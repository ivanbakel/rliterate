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
    pub fn link(file_map: &'a parser::FileMap) -> LinkResult<Self> {
        let mut linked_file_map = HashMap::new();
    
        for (path, lit_file) in file_map.iter() {
            let linked_file = link_lit_file(lit_file)?;
    
            linked_file_map.insert(path, linked_file);
        }
    
        Ok(LinkState {
            file_map: linked_file_map,
        })
    }
}

type LinkMap<'a> = HashMap<&'a str, Vec<&'a str>>;

pub struct LinkedFile<'a> {
    pub title: &'a str,
    pub comment_type: &'static Fn(String) -> String,
    pub sections: Vec<LinkedSection<'a>>,
}

pub struct LinkedSection<'a> {
    pub name: &'a str,
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
pub struct LinkedLine<'a> {
    parts: Vec<LinkPart>,
    slices: Vec<&'a str>,
    text: &'a str,
}

impl<'a> LinkedLine<'a> {
    fn get_links(&self) -> Vec<&'a str> {
        let mut links = Vec::new();

        for i in 0..self.parts.len() {
            if self.parts[i].is_link() {
                links.push(self.slices[i]);
            }
        }

        links
    }

    pub fn split_links<'b>(&'b self) -> SplitLinks<'a, 'b> {
        SplitLinks {
            first: true, 
            current_position: 0_usize,
            parts: &self.parts[..],
            slices: &self.slices[..],
        }
    }

    pub fn get_text<'b>(&'b self) -> &'b str {
        self.text
    }
}

#[derive(Clone)]
pub enum LinkPart {
    Link,
    Text,
}

impl LinkPart {
    fn is_link(&self) -> bool {
        match self {
            &LinkPart::Link => true,
            _ => false
        }
    }
}

type LinkInLine<'a, 'b> = (&'b [&'a str], &'a str, &'b [&'a str]);

pub struct SplitLinks<'a : 'b, 'b> {
    first : bool,
    current_position: usize,
    parts: &'b [LinkPart],
    slices: &'b [&'a str],
}

impl<'a, 'b : 'a> Iterator for SplitLinks<'a, 'b> {
    type Item = LinkInLine<'a, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        
        // First iteration
        if self.first {
            self.current_position = 0;
            self.first = false;
        } else {
            self.current_position += 1;
        }

        while self.current_position < self.parts.len() && !self.parts[self.current_position].is_link() {
            self.current_position += 1;
        }

        if self.current_position >= self.slices.len() {
            return None;
        }
        
        Some((
            &self.slices[0..self.current_position], 
            self.slices[self.current_position], 
            &self.slices[self.current_position..self.slices.len()]
        ))
    }
}

type LinkResult<T> = Result<T, LinkError>;

#[derive(Debug)]
pub enum LinkError {
    InfiniteCodeLoop,
    BadLinkName,
}

fn link_lit_file<'a>(lit_file: &'a LitFile) -> LinkResult<LinkedFile<'a>> {
    let mut link_map = LinkMap::new();
    
    let linked_sections : Vec<LinkedSection<'a>> = lit_file.sections.iter().map(|section| {
        LinkedSection {
            name: section.name.as_str(),
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
        title: &lit_file.title,
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
    lines.iter().map(|line| {
        let mut linked_line = grammar::link_line(line).unwrap();
        linked_line.text = line;
        linked_line
    }).collect()
}

