use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};

#[derive(Default)]
struct App {
    should_quit: bool,
    stablecoin_name: String,
    total_supply: u64,
    paused: bool,
    preset: u8,
    blacklist_count: u32,
    minter_count: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::default();

    loop {
        terminal.draw(|f| ui(f, &app))?;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char('p') => {},
                    _ => {}
                }
            }
        }
        if app.should_quit { break; }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
        .split(f.size());

    let title = Paragraph::new(format!("SSS Token Admin - {}", app.stablecoin_name))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let status = Paragraph::new(format!(
        "Supply: {} | Preset: SSS-{} | Paused: {} | Blacklist: {} | Minters: {}",
        app.total_supply, app.preset, app.paused, app.blacklist_count, app.minter_count
    )).block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[1]);

    let controls = Paragraph::new("Controls: [q]uit [p]ause")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[2]);
}
