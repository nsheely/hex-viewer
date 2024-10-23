// src/app.rs

use crate::parsers::{parse_file, ParsedFile};
use hex;
use std::error::Error;
use std::fs::File;
use twoway::find_bytes;
use std::ops::Range;

/// Application modes
pub enum AppMode {
    Normal,
    Search,
    Goto,
    Help,
}

/// Types of searches
pub enum SearchType {
    Ascii,
    Hex,
}

/// Available themes
pub enum Theme {
    Light,
    Dark,
}

/// Application state
pub struct App {
    pub running: bool,
    pub file_path: String,
    pub parsed_file: ParsedFile, // Either Generic(Vec<u8>) or Lazy(File)
    pub scroll_offset: usize,
    pub bytes_per_line: usize,
    pub mode: AppMode,
    pub input_buffer: String,
    pub search_results: Vec<Range<usize>>, // Changed to store ranges
    pub search_type: SearchType,
    pub file_size: usize,
    pub theme: Theme,
    pub message: Option<String>, // New field for temporary messages
}

impl App {
    /// Initializes a new App instance
    pub fn new(file_path: String, bytes_per_line: usize, theme: Theme) -> Result<Self, Box<dyn Error>> {
        let metadata = std::fs::metadata(&file_path)?;
        let file_size = metadata.len() as usize;

        // Define a threshold for lazy loading (e.g., 10 MB)
        let threshold = 10 * 1024 * 1024;

        let parsed_file = if file_size > threshold {
            ParsedFile::Lazy(File::open(&file_path)?)
        } else {
            parse_file(&file_path)? // parse_file already returns ParsedFile
        };

        Ok(Self {
            running: true,
            file_path,
            parsed_file,
            scroll_offset: 0,
            bytes_per_line,
            mode: AppMode::Normal,
            input_buffer: String::new(),
            search_results: Vec::new(),
            search_type: SearchType::Ascii,
            file_size,
            theme,
            message: None, // Initialize message as None
        })
    }

    /// Scrolls up by one line
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scrolls down by one line
    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.max_scroll_offset() {
            self.scroll_offset += 1;
        }
    }

    /// Calculates the maximum scroll offset based on file size and bytes per line
    pub fn max_scroll_offset(&self) -> usize {
        let total_lines = (self.file_size + self.bytes_per_line - 1) / self.bytes_per_line;
        if total_lines == 0 {
            0
        } else {
            total_lines.saturating_sub(1)
        }
    }

    /// Ensures that scroll_offset is within valid bounds
    pub fn clamp_scroll_offset(&mut self) {
        let max_offset = self.max_scroll_offset();
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    /// Performs search based on the current search type and input buffer
    pub fn perform_search(&mut self) {
        self.search_results.clear();
        if self.input_buffer.is_empty() {
            self.message = Some("Search query cannot be empty.".to_string());
            return;
        }
        match self.search_type {
            SearchType::Ascii => {
                let query = self.input_buffer.clone();
                let query_bytes = query.as_bytes();
                let data = self.parsed_file.data();

                // Use twoway for efficient searching
                let mut pos = 0;
                while pos + query_bytes.len() <= data.len() {
                    if let Some(idx) = find_bytes(&data[pos..], query_bytes) {
                        let absolute_start = pos + idx;
                        let absolute_end = absolute_start + query_bytes.len();
                        self.search_results.push(absolute_start..absolute_end);
                        pos = absolute_end;
                    } else {
                        break;
                    }
                }
            }
            SearchType::Hex => {
                let query = self.input_buffer.replace(" ", "");
                if query.is_empty() {
                    self.message = Some("Hex search query cannot be empty.".to_string());
                    return;
                }
                match hex::decode(&query) {
                    Ok(query_bytes) => {
                        let data = self.parsed_file.data();

                        // Use twoway for efficient searching
                        let mut pos = 0;
                        while pos + query_bytes.len() <= data.len() {
                            if let Some(idx) = find_bytes(&data[pos..], &query_bytes) {
                                let absolute_start = pos + idx;
                                let absolute_end = absolute_start + query_bytes.len();
                                self.search_results.push(absolute_start..absolute_end);
                                pos = absolute_end;
                            } else {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        self.message = Some("Invalid hexadecimal input for search.".to_string());
                    }
                }
            }
        }

        // Provide feedback if no matches are found
        if self.search_results.is_empty() {
            self.message = Some("No matches found for the search query.".to_string());
        }
    }

    /// Jumps to a specific offset provided by the user
    pub fn jump_to_offset(&mut self) {
        if let Ok(offset) = usize::from_str_radix(&self.input_buffer, 16) {
            let max_offset = self.max_scroll_offset();
            let target_line = offset / self.bytes_per_line;
            self.scroll_offset = usize::min(target_line, max_offset);
        } else {
            self.message = Some("Invalid hexadecimal offset input.".to_string());
        }
    }

    /// Toggles between Light and Dark themes
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    }

    /// Retrieves the data to display based on the current scroll offset and visible height
    pub fn get_display_data(&mut self, visible_height: usize) -> Vec<u8> {
        self.parsed_file.get_chunk(self.scroll_offset, self.bytes_per_line, visible_height)
    }
}
