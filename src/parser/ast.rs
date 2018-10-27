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

use parser::{ParseState, ParseResult, ParseError, get_input_file};
use parser::grammar::{LitBlock, CodeBlock, Command, BlockModifier};
use output::css::{CustomCss, CssSettings};

use std::path::{Path, PathBuf};

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! once {
        ( $variable:expr, $ifnothing:expr ) => {
            if $variable.is_some() {
                return Err(ParseError::DuplicateCommand);
            } else {
                $variable = Some($ifnothing);
            }
        };
        ( $variable:expr, $f:ident, $ifnothing:expr, $error:expr ) => {
            if $variable.$f() {
                return Err($error);
            } else {
                $variable = $ifnothing;
            }
        };
    }
    
    #[macro_export]
    macro_rules! require {
        ( $($variable:ident),* ) => {
            $(
                let $variable = $variable.ok_or(ParseError::MissingCommand(stringify!($variable)))?;
            )*
        };
    }
}

pub struct LitFile {
    pub title: String,
    pub code_type: String,
    pub file_extension: String,
    pub comment_type: Option<&'static Fn(String) -> String>,
    pub sections: Vec<Section>,
    pub compiler: Option<CompilerSettings>,
    pub book_status: BookStatus,
}

impl LitFile {
    pub fn parse<'a>(parse_state: &mut ParseState, lines: Vec<LitBlock<'a>>) -> ParseResult<(Self, CssSettings)> {
        let mut title = None;
        let mut code_type_and_file_extension = None;
        let mut comment_type = None;
        let mut compiler_command = None;
        let mut error_format = None;
        let mut is_book = false;
        let mut custom_css = CustomCss::None;
        let mut custom_colorscheme = None;
    
        let mut sections = Vec::new();
        let mut current_section = Section {
            id: 0_usize,
            name: SectionName::Implicit,
            blocks: Vec::new()
        };
    
        let mut chapters = Vec::new();
    
        for line in lines {
            match line {
                LitBlock::Command(command) => match command {
                    Command::Title(title_slice) => {
                        once!(title, title_slice.to_owned())
                    },
                    Command::CodeType { code_type: ctype, file_extension: extension } => {
                        once!(code_type_and_file_extension, (ctype.to_owned(), extension.to_owned()))
                    },
                    Command::CommentType(formatter) => {
                        once!(comment_type, generate_comment_type(formatter))
                    },
                    Command::Compiler(command) => {
                        once!(compiler_command, command.to_owned())
                    },
                    Command::ErrorFormat(formatter) => {
                        once!(error_format, generate_error_format(formatter))
                    },
                    Command::Section(name) => {
                        let last_id = current_section.id;
                        sections.push(current_section);
    
                        current_section = Section {
                            id: last_id + 1_usize,
                            name: SectionName::parse(name),
                            blocks: Vec::new()
                        };
                    },
                    Command::Book => {
                        is_book = true;
                    },
                    Command::AddCss(css_file) => {
                        once!(custom_css, is_not_none, CustomCss::Add(css_file.to_owned()), ParseError::DuplicateCssCommand)
                    },
                    Command::OverwriteCss(css_file) => {
                        once!(custom_css, is_not_none, CustomCss::Overwrite(css_file.to_owned()), ParseError::DuplicateCssCommand)
                    },
                    Command::Colorscheme(css_file) => {
                        once!(custom_colorscheme, css_file.to_owned())
                    },
                },
                LitBlock::Code(code) => {
                    current_section.blocks.push(Block::parse_code(code));
                },
                LitBlock::Prose(lines) => {
                    current_section.blocks.push(Block::parse_prose(lines));
                },
                LitBlock::Chapter { title: chapter_title, file_name: chapter_file } => {
                    let file_path = get_input_file(Path::new(chapter_file))?;
                    
                    parse_state.parse_file(&file_path)?;

                    parse_state.file_map.get_mut(&file_path).unwrap().title 
                        = chapter_title.to_owned();

                    chapters.push(file_path);
                },
            }
        }
    
        let compiler_settings = compiler_command.and_then(|command| {
            let error_format = error_format.or(try_guess_error_format(&command))?;

            Some(CompilerSettings {
                command: command, 
                formatter: error_format
            })
        });

        // Finish off the last section
        sections.push(current_section);
    
        // Error if required fields haven't been populated
        require!(title, code_type_and_file_extension);

        // Generate the book status
        let book_status = if is_book || !chapters.is_empty() {
            BookStatus::IsBook(chapters)
        } else {
            BookStatus::NotBook
        };

        let (code_type, file_extension) = code_type_and_file_extension;

        Ok((LitFile {
                title: title,
                code_type: code_type,
                file_extension: file_extension,
                comment_type: comment_type,
                sections: sections,
                compiler: compiler_settings,
                book_status: book_status,
            },
            CssSettings {
                custom_css: custom_css,
                custom_colorscheme: custom_colorscheme
            }))
    }
}

pub enum BookStatus {
    NotBook,
    IsBook(Vec<PathBuf>)
}

pub struct CompilerSettings {
    pub command: String,
    formatter: &'static Fn(String, String, String, String) -> String
}

pub enum SectionName {
    Implicit,
    Declared(String),
    None,
}

impl SectionName {
    fn parse(name: Option<&str>) -> Self {
        match name {
            Some(slice) => SectionName::Declared(slice.to_owned()),
            None => SectionName::None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            &SectionName::Declared(ref name) => Some(name.as_str()),
            &SectionName::Implicit | &SectionName::None => None,
        }
    }
}

pub struct Section {
    pub id: usize,
    pub name: SectionName,
    pub blocks: Vec<Block>
}

pub enum Block {
    Code { name: String, modifiers: BlockModifier, lines: Vec<String> },
    Prose { lines: Vec<String> }
}

impl Block {
    fn parse_code<'a>(code_block: CodeBlock<'a>) -> Self {
        Block::Code {
            name : code_block.block_name.to_owned(),
            modifiers : code_block.modifiers,
            lines : code_block.contents.into_iter().map(|slice| slice.to_owned()).collect()
        }
    }

    fn parse_prose<'a>(prose_block: Vec<&str>) -> Self {
        Block::Prose {
            lines : prose_block.into_iter().map(|slice| slice.to_owned()).collect()
        }
    }
}

fn try_guess_error_format(compiler: &String) -> Option<&'static Fn(String, String, String, String) -> String> {
    None
}

fn generate_error_format(format_string: &str) -> &'static (Fn(String, String, String, String) -> String) {
    panic!()
}

fn generate_comment_type(format_string: &str) -> &'static (Fn(String) -> String) {
    panic!()
}
