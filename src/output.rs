use parser;
use ast::{LitFile};

use clap::{ArgMatches};
use std::path::{Path, PathBuf};
use std::env;

pub struct CssSettings {
    pub custom_css: CustomCss,
    pub custom_colorscheme: Option<String>,
}

impl CssSettings {
    pub fn is_default(&self) -> bool {
        !self.custom_css.is_not_none() && self.custom_colorscheme.is_none()
    }
}

pub enum CustomCss {
    None,
    Add(String),
    Overwrite(String),
}

impl CustomCss {
    pub fn is_not_none(&self) -> bool {
        match self {
            &CustomCss::None => false,
            _ => true
        }
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

    pub fn process(&self, parse_state: parser::ParseState) {
        for (path, lit_file) in parse_state.file_map.iter() {
            if let Some(ref settings) = self.tangle {
                tangle_file(settings, path, lit_file);
            }

            if let Some(ref weave_type) = self.weave {
                weave_file(weave_type, path, lit_file);
            }
        }
    }

}

enum WeaveType {
    Markdown,
    HtmlViaMarkdown(String),
    StraightToHtml,
}

fn weave_file(weave_type: &WeaveType, file_name: &PathBuf, file: &LitFile) {
}


struct TangleSettings {
    compile: bool,
    line_numbers: Option<&'static Fn(usize) -> String>,
}
    
fn tangle_file(settings: &TangleSettings, file_name: &PathBuf, file: &LitFile) {
}

fn generate_line_numbers(format_string: &str) -> (&'static Fn(usize) -> String) {
    panic!()
}

