use parser::{CompilerSettings};
use output::{OutputResult, OutputError};
use output::canon;
use output::canon::{CanonicalCodeBlock, BlockMap};
use link::{LinkedFile};

use subprocess;

use std::collections::{HashMap};
use std::path::{Path};
use std::fs;
use std::io::{Write};

pub fn tangle_file<'a>(settings: &Settings, file: &LinkedFile<'a>, out_dir: &Path) -> OutputResult<()> {
    let canonical_code_blocks = canon::canonicalise_code_blocks(&file.sections[..]);

    for (name, block) in canonical_code_blocks.iter()
        .filter(|(key, block)| block.is_file() && block.print_to_tangle()) {
        let output_file_path = out_dir.join(name);
        
        // To avoid cluttering a workspace during linting, we do not produce the tangle output when
        // compiling
        if settings.compile {
            //Compile the file
            settings.compile_file(file.compiler)?;
        } else {
            // Print the file out
            let to_file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(&output_file_path)?;
            settings.print_file(to_file, name, block, &canonical_code_blocks)?;
        }
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
        self.print_block(&mut file, name, file_block, blocks, vec![], vec![])
    }
    
    fn print_block<'a>(&self, 
                       file: &mut fs::File,
                       name: &'a str, 
                       block: &CanonicalCodeBlock<'a>, 
                       blocks: &BlockMap<'a>,
                       prependix: Vec<&'a str>,
                       appendix: Vec<&'a str>) -> OutputResult<()> {
        if block.print_header() {
            Self::print_line(file, &prependix[..], &format!("// {}", name), &appendix[..]);
        }

        for line in block.contents() {
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
                Self::print_line(file, &prependix[..], line.get_text(), &appendix[..]);
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

    fn compile_file(&self, compiler_settings: &Option<CompilerSettings>) -> OutputResult<()> {
        if let Some(ref compiler_settings) = compiler_settings {
            let compiler_result = subprocess::Exec::shell(&compiler_settings.command)
                .join()?;

            match compiler_result {
                subprocess::ExitStatus::Exited(0) => Ok(()),
                subprocess::ExitStatus::Exited(code) => Err(OutputError::FailedCompiler(code)),
                subprocess::ExitStatus::Signaled(signal) => Err(OutputError::TerminatedCompiler(signal)),
                _ => unreachable!(),
            }
        } else {
            Err(OutputError::NoCompilerCommand)
        }
    }
}
    
