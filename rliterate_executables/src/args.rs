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

use clap::{App, Arg, ArgGroup, ArgMatches};

use std::path;

pub fn get_main_arg_parser() -> App<'static, 'static> {
    let mut app = App::new("Literate")
        .about("Literate programming tool, based on the original Dlang `literate`.")
        .before_help("Consult the `literate` docs for information about the .lit format.")
        .version(crate_version!())
        .arg(
            Arg::with_name(constants::INPUT)
            .help("The input file or directory. If a directory is given, all the .lit files in it will be processed")
            .index(1)
            .required(true))
        .arg(
            Arg::with_name(constants::OUTPUT_DIRECTORY)
            .help("The directory to write generated files to.")
            .short("odir")
            .long("out-dir")
            .required(false)
            .takes_value(true))
        .arg(
            Arg::with_name(constants::RECURSE)
            .help("Recurse into subdirectories.")
            .short("r")
            .long("recurse")
            .required(false));

    add_common_cli_options(app)
}

pub fn get_cargo_arg_parser() -> App<'static, 'static> {
    let mut app = App::new("cargo lit")
        .about("A cargo subcommand for processing a literate project's `.lit` files and generating source code.")
        .before_help("Consult the help for `literate` for help with using this subcommand.")
        .version(crate_version!());

    add_common_cli_options(app)
}

fn add_common_cli_options(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
            Arg::with_name(constants::NO_OUTPUT)
            .help("Don't produce any output files.")
            .short("no")
            .long("no-output")
            .required(false))
        .arg(
            Arg::with_name(constants::COMPILER)
            .help("Run any compiler commands for linting the code output.")
            .short("c")
            .long("compiler")
            .required(false)
            .conflicts_with(constants::WEAVE))
        .arg(
            Arg::with_name(constants::TANGLE)
            .help("Only produce the code output.")
            .short("t")
            .long("tangle"))
        .arg(
            Arg::with_name(constants::LINE_NUMBERS)
            .help("Set the format string for line numbers in the code output")
            .short("l")
            .long("linenums")
            .required(false)
            .takes_value(true)
            .conflicts_with(constants::WEAVE))
        .arg(
            Arg::with_name(constants::WEAVE)
            .help("Only produce the documentation output.")
            .short("w")
            .long("weave"))
        .arg(Arg::with_name(constants::WEAVE_OUTPUT)
             .help("Set the type of documentation output - valid options are markdown and html.")
             .long("weave-output")
             .required(false)
             .takes_value(true)
             .conflicts_with(constants::TANGLE))
        .arg(Arg::with_name(constants::MD_COMPILER)
             .help("Set the markdown compiler used to generate html output.")
             .long("markdown-compiler")
             .required(false)
             .takes_value(true)
             .conflicts_with(constants::TANGLE))
        .group(
            ArgGroup::with_name(constants::OUTPUT_TYPE)
            .args(&[constants::TANGLE, constants::WEAVE])
            .required(false)
            .multiple(false))
}

pub mod constants {
    pub const INPUT : &'static str = "input";
    pub const NO_OUTPUT : &'static str = "no_output";
    pub const RECURSE : &'static str = "recurse";
    pub const COMPILER : &'static str = "compiler";
    pub const OUTPUT_DIRECTORY : &'static str = "output_directory";
    pub const LINE_NUMBERS : &'static str = "line_numbers";
    pub const TANGLE : &'static str = "tangle";
    pub const WEAVE : &'static str = "weave";
    pub const WEAVE_OUTPUT : &'static str = "weave_output";
    pub const HTML : &'static str = "html";
    pub const MD : &'static str = "md";
    pub const MARKDOWN : &'static str = "markdown";
    pub const MD_COMPILER : &'static str = "md_compiler";
    pub const OUTPUT_TYPE : &'static str = "output_type";
}    

pub fn input_from_args(input_path: &path::Path, args: &ArgMatches<'static>) -> rliterate_core::input::InputSettings {
    rliterate_core::input::InputSettings {
        input_path: input_path.to_owned(),
        recurse: args.is_present(constants::RECURSE),
    }
}
    
pub fn output_from_args(output_dir : &path::Path, args: &ArgMatches<'static>) 
  -> rliterate_core::output::Result<rliterate_core::output::Globals> {
    trace!("Started parsing command-line arguments...");

    let weave = if args.is_present(constants::TANGLE) {
        info!("Setting the tangle-only flag");
        None
    } else {
        let md_compiler = args.value_of(constants::MD_COMPILER)
            .map(|contents| contents.to_owned());

        let weave_type = if let Some(weave_output_type) = args.value_of(constants::WEAVE_OUTPUT) {
            match weave_output_type {
                constants::MARKDOWN | constants::MD => rliterate_core::output::weave::Type::Markdown,
                constants::HTML => rliterate_core::output::weave::Type::HtmlViaMarkdown(md_compiler),
                _ => return Err(rliterate_core::output::Error::BadCLIArgument(format!("Unknown documentation output type: {}", weave_output_type))),
            }
        } else {
            info!("No output type specified, defaulting to HTML");
            rliterate_core::output::weave::Type::HtmlViaMarkdown(md_compiler)
        };

        Some(rliterate_core::output::weave::Globals {
            weave_type: weave_type,
            out_dir: output_dir.to_path_buf(),
            css: rliterate_core::output::css::Globals::default(),
        })
    };

    let tangle = if args.is_present(constants::WEAVE) {
        info!("Setting the weave-only flag");
        None
    } else {
        let line_number_format = args.value_of(constants::LINE_NUMBERS).map(|line_number_format_string| {
            rliterate_core::parser::generate_line_number_format(line_number_format_string)
        });
        Some(rliterate_core::output::tangle::Globals {
            compile: args.is_present(constants::COMPILER),
            line_number_format,
            out_dir: output_dir.to_path_buf(),
        })
    };

    trace!("Finished parsing command-line arguments");
    Ok(rliterate_core::output::Globals {
        generate_output: !args.is_present(constants::NO_OUTPUT),
        weave: weave,
        tangle: tangle,
    })
}

