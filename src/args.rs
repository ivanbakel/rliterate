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

use clap::{App, Arg, ArgGroup};

pub fn get_main_arg_parser() -> App<'static, 'static> {
    let mut app = App::new("Literate")
        .about("Literate programming tool, based on the original Dlang `literate`.")
        .before_help("Consult the `literate` docs for information about the .lit format.")
        .version(crate_version!())
        .arg(
            Arg::with_name(input)
            .help("The input file or directory. If a directory is given, all the .lit files in it will be processed")
            .index(1)
            .required(true))
        .arg(
            Arg::with_name(output_directory)
            .help("The directory to write generated files to.")
            .short("odir")
            .long("out-dir")
            .required(false)
            .takes_value(true));

    add_common_cli_options(app)
}

fn add_common_cli_options(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
            Arg::with_name(no_output)
            .help("Don't produce any output files.")
            .short("no")
            .long("no-output")
            .required(false))
        .arg(
            Arg::with_name(compiler)
            .help("Run any compiler commands for linting the code output.")
            .short("c")
            .long("compiler")
            .required(false)
            .conflicts_with(weave))
        .arg(
            Arg::with_name(line_numbers)
            .help("Set the format string for line numbers in the code output")
            .short("l")
            .long("linenums")
            .required(false)
            .takes_value(true))
        .arg(
            Arg::with_name(tangle)
            .help("Only produce the code output.")
            .short("t")
            .long("tangle"))
        .arg(
            Arg::with_name(weave)
            .help("Only produce the documentation output.")
            .short("w")
            .long("weave"))
        .arg(Arg::with_name(weave_output)
             .help("Set the type of documentation output - valid options are markdown and html.")
             .long("weave-output")
             .required(false)
             .takes_value(true)
             .conflicts_with(tangle))
        .arg(Arg::with_name(md_compiler)
             .help("Set the markdown compiler used to generate html output.")
             .long("markdown-compiler")
             .required(false)
             .takes_value(true)
             .conflicts_with(tangle))
        .group(
            ArgGroup::with_name(output_type)
            .args(&[tangle, weave])
            .required(false)
            .multiple(false))
}

pub const input : &'static str = "input";
pub const no_output : &'static str = "no_output";
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
