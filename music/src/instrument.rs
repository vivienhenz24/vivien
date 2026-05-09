use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::time::Duration;

use opencv::core;
use rodio::{OutputStream, Sink, Source};

const SAMPLE_RATE: u32 = 44100;
const AMPLITUDE: f32 = 0.22;
const ATTACK_SPEED: f32 = 0.25;   // xylophone snap
const RELEASE_DECAY: f32 = 0.994; // ~200ms ring-out

// Right hand: thumb=C  index=D  middle=E  ring=F  pinky=G
const RIGHT_BASE: [f32; 5] = [261.63, 293.66, 329.63, 349.23, 392.00];
// Left hand:  thumb=B  index=A  middle=G  ring=F  pinky=E
const LEFT_BASE:  [f32; 5] = [246.94, 220.00, 196.00, 174.61, 164.81];

pub const RIGHT_NAMES: [&str; 5] = ["C", "D", "E", "F", "G"];
pub const LEFT_NAMES:  [&str; 5] = ["B", "A", "G", "F", "E"];

// MediaPipe landmark indices for each fingertip and the joint just below it.
// Finger is "up/open"  when tip.y < bend.y  (tip above the mid-joint in screen space).
// Finger is "down/held" when tip.y >= bend.y (tip has curled below the mid-joint).
pub const TIPS:  [usize; 5] = [4,  8,  12, 16, 20]; // thumb → pinky
pub const BENDS: [usize; 5] = [3,  6,  10, 14, 18]; // IP / PIP joints

// ── TrackedHand ───────────────────────────────────────────────────────────────

/// One tracked hand per frame: smoothed screen landmarks + debounced held flags.
/// `held[i] = true`  → finger i is bent down  → note sustained.
/// `held[i] = false` → finger i is raised up   → note off / release.
pub struct TrackedHand {
    pub handedness: f32,
    pub screen_landmarks: Vec<core::Point3f>,
    pub held: [bool; 5],
}

impl TrackedHand {
    // After horizontal flip, thumb of physical right hand appears on the LEFT of the
    // image. MediaPipe labels that as a LEFT hand (< 0.5). So we invert here.
    pub fn is_right(&self) -> bool { self.handedness < 0.5 }
}

// ── Oscillator ────────────────────────────────────────────────────────────────

struct OscState {
    active: AtomicBool,
    freq_bits: AtomicU32,
}

struct FingerOsc {
    state: Arc<OscState>,
    phases: [f32; 4],
    envelope: f32,
    was_active: bool,
    attacking: bool,
}

impl FingerOsc {
    fn new(state: Arc<OscState>) -> Self {
        Self { state, phases: [0.0; 4], envelope: 0.0, was_active: false, attacking: false }
    }
}

// Bright upper harmonics for a xylophone-like tone
const HARM_AMPS: [f32; 4] = [0.5, 1.0, 0.8, 0.4];
const HARM_NORM: f32 = 1.0 / (0.5 + 1.0 + 0.8 + 0.4);

impl Iterator for FingerOsc {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let active = self.state.active.load(Ordering::Relaxed);
        let freq   = f32::from_bits(self.state.freq_bits.load(Ordering::Relaxed));

        if active && !self.was_active { self.attacking = true; }
        self.was_active = active;

        if self.attacking {
            self.envelope += (AMPLITUDE - self.envelope) * ATTACK_SPEED;
            if AMPLITUDE - self.envelope < 0.0005 { self.attacking = false; }
        } else if active {
            self.envelope += (AMPLITUDE - self.envelope) * 0.001;
        } else {
            self.envelope *= RELEASE_DECAY;
        }

        let sr = SAMPLE_RATE as f32;
        let mut s = 0.0f32;
        for h in 0..4 {
            s += HARM_AMPS[h] * (std::f32::consts::TAU * self.phases[h]).sin();
            self.phases[h] = (self.phases[h] + (h as f32 + 1.0) * freq / sr).fract();
        }
        Some(self.envelope * s * HARM_NORM)
    }
}

impl Source for FingerOsc {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { 1 }
    fn sample_rate(&self) -> u32 { SAMPLE_RATE }
    fn total_duration(&self) -> Option<Duration> { None }
}

// ── Instrument ────────────────────────────────────────────────────────────────

pub struct Instrument {
    _stream: OutputStream,
    right: [Arc<OscState>; 5],
    left:  [Arc<OscState>; 5],
}

impl Instrument {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, handle) = OutputStream::try_default()?;
        let make = |base: &[f32; 5]| -> [Arc<OscState>; 5] {
            std::array::from_fn(|i| {
                let s = Arc::new(OscState {
                    active: AtomicBool::new(false),
                    freq_bits: AtomicU32::new(base[i].to_bits()),
                });
                let sink = Sink::try_new(&handle).expect("sink");
                sink.append(FingerOsc::new(s.clone()));
                sink.detach();
                s
            })
        };
        Ok(Self { _stream: stream, right: make(&RIGHT_BASE), left: make(&LEFT_BASE) })
    }

    pub fn update(&self, hands: &[TrackedHand], frame_height: i32) {
        let mut saw_right = false;
        let mut saw_left  = false;
        for hand in hands {
            if hand.is_right() { drive_hand(&self.right, hand, &RIGHT_BASE, frame_height); saw_right = true; }
            else                { drive_hand(&self.left,  hand, &LEFT_BASE,  frame_height); saw_left  = true; }
        }
        if !saw_right { silence(&self.right); }
        if !saw_left  { silence(&self.left);  }
    }
}

fn drive_hand(states: &[Arc<OscState>; 5], hand: &TrackedHand, base: &[f32; 5], frame_h: i32) {
    let wrist_y = hand.screen_landmarks.first().map(|p| p.y).unwrap_or(0.0);
    let octave = if wrist_y < frame_h as f32 * 0.33 { 2.0f32 }
                 else if wrist_y < frame_h as f32 * 0.66 { 1.0 }
                 else { 0.5 };
    for i in 0..5 {
        states[i].freq_bits.store((base[i] * octave).to_bits(), Ordering::Relaxed);
        states[i].active.store(hand.held[i], Ordering::Relaxed);
    }
}

fn silence(states: &[Arc<OscState>; 5]) {
    for s in states { s.active.store(false, Ordering::Relaxed); }
}
