use output::{OutputResult};
use link::{LinkedFile};

use std::path::{PathBuf};

pub enum Type {
    Markdown,
    HtmlViaMarkdown(String),
    StraightToHtml,
}

pub fn weave_file<'a>(weave_type: &Type, file_name: &PathBuf, file: &LinkedFile<'a>) -> OutputResult<()> {
    Ok(())
}

