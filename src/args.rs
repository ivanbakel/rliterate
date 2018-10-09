use clap::{App, Arg, ArgGroup};

pub fn get_arg_parser() -> App<'static, 'static> {
    App::new("Literate")
        .about("Literate programming tool, based on the original Dlang `literate`.")
        .before_help("Consult the `literate` docs for information about the .lit format.")
        .version(crate_version!())
        .arg(
            Arg::with_name("input")
            .index(1)
            .required(true))
        .arg(
            Arg::with_name("no_output")
            .help("Don't produce any output files.")
            .short("no")
            .long("no-output")
            .required(false))
        .arg(
            Arg::with_name("compiler")
            .help("Run any compiler commands for linting the code output.")
            .short("c")
            .long("compiler")
            .required(false)
            .conflicts_with("weave"))
        .arg(
            Arg::with_name("output_directory")
            .help("The directory to write generated files to.")
            .short("odir")
            .long("out-dir")
            .required(false)
            .takes_value(true))
        .arg(
            Arg::with_name("line_numbers")
            .help("Set the format string for line numbers in the code output")
            .short("l")
            .long("linenums")
            .required(false)
            .takes_value(true))
        .arg(
            Arg::with_name("tangle")
            .help("Only produce the code output.")
            .short("t")
            .long("tangle"))
        .arg(
            Arg::with_name("weave")
            .help("Only produce the documentation output.")
            .short("w")
            .long("weave"))
        .arg(Arg::with_name("weave_output")
             .help("Set the type of documentation output - valid options are markdown and html.")
             .long("weave-output")
             .required(false)
             .conflicts_with("tangle"))
        .arg(Arg::with_name("md_compiler")
             .help("Set the markdown compiler used to generate html output.")
             .long("markdown-compiler")
             .required(false)
             .conflicts_with("tangle"))
        .group(
            ArgGroup::with_name("output_type")
            .args(&["tangle", "weave"])
            .required(false)
            .multiple(false))
}
