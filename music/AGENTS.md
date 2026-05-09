# Repository Notes

This project is a small Rust camera app built on the `opencv` crate.

## Current behavior

- `cargo run` starts the app.
- The app opens the default camera and shows a live preview window.
- Frames are mirrored horizontally before preview and recording.
- Recording is enabled by default and writes to `recordings/capture.mp4`.
- Press `r` to toggle recording.
- Press `q` or `Esc` to quit.

## Architecture

The app is intentionally split into small stages so future OpenCV work can be added without rewriting the whole program.

- `src/main.rs`
  Starts the app with the default camera configuration.
- `src/app.rs`
  Owns the preview window, main loop, recording toggle, frame mirroring, and shutdown flow.
- `src/camera.rs`
  Runs the capture worker thread. This thread exclusively owns `opencv::videoio::VideoCapture`.
- `src/recorder.rs`
  Runs the recorder worker thread. This thread exclusively owns `opencv::videoio::VideoWriter`.
- `src/frame.rs`
  Defines `FramePacket`, which carries `frame_id`, capture timestamp, and the frame `Mat`.

## Data flow

1. The capture worker opens the camera and reads frames continuously.
2. Frames are sent through a bounded channel to the app thread.
3. The app thread flips each frame horizontally.
4. The app thread shows the flipped frame in the OpenCV preview window.
5. If recording is enabled, the app sends a cloned frame to the recorder worker.
6. The recorder worker writes frames to disk.

## Design intent

This is not a one-loop prototype anymore. The current structure is meant to be a clean base for:

- adding OpenCV effects in `src/app.rs`
- inserting analysis or tracking stages
- swapping the preview layer later if needed
- keeping capture and recording isolated from UI logic

The app currently uses bounded channels to avoid unbounded lag and memory growth.

## Debugging already added

Startup diagnostics are printed to stderr:

- preview window creation
- camera backend attempts
- backend that actually opened
- requested vs actual resolution/fps
- waiting for first frame
- repeated empty-frame warnings
- recorder startup
- timeout if no first frame arrives within 5 seconds

If camera startup fails on macOS, the likely suspects are camera permissions or backend-specific capture issues. The logs in `src/camera.rs` and `src/app.rs` are already set up to narrow that down.

## Practical notes for future agents

- Keep camera ownership inside `src/camera.rs`.
- Keep recorder ownership inside `src/recorder.rs`.
- Put image transforms in `src/app.rs` unless you are intentionally introducing a dedicated processing stage.
- If you change frame size, codec, or color format, check both preview and recorder paths.
- If recording behavior changes, make the frame-drop policy explicit rather than accidental.
