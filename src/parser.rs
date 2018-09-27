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

pub struct CodeBlock<'a> {
    pub block_name: &'a str,
    pub modifiers: BlockModifier,
    pub contents: Vec<&'a str>
}

pub enum LitBlock<'a> {
    Command(Command<'a>),
    Code(CodeBlock<'a>),
    Prose(Vec<&'a str>)
}

