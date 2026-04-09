// Copyright (C) 2026 Raimo Geisel
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Entry point for **narsil** — a terminal-based system resource monitor.
//!
//! Initialises the crossterm raw-mode terminal, runs the main event/tick loop,
//! and restores the terminal on both normal exit and errors.
//!
//! # CLI flags
//!
//! | Flag | Default | Description |
//! |------|---------|-------------|
//! | `--interval <ms>` | 1000 | Refresh interval in milliseconds |

mod app;
mod metrics;
mod ui;

#[cfg(test)]
mod tests;

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

/// Number of tabs: 7 on Linux (includes GPU), 6 on other platforms.
#[cfg(target_os = "linux")]
const TAB_COUNT: usize = 7;
#[cfg(not(target_os = "linux"))]
const TAB_COUNT: usize = 6;

/// Default refresh interval in milliseconds.
const DEFAULT_INTERVAL_MS: u64 = 1000;

/// Parses `--interval <ms>` from the process arguments.
///
/// Returns [`DEFAULT_INTERVAL_MS`] when the flag is absent, and exits with a
/// human-readable error message when the supplied value is not a positive
/// integer.
fn parse_interval_ms() -> u64 {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--interval" {
            match args.next() {
                Some(val) => match val.parse::<u64>() {
                    Ok(0) => {
                        eprintln!("error: --interval must be greater than 0");
                        std::process::exit(1);
                    }
                    Ok(ms) => return ms,
                    Err(_) => {
                        eprintln!("error: --interval expects a positive integer (milliseconds), got {:?}", val);
                        std::process::exit(1);
                    }
                },
                None => {
                    eprintln!("error: --interval requires a value");
                    std::process::exit(1);
                }
            }
        } else if arg == "--help" || arg == "-h" {
            println!("Usage: narsil [--interval <ms>]");
            println!();
            println!("Options:");
            println!("  --interval <ms>   Refresh interval in milliseconds [default: {}]", DEFAULT_INTERVAL_MS);
            println!("  -h, --help        Print this help message");
            std::process::exit(0);
        }
    }
    DEFAULT_INTERVAL_MS
}

/// Initialises the terminal (raw mode, alternate screen, mouse capture),
/// delegates to [`run_app`], and guarantees terminal restoration on exit.
fn main() -> Result<()> {
    let interval_ms = parse_interval_ms();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, interval_ms);

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

/// Runs the main event/tick loop.
///
/// Draws the UI every frame, polls for crossterm [`Event`]s, dispatches
/// keyboard input to update [`App`] state, and calls [`App::on_tick`] once
/// per tick to refresh all metrics.
///
/// `interval_ms` controls how often [`App::on_tick`] is called.
fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, interval_ms: u64) -> Result<()>
where
    <B as ratatui::backend::Backend>::Error: Send + Sync + 'static,
{
    let mut app = App::new();
    app.tick_rate_ms = interval_ms;
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
                        app.selected_tab = (app.selected_tab + 1) % TAB_COUNT;
                    }
                    (KeyCode::BackTab, _) | (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                        app.selected_tab = (app.selected_tab + TAB_COUNT - 1) % TAB_COUNT;
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
                            #[cfg(target_os = "linux")]
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
                            #[cfg(target_os = "linux")]
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
                    #[cfg(target_os = "linux")]
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

