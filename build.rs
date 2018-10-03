extern crate peg;

fn main() {
    peg::cargo_build("src/parser/literate.rustpeg");
    peg::cargo_build("src/link_parsing.rustpeg");
}

