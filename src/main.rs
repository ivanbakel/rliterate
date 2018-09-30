#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;
extern crate peg;

mod args;
mod grammar; 
#[macro_use]
mod ast;
mod parser;
mod link;
mod output;

fn main() {
    let args : clap::ArgMatches<'static> = args::get_arg_parser().get_matches();

    let input_path = args.value_of("input").unwrap();
    
    let parse_state = parser::ParseState::from_input_path(input_path);

    parse_state.map(|state| {
            let output_settings = output::OutputSettings::from_args(&args);

            let linked_state = link::link(state);

            output_settings.process(linked_state)
        });
}

