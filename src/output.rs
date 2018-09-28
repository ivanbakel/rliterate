use parser;
use ast::{LitFile};

use clap::{ArgMatches};
use std::path::{Path, PathBuf};
use std::env;

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
                line_numbers: line_numbers
            })
        };

        OutputSettings {
            out_dir: output_dir,
            generate_output: args.is_present("no_output"),
            weave: weave,
            tangle: tangle,
        }
    }

    pub fn process(&self, parse_state: parser::ParseState) {
    
    }

    fn weave(&self, file_name: &PathBuf, file: &mut LitFile) {
    }

    fn tangle(&self, file_name: &PathBuf, file: &mut LitFile) {
    }
}

enum WeaveType {
    Markdown,
    HtmlViaMarkdown(String),
    StraightToHtml,
}

struct TangleSettings {
    line_numbers: Option<&'static Fn(usize) -> String>,
}

fn generate_line_numbers(format_string: &str) -> (&'static Fn(usize) -> String) {
    panic!()
}

