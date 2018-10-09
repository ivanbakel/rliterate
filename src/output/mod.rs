pub mod css;
mod canon;
mod tangle;
mod weave;

use super::link;
use super::args;

use clap::{ArgMatches};
use subprocess::{PopenError};
use std::path::{Path, PathBuf};
use std::env;
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
    pub fn from_args(args: &ArgMatches<'static>) -> OutputResult<Self> {
        let output_dir = args.value_of(args::output_directory)
            .map_or(
                env::current_dir().unwrap(),
                |odir| Path::new(odir).to_path_buf()
            );

        let weave = if args.is_present(args::tangle) {
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
                weave::Type::HtmlViaMarkdown(md_compiler)
            };

            Some(weave::Settings {
                weave_type: weave_type,
                css: css::CssSettings::default(),
            })
        };

        let tangle = if args.is_present(args::weave) {
            None
        } else {
            let line_numbers = args.value_of(args::line_numbers)
                .map(|format_string| generate_line_numbers(format_string));

            Some(tangle::Settings {
                compile: args.is_present(args::compiler),
                line_numbers: line_numbers,
            })
        };

        Ok(OutputSettings {
            out_dir: output_dir,
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
        for (path, linked_file) in link_state.file_map.iter() {
            let canonical_code_blocks = canon::canonicalise_code_blocks(&linked_file.sections[..]);

            if self.generate_output {
                if let Some(ref settings) = self.tangle {
                    tangle::tangle_blocks(settings, &canonical_code_blocks, &self.out_dir, &linked_file.compiler)?;
                }

                if let Some(ref weave_type) = self.weave {
                    weave::weave_file_with_blocks(weave_type, path, linked_file, &canonical_code_blocks, &self.out_dir)?;
                }
            }
        }

        Ok(())
    }

}

fn generate_line_numbers(format_string: &str) -> (&'static Fn(usize) -> String) {
    panic!()
}

