#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;
extern crate peg;
extern crate pulldown_cmark;

mod args;
mod grammar; 
#[macro_use]
mod ast;
mod parser;
pub mod link;
mod output;

fn main() -> Result<(), ProgramError> {
    let args : clap::ArgMatches<'static> = args::get_arg_parser().get_matches();

    let input_path = args.value_of("input").unwrap();
    
    let parse_state = parser::ParseState::from_input_path(input_path)?;
    
    let linked_state = link::LinkState::link(&parse_state.file_map)?;

    let output_settings = output::OutputSettings::from_args(&args);
    output_settings.process(linked_state)?;

    Ok(())
}

#[derive(Debug)]
enum ProgramError {
    ParserError,
    LinkerError,
    OutputError
}

impl From<parser::ParseError> for ProgramError {
    fn from(err: parser::ParseError) -> Self {
        ProgramError::ParserError
    }
}

impl From<link::LinkError> for ProgramError {
    fn from(err: link::LinkError) -> Self {
        ProgramError::LinkerError
    }
}

impl From<output::OutputError> for ProgramError {
    fn from(err: output::OutputError) -> Self {
        ProgramError::OutputError
    }
}

