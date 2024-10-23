// src/parsers/generic.rs

use super::{FileParser, ParsedFile};
use std::fs::File;
use std::io::Read;

/// Generic parser that reads the entire file into memory
pub struct GenericParser;

impl FileParser for GenericParser {
    fn parse(file: &mut File) -> Result<ParsedFile, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(ParsedFile::Generic(buffer))
    }
}
