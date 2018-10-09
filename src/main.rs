#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;
extern crate peg;
extern crate pulldown_cmark;
extern crate prettify_cmark;
extern crate subprocess;

mod args;
mod parser;
pub mod link;
mod output;

fn main() -> Result<(), ProgramError> {
    let args : clap::ArgMatches<'static> = args::get_arg_parser().get_matches();

    let input_path = args.value_of("input").unwrap();
    
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

