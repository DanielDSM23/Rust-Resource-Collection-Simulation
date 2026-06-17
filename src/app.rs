use std::io;
use crossterm::{execute, event, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use std::time::Duration;
use ratatui::{backend::CrosstermBackend, Terminal};

const FRAME_RATE_MS: u64 = 50;

pub struct App {
    width: usize,
    height: usize,
}

impl App {
    pub fn new (width: usize, height: usize) -> Self {
        App {width, height}
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        drain_pending_events()?;
        Ok(())
    }
}

fn drain_pending_events() -> anyhow::Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }

    Ok(())
}