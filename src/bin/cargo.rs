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

extern crate cargo_metadata;
extern crate env_logger;

extern crate rliterate;

use std::env;
use std::path::{Path};

use rliterate::args;
use rliterate::output;
use rliterate::{ProgramError};

fn main() -> Result<(), ProgramError> {
    env_logger::init();

    let manifest_path = env::current_dir().ok().map(|path| path.join("Cargo.toml"));

    let metadata = cargo_metadata::metadata(manifest_path.as_ref().map(Path::new))
            .map_err(|err| ProgramError::Other(err.description().to_owned()))?;

    let lit_folder = Path::new(&metadata.workspace_root).join("lit");
    let src_folder = Path::new(&metadata.workspace_root).join("src");

    let args : clap::ArgMatches<'static> = args::get_main_arg_parser().get_matches();

    let output_settings = output::OutputSettings::from_args(&src_folder, &args)?;
    
    rliterate::run(&lit_folder, output_settings)
}

