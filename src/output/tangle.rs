use output::{OutputResult};
use parser::{BlockModifier};
use link::{LinkedFile, LinkedSection, LinkedBlock, LinkedLine};

use std::collections::{HashMap};
use std::path::{Path};
use std::fs;

pub struct Settings {
    pub compile: bool,
    pub line_numbers: Option<&'static Fn(usize) -> String>,
}
    
pub fn tangle_file<'a>(settings: &Settings, file: &LinkedFile<'a>) -> OutputResult<()> {
    let canonical_code_blocks = canonicalise_code_blocks(&file.sections[..]);

    for (name, block) in canonical_code_blocks.iter() {
        if block.is_file() {
            let to_file = fs::OpenOptions::new().write(true).open(name)?;
        }
    }

    Ok(())
}

struct CanonicalCodeBlock<'a> {
    print_header: bool,
    print_to_tangle: bool,
    print_to_weave: bool,
    form: CCBForm,
    contents: Vec<LinkedLine<'a>>,
}

enum CCBForm {
    File,
    Block,
}

impl<'a> CanonicalCodeBlock<'a> {
    fn from_form(form : CCBForm) -> Self {
        CanonicalCodeBlock {
            print_header: true,
            print_to_tangle: true,
            print_to_weave: true,
            form: form,
            contents: Vec::new(),
        }
    }

    fn is_file(&self) -> bool {
        match self.form {
            CCBForm::File => true,
            _ => false,
        }
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
}

fn canonicalise_code_blocks<'a>(sections: &[LinkedSection<'a>]) -> HashMap<&'a String, CanonicalCodeBlock<'a>> {
    let mut block_map : HashMap<&'a String, CanonicalCodeBlock<'a>> = HashMap::new();
    
    for section in sections {
        for block in section.blocks.iter() {
            match block {
                LinkedBlock::Code { ref name, modifiers, ref lines } => {
                    if block_map.contains_key(name) {
                        let canonical = block_map.get_mut(name).unwrap();

                        if modifiers.contains(BlockModifier::Redef) {
                            canonical.replace_lines(lines);
                            canonical.set_modifiers(*modifiers);
                        } else {
                            if modifiers.contains(BlockModifier::Append) {
                                canonical.add_modifiers(*modifiers);
                            }
                            canonical.append_lines(lines);
                        }
                    } else {
                        let form = if Path::new(name).is_file() {
                            CCBForm::File
                        } else {
                            CCBForm::Block
                        };

                        let mut canonical = CanonicalCodeBlock::from_form(form);
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
