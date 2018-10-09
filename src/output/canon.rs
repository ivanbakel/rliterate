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
    pub first_defined_in: &'a str,
    appended_to_in: Vec<&'a str>,
    redefined_in: Vec<&'a str>,
}

pub enum CCBForm {
    File,
    Block,
}

impl<'a> CanonicalCodeBlock<'a> {
    fn from_form(form : CCBForm, in_section: &'a str) -> Self {
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

    pub fn contents(&self) -> Iter<LinkedLine<'a>> {
        self.contents.iter()
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

    fn mark_redefined(&mut self, in_section: &'a str) {
        self.redefined_in.push(in_section);
    }
    
    fn mark_appended(&mut self, in_section: &'a str) {
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
                            canonical.mark_redefined(&section.name);
                        } else {
                            if modifiers.contains(BlockModifier::Append) {
                                canonical.add_modifiers(*modifiers);
                            }
                            canonical.append_lines(lines);
                            canonical.mark_appended(&section.name);
                        }
                    } else {
                        let form = if Path::new(name).extension().is_some() {
                            CCBForm::File
                        } else {
                            CCBForm::Block
                        };

                        let mut canonical = CanonicalCodeBlock::from_form(form, &section.name);
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
