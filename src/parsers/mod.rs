// src/parsers/mod.rs

pub mod generic;

use crate::utils::read_file_chunk;
use std::fs::File;

/// Trait for parsing different file types
pub trait FileParser {
    fn parse(file: &mut File) -> Result<ParsedFile, Box<dyn std::error::Error>>;
}

/// Enum representing the parsed file content
pub enum ParsedFile {
    Generic(Vec<u8>),
    Lazy(File), // For lazy loading large files
    // Future variants for other file types
}

impl ParsedFile {
    /// Returns a byte slice of the file data
    pub fn data(&self) -> &[u8] {
        match self {
            ParsedFile::Generic(data) => data.as_slice(),
            ParsedFile::Lazy(_) => &[], // For Lazy loading, data is fetched via get_chunk
            // Handle other variants
        }
    }

    /// Retrieves a chunk of data based on the current scroll offset
    pub fn get_chunk(&mut self, offset: usize, bytes_per_line: usize, lines: usize) -> Vec<u8> {
        match self {
            ParsedFile::Generic(data) => {
                let start = offset * bytes_per_line;
                let end = usize::min(start + (bytes_per_line * lines), data.len());
                if start >= data.len() {
                    Vec::new()
                } else {
                    data[start..end].to_vec()
                }
            }
            ParsedFile::Lazy(file) => read_file_chunk(file, offset, bytes_per_line, lines),
            // Handle other variants
        }
    }
}

/// Parses the file and returns a `ParsedFile` instance
pub fn parse_file(path: &str) -> Result<ParsedFile, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    // For now, always use the generic parser
    generic::GenericParser::parse(&mut file)
}
