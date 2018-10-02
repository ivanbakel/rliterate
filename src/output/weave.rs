use output::{OutputResult};
use output::css;
use link::{LinkedFile, LinkedBlock, LinkedLine};

use pulldown_cmark as cmark;

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

pub fn weave_file<'a>(settings: &Settings, file_name: &PathBuf, file: &LinkedFile<'a>, out_dir: &Path) -> OutputResult<()> {
    match settings.weave_type {
        Type::HtmlViaMarkdown(ref maybe_command) => {
            let markdown = MarkDown::build(settings, file);

            let mut html_filename = file_name.clone();
            html_filename.set_extension("html");

            let html_file = fs::OpenOptions::new().write(true).open(out_dir.join(html_filename))?;

            if let Some(ref command) = maybe_command {
                call_markdown_compiler(command, html_file, markdown)
            } else {
                compile_markdown(html_file, markdown)
            }
        },
        _ => {
            panic!()
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
    fn build(settings: &Settings, file: &'m LinkedFile<'m>) -> Self {
        let mut file_contents : Vec<cmark::Event<'m>> = vec![];
    
        file_contents.append(&mut build_title(file.title));
        
        for section in file.sections.iter() {
            file_contents.append(&mut build_section_header(settings, section.name));
    
            for block in section.blocks.iter() {
                match block {
                    &LinkedBlock::Code { ref name, ref lines, ..} => {
                        file_contents.append(&mut build_code_block(settings, name, lines));
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

fn build_code_block<'a>(settings: &Settings, name: &'a str, lines: &'a [LinkedLine<'a>]) -> Vec<cmark::Event<'a>> {
    let mut code_block : Vec<cmark::Event<'a>> = vec![];

    code_block.push(cmark::Event::Start(cmark::Tag::CodeBlock(Cow::Borrowed(name))));
    code_block.append(&mut lines.iter().map(|line| {
        cmark::Event::Text(Cow::Borrowed(line.get_text()))
    }).collect());
    code_block.push(cmark::Event::End(cmark::Tag::CodeBlock(Cow::Borrowed(name))));

    code_block
}

fn call_markdown_compiler<'m>(command: &String, mut file: fs::File, markdown: MarkDown<'m>) -> OutputResult<()> {
    Ok(())
}

fn compile_markdown<'m>(mut file: fs::File, markdown: MarkDown<'m>) -> OutputResult<()> {
    //TODO: Once the relevant pull request is merged on pulldown_cmark, change this to write
    //directly to the file
    
    let mut compiled_html = String::new();

    cmark::html::push_html(&mut compiled_html, markdown.into_iter());

    file.write(&compiled_html.as_bytes())?;

    Ok(())
}
