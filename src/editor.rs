use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::process::exit;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: color::Rgb = color::Rgb(255, 255, 255);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(23, 23, 23);
const EMPTY_LINE_COLOR: color::Rgb = color::Rgb(204, 102, 255);

pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
}

impl Mode {
    fn to_string(&self) -> String {
        match self {
            Self::Normal => String::from("Normal"),
            Self::Insert => String::from("Insert"),
            Self::Command => String::from("Command"),
            Self::Visual => String::from("Visual"),
        }
    }
}

#[derive(Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    mode: Mode,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("Press Ctrl-Q to quit");

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document: if args.len() > 1 {
                match Document::open_file(&args[1]) {
                    Ok(doc) => doc,
                    Err(error) => {
                        initial_status = format!("Error opening file: {}", error);
                        Document::default()
                    }
                }
            } else {
                Document::default()
            },
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            mode: Mode::Normal,
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::show_cursor();
        Terminal::flush()
    }
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match self.mode {
            Mode::Normal => match pressed_key {
                Key::Char('i') => self.mode = Mode::Insert,
                // Key::Char(':') => self.mode = Mode::Command,
                // Key::Char('v') => self.mode = Mode::Visual,
                Key::Up
                | Key::Down
                | Key::Left
                | Key::Right
                | Key::Char('h')
                | Key::Char('j')
                | Key::Char('k')
                | Key::Char('l')
                | Key::PageUp
                | Key::PageDown
                | Key::End
                | Key::Home => self.move_cursor(pressed_key),
                Key::Ctrl('q') => self.should_quit = true,
                _ => (),
            },

            Mode::Insert => match pressed_key {
                Key::Esc => self.mode = Mode::Normal,
                Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(pressed_key),
                _ => (),
            },

            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().cols as usize;
        let height = self.terminal.size().rows as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut y, mut x } = self.cursor_position;
        let height = self.document.len();
        let terminal_height = self.terminal.size().rows as usize;
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up | Key::Char('k') => y = y.saturating_sub(1),
            Key::Down | Key::Char('j') => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left | Key::Char('h') => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right | Key::Char('l') => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }

        // If the cursor is at the end of a line, it should stay there when the user presses the down arrow key.
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("ggedit v{}", VERSION);
        let width = self.terminal.size().cols as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row) {
        let terminal_width = self.terminal.size().cols as usize;
        let start = self.offset.x;
        let end = self.offset.x + terminal_width;
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().rows;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                Terminal::set_fg_color(EMPTY_LINE_COLOR);
                println!("~\r");
                Terminal::reset_fg_color();
            }
        }
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().cols as usize;
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!("{} - {} lines", file_name, self.document.len());

        let mode_indicator: String = format!(" [ {} ] ", self.mode.to_string());

        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len - mode_indicator.len()));
        }
        status = format!("{}{}{}", status, mode_indicator, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().cols as usize);
            print!("{}", text);
        }
    }
}

fn die(_e: std::io::Error) {
    Terminal::clear_screen();
    exit(0);
}
