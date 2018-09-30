use parser;
use parser::{LitFile, Block, BlockModifier};

use std::collections::{HashMap};
use std::path::{PathBuf};

mod grammar {
    use super::*;

    include!(concat!(env!("OUT_DIR"), "/link_parsing.rs"));
}

pub struct LinkState<'a> {
    pub file_map: HashMap<&'a PathBuf, LinkedFile<'a>>,
}

impl<'a> LinkState<'a> {
    pub fn link(parse_state: &'a parser::ParseState) -> LinkResult<Self> {
        let mut file_map = HashMap::new();
    
        for (path, lit_file) in parse_state.file_map.iter() {
            let linked_file = link_lit_file(lit_file)?;
    
            file_map.insert(path, linked_file);
        }
    
        Ok(LinkState {
            file_map: file_map,
        })
    }
}

type LinkMap<'a> = HashMap<&'a str, Vec<&'a str>>;

pub struct LinkedFile<'a> {
    pub comment_type: &'static Fn(String) -> String,
    pub sections: Vec<LinkedSection<'a>>,
}

pub struct LinkedSection<'a> {
    pub blocks: Vec<LinkedBlock<'a>>,
}

pub enum LinkedBlock<'a> {
    Code { name: &'a String, modifiers: BlockModifier, lines: Vec<LinkedLine<'a>> },
    Prose { lines: Vec<LinkedLine<'a>> },
}

impl<'a> LinkedBlock<'a> {
    fn get_lines(&self) -> &[LinkedLine<'a>] {
        match self {
            &LinkedBlock::Code { ref lines, .. } => &lines[..],
            &LinkedBlock::Prose { ref lines } => &lines[..]
        }
    }
}

#[derive(Clone)]
pub struct LinkedLine<'a>(Vec<LinkPart<'a>>);

impl<'a> LinkedLine<'a> {
    fn get_links(&self) -> Vec<&'a str> {
        let LinkedLine(parts) = self;
        parts.iter().filter_map(|part| {
            match part {
                &LinkPart::Link(contents) => Some(contents),
                _ => None
            }
        }).collect()
    }
}

#[derive(Clone)]
pub enum LinkPart<'a> {
    Link(&'a str),
    Text(&'a str),
}

type LinkResult<T> = Result<T, LinkError>;

pub enum LinkError {
    InfiniteCodeLoop,
    BadLinkName,
}

fn link_lit_file<'a>(lit_file: &'a LitFile) -> LinkResult<LinkedFile<'a>> {
    let mut link_map = LinkMap::new();
    
    let linked_sections : Vec<LinkedSection<'a>> = lit_file.sections.iter().map(|section| {
        LinkedSection {
            blocks: section.blocks.iter().map(|block| {
                link_block(block, &mut link_map)       
            }).collect()
        }
    }).collect();

    let all_links : Vec<&'a str> = linked_sections.iter().flat_map(|section| {
        section.blocks.iter().flat_map(|block| {
            block.get_lines().iter().flat_map(|line| {
                line.get_links()
            })
        })
    }).collect();

    //TODO: Detect a cycle!

    for link in all_links {
        if !(link_map.contains_key(link)) {
            Err(LinkError::BadLinkName)?;
        }
    }

    Ok(LinkedFile {
        comment_type: lit_file.comment_type,
        sections: linked_sections,
    })
}

fn link_block<'a>(block: &'a Block, link_map : &mut LinkMap<'a>) -> LinkedBlock<'a> {
    match block {
        &Block::Code { ref name, modifiers, ref lines } => {
            let linked_lines = link_lines(lines);
            let mut linked_to : Vec<&'a str> = linked_lines.iter()
                .flat_map(|line| line.get_links())
                .collect();
    
            let key = name.as_str();

            if link_map.contains_key(key) {
                let link_record = link_map.get_mut(key).unwrap();
                link_record.append(&mut linked_to);
            } else {
                link_map.insert(name, linked_to);
            }
    
            LinkedBlock::Code {
                name: name,
                modifiers: modifiers,
                lines: linked_lines,
            }
        },
        &Block::Prose { ref lines } => {
            LinkedBlock::Prose {
                lines: link_lines(lines)
            }
        }
    }
} 

fn link_lines<'a>(lines: &'a [String]) -> Vec<LinkedLine<'a>> {
    lines.iter().map(|line| grammar::link_line(line).unwrap()).collect()
}

