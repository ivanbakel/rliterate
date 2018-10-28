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

extern crate env_logger;

use std::env;
use std::path;

use rliterate_core::{run};
use rliterate_runtime::args;

fn main() -> rliterate_core::Result<()> {
    env_logger::init();

    let args : clap::ArgMatches<'static> = args::get_main_arg_parser().get_matches();

    let input_path = path::Path::new(args.value_of(args::constants::input).unwrap());
    let output_path = args.value_of(args::constants::output_directory)
            .map_or(
                env::current_dir().unwrap(),
                |out_dir| path::Path::new(out_dir).to_path_buf()
            );

    let input_settings = args::input_from_args(input_path, &args);
    let output_settings = args::output_from_args(&output_path, &args)?;

    run(input_settings, output_settings)
}
