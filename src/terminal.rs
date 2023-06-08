use crate::Position;
use std::default;
use std::io::{self, Write};
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

pub struct Size {
    pub height: u16,
    pub width: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<std::io::Stdout>,
}

pub enum CursorStyle {
    Bar,
    Block,
    Underline,
}

impl default::Default for Terminal {
    fn default() -> Terminal {
        let size = termion::terminal_size().unwrap();
        Terminal {
            size: Size {
                width: size.0,
                height: size.1.saturating_sub(2),
            },
            _stdout: io::stdout().into_raw_mode().unwrap(),
        }
    }
}

impl Terminal {
    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn cursor_position(position: &Position) {
        #[allow(clippy::cast_possible_truncation)]
        let Position { mut x, mut y } = position;
        x = x.saturating_add(1);
        y = y.saturating_add(1);

        let x = x as u16;
        let y = y as u16;

        print!("{}", termion::cursor::Goto(x, y));
    }

    pub fn flush() -> Result<(), std::io::Error> {
        io::stdout().flush()
    }

    pub fn read_key() -> Result<Key, std::io::Error> {
        loop {
            if let Some(key) = io::stdin().lock().keys().next() {
                return key;
            }
        }
    }

    pub fn hide_cursor() {
        print!("{}", termion::cursor::Hide);
    }

    pub fn show_cursor() {
        print!("{}", termion::cursor::Show);
    }

    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }

    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }

    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color));
    }

    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }

    pub fn change_cursor_style(style: &CursorStyle) {
        match *style {
            CursorStyle::Bar => print!("{}", cursor::BlinkingBar),
            CursorStyle::Block => print!("{}", cursor::SteadyBlock),
            CursorStyle::Underline => print!("{}", cursor::SteadyUnderline),
        }
    }
}
