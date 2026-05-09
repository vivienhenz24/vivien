use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use opencv::{core::Mat, prelude::*, videoio};

use crate::frame::FramePacket;

const CAPTURE_BUFFER_SIZE: usize = 2;
const STOP_POLL_INTERVAL: Duration = Duration::from_millis(20);
const EMPTY_FRAME_LOG_INTERVAL: u64 = 60;

#[derive(Clone, Copy)]
pub struct CameraConfig {
    pub device_index: i32,
    pub width: i32,
    pub height: i32,
    pub fps: f64,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            device_index: 0,
            width: 1280,
            height: 720,
            fps: 30.0,
        }
    }
}

pub enum CaptureEvent {
    Frame(FramePacket),
    Ended,
    Error(opencv::Error),
}

pub enum CaptureCommand {
    Shutdown,
}

pub struct CaptureWorker {
    pub events: Receiver<CaptureEvent>,
    pub commands: SyncSender<CaptureCommand>,
    handle: Option<JoinHandle<()>>,
}

impl CaptureWorker {
    pub fn spawn(config: CameraConfig) -> Self {
        let (event_tx, event_rx) = sync_channel(CAPTURE_BUFFER_SIZE);
        let (command_tx, command_rx) = sync_channel(1);
        let handle = thread::spawn(move || run_capture(config, event_tx, command_rx));

        Self {
            events: event_rx,
            commands: command_tx,
            handle: Some(handle),
        }
    }

    pub fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for CaptureWorker {
    fn drop(&mut self) {
        let _ = self.commands.try_send(CaptureCommand::Shutdown);
        self.join();
    }
}

fn run_capture(
    config: CameraConfig,
    event_tx: SyncSender<CaptureEvent>,
    command_rx: Receiver<CaptureCommand>,
) {
    if let Err(error) = capture_loop(config, event_tx.clone(), command_rx) {
        let _ = event_tx.send(CaptureEvent::Error(error));
    }
}

fn capture_loop(
    config: CameraConfig,
    event_tx: SyncSender<CaptureEvent>,
    command_rx: Receiver<CaptureCommand>,
) -> opencv::Result<()> {
    eprintln!(
        "[camera] starting capture device={} requested={}x{}@{}",
        config.device_index, config.width, config.height, config.fps
    );

    let mut camera = open_camera(config.device_index)?;
    let backend_name = camera.get_backend_name()?;
    eprintln!("[camera] opened backend={backend_name}");

    camera.set(videoio::CAP_PROP_FRAME_WIDTH, config.width as f64)?;
    camera.set(videoio::CAP_PROP_FRAME_HEIGHT, config.height as f64)?;
    camera.set(videoio::CAP_PROP_FPS, config.fps)?;
    eprintln!(
        "[camera] actual={}x{}@{:.2}",
        camera.get(videoio::CAP_PROP_FRAME_WIDTH)?,
        camera.get(videoio::CAP_PROP_FRAME_HEIGHT)?,
        camera.get(videoio::CAP_PROP_FPS)?
    );

    let mut frame_id = 0_u64;
    let mut empty_frame_count = 0_u64;

    loop {
        match command_rx.recv_timeout(STOP_POLL_INTERVAL) {
            Ok(CaptureCommand::Shutdown) => break,
            Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }

        let mut image = Mat::default();
        if frame_id == 0 && empty_frame_count == 0 {
            eprintln!("[camera] waiting for first frame");
        }
        camera.read(&mut image)?;
        if image.empty() {
            empty_frame_count += 1;
            if empty_frame_count % EMPTY_FRAME_LOG_INTERVAL == 0 {
                eprintln!("[camera] still receiving empty frames count={empty_frame_count}");
            }
            continue;
        }

        if frame_id == 0 {
            let size = image.size()?;
            eprintln!(
                "[camera] received first frame size={}x{}",
                size.width, size.height
            );
        }

        let packet = FramePacket::new(frame_id, image);
        frame_id += 1;

        match event_tx.try_send(CaptureEvent::Frame(packet)) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => break,
        }
    }

    camera.release()?;
    let _ = event_tx.send(CaptureEvent::Ended);
    Ok(())
}

fn open_camera(device_index: i32) -> opencv::Result<videoio::VideoCapture> {
    #[cfg(target_os = "macos")]
    let backends = [videoio::CAP_AVFOUNDATION, videoio::CAP_ANY];

    #[cfg(not(target_os = "macos"))]
    let backends = [videoio::CAP_ANY];

    let mut last_error = None;

    for backend in backends {
        eprintln!("[camera] trying backend={}", backend_label(backend));
        match videoio::VideoCapture::new(device_index, backend) {
            Ok(camera) => {
                if camera.is_opened()? {
                    return Ok(camera);
                }
                last_error = Some(opencv::Error::new(
                    0,
                    format!("backend {} did not open the camera", backend_label(backend)),
                ));
            }
            Err(error) => {
                eprintln!(
                    "[camera] backend={} failed: {}",
                    backend_label(backend),
                    error
                );
                last_error = Some(error);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| opencv::Error::new(0, "failed to open the default camera")))
}

fn backend_label(backend: i32) -> &'static str {
    match backend {
        #[cfg(target_os = "macos")]
        videoio::CAP_AVFOUNDATION => "AVFOUNDATION",
        videoio::CAP_ANY => "ANY",
        _ => "UNKNOWN",
    }
}
