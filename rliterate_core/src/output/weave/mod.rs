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

mod markdown;
use self::markdown::{MarkDown};
mod html;

use output;
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

pub fn weave_file_with_blocks<'a>(settings: &Settings, file_name: &PathBuf, file: &LinkedFile<'a>, block_map: &BlockMap, out_dir: &Path) -> output::Result<()> {
    trace!("Starting the weave...");
    match settings.weave_type {
        Type::HtmlViaMarkdown(ref maybe_command) => {
            let markdown = MarkDown::build(settings, file, block_map);

            let mut html_filename = out_dir.join(file_name.file_stem().unwrap());
            html_filename.set_extension("html");
            info!("Writing HTML documentation to \"{}\"", html_filename.to_string_lossy());

            let html_file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(html_filename)?;

            let compiled_markdown = if let Some(ref command) = maybe_command {
                call_markdown_compiler(command, markdown)
            } else {
                compile_markdown(markdown)
            }?;

            html::print(html_file, compiled_markdown, &file.title, &settings.css)?;
        },
        Type::Markdown => {
            let markdown = MarkDown::build(settings, file, block_map);

            let mut md_filename = out_dir.join(file_name.file_stem().unwrap());
            md_filename.set_extension("md");
            info!("Writing Markdown documentation to \"{}\"", md_filename.to_string_lossy());

            let md_file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(md_filename)?;

            print_markdown(md_file, markdown)?;
        }
    }
    trace!("Finished the weave");
    Ok(())
}

impl From<subprocess::PopenError> for output::Error {
    fn from(err: subprocess::PopenError) -> output::Error {
        output::Error::BadCommand(err)
    }
}

fn call_markdown_compiler<'m>(command: &str, markdown: MarkDown<'m>) -> output::Result<String> {
    // Setup the markdown to be fed into the command
    let mut printed_markdown = String::new();
    
    {
        let mut pretty_printer = prettify_cmark::PrettyPrinter::new(&mut printed_markdown);
        pretty_printer.push_events(markdown.into_iter()).unwrap();
    }
   
    trace!("Invoking the requested markdown compiler \"{}\"...", command);
    let process_result = subprocess::Exec::shell(command)
        .stdin(printed_markdown.as_str())
        .stdout(subprocess::Redirection::Pipe)
        .capture()?;

    trace!("Finished invoking the requested markdown compiler");

    match process_result.exit_status {
        subprocess::ExitStatus::Exited(0) => Ok(process_result.stdout_str()),
        subprocess::ExitStatus::Exited(code) => Err(output::Error::FailedCommand(code)),
        subprocess::ExitStatus::Signaled(signal) => Err(output::Error::TerminatedCommand(signal)),
        _ => unreachable!(),
    }
}

fn compile_markdown<'m>(markdown: MarkDown<'m>) -> output::Result<String> {
    //TODO: Once the relevant pull request is merged on pulldown_cmark, change this to write
    //directly to the file
    
    let mut compiled_html = String::new();

    trace!("Invoking pulldown_cmark...");
    cmark::html::push_html(&mut compiled_html, markdown.into_iter());

    trace!("Finished invoking pulldown_cmark");

    Ok(compiled_html)
}

fn print_markdown<'m>(mut file: fs::File, markdown: MarkDown<'m>) -> output::Result<()> {
    let mut pretty_printer = prettify_cmark::PrettyPrinter::new(String::new());
    pretty_printer.push_events(markdown.into_iter()).unwrap();
    write!(file, "{}", pretty_printer.into_inner());
    Ok(())
}
