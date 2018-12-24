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

use output;
use output::css;

use maud::html;
use maud::{DOCTYPE, PreEscaped};

use std::fs;
use std::io::{Write};

static DEFAULT_CSS_STYLE : &'static str = include_str!("default.css");
static KATEX_HTML : &'static str = include_str!("katex.html");

pub fn print(mut file: fs::File, html: String, title: &str, css: &css::Globals) -> output::Result<()> {
    let markup = html! {
        (DOCTYPE);
        head {
            title { (title) };
            (PreEscaped(KATEX_HTML))
            (match css.custom_css {
                css::CustomCss::None => html! {},
                css::CustomCss::Add(ref file_path) => {
                    html! {
                        style {
                            (DEFAULT_CSS_STYLE)
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
