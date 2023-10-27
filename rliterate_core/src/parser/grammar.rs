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

peg::parser!{grammar grammar() for str {

    rule newline() = "\n" / "\r\n" / ![_]

    rule whitespace() = " " / "\t"

    // Optional whitespace
    rule _ = whitespace()*

    // Significant whitespace
    rule __ = whitespace()+

    rule non_whitespace() = (!(whitespace() / newline()) [_])

    rule comment() = "//" (!newline() [_])*

    rule line(contents : rule<()>) = contents() _ comment()?

    rule named_line<T>(contents : rule<T>) -> T = this:contents() _ comment()? { this }

    rule line_slice() -> &'input str
        = $((!newline() [_])+)

    /// Commands

    rule arg_separator() -> () = "---"

    rule name() -> &'input str
        = $(((!arg_separator() non_whitespace())+) ++ (__))

    rule delimited_name(delim : rule<()>) -> &'input str
        = $(((!delim() non_whitespace())+) ++ (__))

    rule file_extension() -> &'input str
        = $("." ['A'..='Z'|'a'..='z'|'_'|'-']+)

    rule comment_pattern() -> &'input str
        = line_slice()

    rule line_num_pattern() -> &'input str
        = line_slice()

    rule error_pattern() -> &'input str
        = line_slice()

    rule shell_command() -> &'input str
        = line_slice()

    rule css_file() -> &'input str
        = line_slice()

    rule a_command() -> Command<'input>
        = "title" __ title:name() { Command::Title(title) }
        / esses:$("s"+) __ section_name:(name()?)
            { Command::Section { name: section_name, depth: esses.len() - 1 } }
        / "code_type" __ ctype:$(['A'..='Z'|'a'..='z'|'_'|'-']+) _ extension:file_extension()
            { Command::CodeType { code_type: ctype, file_extension: extension } }
        / "comment_type" __ pattern:comment_pattern() { Command::CommentType(pattern) }
        / "line_numbers" __ pattern:line_num_pattern() { Command::LineNumbers(pattern) }
        / "compiler" __ sh_command:shell_command() { Command::Compiler(sh_command) }
        / "error_format" __ err_format:error_pattern() { Command::ErrorFormat(err_format) }
        / "book" __ { Command::Book }
        / "add_css" __ file:css_file() { Command::AddCss(file) }
        / "overwrite_css" __ file:css_file() { Command::OverwriteCss(file) }
        / "colorscheme" __ file:css_file() { Command::Colorscheme(file) }
        / expected!("A valid command")

    rule command() -> Command<'input>
        = named_line(<"@" a_command:a_command() { a_command }>)

    /// Code blocks

    rule codeblock_delim() -> () = "---"

    rule block_name() -> &'input str
        = delimited_name(<arg_separator() / block_modifier_unit()>)

    rule block_modifier() -> BlockModifier
        = append()
        / redef()
        / arg_separator() _ mods:(mods:block_mods() ++ __ { mods })
            { mods.into_iter().fold(BlockModifier::empty(), |set, flag| set | flag) }

    rule block_modifier_unit() -> () = block_modifier() { () }

    rule possible_block_modifier() -> BlockModifier
        =  modifier:block_modifier() { modifier }
        / _ { BlockModifier::empty() }

    rule block_mods() -> BlockModifier
        = append()
        / redef()
        / "noTangle" { BlockModifier::NOTANGLE }
        / "noWeave" { BlockModifier::NOWEAVE }
        / "noHeader" { BlockModifier::NOHEADER}
    rule append() -> BlockModifier
        = "+=" { BlockModifier::APPEND }
    rule redef() -> BlockModifier
        = "-=" { BlockModifier::REDEF }

    rule codeblock_header() -> (&'input str, BlockModifier)
        = named_line(
            <codeblock_delim() _ name:block_name() _ mods:possible_block_modifier()
                { (name, mods) }>)

    /// Prose

    rule prose() -> &'input str
        = $(!"@" (!(newline() / "//") [_])*)

    rule prose_line() -> &'input str
        = named_line(<prose()>)

    /// Chapter links

    rule chapter_file() -> &'input str
        = $((!"(" [_])+)
    rule chapter() -> (&'input str, &'input str)
        = "[" _ title:name() _ "](" filename:chapter_file() ")" { (title, filename) }

    /// The whole file

    pub rule partial_line() -> PartialLitLine<'input>
        = block_header:codeblock_header()
            { let (block_name, mods) = block_header;
                PartialLitLine::CodeBlockStart(block_name, mods)
            }
        / line(<codeblock_delim()>) { PartialLitLine::CodeBlockEnd }
        / line:(line_slice() / $("")) { PartialLitLine::Line(line) }

    pub rule lit_line() -> LitLine<'input>
        = command:command() { LitLine::Command(command) }
        / chapter:named_line(<chapter()>)
            { let (title, file) = chapter;
              LitLine::Chapter { title: title, file_name: file, }
            }
        / prose:prose_line() { LitLine::Prose(prose) }
}}

use self::grammar::*;

pub type ParseError = peg::error::ParseError<<str as peg::Parse>::PositionRepr>;

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
        const APPEND   = 0b00000001;
        const REDEF    = 0b00000010;
        const NOTANGLE = 0b00000100;
        const NOWEAVE  = 0b00001000;
        const NOHEADER = 0b00010000;
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

