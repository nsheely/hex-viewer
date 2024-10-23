// src/main.rs

mod app;
mod event;
mod parsers;
mod ui;
mod utils;

use crate::app::{App, Theme};
use crate::event::handle_event;
use crate::ui::draw_ui;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::{error::Error, io, panic};

/// Command-line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// File path to view
    file_path: String,

    /// Number of bytes per line in the hex view
    #[arg(short, long, default_value_t = 16)]
    bytes_per_line: usize,

    /// Theme: light or dark
    #[arg(short, long, default_value = "dark")]
    theme: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set a panic hook to restore terminal state in case of panic
    panic::set_hook(Box::new(|info| {
        let mut stdout = io::stdout();
        let _ = disable_raw_mode();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("Application panicked: {:?}", info);
    }));

    // Parse command-line arguments
    let cli = Cli::parse();

    // Determine theme
    let theme = match cli.theme.to_lowercase().as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => {
            eprintln!("Unknown theme '{}'. Falling back to Dark theme.", cli.theme);
            Theme::Dark
        }
    };

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize app state
    let mut app = match App::new(cli.file_path, cli.bytes_per_line, theme) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to initialize application: {}", e);
            // Restore terminal before exiting
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
            return Err(e);
        }
    };

    // Run application
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle errors
    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    while app.running {
        terminal.draw(|f| draw_ui(f, app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            let event = crossterm::event::read()?;
            if !handle_event(event, app) {
                break;
            }
            app.clamp_scroll_offset(); // Ensure scroll_offset is valid
        }
    }
    Ok(())
}
