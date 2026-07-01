mod app;
mod base;
mod map;
mod messages;
mod simulation;
mod robots;
mod ui;

use app::App;

fn main() -> anyhow::Result<()> {
    let mut app = App::new(60, 30);
    app.run()
}
