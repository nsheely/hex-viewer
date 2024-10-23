// src/ui.rs

use crate::app::{App, AppMode, Theme};
use crate::utils::format_hex_dump;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Line, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_ui<'a>(f: &mut Frame<'a>, app: &mut App) {
    match app.mode {
        AppMode::Help => {
            // Render Help Mode as a centered, prominent block
            let help_text = vec![
                Line::from(Span::styled(
                    "Hex Viewer Help ('h' to leave this view)",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::raw("This application allows you to view files in a hexadecimal format.")),
                Line::from(""),
                Line::from(Span::styled("Keybindings:", Style::default().add_modifier(Modifier::UNDERLINED))),
                Line::from("  ↑ / ↓ : Scroll Up/Down"),
                Line::from("  /     : Enter ASCII search mode"),
                Line::from("  x     : Enter Hex search mode"),
                Line::from("  :     : Go to Offset"),
                Line::from("  t     : Toggle Theme (Light/Dark)"),
                Line::from("  h     : Toggle Help"),
                Line::from("  q     : Quit"),
                Line::from(""),
                Line::from(Span::styled("Usage:", Style::default().add_modifier(Modifier::UNDERLINED))),
                Line::from("  - Navigate using arrow keys or mouse wheel."),
                Line::from("  - Search for ASCII strings or hexadecimal patterns to highlight them."),
                Line::from("  - Jump directly to a specific offset within the file."),
                Line::from("  - Toggle between Light and Dark themes for better visibility."),
                Line::from(""),
                Line::from("Additional Information:"),
                Line::from("  - Press 'h' or 'Esc' to return to Normal Mode."),
                Line::from("  - Search results are highlighted based on your query."),
            ];
            let help_block = Paragraph::new(Text::from(help_text))
                .block(Block::default().borders(Borders::ALL).title("Help"))
                .style(match app.theme {
                    Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
                    Theme::Dark => Style::default().fg(Color::White).bg(Color::Black),
                })
                .wrap(ratatui::widgets::Wrap { trim: true }); // Enable text wrapping

            // Calculate a centered rectangle for the Help block
            let width = 80;  // Width in percentage
            let height = 30; // Height in percentage
            let size = f.area();
            let rect = centered_rect(width, height, size);
            f.render_widget(help_block, rect);
        }
        _ => {
            // Normal mode layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Metadata
                        Constraint::Length(3), // Input or Help
                        Constraint::Min(0),    // Content
                        Constraint::Length(3), // Message (increased from 1 to 3)
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // Render metadata
            let metadata = render_metadata(app);
            f.render_widget(metadata, chunks[0]);

            // Render input box
            let input = render_input(app);
            f.render_widget(input, chunks[1]);

            // Render content
            let content = render_content(app, chunks[2].height as usize);
            f.render_widget(content, chunks[2]);

            // Render message box
            if let Some(message) = &app.message {
                let message_paragraph = Paragraph::new(message.clone())
                    .block(Block::default().borders(Borders::ALL).title("Message"))
                    .style(Style::default().fg(Color::Red))
                    .alignment(ratatui::layout::Alignment::Left)
                    .wrap(ratatui::widgets::Wrap { trim: true }); // Enable text wrapping
                f.render_widget(message_paragraph, chunks[3]);
            } else {
                // Clear the message box if there's no message
                let empty = Paragraph::new("");
                f.render_widget(empty, chunks[3]);
            }
        }
    }
}

/// Helper function to create a centered rectangular area
fn centered_rect(width_percent: u16, height_percent: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let vertical_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height_percent) / 2),
                Constraint::Percentage(height_percent),
                Constraint::Percentage((100 - height_percent) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let horizontal_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - width_percent) / 2),
                Constraint::Percentage(width_percent),
                Constraint::Percentage((100 - width_percent) / 2),
            ]
            .as_ref(),
        )
        .split(vertical_split[1]);

    horizontal_split[1]
}

fn render_metadata(app: &App) -> Paragraph<'_> {
    let total_lines = (app.file_size + app.bytes_per_line - 1) / app.bytes_per_line;
    let percentage = if app.file_size == 0 {
        0.0
    } else {
        (app.scroll_offset * app.bytes_per_line) as f64 / app.file_size as f64 * 100.0
    };
    let text = format!(
        "File: {} | Size: {} bytes | Offset: {:#08x} | {}/{} lines ({:.2}%)",
        app.file_path,
        app.file_size,
        app.scroll_offset * app.bytes_per_line,
        app.scroll_offset + 1,
        total_lines,
        percentage
    );
    Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Metadata"))
        .style(match app.theme {
            Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
            Theme::Dark => Style::default().fg(Color::White).bg(Color::Black),
        })
}

fn render_input(app: &App) -> Paragraph<'_> {
    let (title, content) = match app.mode {
        AppMode::Search => (
            "Search",
            format!("/{}", app.input_buffer),
        ),
        AppMode::Goto => (
            "Go To Offset",
            format!(":{}", app.input_buffer),
        ),
        _ => (
            "Normal Mode",
            String::from("Press '/' to search, 'x' for Hex search, ':' to go to offset, 't' to toggle theme, 'h' for Help, 'q' to quit"),
        ),
    };
    Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(match app.theme {
            Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
            Theme::Dark => Style::default().fg(Color::White).bg(Color::Black),
        })
}

fn render_content(app: &mut App, visible_height: usize) -> Paragraph<'_> {
    let data = app.get_display_data(visible_height);

    // Handle the case where no data is returned
    if data.is_empty() {
        let empty_message = Paragraph::new("No data to display.")
            .block(Block::default().borders(Borders::ALL).title("Content"))
            .style(match app.theme {
                Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
                Theme::Dark => Style::default().fg(Color::White).bg(Color::Black),
            });
        return empty_message;
    }

    let content = format_hex_dump(
        &data,
        app.scroll_offset,
        visible_height,
        app.bytes_per_line,
        &app.search_results,
    );

    // Handle the case where format_hex_dump returns empty content
    if content.is_empty() {
        let empty_message = Paragraph::new("No data to display.")
            .block(Block::default().borders(Borders::ALL).title("Content"))
            .style(match app.theme {
                Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
                Theme::Dark => Style::default().fg(Color::White).bg(Color::Black),
            });
        return empty_message;
    }

    Paragraph::new(Text::from(content))
        .block(Block::default().borders(Borders::ALL).title("Content"))
        .style(match app.theme {
            Theme::Light => Style::default().fg(Color::Black).bg(Color::White),
            Theme::Dark => Style::default().fg(Color::White).bg(Color::Black),
        })
}
