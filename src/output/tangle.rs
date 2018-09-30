use output::{OutputResult};

pub struct Settings {
    compile: bool,
    line_numbers: Option<&'static Fn(usize) -> String>,
}
    
pub fn tangle_file(settings: &Settings, file: &LitFile) -> OutputResult<()> {
    let canonical_code_blocks = canonicalise_code_blocks(&file.sections);

    for (name, block) in canonical_code_blocks.iter() {
        if block.is_file() {
            fs::OpenOptions::new().write(true).open(name)?;
        }
    }
}

struct CanonicalCodeBlock<'a> {
    print_header: bool,
    print_to_tangle: bool,
    print_to_weave: bool,
    form: CCBForm<'a>,
    contents: Vec<&'a String>,
}

impl<'a> CanonicalCodeBlock<'a> {
    fn from_form(form : CCBForm<'a>) -> Self {
        CanonicalCodeBlock {
            print_header: false,
            print_to_tangle: false,
            print_to_weave: false,
            form: form,
            contents: Vec::new(),
        }
    }

    fn is_file(&self) -> bool {
        match self.form {
            &CCBForm::File => true,
            _ => false,
        }
    }

    fn append_lines(&mut self, lines: &'a [String]) {
        self.contents.extend(lines.iter());
    }

    fn replace_lines(&mut self, lines: &'a [String]) {
        self.contents.clear();
        self.append_lines(lines);
    }

