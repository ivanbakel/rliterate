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
use parser::{ParseState, get_input_file};
use parser::grammar::{LitBlock, CodeBlock, Command, BlockModifier};
use output::css;
use output::css::{CustomCss};

use std::path::{Path, PathBuf};

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! once {
        ( $variable:expr, $ifnothing:expr ) => {
            if $variable.is_some() {
                return Err(parser::Error::DuplicateCommand);
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
                let $variable = $variable.ok_or(parser::Error::MissingCommand(stringify!($variable)))?;
            )*
        };
    }
}

pub type FormatFn<T> = Box<dyn Fn(T) -> String>;

pub struct Metadata {
    pub relative_directory: PathBuf,
    pub title: String,
    pub code_type: String,
    pub file_extension: String,
    pub comment_type: Option<FormatFn<String>>,
    pub line_number_format: Option<FormatFn<usize>>,
    pub compiler: Option<CompilerSettings>,
    pub book_status: BookStatus,
}

pub struct LitFile {
    pub metadata: Metadata,
    pub sections: Vec<Section>,
}

impl std::ops::Deref for LitFile {
    type Target = Metadata;

    fn deref(&self) -> &Metadata {
        &self.metadata
    }
}


impl LitFile {
    pub fn parse<'a>(parse_state: &mut ParseState, lines: Vec<LitBlock<'a>>) -> parser::Result<(Self, css::Globals)> {
        let mut title = None;
        let mut code_type_and_file_extension = None;
        let mut comment_type = None;
        let mut compiler_command = None;
        let mut error_format = None;
        let mut line_number_format = None;
        let mut is_book = false;
        let mut custom_css = CustomCss::None;
        let mut custom_colorscheme = None;
    
        let mut sections = Vec::new();
        let mut current_section = Section {
            id: 0_usize,
            depth: 0_usize,
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
                    Command::LineNumbers(formatter) => {
                        once!(line_number_format, generate_line_number_format(formatter))
                    },
                    Command::Section { name, depth } => {
                        let last_id = current_section.id;
                        sections.push(current_section);
                        
                        debug!("Started a new section named \"{:?}\", id: {}", 
                               name,
                               last_id + 1);
    
                        current_section = Section {
                            id: last_id + 1_usize,
                            depth: depth,
                            name: SectionName::parse(name),
                            blocks: Vec::new()
                        };
                    },
                    Command::Book => {
                        is_book = true;
                    },
                    Command::AddCss(css_file) => {
                        once!(custom_css, is_not_none, CustomCss::Add(css_file.to_owned()), parser::Error::DuplicateCssCommand)
                    },
                    Command::OverwriteCss(css_file) => {
                        once!(custom_css, is_not_none, CustomCss::Overwrite(css_file.to_owned()), parser::Error::DuplicateCssCommand)
                    },
                    Command::Colorscheme(css_file) => {
                        once!(custom_colorscheme, css_file.to_owned())
                    },
                },
                LitBlock::Code(code) => {
                    debug!("Added a code block to section \"{:?}\", id: {}", 
                           current_section.name.as_str(),
                           current_section.id);
                    current_section.blocks.push(Block::parse_code(code));
                },
                LitBlock::Prose(lines) => {
                    debug!("Added a prose block to section \"{:?}\", id: {}", 
                           current_section.name.as_str(),
                           current_section.id);
                    current_section.blocks.push(Block::parse_prose(lines));
                },
                LitBlock::Chapter { title: chapter_title, file_name: chapter_file } => {
                    let file_path = get_input_file(Path::new(chapter_file))?;
                   
                    if file_path.is_absolute() {
                      return Err(parser::Error::BadChapterPath);
                    }
                    //Constrain the borrow region
                    {
                        let relative_directory = file_path.parent().unwrap();
                        parse_state.parse_file(&file_path, &relative_directory)?;
                    }

                    parse_state.file_map.get_mut(&file_path).unwrap().set_title(chapter_title.to_owned());

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
                metadata: Metadata {
                    relative_directory: PathBuf::new(),
                    title: title,
                    code_type: code_type,
                    file_extension: file_extension,
                    comment_type: comment_type,
                    line_number_format: line_number_format,
                    compiler: compiler_settings,
                    book_status: book_status,
                },
                sections: sections,
            },
            css::Globals {
                custom_css: custom_css,
                custom_colorscheme: custom_colorscheme
            }))
    }

    fn set_title(&mut self, new_title: String) {
        self.metadata.title = new_title;
    }
}

pub enum BookStatus {
    NotBook,
    IsBook(Vec<PathBuf>)
}

pub struct CompilerSettings {
    pub command: String,
    #[allow(dead_code)] // TODO: make use of this field
    formatter: &'static dyn Fn(String, String, String, String) -> String
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
    pub depth: usize,
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

fn try_guess_error_format(_compiler: &String) -> Option<&'static dyn Fn(String, String, String, String) -> String> {
    None
}

fn generate_error_format(_format_string: &str) -> &'static dyn Fn(String, String, String, String) -> String {
    panic!()
}

fn generate_comment_type(format_string: &str) -> FormatFn<String> {
  let owned_format = format_string.to_owned();
  Box::new(move |comment| { 
    owned_format.replace("%s", &comment)
  })
}

pub fn generate_line_number_format(format_string: &str) -> FormatFn<usize> {
  let owned_format = format_string.to_owned();
  Box::new(move |line_number| { 
    owned_format.replace("%l", &line_number.to_string())
  })
}
