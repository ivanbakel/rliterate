extern crate peg;

fn main() {
    peg::cargo_build("src/literate.rustpeg");
    peg::cargo_build("src/link_parsing.rustpeg");
}

