#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;
extern crate peg;

mod args;
mod grammar {
    include!(concat!(env!("OUT_DIR"), "/literate.rs"));
}
mod parser;

fn main() {
    println!("Hello, world!");
}
