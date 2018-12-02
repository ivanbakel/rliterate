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
use std::fmt;

pub struct Settings {
    pub custom_css: CustomCss,
    pub custom_colorscheme: Option<String>,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "css file: {}, colorscheme file: {}", self.custom_css, 
        if let Some(ref cs) = self.custom_colorscheme {
            cs
        } else {
            "none"
        })
    }
}

impl Settings {
    pub fn is_default(&self) -> bool {
        !self.custom_css.is_not_none() && self.custom_colorscheme.is_none()
    }

    pub fn default() -> Self {
        Settings {
            custom_css: CustomCss::None,
            custom_colorscheme: None,
        }
    }
}

pub enum CustomCss {
    None,
    Add(String),
    Overwrite(String),
}

impl fmt::Display for CustomCss {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CustomCss::None => write!(f, "none"),
            CustomCss::Add(ref add_file) => write!(f, "appending {}", add_file),
            CustomCss::Overwrite(ref overwrite_file) => write!(f, "overwriting by {}", overwrite_file),
        }
    }
}

impl CustomCss {
    pub fn is_not_none(&self) -> bool {
        match self {
            &CustomCss::None => false,
            _ => true
        }
    }
}

