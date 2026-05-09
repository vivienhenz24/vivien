mod app;
mod camera;
mod frame;
mod handpose;
mod instrument;
mod palm;
mod recorder;

use app::App;
use camera::CameraConfig;

fn main() -> opencv::Result<()> {
    let mut app = App::new(CameraConfig::default())?;
    app.run()
}
