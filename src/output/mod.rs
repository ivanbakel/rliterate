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
mod tangle;
mod weave;

use super::link;
use super::args;

use clap::{ArgMatches};
use subprocess::{PopenError};
use std::path::{Path, PathBuf};
use std::io;

type OutputResult<T> = Result<T, OutputError>;

#[derive(Debug)]
pub enum OutputError {
    BadCLIArgument(String),
    FileSystem(io::Error),
    BadCommand(PopenError),
    FailedCommand(u32),
    TerminatedCommand(u8),
    NoCompilerCommand,
    FailedCompiler(u32),
    TerminatedCompiler(u8),
}

impl From<io::Error> for OutputError {
    fn from(io_error: io::Error) -> OutputError {
        OutputError::FileSystem(io_error)
    }
}

pub struct OutputSettings {
    out_dir: PathBuf,
    generate_output: bool,
    weave: Option<weave::Settings>,
    tangle: Option<tangle::Settings>, 
}

impl OutputSettings {
    pub fn from_args(output_dir : &Path, args: &ArgMatches<'static>) -> OutputResult<Self> {
        trace!("Started parsing command-line arguments...");

        let weave = if args.is_present(args::tangle) {
            info!("Setting the tangle-only flag");
            None
        } else {
            let md_compiler = args.value_of(args::md_compiler)
                .map(|contents| contents.to_owned());

            let weave_type = if let Some(weave_output_type) = args.value_of(args::weave_output) {
                match weave_output_type {
                    args::markdown | args::md => weave::Type::Markdown,
                    args::html => weave::Type::HtmlViaMarkdown(md_compiler),
                    _ => return Err(OutputError::BadCLIArgument(format!("Unknown documentation output type: {}", weave_output_type))),
                }
            } else {
                info!("No output type specified, defaulting to HTML");
                weave::Type::HtmlViaMarkdown(md_compiler)
            };

            Some(weave::Settings {
                weave_type: weave_type,
                css: css::CssSettings::default(),
            })
        };

        let tangle = if args.is_present(args::weave) {
            info!("Setting the weave-only flag");
            None
        } else {
            let line_numbers = args.value_of(args::line_numbers)
                .map(|format_string| generate_line_numbers(format_string));

            Some(tangle::Settings {
                compile: args.is_present(args::compiler),
                line_numbers: line_numbers,
            })
        };

        trace!("Finished parsing command-line arguments");
        Ok(OutputSettings {
            out_dir: output_dir.to_path_buf(),
            generate_output: args.is_present(args::no_output),
            weave: weave,
            tangle: tangle,
        })
    }

    pub fn set_css(&mut self, settings: css::CssSettings) {
        if let Some(ref mut weave) = self.weave {
            weave.css = settings;
        }
    }

    pub fn process<'a>(&self, link_state: link::LinkState<'a>) -> OutputResult<()> {
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

fn generate_line_numbers(format_string: &str) -> (&'static Fn(usize) -> String) {
    panic!()
}

