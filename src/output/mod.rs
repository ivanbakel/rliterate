use parser;
use ast::{LitFile, Section, Block};
use grammar::{BlockModifier};

use clap::{ArgMatches};
use std::collections::{HashMap};
use std::path::{Path, PathBuf};
use std::env;

type OutputResult<T> = Result<T, OutputError>;

enum OutputError {
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
    weave: Option<WeaveType>,
    tangle: Option<TangleSettings>, 
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
            Some(WeaveType::StraightToHtml)
        };

        let tangle = if args.is_present("weave") {
            None
        } else {
            let line_numbers = args.value_of("line_numbers")
                .map(|format_string| generate_line_numbers(format_string));

            Some(TangleSettings {
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

    pub fn process(&self, parse_state: parser::ParseState) -> OutputResult<()> {
        for (path, lit_file) in parse_state.file_map.iter() {
            let linked_file = link_lit_file(lit_file)?;

            if let Some(ref settings) = self.tangle {
                tangle_file(settings, path, linked_file);
             }

            if let Some(ref weave_type) = self.weave {
                weave_file(weave_type, path, linked_file);
            }
        }
    }

}

fn generate_line_numbers(format_string: &str) -> (&'static Fn(usize) -> String) {
    panic!()
}

