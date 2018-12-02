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

// Load the grammar file
include!(concat!(env!("OUT_DIR"), "/literate.rs"));

pub enum Command<'a> {
    Title(&'a str),
    Section { name: Option<&'a str>, depth: usize },
    CodeType { code_type:&'a str, file_extension:&'a str},
    CommentType(&'a str),
    Compiler(&'a str),
    ErrorFormat(&'a str),
    LineNumbers(&'a str),
    Book,
    AddCss(&'a str),
    OverwriteCss(&'a str),
    Colorscheme(&'a str),
}

bitflags! {
    pub struct BlockModifier: u32 {
        const Append   = 0b00000001;
        const Redef    = 0b00000010;
        const NoTangle = 0b00000100;
        const NoWeave  = 0b00001000;
        const NoHeader = 0b00010000;
    }
}

pub enum PartialLitLine<'a> {
    CodeBlockStart(&'a str, BlockModifier),
    CodeBlockEnd,
    Line(&'a str),
}

pub enum LitLine<'a> {
    Command(Command<'a>),
    Chapter { title: &'a str, file_name: &'a str },
    Prose(&'a str),
}

fn partial_lines<'a>(input: &'a str) -> Result<Vec<PartialLitLine<'a>>, ParseError> {
    let mut partial_lines = vec![];

    for line in input.lines() {
        partial_lines.push(partial_line(line)?);
    }

    Ok(partial_lines)
}

pub fn lit_file<'a>(input: &'a str) -> Result<Vec<LitBlock<'a>>, ParseError> {
    let partial_lines = partial_lines(input)?;
    parse_lines(partial_lines)
}

fn parse_lines<'a>(lines: Vec<PartialLitLine<'a>>) -> Result<Vec<LitBlock<'a>>, ParseError> {
    let mut blocks = vec![];

    let mut prose_lines: Vec<&'a str> = vec![];
   
    macro_rules! close_prose {
        () => {
            blocks.push(LitBlock::Prose(prose_lines));
            prose_lines = vec![];
        }
    }

    let mut line_iter = lines.into_iter();
            
    while let Some(line) = line_iter.next() {
        match line {
            PartialLitLine::CodeBlockStart(name, modifiers) => {
                close_prose!();

                let mut code = CodeBlock {
                        block_name: name,
                        modifiers: modifiers,
                        contents: vec![],
                };
                while let Some(code_line) = line_iter.next() {
                    match code_line {
                        PartialLitLine::CodeBlockEnd => {
                            break;
                        },
                        PartialLitLine::Line(line) => {
                            code.contents.push(line);
                        }
                        PartialLitLine::CodeBlockStart(..) => {
                            error!("Tried to start a new code block without ending the previous one.");
                            panic!()
                        }
                    }
                }

                blocks.push(LitBlock::Code(code));
            },
            PartialLitLine::CodeBlockEnd => {
                error!("Found a code block with no name");
                panic!()
            },
            PartialLitLine::Line(line) => {
                let line = lit_line(line)?;

                match line {
                    LitLine::Command(command) => {
                        close_prose!();
                        blocks.push(LitBlock::Command(command));
                    },
                    LitLine::Chapter { title, file_name } => {
                        close_prose!();
                        blocks.push(LitBlock::Chapter { title, file_name });
                    },
                    LitLine::Prose(line) => {
                        prose_lines.push(line);
                    },
                }
            },
        }
    }

    Ok(blocks)
}

pub struct CodeBlock<'a> {
    pub block_name: &'a str,
    pub modifiers: BlockModifier,
    pub contents: Vec<&'a str>
}

pub enum LitBlock<'a> {
    Command(Command<'a>),
    Chapter { title: &'a str, file_name: &'a str },
    Code(CodeBlock<'a>),
    Prose(Vec<&'a str>),
}

