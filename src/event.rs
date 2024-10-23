// src/event.rs

use crate::app::{App, AppMode, SearchType};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, MouseEvent, MouseEventKind};

pub fn handle_event(event: CrosstermEvent, app: &mut App) -> bool {
    match app.mode {
        AppMode::Normal => match event {
            CrosstermEvent::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Char('q') => {
                    app.running = false;
                    false
                }
                KeyCode::Up => {
                    app.scroll_up();
                    app.message = None; // Clear message
                    true
                }
                KeyCode::Down => {
                    app.scroll_down();
                    app.message = None; // Clear message
                    true
                }
                KeyCode::Char('/') => {
                    app.mode = AppMode::Search;
                    app.search_type = SearchType::Ascii;
                    app.input_buffer.clear();
                    app.message = None; // Clear message
                    true
                }
                KeyCode::Char(':') => {
                    app.mode = AppMode::Goto;
                    app.input_buffer.clear();
                    app.message = None; // Clear message
                    true
                }
                KeyCode::Char('x') => {
                    app.mode = AppMode::Search;
                    app.search_type = SearchType::Hex;
                    app.input_buffer.clear();
                    app.message = None; // Clear message
                    true
                }
                KeyCode::Char('h') => { // Press 'h' to enter Help mode
                    app.mode = AppMode::Help;
                    app.message = None; // Clear message
                    true
                }
                KeyCode::Char('t') => { // Press 't' to toggle theme
                    app.toggle_theme();
                    app.message = None; // Clear message
                    true
                }
                _ => true,
            },
            CrosstermEvent::Mouse(MouseEvent { kind, .. }) => match kind {
                MouseEventKind::ScrollUp => {
                    app.scroll_up();
                    app.message = None; // Clear message
                    true
                }
                MouseEventKind::ScrollDown => {
                    app.scroll_down();
                    app.message = None; // Clear message
                    true
                }
                _ => true,
            },
            _ => true,
        },
        AppMode::Search | AppMode::Goto => match event {
            CrosstermEvent::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Enter => {
                    match app.mode {
                        AppMode::Search => {
                            if app.input_buffer.is_empty() {
                                // Message is already set in perform_search
                            } else {
                                app.perform_search();
                            }
                        }
                        AppMode::Goto => {
                            if app.input_buffer.is_empty() {
                                // Message is already set in jump_to_offset
                            } else {
                                app.jump_to_offset();
                            }
                        }
                        _ => {}
                    }
                    app.mode = AppMode::Normal;
                    true
                }
                KeyCode::Char(c) => {
                    app.input_buffer.push(c);
                    true
                }
                KeyCode::Backspace => {
                    app.input_buffer.pop();
                    true
                }
                KeyCode::Esc => {
                    app.mode = AppMode::Normal;
                    app.message = None; // Clear message
                    true
                }
                _ => true,
            },
            _ => true,
        },
        AppMode::Help => match event {
            CrosstermEvent::Key(KeyEvent { code, .. }) => match code {
                KeyCode::Char('h') | KeyCode::Esc => { // Press 'h' or 'Esc' to exit Help mode
                    app.mode = AppMode::Normal;
                    app.message = None; // Clear message
                    true
                }
                _ => true,
            },
            _ => true,
        },
    }
}
