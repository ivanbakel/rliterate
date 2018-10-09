use output::{OutputResult, OutputError};
use output::css;
use output::canon::{BlockMap};
use link::{LinkedFile, LinkedBlock, LinkedLine};

use pulldown_cmark as cmark;
use prettify_cmark;
use subprocess;

use std::borrow::{Cow};
use std::path::{PathBuf, Path};
use std::fs;
use std::io::{Write};
use std::vec;

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

            if let Some(ref command) = maybe_command {
                call_markdown_compiler(command, html_file, markdown)
            } else {
                compile_markdown(html_file, markdown)
            }
        },
        Type::Markdown => {
            let markdown = MarkDown::build(settings, file, block_map);

            let mut md_filename = out_dir.join(file_name.file_stem().unwrap());
            md_filename.set_extension("md");

            let md_file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(md_filename)?;

            print_markdown(md_file, markdown);

            Ok(())
        }
    }
}

// Internal markdown representation
// Yields the markdown parsed by the cmark parser, and then some stuff that we're going to shove in
// as well ourselves
struct MarkDown<'m> {
    file_contents: Vec<cmark::Event<'m>>,
}

impl<'m> MarkDown<'m> {
    fn build(settings: &Settings, file: &'m LinkedFile<'m>, block_map: &'m BlockMap) -> Self {
        let mut file_contents : Vec<cmark::Event<'m>> = vec![];
    
        file_contents.append(&mut build_title(file.title));
        
        for section in file.sections.iter() {
            file_contents.append(&mut build_section_header(settings, section.name));
    
            for block in section.blocks.iter() {
                match block {
                    &LinkedBlock::Code { ref name, ref lines, ..} => {
                        file_contents.append(&mut build_code_block(settings, name, lines, block_map, &file.code_type));
                    },
                    &LinkedBlock::Prose { ref lines } => {
                        file_contents.append(&mut lines.iter().flat_map(|line| {
                            cmark::Parser::new(line.get_text())
                        }).collect());
                    },
                }
            }
        }
    
        MarkDown {
            file_contents: file_contents,
        }
    }

    fn into_iter(self) -> vec::IntoIter<cmark::Event<'m>> {
        self.file_contents.into_iter()
    }
}

fn build_title<'a>(title: &'a str) -> Vec<cmark::Event<'a>> {
    vec![
        cmark::Event::Start(cmark::Tag::Header(1)),
        cmark::Event::Text(Cow::Borrowed(title)),
        cmark::Event::End(cmark::Tag::Header(1)),
    ]
}

fn build_section_header<'a>(settings: &Settings, name: &'a str) -> Vec<cmark::Event<'a>> {
    vec![
        cmark::Event::Start(cmark::Tag::Header(4)),
        cmark::Event::Text(Cow::Borrowed(name)),
        cmark::Event::End(cmark::Tag::Header(4)),
    ]
}

fn build_code_block<'a>(settings: &Settings, name: &'a str, lines: &'a [LinkedLine<'a>], block_map: &'a BlockMap, code_type: &'a str) -> Vec<cmark::Event<'a>> {
    let mut code_block : Vec<cmark::Event<'a>> = vec![];

    code_block.push(cmark::Event::Start(cmark::Tag::CodeBlock(Cow::Borrowed(code_type))));
    code_block.append(&mut lines.iter().flat_map(|line| {
        vec![
            cmark::Event::Text(Cow::Borrowed(line.get_text())),
            cmark::Event::SoftBreak,
        ]
    }).collect());
    code_block.push(cmark::Event::End(cmark::Tag::CodeBlock(Cow::Borrowed(code_type))));
    code_block.push(cmark::Event::SoftBreak);

    code_block
}

impl From<subprocess::PopenError> for OutputError {
    fn from(err: subprocess::PopenError) -> OutputError {
        OutputError::BadCommand(err)
    }
}

fn call_markdown_compiler<'m>(command: &str, file: fs::File, markdown: MarkDown<'m>) -> OutputResult<()> {
    // Setup the markdown to be fed into the command
    let mut printed_markdown = String::new();
    
    {
        let mut pretty_printer = prettify_cmark::PrettyPrinter::new(&mut printed_markdown);
        pretty_printer.push_events(markdown.file_contents.into_iter()).unwrap();
    }
    
    let process_result = subprocess::Exec::shell(command)
        .stdin(printed_markdown.as_str())
        .stdout(file)
        .join()?;

    match process_result {
        subprocess::ExitStatus::Exited(0) => Ok(()),
        subprocess::ExitStatus::Exited(code) => Err(OutputError::FailedCommand(code)),
        subprocess::ExitStatus::Signaled(signal) => Err(OutputError::TerminatedCommand(signal)),
        _ => unreachable!(),
    }
}

fn compile_markdown<'m>(mut file: fs::File, markdown: MarkDown<'m>) -> OutputResult<()> {
    //TODO: Once the relevant pull request is merged on pulldown_cmark, change this to write
    //directly to the file
    
    let mut compiled_html = String::new();

    cmark::html::push_html(&mut compiled_html, markdown.into_iter());

    file.write(&compiled_html.as_bytes())?;

    Ok(())
}

fn print_markdown<'m>(mut file: fs::File, markdown: MarkDown<'m>) -> OutputResult<()> {
    let mut pretty_printer = prettify_cmark::PrettyPrinter::new(String::new());
    pretty_printer.push_events(markdown.file_contents.into_iter()).unwrap();
    write!(file, "{}", pretty_printer.into_inner());
    Ok(())
}
