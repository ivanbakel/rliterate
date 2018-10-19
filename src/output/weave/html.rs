use output::{OutputResult};
use output::css;

use maud::html;
use maud::{DOCTYPE, PreEscaped};

use std::fs;
use std::io::{Write};

static default_css_style : &'static str = include_str!("default.css");
static katex_html : &'static str = include_str!("katex.html");

pub fn print(mut file: fs::File, html: String, title: &str, css: &css::CssSettings) -> OutputResult<()> {
    let markup = html! {
        DOCTYPE;
        head {
            title { (title) };
            (PreEscaped(katex_html))
            (match css.custom_css {
                css::CustomCss::None => html! {},
                css::CustomCss::Add(ref file_path) => {
                    html! {
                        style {
                            (default_css_style)
                            (fs::read_to_string(file_path)?)
                        }
                    }
                },
                css::CustomCss::Overwrite(ref file_path) => {
                    html! {
                        style {
                            (fs::read_to_string(file_path)?)
                        }
                    }
                },
            })
            (if let Some(ref file_path) = css.custom_colorscheme {
                html! {
                    style {
                        (fs::read_to_string(file_path)?)
                    }
                }
            } else {
                html! {}
            })
        };
        body {
            (PreEscaped(html))
        };
    };

    write!(file, "{}", markup.into_string())?;
    
    Ok(())
}
