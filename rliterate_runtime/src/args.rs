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
            Arg::with_name(constants::input)
            .help("The input file or directory. If a directory is given, all the .lit files in it will be processed")
            .index(1)
            .required(true))
        .arg(
            Arg::with_name(constants::output_directory)
            .help("The directory to write generated files to.")
            .short("odir")
            .long("out-dir")
            .required(false)
            .takes_value(true))
        .arg(
            Arg::with_name(constants::recurse)
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
            Arg::with_name(constants::no_output)
            .help("Don't produce any output files.")
            .short("no")
            .long("no-output")
            .required(false))
        .arg(
            Arg::with_name(constants::compiler)
            .help("Run any compiler commands for linting the code output.")
            .short("c")
            .long("compiler")
            .required(false)
            .conflicts_with(constants::weave))
        .arg(
            Arg::with_name(constants::tangle)
            .help("Only produce the code output.")
            .short("t")
            .long("tangle"))
        .arg(
            Arg::with_name(constants::weave)
            .help("Only produce the documentation output.")
            .short("w")
            .long("weave"))
        .arg(Arg::with_name(constants::weave_output)
             .help("Set the type of documentation output - valid options are markdown and html.")
             .long("weave-output")
             .required(false)
             .takes_value(true)
             .conflicts_with(constants::tangle))
        .arg(Arg::with_name(constants::md_compiler)
             .help("Set the markdown compiler used to generate html output.")
             .long("markdown-compiler")
             .required(false)
             .takes_value(true)
             .conflicts_with(constants::tangle))
        .group(
            ArgGroup::with_name(constants::output_type)
            .args(&[constants::tangle, constants::weave])
            .required(false)
            .multiple(false))
}

pub mod constants {
    pub const input : &'static str = "input";
    pub const no_output : &'static str = "no_output";
    pub const recurse : &'static str = "recurse";
    pub const compiler : &'static str = "compiler";
    pub const output_directory : &'static str = "output_directory";
    pub const line_numbers : &'static str = "line_numbers";
    pub const tangle : &'static str = "tangle";
    pub const weave : &'static str = "weave";
    pub const weave_output : &'static str = "weave_output";
    pub const html : &'static str = "html";
    pub const md : &'static str = "md";
    pub const markdown : &'static str = "markdown";
    pub const md_compiler : &'static str = "md_compiler";
    pub const output_type : &'static str = "output_type";
}    

pub fn input_from_args(input_path: &path::Path, args: &ArgMatches<'static>) -> rliterate_core::input::InputSettings {
    rliterate_core::input::InputSettings {
        input_path: input_path.to_owned(),
        recurse: args.is_present(constants::recurse),
    }
}
    
pub fn output_from_args(output_dir : &path::Path, args: &ArgMatches<'static>) 
  -> rliterate_core::output::Result<rliterate_core::output::Globals> {
    trace!("Started parsing command-line arguments...");

    let weave = if args.is_present(constants::tangle) {
        info!("Setting the tangle-only flag");
        None
    } else {
        let md_compiler = args.value_of(constants::md_compiler)
            .map(|contents| contents.to_owned());

        let weave_type = if let Some(weave_output_type) = args.value_of(constants::weave_output) {
            match weave_output_type {
                constants::markdown | constants::md => rliterate_core::output::weave::Type::Markdown,
                constants::html => rliterate_core::output::weave::Type::HtmlViaMarkdown(md_compiler),
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

    let tangle = if args.is_present(constants::weave) {
        info!("Setting the weave-only flag");
        None
    } else {
        Some(rliterate_core::output::tangle::Globals {
            compile: args.is_present(constants::compiler),
            out_dir: output_dir.to_path_buf(),
        })
    };

    trace!("Finished parsing command-line arguments");
    Ok(rliterate_core::output::Globals {
        generate_output: !args.is_present(constants::no_output),
        weave: weave,
        tangle: tangle,
    })
}

fn generate_line_numbers(format_string: &str) -> (&'static Fn(usize) -> String) {
    panic!()
}

