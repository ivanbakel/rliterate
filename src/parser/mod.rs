#[macro_use]
mod ast;
pub use self::ast::{LitFile, Section, Block};
mod grammar;
pub use self::grammar::{BlockModifier};

use output::css::{CssSettings};

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    MissingCommand(&'static str),
    DuplicateCommand,
    DuplicateCssCommand,
    ConflictingCss,
    FileSystem,
    FileLoop,
    FileRepeat,
    GrammarError(grammar::ParseError),
}

impl From<grammar::ParseError> for ParseError {
    fn from(err: grammar::ParseError) -> ParseError {
        ParseError::GrammarError(err)
    }
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> ParseError {
        ParseError::FileSystem
    }
}

pub struct ParseState {    
    in_progress: HashSet<PathBuf>,
    pub file_map: FileMap,
    pub css_settings: Option<CssSettings>,
}

pub type FileMap = HashMap<PathBuf, LitFile>;

impl ParseState {
    pub fn new() -> Self {
        ParseState {
            in_progress: HashSet::new(),
            file_map: HashMap::new(),
            css_settings: None,
        }
    }

    pub fn from_input_path(input_path: &str) -> ParseResult<Self> {
        let mut parse_state = ParseState::new();  

        for file in get_input_files(input_path)?.into_iter() {
            parse_state.parse_file(&file)?;
        }

        Ok(parse_state)
    }

    pub fn parse_file(&mut self, file_path: &PathBuf) -> ParseResult<()> {
        if self.in_progress.contains(file_path) {
            Err(ParseError::FileLoop)
        } else if self.file_map.contains_key(file_path) {
            Err(ParseError::FileRepeat)
        } else {
            
            let file_contents = fs::read_to_string(file_path)?; 
            let lit_blocks = grammar::lit_file(&file_contents)?;
            
            self.in_progress.insert(file_path.clone());

            let (lit_file, settings) = LitFile::parse(self, lit_blocks)?;

            once!(self.css_settings, is_some, Some(settings), ParseError::ConflictingCss);

            self.in_progress.remove(file_path);
            self.file_map.insert(file_path.clone(), lit_file);
            Ok(())
        }
    }
}

fn get_input_files(input_path: &str) -> ParseResult<Vec<PathBuf>> {
    let path_buf = Path::new(input_path).to_path_buf();

    if path_buf.is_file() {
        Ok(vec![path_buf])
    } else if path_buf.is_dir() {
        let files = fs::read_dir(path_buf).
            map_err(|_| ParseError::FileSystem)?;

        let mut paths = Vec::new();

        for entry in files {
            let dir_entry = entry.map_err(|_| ParseError::FileSystem)?;

            let entry_path = dir_entry.path();

            if entry_path.is_file() && entry_path.extension().map_or(false, |extension| extension == "lit") {
                paths.push(entry_path);
            }
        }

        Ok(paths)
    } else {
        Err(ParseError::FileSystem)
    }
}

pub fn get_input_file(input_path: &str) -> ParseResult<PathBuf> {
    let path_buf = Path::new(input_path).to_path_buf();

    if path_buf.is_file() {
        Ok(path_buf)
    } else {
        Err(ParseError::FileSystem)
    }
}