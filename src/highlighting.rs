use termion::color::{self, Bg, Fg};

#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
    SearchMatch,
    String,
    Character,
    Comment,
}

impl Type {
    pub fn to_bg_color(&self) -> Option<color::Bg<color::Rgb>> {
        match self {
            Type::None => None,
            Type::Number => None,
            Type::Character => None,
            Type::SearchMatch => Some(Bg(color::Rgb(255, 255, 0))),
            Type::String => None,
            Type::Comment => None,
        }
    }

    pub fn to_fg_color(&self) -> Option<color::Fg<color::Rgb>> {
        match self {
            Type::None => None,
            Type::Character => Some(Fg(color::Rgb(255, 234, 96))),
            Type::Comment => Some(Fg(color::Rgb(124, 124, 124))),
            Type::String => Some(Fg(color::Rgb(211, 54, 130))),
            Type::Number => Some(Fg(color::Rgb(232, 165, 165))),
            Type::SearchMatch => Some(Fg(color::Rgb(0, 0, 0))),
        }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        format!(
            "{}{}",
            match self.to_bg_color() {
                None => Bg(color::Reset).to_string(),
                Some(color) => color.to_string(),
            },
            match self.to_fg_color() {
                None => Fg(color::Reset).to_string(),
                Some(color) => color.to_string(),
            }
        )
    }
}
