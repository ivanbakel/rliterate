pub enum WeaveType {
    Markdown,
    HtmlViaMarkdown(String),
    StraightToHtml,
}

pub fn weave_file(weave_type: &WeaveType, file_name: &PathBuf, file: &LitFile) {
}

