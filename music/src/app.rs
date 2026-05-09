use std::path::PathBuf;
use std::sync::mpsc::{RecvTimeoutError, TryRecvError};
use std::time::Duration;

use opencv::{core, highgui, imgproc, prelude::*};

use crate::camera::{CameraConfig, CaptureCommand, CaptureEvent, CaptureWorker};
use crate::handpose::HandPoseDetector;
use crate::instrument::{Instrument, LEFT_NAMES, RIGHT_NAMES, TIPS, BENDS, TrackedHand};
use crate::palm::PalmDetector;
use crate::recorder::{RecorderCommand, RecorderEvent, RecorderWorker};

const WINDOW_NAME: &str = "Musical Instrument";
const PALM_MODEL: &str = "models/palm_detection_mediapipe_2023feb.onnx";
const HAND_MODEL: &str = "models/handpose_estimation_mediapipe_2023feb.onnx";
const OUTPUT_PATH: &str = "recordings/capture.mp4";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

const REDETECT_CONFIDENCE: f32 = 0.6;
// Re-scan for palms at least every N frames so a second hand entering frame gets picked up.
const PALM_SCAN_INTERVAL: u64 = 12;

const SKELETON: &[(usize, usize)] = &[
    (0, 1), (1, 2), (2, 3), (3, 4),
    (0, 5), (5, 6), (6, 7), (7, 8),
    (0, 9), (9, 10), (10, 11), (11, 12),
    (0, 13), (13, 14), (14, 15), (15, 16),
    (0, 17), (17, 18), (18, 19), (19, 20),
    (5, 9), (9, 13), (13, 17),
];

// ── Schmitt-trigger debounce ──────────────────────────────────────────────────

struct FingerTracker {
    counts: [u8; 5],
    states: [bool; 5],
}

impl FingerTracker {
    const MAX: u8 = 3;
    const ON:  u8 = 2; // 2 consecutive frames to activate
    const OFF: u8 = 1;

    fn new() -> Self { Self { counts: [0; 5], states: [false; 5] } }

    fn update(&mut self, raw: [bool; 5]) -> [bool; 5] {
        for i in 0..5 {
            if raw[i] { self.counts[i] = (self.counts[i] + 1).min(Self::MAX); }
            else       { self.counts[i] = self.counts[i].saturating_sub(1); }

            if      self.counts[i] >= Self::ON  { self.states[i] = true;  }
            else if self.counts[i] <= Self::OFF { self.states[i] = false; }
        }
        self.states
    }

    fn reset(&mut self) { self.counts = [0; 5]; self.states = [false; 5]; }
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    capture: CaptureWorker,
    recorder: Option<RecorderWorker>,
    recording_enabled: bool,
    palm_detector: PalmDetector,
    hand_detector: HandPoseDetector,
    instrument: Instrument,
    frame_count: u64,
    last_palm_detect_frame: u64,
    cached_palms: Vec<crate::palm::PalmDetection>,
    last_min_confidence: f32,
    right_smooth: Option<Vec<core::Point3f>>,
    left_smooth:  Option<Vec<core::Point3f>>,
    right_tracker: FingerTracker,
    left_tracker:  FingerTracker,
}

impl App {
    pub fn new(config: CameraConfig) -> opencv::Result<Self> {
        let instrument = Instrument::new()
            .map_err(|e| opencv::Error::new(0, format!("audio init: {e}")))?;
        Ok(Self {
            capture: CaptureWorker::spawn(config),
            recorder: None,
            recording_enabled: true,
            palm_detector: PalmDetector::new(PathBuf::from(PALM_MODEL).as_path())?,
            hand_detector: HandPoseDetector::new(PathBuf::from(HAND_MODEL).as_path())?,
            instrument,
            frame_count: 0,
            last_palm_detect_frame: 0,
            cached_palms: Vec::new(),
            last_min_confidence: 1.0,
            right_smooth: None,
            left_smooth:  None,
            right_tracker: FingerTracker::new(),
            left_tracker:  FingerTracker::new(),
        })
    }

