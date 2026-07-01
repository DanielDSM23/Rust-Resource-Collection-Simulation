use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::map::{ResourceKind, Tile};
use crate::simulation::SimState;

pub fn render(frame: &mut Frame, state: &SimState) {
    let area = frame.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(35)])
        .split(main_chunks[0]);

    render_map(frame, state, top_chunks[0]);
    render_log(frame, state, top_chunks[1]);
    render_hud(frame, state, main_chunks[1]);
}

fn render_map(frame: &mut Frame, state: &SimState, area: Rect) {
    let block = Block::default()
        .title(" Resource Collection Simulation ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let max_x = (inner.width as usize).min(state.map.width);
    let max_y = (inner.height as usize).min(state.map.height);
    let mut lines: Vec<Line> = Vec::with_capacity(max_y);

    for y in 0..max_y {
        let mut spans: Vec<Span> = Vec::with_capacity(max_x);

        for x in 0..max_x {
            let robot_span = state
                .robot_positions
                .iter()
                .find(|r| r.pos == (x, y))
                .map(|r| {
                    if r.is_collector {
                        Span::styled("o", Style::default().fg(Color::Magenta))
                    } else {
                        Span::styled("x", Style::default().fg(Color::Red))
                    }
                });

            if let Some(span) = robot_span {
                spans.push(span);
                continue;
            }

            let span = match state.map.get(x, y) {
                Tile::Empty => Span::raw(" "),
                Tile::Obstacle => Span::styled("O", Style::default().fg(Color::Cyan)),
                Tile::Base => Span::styled("#", Style::default().fg(Color::LightGreen)),
                Tile::Resource { kind, .. } => match kind {
                    ResourceKind::Energy => Span::styled("E", Style::default().fg(Color::Green)),
                    ResourceKind::Crystal => {
                        Span::styled("C", Style::default().fg(Color::LightMagenta))
                    }
                },
            };
            spans.push(span);
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_hud(frame: &mut Frame, state: &SimState, area: Rect) {
    let text = format!(
        " Energy: {:>5}   Crystals: {:>5}   Tick: {:>6}   Base: ({:>2}, {:>2})   Press any key to quit",
        state.base.total_energy,
        state.base.total_crystals,
        state.tick,
        state.base.pos.0,
        state.base.pos.1
    );

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

fn render_log(frame: &mut Frame, state: &SimState, area: Rect) {
    let max_lines = area.height.saturating_sub(2) as usize;

    let lines = if state.event_log.is_empty() {
        vec![Line::from(Span::raw("Aucun evenement pour le moment"))]
    } else {
        let total = state.event_log.len();
        let auto_scroll = if state.event_log_scroll == 0 && total > max_lines {
            total.saturating_sub(max_lines)
        } else {
            state.event_log_scroll.min(total.saturating_sub(max_lines))
        };

        state
            .event_log
            .iter()
            .skip(auto_scroll)
            .take(max_lines)
            .map(|entry| Line::from(Span::raw(entry.clone())))
            .collect()
    };

    let title = if state.event_log.len() > max_lines {
        let total = state.event_log.len();
        let auto_scroll = if state.event_log_scroll == 0 && total > max_lines {
            total.saturating_sub(max_lines)
        } else {
            state.event_log_scroll.min(total.saturating_sub(max_lines))
        };

        format!(" Evenements [{}/{}] ", auto_scroll + 1, total)
    } else {
        " Evenements ".to_string()
    };

    let paragraph = Paragraph::new(lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}
