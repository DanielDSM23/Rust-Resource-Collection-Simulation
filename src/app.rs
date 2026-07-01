use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::simulation::Simulation;
use crate::ui::render;

const FRAME_RATE_MS: u64 = 50;

pub struct App {
    width: usize,
    height: usize,
}

impl App {
    pub fn new(width: usize, height: usize) -> Self {
        App { width, height }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        drain_pending_events()?;

        let mut sim = Simulation::new(self.width, self.height);
        sim.start();
        let state = sim.state.clone();

        let result = (|| {
            loop {
                let frame_start = Instant::now();

                {
                    let s = state.lock().unwrap();
                    terminal.draw(|f| render(f, &s))?;
                }

                let elapsed = frame_start.elapsed();
                let timeout = Duration::from_millis(FRAME_RATE_MS)
                    .checked_sub(elapsed)
                    .unwrap_or_default();

                if event::poll(timeout)? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
                        break;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        })();

        sim.stop();

        let restore_result = (|| {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            Ok::<(), anyhow::Error>(())
        })();

        result.and(restore_result)
    }
}

fn drain_pending_events() -> anyhow::Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }

    Ok(())
}