    pub fn run(&mut self) -> opencv::Result<()> {
        highgui::named_window(WINDOW_NAME, highgui::WINDOW_AUTOSIZE)?;

        loop {
            self.poll_recorder_events()?;

            let event = match self.capture.events.recv_timeout(STARTUP_TIMEOUT) {
                Ok(e) => e,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(opencv::Error::new(0, "timed out waiting for first camera frame"));
                }
                Err(RecvTimeoutError::Disconnected) => break,
            };

            match event {
                CaptureEvent::Frame(frame) => {
                    let frame = flip_frame(frame)?;

                    if self.recorder.is_none() {
                        let sz = frame.image.size()?;
                        self.recorder = Some(RecorderWorker::spawn(
                            PathBuf::from(OUTPUT_PATH), sz, 30.0,
                        ));
                    }
                    if self.recording_enabled {
                        send_to_recorder(self.recorder.as_ref(), &frame)?;
                    }

                    let hands = self.detect_hands(&frame.image);
                    let sz = frame.image.size()?;
                    self.instrument.update(&hands, sz.height);

                    let mut preview = frame.image.try_clone()?;
                    draw_overlay(&mut preview, &hands)?;
                    highgui::imshow(WINDOW_NAME, &preview)?;

                    match highgui::wait_key(1)? {
                        k if k == 'q' as i32 || k == 27 => break,
                        k if k == 'r' as i32 => self.recording_enabled = !self.recording_enabled,
                        _ => {}
                    }
                }
                CaptureEvent::Ended => break,
                CaptureEvent::Error(e) => return Err(e),
            }
        }

