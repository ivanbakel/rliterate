use output::{OutputResult};
use parser::{BlockModifier};
use link;
use link::{LinkedFile, LinkedSection, LinkedBlock, LinkedLine};

use std::collections::{HashMap};
use std::path::{Path};
use std::fs;
use std::io::{Write};

pub fn tangle_file<'a>(settings: &Settings, file: &LinkedFile<'a>) -> OutputResult<()> {
    let canonical_code_blocks = canonicalise_code_blocks(&file.sections[..]);

    for (name, block) in canonical_code_blocks.iter()
        .filter(|(key, block)| block.is_file() && block.print_to_tangle()) {
        let to_file = fs::OpenOptions::new().write(true).open(name)?;

        settings.print_file(to_file, name, block, &canonical_code_blocks)?;
    }

    Ok(())
}

pub struct Settings {
    pub compile: bool,
    pub line_numbers: Option<&'static Fn(usize) -> String>,
}

impl Settings {
    fn print_file<'a>(&self,
                      mut file: fs::File,
                      name: &'a str, 
                      file_block: &CanonicalCodeBlock<'a>, 
                      blocks: &BlockMap<'a>) -> OutputResult<()> {
        self.print_block(&mut file, name, file_block, blocks, vec![], vec![])?;
        
        //TODO: Compile the file!
        Ok(())
    }
    
    fn print_block<'a>(&self, 
                       file: &mut fs::File,
                       name: &'a str, 
                       block: &CanonicalCodeBlock<'a>, 
                       blocks: &BlockMap<'a>,
                       prependix: Vec<&'a str>,
                       appendix: Vec<&'a str>) -> OutputResult<()> {
        if block.print_header {
            Self::print_line(file, &prependix[..], name, &appendix[..]);
        }

        for line in block.contents.iter() {
            let mut printed_link = false;

            for (pre_link, link, post_link) in line.split_links() {
                printed_link = true;

                let mut sub_pre = prependix.clone();
                sub_pre.extend_from_slice(pre_link);

                let mut sub_app = appendix.clone();
                sub_app.extend_from_slice(post_link);

                self.print_block(file, link, blocks.get(link).unwrap(), blocks, sub_pre, sub_app)?;
            }

            if !printed_link {
                Self::print_line(file, &prependix[..], name, &appendix[..]);
            }
        }

        Ok(())
    }

    fn print_line(file: &mut fs::File, prependix: &[&str], line: &str, appendix: &[&str]) {
        for pre in prependix {
            write!(file, "{}", pre);
        }

        write!(file, "{}", line);

        for post in appendix {
            write!(file, "{}", post);
        }

        writeln!(file, "");
    }
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

    fn print_to_tangle(&self) -> bool {
        self.print_to_tangle
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

type BlockMap<'a> = HashMap<&'a str, CanonicalCodeBlock<'a>>;

fn canonicalise_code_blocks<'a>(sections: &[LinkedSection<'a>]) -> BlockMap<'a> {
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
