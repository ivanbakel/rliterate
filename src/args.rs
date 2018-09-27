use clap::{App, Arg, ArgGroup};

pub fn get_arg_parser() -> App<'static, 'static> {
    App::new("Literate")
        .about("Literate programming tool, based on the original Dlang `literate`.")
        .before_help("Consult the `literate` docs for information about the .lit format.")
        .version(crate_version!())
        .arg(
            Arg::with_name("no_output")
            .short("no")
            .long("no-output")
            .required(false))
        .arg(
            Arg::with_name("compiler")
            .short("c")
            .long("compiler")
            .required(false))
        .arg(
            Arg::with_name("output_directory")
            .short("odir")
            .long("out-dir")
            .required(false)
            .takes_value(true))
        .arg(
            Arg::with_name("line_numbers")
            .short("l")
            .long("linenums")
            .required(false)
            .takes_value(true))
        .arg(
            Arg::with_name("tangle")
            .short("t")
            .long("tangle"))
        .arg(
            Arg::with_name("weave")
            .short("w")
            .long("weave"))
        .group(
            ArgGroup::with_name("output_type")
            .args(&["tangle", "weave"])
            .required(false)
            .multiple(false))
}
