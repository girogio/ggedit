use termion::color::{self, Bg, Color};

#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
    SearchMatch,
}

impl Type {
    pub fn to_bg_color(&self) -> Option<color::Rgb> {
        match self {
            Type::None => None,
            Type::Number => None,
            Type::SearchMatch => Some(color::Rgb(255, 255, 0)),
        }
    }

    pub fn to_fg_color(&self) -> Option<color::Rgb> {
        match self {
            Type::None => None,
            Type::Number => Some(color::Rgb(232, 165, 165)),
            Type::SearchMatch => Some(color::Rgb(0, 0, 0)),
        }
    }
}
