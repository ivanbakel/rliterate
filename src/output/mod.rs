use parser;
use ast::{LitFile, Section, Block};
use grammar::{BlockModifier};

pub mod css;
mod tangle;
mod weave;
use super::link;

use clap::{ArgMatches};
use std::path::{Path, PathBuf};
use std::env;
use std::io;

type OutputResult<T> = Result<T, OutputError>;

pub enum OutputError {
    FileSystem(io::Error),
}

impl From<io::Error> for OutputError {
    fn from(io_error: io::Error) -> OutputError {
        OutputError::FileSystem(io_error)
    }
}

pub struct OutputSettings {
    out_dir: PathBuf,
    generate_output: bool,
    weave: Option<weave::Type>,
    tangle: Option<tangle::Settings>, 
}

impl OutputSettings {
    pub fn from_args(args: &ArgMatches<'static>) -> Self {
        let output_dir = args.value_of("output_directory")
            .map_or(
                env::current_dir().unwrap(),
                |odir| Path::new(odir).to_path_buf()
            );

        let weave = if args.is_present("tangle") {
            None
        } else {
            Some(weave::Type::StraightToHtml)
        };

        let tangle = if args.is_present("weave") {
            None
        } else {
            let line_numbers = args.value_of("line_numbers")
                .map(|format_string| generate_line_numbers(format_string));

            Some(tangle::Settings {
                compile: args.is_present("compiler"),
                line_numbers: line_numbers,
            })
        };

        OutputSettings {
            out_dir: output_dir,
            generate_output: args.is_present("no_output"),
            weave: weave,
            tangle: tangle,
        }
    }

    pub fn process<'a>(&self, link_state: link::LinkState<'a>) -> OutputResult<()> {
        for (path, linked_file) in link_state.file_map.iter() {

            if let Some(ref settings) = self.tangle {
                tangle::tangle_file(settings, linked_file)?;
             }

            if let Some(ref weave_type) = self.weave {
                weave::weave_file(weave_type, path, linked_file)?;
            }
        }

        Ok(())
    }

}

fn generate_line_numbers(format_string: &str) -> (&'static Fn(usize) -> String) {
    panic!()
}
