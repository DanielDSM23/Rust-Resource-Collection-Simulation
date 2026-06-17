mod app;
use app::App;

fn main() -> anyhow::Result<()> {
    let mut app = App::new(60, 30);
    app.run()
}