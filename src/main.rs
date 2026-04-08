mod app;
mod metrics;
mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new();
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(app.tick_rate_ms);

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    (KeyCode::Tab, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                        app.selected_tab = (app.selected_tab + 1) % 7;
                    }
                    (KeyCode::BackTab, _) | (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                        app.selected_tab = (app.selected_tab + 6) % 7;
                    }
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                        match app.selected_tab {
                            4 => {
                                app.disk_scroll = (app.disk_scroll + 1)
                                    .min(app.disks.len().saturating_sub(1));
                            }
                            5 => {
                                app.process_scroll = (app.process_scroll + 1)
                                    .min(app.processes.len().saturating_sub(1));
                            }
                            6 => {
                                app.gpu_scroll = (app.gpu_scroll + 1)
                                    .min(app.gpus.len().saturating_sub(1));
                            }
                            _ => {}
                        }
                    }
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                        match app.selected_tab {
                            4 => {
                                app.disk_scroll = app.disk_scroll.saturating_sub(1);
                            }
                            5 => {
                                app.process_scroll = app.process_scroll.saturating_sub(1);
                            }
                            6 => {
                                app.gpu_scroll = app.gpu_scroll.saturating_sub(1);
                            }
                            _ => {}
                        }
                    }
                    (KeyCode::Char('1'), _) => app.selected_tab = 0,
                    (KeyCode::Char('2'), _) => app.selected_tab = 1,
                    (KeyCode::Char('3'), _) => app.selected_tab = 2,
                    (KeyCode::Char('4'), _) => app.selected_tab = 3,
                    (KeyCode::Char('5'), _) => app.selected_tab = 4,
                    (KeyCode::Char('6'), _) => app.selected_tab = 5,
                    (KeyCode::Char('7'), _) => app.selected_tab = 6,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