        self.shutdown();
        highgui::destroy_window(WINDOW_NAME)?;
        Ok(())
    }

    fn detect_hands(&mut self, image: &core::Mat) -> Vec<TrackedHand> {
        // ── 1. Palm re-detection ─────────────────────────────────────────────
        // Always re-scan on a fixed interval so a second hand entering frame
        // gets picked up. Also re-scan immediately when confidence drops.
        let frames_since = self.frame_count.saturating_sub(self.last_palm_detect_frame);
        let should_redetect =
            frames_since >= PALM_SCAN_INTERVAL
            || self.last_min_confidence < REDETECT_CONFIDENCE;

        if should_redetect {
            self.last_palm_detect_frame = self.frame_count;
            match self.palm_detector.detect(image) {
                Ok(palms) => {
                    eprintln!("[detect] palm scan frame={} found={}", self.frame_count, palms.len());
                    self.cached_palms = palms;
                }
                Err(e) => eprintln!("[detect] palm error: {e}"),
            }
        }
        self.frame_count += 1;

        // ── 2. Hand pose inference ───────────────────────────────────────────
        let mut raw_poses = Vec::with_capacity(self.cached_palms.len());
        let mut min_conf = 1.0f32;

        for palm in &self.cached_palms {
            match self.hand_detector.infer(image, palm) {
                Ok(Some(pose)) => { min_conf = min_conf.min(pose.confidence); raw_poses.push(pose); }
                Ok(None) => { min_conf = 0.0; }
                Err(e) => eprintln!("[detect] hand error: {e}"),
            }
        }
        self.last_min_confidence = min_conf;

        // ── 3. Smooth → angle check → debounce ──────────────────────────────
        let mut tracked = Vec::with_capacity(raw_poses.len());
        let mut saw_right = false;
        let mut saw_left  = false;

        for mut pose in raw_poses {
            let is_right = pose.handedness > 0.5;
            let smooth   = if is_right { &mut self.right_smooth  } else { &mut self.left_smooth  };
            let tracker  = if is_right { &mut self.right_tracker } else { &mut self.left_tracker };

            // EMA on screen landmarks
            if let Some(prev) = smooth.as_ref() {
                for (cur, p) in pose.screen_landmarks.iter_mut().zip(prev.iter()) {
                    cur.x = 0.6 * cur.x + 0.4 * p.x;
                    cur.y = 0.6 * cur.y + 0.4 * p.y;
                    cur.z = 0.6 * cur.z + 0.4 * p.z;
                }
            }
            *smooth = Some(pose.screen_landmarks.clone());

            // Finger is "up/open" when tip.y < bend.y (tip above mid-joint in screen space).
            // Finger is "down/held" when tip has curled below the mid-joint.
            let lms = &pose.screen_landmarks;
            let bent: [bool; 5] = std::array::from_fn(|i| lms[TIPS[i]].y >= lms[BENDS[i]].y);

            // Debounce the held state
            let held = tracker.update(bent);

            if is_right { saw_right = true; } else { saw_left = true; }

            tracked.push(TrackedHand {
                handedness: pose.handedness,
                screen_landmarks: pose.screen_landmarks,
                held,
            });
        }

        if !saw_right { self.right_tracker.reset(); self.right_smooth = None; }
        if !saw_left  { self.left_tracker.reset();  self.left_smooth  = None; }

        tracked
    }

    fn poll_recorder_events(&mut self) -> opencv::Result<()> {
        if let Some(recorder) = &self.recorder {
            match recorder.events.try_recv() {
                Ok(RecorderEvent::Error(e)) => return Err(e),
                Ok(RecorderEvent::Finished) => self.recorder = None,
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => self.recorder = None,
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) {
        let _ = self.capture.commands.send(CaptureCommand::Shutdown);
        self.capture.join();
        if let Some(mut r) = self.recorder.take() {
            let _ = r.commands.send(RecorderCommand::Shutdown);
            r.join();
        }
    }
}

fn flip_frame(mut frame: crate::frame::FramePacket) -> opencv::Result<crate::frame::FramePacket> {
    let mut flipped = core::Mat::default();
    core::flip(&frame.image, &mut flipped, 1)?;
    frame.image = flipped;
    Ok(frame)
}

fn send_to_recorder(
    recorder: Option<&RecorderWorker>,
    frame: &crate::frame::FramePacket,
) -> opencv::Result<()> {
    if let Some(r) = recorder {
        let _ = r.commands.try_send(RecorderCommand::Frame(frame.duplicate()?));
    }
    Ok(())
}

fn draw_overlay(frame: &mut core::Mat, hands: &[TrackedHand]) -> opencv::Result<()> {
    for hand in hands {
        let lms = &hand.screen_landmarks;
        if lms.len() < 21 { continue; }

        let is_right = hand.is_right();
        let note_names: &[&str; 5] = if is_right { &RIGHT_NAMES } else { &LEFT_NAMES };

        let line_color = if is_right {
            core::Scalar::new(0.0, 180.0, 255.0, 0.0)
        } else {
            core::Scalar::new(200.0, 130.0, 50.0, 0.0)
        };

        for &(a, b) in SKELETON {
            let pa = core::Point::new(lms[a].x as i32, lms[a].y as i32);
            let pb = core::Point::new(lms[b].x as i32, lms[b].y as i32);
            imgproc::line(frame, pa, pb, line_color, 2, imgproc::LINE_AA, 0)?;
        }

        for (i, &tip_idx) in TIPS.iter().enumerate() {
            let lm = lms[tip_idx];
            let pt = core::Point::new(lm.x as i32, lm.y as i32);

            let (color, radius) = if hand.held[i] {
                if is_right { (core::Scalar::new(0.0, 255.0, 100.0, 0.0), 10) }
                else        { (core::Scalar::new(80.0, 210.0, 255.0, 0.0), 10) }
            } else {
                (core::Scalar::new(80.0, 80.0, 80.0, 0.0), 5)
            };

            imgproc::circle(frame, pt, radius, color, -1, imgproc::LINE_AA, 0)?;

            if hand.held[i] {
                let label = core::Point::new(pt.x + 10, pt.y - 10);
                imgproc::put_text(frame, note_names[i], label,
                    imgproc::FONT_HERSHEY_SIMPLEX, 0.75, color, 2, imgproc::LINE_AA, false)?;
            }
        }
    }
    Ok(())
}
