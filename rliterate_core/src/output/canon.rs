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

use parser::{BlockModifier};
use link::{LinkedSection, LinkedBlock, LinkedLine};

use std::collections::{HashMap};
use std::slice::{Iter};
use std::path::{Path};

pub struct CanonicalCodeBlock<'a> {
    print_header: bool,
    print_to_tangle: bool,
    print_to_weave: bool,
    form: CCBForm,
    contents: Vec<LinkedLine<'a>>,
    pub first_defined_in: usize,
    appended_to_in: Vec<usize>,
    redefined_in: Vec<usize>,
}

pub enum CCBForm {
    File,
    Block,
}

impl<'a> CanonicalCodeBlock<'a> {
    fn from_form(form : CCBForm, in_section: usize) -> Self {
        CanonicalCodeBlock {
            print_header: true,
            print_to_tangle: true,
            print_to_weave: true,
            form: form,
            contents: Vec::new(),
            first_defined_in: in_section,
            appended_to_in: Vec::new(),
            redefined_in: Vec::new(),
        }
    }

    pub fn is_file(&self) -> bool {
        match self.form {
            CCBForm::File => true,
            _ => false,
        }
    }

    pub fn print_to_tangle(&self) -> bool {
        self.print_to_tangle
    }

    pub fn print_header(&self) -> bool {
        self.print_header
    }

    pub fn contents(&self) -> &[LinkedLine<'a>] {
        &self.contents[..]
    }
    
    fn append_lines(&mut self, lines: &[LinkedLine<'a>]) {
        self.contents.extend_from_slice(lines);
    }

    fn replace_lines(&mut self, lines: &[LinkedLine<'a>]) {
        self.contents.clear();
        self.append_lines(lines);
    }

    fn set_modifiers(&mut self, modifiers: BlockModifier) {
        self.print_header = !modifiers.contains(BlockModifier::NoHeader);
        self.print_to_tangle = !modifiers.contains(BlockModifier::NoTangle);
        self.print_to_weave = !modifiers.contains(BlockModifier::NoWeave);
    }
    
    fn add_modifiers(&mut self, modifiers: BlockModifier) {
        self.print_header &= !modifiers.contains(BlockModifier::NoHeader);
        self.print_to_tangle &= !modifiers.contains(BlockModifier::NoTangle);
        self.print_to_weave &= !modifiers.contains(BlockModifier::NoWeave);
    }

    fn mark_redefined(&mut self, in_section: usize) {
        self.redefined_in.push(in_section);
    }
    
    fn mark_appended(&mut self, in_section: usize) {
        self.appended_to_in.push(in_section);
    }
}

pub type BlockMap<'a> = HashMap<&'a str, CanonicalCodeBlock<'a>>;

pub fn canonicalise_code_blocks<'a>(sections: &[LinkedSection<'a>]) -> BlockMap<'a> {
    let mut block_map : BlockMap<'a> = HashMap::new();
    
    for section in sections {
        for block in section.blocks.iter() {
            match block {
                LinkedBlock::Code { ref name, modifiers, ref lines } => {
                    let name = name.as_str();
                    if block_map.contains_key(name) {
                        let canonical = block_map.get_mut(name).unwrap();

                        if modifiers.contains(BlockModifier::Redef) {
                            canonical.replace_lines(lines);
                            canonical.set_modifiers(*modifiers);
                            canonical.mark_redefined(section.id);
                        } else {
                            if modifiers.contains(BlockModifier::Append) {
                                canonical.add_modifiers(*modifiers);
                            }
                            canonical.append_lines(lines);
                            canonical.mark_appended(section.id);
                        }
                    } else {
                        let form = if Path::new(name).extension().is_some() {
                            CCBForm::File
                        } else {
                            CCBForm::Block
                        };

                        let mut canonical = CanonicalCodeBlock::from_form(form, section.id);
                        canonical.replace_lines(lines);
                        canonical.set_modifiers(*modifiers);

                        block_map.insert(name, canonical);
                    }
                },
                _ => {}
            }
        }
    }

    block_map
}
