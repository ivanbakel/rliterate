/*
 * Copyright (c) 2018 Isaac van Bakel
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#[macro_use]
mod ast;
pub use self::ast::{LitFile, Section, Block, CompilerSettings};
mod grammar;
pub use self::grammar::{BlockModifier};

use input;
use output::css::{CssSettings};

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingCommand(&'static str),
    DuplicateCommand,
    DuplicateCssCommand,
    ConflictingCss,
    FileSystem(io::Error),
    FileLoop,
    FileRepeat,
    GrammarError(grammar::ParseError),
    FormatError,
}

impl From<grammar::ParseError> for Error {
    fn from(err: grammar::ParseError) -> Error {
        Error::GrammarError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::FileSystem(err)
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

    pub fn from_input(input_settings: input::InputSettings) -> Result<Self> {
        trace!("Loading files from input path \"{}\"", input_settings.input_path.to_string_lossy());
        let mut parse_state = ParseState::new();  

        for file in get_input_files(&input_settings.input_path, input_settings.recurse)?.into_iter() {
            parse_state.parse_file(&file)?;
        }

        Ok(parse_state)
    }

    pub fn parse_file(&mut self, file_path: &PathBuf) -> Result<()> {
        if self.in_progress.contains(file_path) {
            error!("Found an include loop when trying to load file \"{}\"", file_path.to_string_lossy());
            Err(Error::FileLoop)
        } else if self.file_map.contains_key(file_path) {
            error!("Ended up trying to parse file \"{}\" twice", file_path.to_string_lossy());
            Err(Error::FileRepeat)
        } else { 
            trace!("Parsing file \"{}\"", file_path.to_string_lossy());
            let file_contents = fs::read_to_string(file_path)?; 
            let lit_blocks = grammar::lit_file(&file_contents)?;
            
            self.in_progress.insert(file_path.clone());

            let (lit_file, settings) = LitFile::parse(self, lit_blocks)?;

            if !settings.is_default() {
                once!(self.css_settings, is_some, Some(settings), Error::ConflictingCss);
            }

            self.in_progress.remove(file_path);
            info!("Finished parsing \"{}\"", file_path.to_string_lossy());
            self.file_map.insert(file_path.clone(), lit_file);
            Ok(())
        }
    }
}

fn get_input_files(input_path: &Path, recurse: bool) -> Result<Vec<PathBuf>> {
    let path_buf = input_path.to_path_buf();

    if path_buf.is_file() {
        info!("\"{}\" is a file, taking it as the only input", input_path.to_string_lossy());
        Ok(vec![path_buf])
    } else if path_buf.is_dir() {
        trace!("\"{}\" is a directory, traversing it...", input_path.to_string_lossy());
        let files = fs::read_dir(path_buf)?;

        let mut paths = Vec::new();

        for entry in files {
            let dir_entry = entry?;

            let entry_path = dir_entry.path();

            if entry_path.is_file() && entry_path.extension().map_or(false, |extension| extension == "lit") {
                info!("Adding \"{}\" as an input file", entry_path.to_string_lossy());
                paths.push(entry_path);
            } else if entry_path.is_dir() && recurse {
                info!("Recursing into \"{}\" as an input directory", entry_path.to_string_lossy());
                paths.append(&mut get_input_files(&entry_path, recurse)?);
            }
        }
        trace!("Finished traversing \"{}\" for input files", input_path.to_string_lossy());

        Ok(paths)
    } else {
        Err(Error::FileSystem(io::Error::new(io::ErrorKind::Other, format!("Could not process input path: {}", input_path.to_string_lossy()))))
    }
}

pub fn get_input_file(input_path: &Path) -> Result<PathBuf> {
    let path_buf = input_path.to_path_buf();

    if path_buf.is_file() {
        Ok(path_buf)
    } else {
        Err(Error::FileSystem(io::Error::new(io::ErrorKind::Other, format!("Input path was not a file: {}", input_path.to_string_lossy()))))
    }
}
