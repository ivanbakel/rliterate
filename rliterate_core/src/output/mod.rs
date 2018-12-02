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

pub mod css;
mod canon;
pub mod tangle;
pub mod weave;

use super::link;

use subprocess::{PopenError};
use std::path::{Path, PathBuf};
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    BadCLIArgument(String),
    FileSystem(io::Error),
    BadCommand(PopenError),
    FailedCommand(u32),
    TerminatedCommand(u8),
    NoCompilerCommand,
    FailedCompiler(u32),
    TerminatedCompiler(u8),
}

impl From<io::Error> for Error {
    fn from(io_error: io::Error) -> Error {
        Error::FileSystem(io_error)
    }
}

pub struct Settings {
    pub out_dir: PathBuf,
    pub generate_output: bool,
    pub weave: Option<weave::Settings>,
    pub tangle: Option<tangle::Settings>, 
}

impl Settings {
    pub fn set_css(&mut self, settings: css::Settings) {
        if let Some(ref mut weave) = self.weave {
            weave.css = settings;
        }
    }

    pub fn process<'a>(&self, link_state: link::LinkState<'a>) -> Result<()> {
        trace!("Started outputting files...");
        for (path, linked_file) in link_state.file_map.iter() {
            let canonical_code_blocks = canon::canonicalise_code_blocks(&linked_file.sections[..]);

            if self.generate_output {
                trace!("Generating output for \"{}\"...", path.to_string_lossy());
                if let Some(ref settings) = self.tangle {
                    tangle::tangle_blocks(settings, &canonical_code_blocks, &self.out_dir, &linked_file.compiler)?;
                }

                if let Some(ref weave_type) = self.weave {
                    weave::weave_file_with_blocks(weave_type, path, linked_file, &canonical_code_blocks, &self.out_dir)?;
                }
                trace!("Finished generating output for \"{}\"", path.to_string_lossy());
            }
        }

        trace!("Finished outputting files");
        Ok(())
    }

}
