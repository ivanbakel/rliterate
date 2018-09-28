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

fn main() {
    let args : clap::ArgMatches<'static> = args::get_arg_parser().get_matches();

    //TODO: get the source files
    let input_path = args.value_of("input").unwrap();
    //TODO: parse the source files
    //TODO: construct the AST
    
    let parse_state = parser::ParseState::from_input_path(input_path);

    //TODO: link blocks together
    //TODO: print it all somewhere
}

