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

use parser::{FormatFn, CompilerSettings};
use output;
use output::canon::{CanonicalCodeBlock, BlockMap};

use subprocess;

use std::path::{PathBuf, Path};
use std::fs;
use std::io::{Write};

pub fn tangle_blocks<'a>(settings: Settings<'a>, 
                         canonical_code_blocks: &BlockMap) -> output::Result<()> {
    trace!("Starting the tangle...");
    for (name, block) in canonical_code_blocks.iter()
        .filter(|(_, block)| block.is_file() && block.print_to_tangle()) {
        let output_relative_dir = settings.global_settings.out_dir.join(settings.relative_directory); 
        std::fs::DirBuilder::new().recursive(true).create(&output_relative_dir)?;
        
        let output_file_path = output_relative_dir.join(name);
        
        info!("Found a file block \"{}\", writing to \"{}\"", name, output_file_path.to_string_lossy());
        // To avoid cluttering a workspace during linting, we do not produce the tangle output when
        // compiling
        if settings.global_settings.compile {
            //Compile the file
            compile_file(settings.compiler, &output_file_path)?;
        } else {
            // Print the file out
            let to_file = fs::OpenOptions::new().create(true).truncate(true).write(true).open(&output_file_path)?;
            print_file(to_file, settings.comment_formatter, name, block, &canonical_code_blocks)?;
        }
    }

    trace!("Finished the tangle");
    Ok(())
}

pub struct Settings<'borrow> {
    pub global_settings: &'borrow Globals,
    pub relative_directory: &'borrow Path,
    pub line_numbers: Option<&'borrow FormatFn<usize>>,
    pub comment_formatter: Option<&'borrow FormatFn<String>>,
    pub compiler: &'borrow Option<CompilerSettings>,
}

pub struct Globals {
    pub compile: bool,
    pub line_numbers: Option<FormatFn<usize>>,
    pub out_dir: PathBuf,
}

fn print_file<'a>(mut file: fs::File,
                  comment_formatter: Option<&'a FormatFn<String>>,
                  name: &'a str, 
                  file_block: &CanonicalCodeBlock<'a>, 
                  blocks: &BlockMap<'a>) -> output::Result<()> {
    trace!("Printing out \"{}\"...", name);
    print_block(&mut file, comment_formatter, name, file_block, blocks, vec![], vec![])?;
    trace!("Finished printing out \"{}\"", name);
    Ok(())
}

fn print_block<'a>(file: &mut fs::File,
                   comment_formatter: Option<&'a FormatFn<String>>,
                   name: &'a str,
                   block: &CanonicalCodeBlock<'a>, 
                   blocks: &BlockMap<'a>,
                   prependix: Vec<&'a str>,
                   appendix: Vec<&'a str>) -> output::Result<()> {
    if block.print_header() {
        if let Some(comment_formatter) = comment_formatter {
            print_line(file, &prependix[..], &comment_formatter(name.to_string()), &appendix[..])?;
        }
    }

    for line in block.contents() {
        let mut printed_link = false;

        for (pre_link, link, post_link) in line.split_links() {
            printed_link = true;

            let mut sub_pre = prependix.clone();
            sub_pre.extend_from_slice(pre_link);

            let mut sub_app = appendix.clone();
            sub_app.extend_from_slice(post_link);

            print_block(file, comment_formatter, link, blocks.get(link).unwrap(), blocks, sub_pre, sub_app)?;
        }

        if !printed_link {
            print_line(file, &prependix[..], line.get_text(), &appendix[..])?;
        }
    }

    Ok(())
}

fn print_line(file: &mut fs::File, prependix: &[&str], line: &str, appendix: &[&str]) -> output::Result<()> {
    for pre in prependix {
        write!(file, "{}", pre)?;
    }

    write!(file, "{}", line)?;

    for post in appendix {
        write!(file, "{}", post)?;
    }

    writeln!(file, "")?;

    Ok(())
}

fn compile_file(compiler_settings: &Option<CompilerSettings>, output_file_path: &Path) -> output::Result<()> {
    if let Some(ref compiler_settings) = compiler_settings {
        trace!("Compiling \"{}\"...", output_file_path.to_string_lossy());
        let compiler_result = subprocess::Exec::shell(&compiler_settings.command)
            .join()?;

        trace!("Finished compiling \"{}\"", output_file_path.to_string_lossy());
        match compiler_result {
            subprocess::ExitStatus::Exited(0) => Ok(()),
            subprocess::ExitStatus::Exited(code) => Err(output::Error::FailedCompiler(code)),
            subprocess::ExitStatus::Signaled(signal) => Err(output::Error::TerminatedCompiler(signal)),
            _ => unreachable!(),
        }
    } else {
        Err(output::Error::NoCompilerCommand)
    }
}
