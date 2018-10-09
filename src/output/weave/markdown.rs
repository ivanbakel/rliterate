use pulldown_cmark as cmark;

use output::canon::{BlockMap};
use link::{LinkedFile, LinkedBlock, LinkedLine};

use std::borrow::{Cow};
use std::vec;

// Internal markdown representation
// Yields the markdown parsed by the cmark parser, and then some stuff that we're going to shove in
// as well ourselves
pub struct MarkDown<'m> {
    file_contents: Vec<cmark::Event<'m>>,
}

impl<'m> MarkDown<'m> {
    pub fn build(settings: &super::Settings, file: &'m LinkedFile<'m>, block_map: &'m BlockMap) -> Self {
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

    pub fn into_iter(self) -> vec::IntoIter<cmark::Event<'m>> {
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

fn build_section_header<'a>(settings: &super::Settings, name: &'a str) -> Vec<cmark::Event<'a>> {
    vec![
        cmark::Event::Start(cmark::Tag::Header(4)),
        cmark::Event::Text(Cow::Borrowed(name)),
        cmark::Event::End(cmark::Tag::Header(4)),
    ]
}

fn build_code_block<'a>(settings: &super::Settings, name: &'a str, lines: &'a [LinkedLine<'a>], block_map: &'a BlockMap, code_type: &'a str) -> Vec<cmark::Event<'a>> {
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


