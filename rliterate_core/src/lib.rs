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

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;
extern crate peg;
extern crate subprocess;
extern crate pulldown_cmark;
extern crate prettify_cmark;
extern crate maud;

pub mod parser;
pub mod link;
pub mod input;
pub mod output;

pub type Result<T> = std::result::Result<T, Error>;

pub fn run(input_settings: input::InputSettings, mut output_settings: output::Globals) -> Result<()> {
    let parse_state = parser::ParseState::from_input(input_settings)?;
    
    let linked_state = link::LinkState::link(&parse_state.file_map)?;
    if let Some(css_settings) = parse_state.css_settings {
        info!("Loaded css settings \"{}\" from file commands.", css_settings);
        output_settings.set_css(css_settings);
    }
    output_settings.process(linked_state)?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Parser(parser::Error),
    Linker(link::Error),
    Output(output::Error),
    Other(String),
}

impl From<parser::Error> for Error {
    fn from(err: parser::Error) -> Self {
        Error::Parser(err)
    }
}

impl From<link::Error> for Error {
    fn from(err: link::Error) -> Self {
        Error::Linker(err)
    }
}

impl From<output::Error> for Error {
    fn from(err: output::Error) -> Self {
        Error::Output(err)
    }
}

