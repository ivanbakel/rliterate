use parser;

use std::collections::{HashMap};

include!(concat!(env!("OUT_DIR"), "/link_parsing.rs"));

pub struct LinkState<'a> {
    pub file_map: HashMap<&'a PathBuf, LinkedFile<'a>>,
}

impl<'a> LinkState<'a> {
    pub fn link(parse_state: &'a parser::ParseState) -> LinkResult<Self> {
        let file_map = HashMap::new();
    
        for (path, lit_file) in parse_state.file_map.iter() {
            let linked_file = link_lit_file(lit_file)?;
    
            file_map.insert(path, linked_file);
        }
    
        Ok(LinkState {
            file_map: file_map,
        })
    }
}

type LinkMap<'a> = HashMap<&'a String, Vec<&'a String>>;

pub struct LinkedFile<'a> {
    pub comment_type: &'static Fn(String) -> String,
    pub sections: Vec<LinkedSection<'a>>,
}

struct LinkedSection<'a> {
    pub blocks: Vec<LinkedBlock<'a>>,
}

enum LinkedBlock<'a> {
    Code { name: String, modifiers: BlockModifier, lines: Vec<LinkedLine<'a>> },
    Prose { lines: Vec<LinkedLine<'a>> },
}

impl<'a> LinkedBlock<'a> {
    fn get_lines(&self) -> &[LinkedLine<'a>] {
        match self {
            &Self::Code { lines, .. } => &lines[..],
            &Self::Prose { lines } => &lines[..]
        }
    }
}

pub struct LinkedLine<'a>(Vec<LinkPart<'a>>);

impl<'a> LinkedLine<'a> {
    fn get_links(&self) -> Vec<&'a str> {
        self.iter().filter_map(|part| {
            match part {
                &Link(contents) => Some(contents),
                _ => None
            }
        }
    }
}

pub enum LinkPart<'a> {
    Link(&'a str),
    Text(&'a str),
}

type LinkResult<T> = Result<T, LinkError>;

enum LinkError {
    InfiniteCodeLoop,
    BadLinkName,
}

fn link_lit_file<'a>(lit_file: &'a LitFile) -> LinkResult<LinkedFile<'a>> {
    let mut link_map = LinkMap::new();
    
    let linked_sections = lit_file.sections().map(|section| {
        LinkedSection {
            blocks: section.blocks.map(|block| {
                link_block(block, &mut link_map)       
            }).collect()
        }
    }).collect();

    let all_links = linked_sections.iter().flat_map(|section| {
        section.blocks.flat_map(|block| {
            block.get_lines().flat_map(|line| {
                line.get_links()
            })
        })
    });

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

fn link_block<'a>(block: &'a Block, link_map : &mut LinkMap) -> LinkedBlock<'a> {
    Block::Code { name, modifiers, lines } => {
        let linked_lines = link_lines(lines);
        let linked_to = linked_lines.flat_map(|line| line.get_links());

        if link_map.contains_key(&name) {
            let link_record = link_map.get_mut(name).unwrap();
            link_record.append(linked_to);
        } else {
            link_map.insert(&name, linked_to);
        }

        LinkedBlock::Code {
            name: name,
            modifiers: modifiers,
            lines: linked_lines,
        }
    },
    Block::Prose { lines } => {
        LinkedBlock::Prose {
            lines: link_lines(lines)
        }
    }
} 

fn link_lines<'a>(lines: &'a [String]) -> Vec<LinkedLine<'a>> {
    lines.iter().map(|line| link_line(line).unwrap()).collect()
}

