// Load the grammar file
include!(concat!(env!("OUT_DIR"), "/literate.rs"));

pub enum Command<'a> {
    Title(&'a str),
    Section(Option<&'a str>),
    CodeType { code_type:&'a str, file_extension:&'a str},
    CommentType(&'a str),
    Compiler(&'a str),
    ErrorFormat(&'a str),
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

    let mut code_block = None;
    let mut prose_block = None;
    
    for line in lines.into_iter() {
        match line {
            PartialLitLine::CodeBlockStart(name, modifiers) => {
                if let Some(current_block) = code_block {
                    blocks.push(LitBlock::Code(current_block));
                }

                if let Some(current_block) = prose_block {
                    blocks.push(LitBlock::Prose(current_block));

                    prose_block = None;
                }

                code_block = Some(CodeBlock {
                    block_name: name,
                    modifiers: modifiers,
                    contents: vec![],
                });
            },
            PartialLitLine::CodeBlockEnd => {
                if let Some(current_block) = code_block {
                    blocks.push(LitBlock::Code(current_block));
                    code_block = None;
                } else {
                    panic!()
                }
            },
            PartialLitLine::Line(line) => {
                if let Some(ref mut current_block) = code_block {
                    current_block.contents.push(line);
                } else {
                    let line = lit_line(line)?;

                    match line {
                        LitLine::Command(command) => {
                            blocks.push(LitBlock::Command(command));
                        },
                        LitLine::Chapter { title, file_name } => {
                            blocks.push(LitBlock::Chapter { title, file_name });
                        },
                        LitLine::Prose(line) => {
                            if let Some(ref mut lines) = prose_block {
                                lines.push(line);
                            } else {
                                prose_block = Some(vec![line]);
                            }
                        }
                    }
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

