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

#![feature(proc_macro_hygiene)]
#![feature(use_extern_macros)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;
extern crate peg;
extern crate subprocess;
extern crate pulldown_cmark;
extern crate prettify_cmark;
extern crate maud;

mod args;
mod parser;
pub mod link;
mod output;

fn main() -> Result<(), ProgramError> {
    let args : clap::ArgMatches<'static> = args::get_arg_parser().get_matches();

    let input_path = args.value_of(args::input).unwrap();
    
    let parse_state = parser::ParseState::from_input_path(input_path)?;
    
    let linked_state = link::LinkState::link(&parse_state.file_map)?;

    let mut output_settings = output::OutputSettings::from_args(&args)?;
    if let Some(css_settings) = parse_state.css_settings {
        output_settings.set_css(css_settings);
    }
    output_settings.process(linked_state)?;

    Ok(())
}

#[derive(Debug)]
enum ProgramError {
    ParserError(parser::ParseError),
    LinkerError(link::LinkError),
    OutputError(output::OutputError),
}

impl From<parser::ParseError> for ProgramError {
    fn from(err: parser::ParseError) -> Self {
        ProgramError::ParserError(err)
    }
}

impl From<link::LinkError> for ProgramError {
    fn from(err: link::LinkError) -> Self {
        ProgramError::LinkerError(err)
    }
}

impl From<output::OutputError> for ProgramError {
    fn from(err: output::OutputError) -> Self {
        ProgramError::OutputError(err)
    }
}

