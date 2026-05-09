use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::thread::{self, JoinHandle};

use opencv::{core::Size, prelude::*, videoio};

use crate::frame::FramePacket;

const RECORDER_BUFFER_SIZE: usize = 8;

pub enum RecorderCommand {
    Frame(FramePacket),
    Shutdown,
}

pub enum RecorderEvent {
    Error(opencv::Error),
    Finished,
}

pub struct RecorderWorker {
    pub commands: SyncSender<RecorderCommand>,
    pub events: Receiver<RecorderEvent>,
    handle: Option<JoinHandle<()>>,
}

impl RecorderWorker {
    pub fn spawn(output_path: PathBuf, frame_size: Size, fps: f64) -> Self {
        let (command_tx, command_rx) = sync_channel(RECORDER_BUFFER_SIZE);
        let (event_tx, event_rx) = sync_channel(1);
        let handle =
            thread::spawn(move || run_recorder(output_path, frame_size, fps, command_rx, event_tx));

        Self {
            commands: command_tx,
            events: event_rx,
            handle: Some(handle),
        }
    }

    pub fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for RecorderWorker {
    fn drop(&mut self) {
        let _ = self.commands.try_send(RecorderCommand::Shutdown);
        self.join();
    }
}

fn run_recorder(
    output_path: PathBuf,
    frame_size: Size,
    fps: f64,
    command_rx: Receiver<RecorderCommand>,
    event_tx: SyncSender<RecorderEvent>,
) {
    if let Err(error) = recorder_loop(output_path, frame_size, fps, command_rx) {
        let _ = event_tx.send(RecorderEvent::Error(error));
        return;
    }

    let _ = event_tx.send(RecorderEvent::Finished);
}

fn recorder_loop(
    output_path: PathBuf,
    frame_size: Size,
    fps: f64,
    command_rx: Receiver<RecorderCommand>,
) -> opencv::Result<()> {
    prepare_output_dir(&output_path)?;

    let fourcc = videoio::VideoWriter::fourcc('m', 'p', '4', 'v')?;
    let output_path = output_path.to_string_lossy().into_owned();
    let mut writer = videoio::VideoWriter::new(&output_path, fourcc, fps, frame_size, true)?;
    if !writer.is_opened()? {
        return Err(opencv::Error::new(0, "failed to open the video writer"));
    }

    while let Ok(command) = command_rx.recv() {
        match command {
            RecorderCommand::Frame(packet) => writer.write(&packet.image)?,
            RecorderCommand::Shutdown => break,
        }
    }

    writer.release()?;
    Ok(())
}

fn prepare_output_dir(output_path: &Path) -> opencv::Result<()> {
    let parent = output_path
        .parent()
        .ok_or_else(|| opencv::Error::new(0, "invalid output path"))?;

    fs::create_dir_all(parent).map_err(|err| {
        opencv::Error::new(0, format!("failed to create output directory: {err}"))
    })?;

    Ok(())
}
