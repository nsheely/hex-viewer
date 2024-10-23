// src/utils.rs

use ratatui::style::{Color, Style};
use ratatui::text::{Span, Line};
use std::ops::Range;
use std::fs::File;
use std::io::{Seek, SeekFrom, Read};

/// Formats the hex dump with color coding and highlights search results.
/// Returns a vector of Lines that can be directly displayed in the Paragraph widget.
pub fn format_hex_dump(
    data: &[u8],
    scroll_offset: usize,
    lines: usize,
    bytes_per_line: usize,
    search_results: &Vec<Range<usize>>,
) -> Vec<Line<'static>> {
    let mut output = Vec::new();
    let start_addr = scroll_offset * bytes_per_line;

    for (i, chunk) in data.chunks(bytes_per_line).enumerate().take(lines) {
        let addr = start_addr + i * bytes_per_line;
        let mut spans = Vec::new();

        // Address
        spans.push(Span::styled(
            format!("{:08x}: ", addr),
            Style::default().fg(Color::Blue),
        ));

        // Hexadecimal representation
        for (j, byte) in chunk.iter().enumerate() {
            let global_index = addr + j;
            let is_match = search_results.iter().any(|range| range.contains(&global_index));
            let style = if is_match {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else {
                Style::default().fg(Color::Cyan)
            };
            spans.push(Span::styled(format!("{:02x} ", byte), style));
        }

        // Padding for incomplete lines
        if chunk.len() < bytes_per_line {
            spans.push(Span::raw("   ".repeat(bytes_per_line - chunk.len())));
        }

        spans.push(Span::raw("  "));

        // ASCII representation
        for (j, byte) in chunk.iter().enumerate() {
            let global_index = addr + j;
            let is_match = search_results.iter().any(|range| range.contains(&global_index));
            let display_char = byte_to_displayable(*byte);
            let style = if is_match {
                Style::default().bg(Color::Yellow).fg(Color::Black)
            } else if display_char == '.' {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Green)
            };
            spans.push(Span::styled(display_char.to_string(), style));
        }

        output.push(Line::from(spans));
    }

    output
}

/// Converts a byte to a displayable character.
/// Printable ASCII characters are displayed as-is, others are represented by a dot.
fn byte_to_displayable(byte: u8) -> char {
    if byte.is_ascii_graphic() || byte == b' ' {
        byte as char
    } else {
        '.'
    }
}

/// Reads the file in chunks for lazy loading.
/// Returns an empty vector if seeking fails or no bytes are read.
pub fn read_file_chunk(file: &mut File, offset: usize, bytes_per_line: usize, lines: usize) -> Vec<u8> {
    let mut buffer = vec![0; bytes_per_line * lines];
    let seek_position = (offset * bytes_per_line) as u64;
    if let Err(e) = file.seek(SeekFrom::Start(seek_position)) {
        eprintln!("Error seeking to position {:#x}: {}", seek_position, e);
        return Vec::new();
    }
    match file.read(&mut buffer) {
        Ok(bytes_read) => {
            buffer.truncate(bytes_read);
            buffer
        }
        Err(e) => {
            eprintln!("Error reading from file: {}", e);
            Vec::new()
        }
    }
}
