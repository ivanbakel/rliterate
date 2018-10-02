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

