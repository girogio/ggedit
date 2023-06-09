use termion::color::{self, Bg, Fg};

#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
    SearchMatch,
}

impl Type {
    pub fn to_bg_color(&self) -> Option<color::Bg<color::Rgb>> {
        match self {
            Type::None => None,
            Type::Number => None,
            Type::SearchMatch => Some(Bg(color::Rgb(255, 255, 0))),
        }
    }

    pub fn to_fg_color(&self) -> Option<color::Fg<color::Rgb>> {
        match self {
            Type::None => None,
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
                Some(color) => color.to_string(),
                None => Bg(color::Reset).to_string(),
            },
            match self.to_fg_color() {
                Some(color) => color.to_string(),
                None => Fg(color::Reset).to_string(),
            }
        )
    }
}
