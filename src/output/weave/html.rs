use output::{OutputResult};
use output::css;

use maud::html;
use maud::{DOCTYPE, PreEscaped};

use std::fs;
use std::io::{Write};

pub fn print(mut file: fs::File, html: String, css: &css::CssSettings) -> OutputResult<()> {
    let markup = html! {
        DOCTYPE;
        head {};
        body {
            (PreEscaped(html))
        };
    };

    write!(file, "{}", markup.into_string())?;
    
    Ok(())
}
