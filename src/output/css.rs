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

pub struct CssSettings {
    pub custom_css: CustomCss,
    pub custom_colorscheme: Option<String>,
}

impl CssSettings {
    pub fn is_default(&self) -> bool {
        !self.custom_css.is_not_none() && self.custom_colorscheme.is_none()
    }

    pub fn default() -> Self {
        CssSettings {
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

impl CustomCss {
    pub fn is_not_none(&self) -> bool {
        match self {
            &CustomCss::None => false,
            _ => true
        }
    }
}

