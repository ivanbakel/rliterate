mod markdown;
use self::markdown::{MarkDown};
mod html;

use output::{OutputResult, OutputError};
use output::css;
use output::canon::{BlockMap};
use link::{LinkedFile, LinkedBlock, LinkedLine};

use pulldown_cmark as cmark;
use prettify_cmark;
use subprocess;

use std::path::{PathBuf, Path};
use std::fs;
use std::io::{Write};

pub struct Settings {
    pub weave_type: Type,
    pub css: css::CssSettings,
}

pub enum Type {
    Markdown,
    HtmlViaMarkdown(Option<String>),
}

pub fn weave_file_with_blocks<'a>(settings: &Settings, file_name: &PathBuf, file: &LinkedFile<'a>, block_map: &BlockMap, out_dir: &Path) -> OutputResult<()> {
    match settings.weave_type {
        Type::HtmlViaMarkdown(ref maybe_command) => {
            let markdown = MarkDown::build(settings, file, block_map);

            let mut html_filename = out_dir.join(file_name.file_stem().unwrap());
            html_filename.set_extension("html");

            let html_file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(html_filename)?;

            let compiled_markdown = if let Some(ref command) = maybe_command {
                call_markdown_compiler(command, markdown)
            } else {
                compile_markdown(markdown)
            }?;

            html::print(html_file, compiled_markdown, &settings.css)?;

            Ok(())
        },
        Type::Markdown => {
            let markdown = MarkDown::build(settings, file, block_map);

            let mut md_filename = out_dir.join(file_name.file_stem().unwrap());
            md_filename.set_extension("md");

            let md_file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(md_filename)?;

            print_markdown(md_file, markdown)?;

            Ok(())
        }
    }
}

impl From<subprocess::PopenError> for OutputError {
    fn from(err: subprocess::PopenError) -> OutputError {
        OutputError::BadCommand(err)
    }
}

fn call_markdown_compiler<'m>(command: &str, markdown: MarkDown<'m>) -> OutputResult<String> {
    // Setup the markdown to be fed into the command
    let mut printed_markdown = String::new();
    
    {
        let mut pretty_printer = prettify_cmark::PrettyPrinter::new(&mut printed_markdown);
        pretty_printer.push_events(markdown.into_iter()).unwrap();
    }
    
    let process_result = subprocess::Exec::shell(command)
        .stdin(printed_markdown.as_str())
        .stdout(subprocess::Redirection::Pipe)
        .capture()?;

    match process_result.exit_status {
        subprocess::ExitStatus::Exited(0) => Ok(process_result.stdout_str()),
        subprocess::ExitStatus::Exited(code) => Err(OutputError::FailedCommand(code)),
        subprocess::ExitStatus::Signaled(signal) => Err(OutputError::TerminatedCommand(signal)),
        _ => unreachable!(),
    }
}

fn compile_markdown<'m>(markdown: MarkDown<'m>) -> OutputResult<String> {
    //TODO: Once the relevant pull request is merged on pulldown_cmark, change this to write
    //directly to the file
    
    let mut compiled_html = String::new();

    cmark::html::push_html(&mut compiled_html, markdown.into_iter());

    Ok(compiled_html)
}

fn print_markdown<'m>(mut file: fs::File, markdown: MarkDown<'m>) -> OutputResult<()> {
    let mut pretty_printer = prettify_cmark::PrettyPrinter::new(String::new());
    pretty_printer.push_events(markdown.into_iter()).unwrap();
    write!(file, "{}", pretty_printer.into_inner());
    Ok(())
}
