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

use parser;
use parser::{LitFile, Block, BlockModifier};

use std::collections::{HashMap, VecDeque};
use std::path::{PathBuf};

peg::parser!{grammar grammar() for str {
    rule whitespace() = " " / "\t"

    rule non_whitespace() = !whitespace() [_]

    rule _ = whitespace()+

    rule name() = ((!link_end() non_whitespace())+) ++ _

    rule link_start() = "@{"
    rule link_end() = "}"

    rule text() -> (LinkPart, &'input str)
      = text:$((!link_start() [_])+) { (LinkPart::Text, text) }

    rule link() -> (LinkPart, &'input str)
      = link_start() _? name:$(name()) _? link_end() { (LinkPart::Link, name) }

    pub rule link_line() -> LinkedLine<'input>
      = parts:((text() / link()) *) {
          let (parts, slices) = parts
            .into_iter()
            .fold((vec![], vec![]),
            |(mut ps, mut ns), (part, name)| {
                ps.push(part);
                ns.push(name);
                (ps, ns)
            });

          LinkedLine {
              parts: parts,
              slices: slices,
              text: "",
          }
        }
    }
}

pub struct LinkState<'a> {
    pub file_map: HashMap<&'a PathBuf, LinkedFile<'a>>,
}

impl<'a> LinkState<'a> {
    pub fn link(file_map: &'a parser::FileMap) -> Result<Self> {
        trace!("Started linking files...");
        let mut linked_file_map = HashMap::new();

        for (path, lit_file) in file_map.iter() {
            info!("Linking up the blocks in \"{}\"", path.to_string_lossy());
            let linked_file = link_lit_file(lit_file)?;

            linked_file_map.insert(path, linked_file);
        }

        trace!("Finished linking files");
        Ok(LinkState {
            file_map: linked_file_map,
        })
    }
}

type LinkMap<'a> = HashMap<&'a str, Vec<&'a str>>;

pub struct LinkedFile<'a> {
    pub metadata: &'a parser::Metadata,
    pub sections: Vec<LinkedSection<'a>>,
    pub link_map: LinkMap<'a>,
}

impl<'a> std::ops::Deref for LinkedFile<'a> {
    type Target = parser::Metadata;

    fn deref(&self) -> &parser::Metadata {
        &self.metadata
    }
}

pub struct LinkedSection<'a> {
    pub id: usize,
    pub depth: usize,
    pub name: Option<&'a str>,
    pub blocks: Vec<LinkedBlock<'a>>,
}

pub enum LinkedBlock<'a> {
    Code { name: &'a String, modifiers: BlockModifier, lines: Vec<LinkedLine<'a>>, },
    Prose { lines: Vec<LinkedLine<'a>>, },
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
            &self.slices[self.current_position+1..self.slices.len()]
        ))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InfiniteCodeLoop,
    BadLinkName,
}

fn link_lit_file<'a>(lit_file: &'a LitFile) -> Result<LinkedFile<'a>> {
    let mut link_map = LinkMap::new();

    let linked_sections : Vec<LinkedSection<'a>> = lit_file.sections.iter().map(|section| {
        LinkedSection {
            id: section.id,
            depth: section.depth,
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

    for link in all_links {
        if !(link_map.contains_key(link)) {
            error!("Found a link to \"{}\", but that block doesn't exist", link);
            Err(Error::BadLinkName)?;
        }
    }

    {
      let mut recursion_stack : VecDeque<&'a str> = VecDeque::new();

      fn check_recursion<'str, I>(links: I, recursion_stack: &mut VecDeque<&'str str>, link_map: &LinkMap<'str>) -> Result<()>
        where I: Iterator, I::Item: std::ops::Deref<Target=&'str str> {
        for link in links {
          // If the stack already contains the link, we have a back-edge
          if recursion_stack.contains(&link) {
            // We push this on to make clear what the recursive element is
            // when rendering the below error message
            recursion_stack.push_back(*link);

            let (front, back) = recursion_stack.as_slices();
            let mut recursion_path = front.join(" -> ");
            let back_path = back.join(" -> ");

            if back_path.len() != 0 {
              recursion_path.push_str(" -> ");
              recursion_path.push_str(&back_path);
            }


            error!("Found a recursive series of links: {}", recursion_path);
            return Err(Error::InfiniteCodeLoop);
          } else {
            recursion_stack.push_back(*link);
            // We checked above that all links are valid - so unwrapping is safe
            check_recursion(link_map.get(*link).unwrap().iter(), recursion_stack, link_map)?;
            recursion_stack.pop_back();
          }
        }

        Ok(())
      }

      check_recursion(link_map.keys(), &mut recursion_stack, &link_map)?;
    }

    Ok(LinkedFile {
        metadata: &lit_file.metadata,
        sections: linked_sections,
        link_map: link_map,
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

