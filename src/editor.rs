use crate::terminal::CursorStyle;
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
    // Visual,
}

impl Mode {
    fn to_string(&self) -> String {
        match self {
            Self::Normal => String::from("Normal"),
            Self::Insert => String::from("Insert"),
            Self::Command => String::from("Command"),
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
    command_buffer: String,
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
            terminal: Terminal::default(),
            document: if args.len() > 1 {
                match Document::open(&args[1]) {
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
            command_buffer: String::new(),
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::hide_cursor();
        {
            let position = &Position::default();
            #[allow(clippy::cast_possible_truncation)]
            let Position { mut x, mut y } = position;
            x = x.saturating_add(1);
            y = y.saturating_add(1);

            let x = x as u16;
            let y = y as u16;

            print!("{}", termion::cursor::Goto(x, y));
        };
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            if !matches!(self.mode, Mode::Command) {
                {
                    let position = &Position {
                        x: self.cursor_position.x.saturating_sub(self.offset.x),
                        y: self.cursor_position.y.saturating_sub(self.offset.y),
                    };
                    #[allow(clippy::cast_possible_truncation)]
                    let Position { mut x, mut y } = position;
                    x = x.saturating_add(1);
                    y = y.saturating_add(1);

                    let x = x as u16;
                    let y = y as u16;

                    print!("{}", termion::cursor::Goto(x, y));
                };
            }
        }
        Terminal::show_cursor();
        Terminal::flush()
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match self.mode {
            // While in normal mode
            Mode::Normal => match pressed_key {
                // Command mutators
                Key::Char('i') => {
                    self.mode = Mode::Insert;
                    Terminal::change_cursor_style(&CursorStyle::Bar);
                }
                Key::Char('a') => {
                    self.move_cursor(Key::Right);
                    self.mode = Mode::Insert;
                    Terminal::change_cursor_style(&CursorStyle::Bar);
                }
                Key::Char(':') => {
                    self.mode = Mode::Command;
                    self.status_message = StatusMessage::from(String::from(":"));
                }
                // Key::Char('v') => self.mode = Mode::Visual,
                Key::Up
                | Key::Down
                | Key::Left
                | Key::Right
                | Key::Char('h')
                | Key::Char('j')
                | Key::Char('k')
                | Key::Char('l')
                | Key::Backspace
                | Key::PageUp
                | Key::PageDown
                | Key::End
                | Key::Home => self.move_cursor(pressed_key),
                Key::Ctrl('q') => self.should_quit = true,
                _ => (),
            },

            // While in insert mode
            Mode::Insert => match pressed_key {
                // Mode mutators
                Key::Esc => {
                    self.mode = Mode::Normal;
                    self.move_cursor(Key::Left);
                    Terminal::change_cursor_style(&CursorStyle::Block);
                }
                // Movement keys
                Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(pressed_key),
                // Insertable characters
                Key::Char(c) => {
                    self.document.insert(&self.cursor_position, c);
                    self.move_cursor(Key::Right);
                }
                // Deletion
                Key::Delete => self.document.delete(&self.cursor_position),
                Key::Backspace => {
                    if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                        self.move_cursor(Key::Left);
                        self.document.delete(&self.cursor_position);
                    }
                }
                _ => (),
            },

            // While in command mode
            Mode::Command => match pressed_key {
                Key::Backspace => {
                    self.command_buffer.pop();
                    self.status_message = StatusMessage::from(format!(":{}", self.command_buffer));
                }
                Key::Esc => {
                    self.mode = Mode::Normal;
                    self.command_buffer.clear();
                    Terminal::change_cursor_style(&CursorStyle::Block);
                }
                Key::Char('\n') => {
                    let command_buffer_args = self
                        .command_buffer
                        .split_ascii_whitespace()
                        .collect::<Vec<&str>>();

                    let force = command_buffer_args.get(0).unwrap().ends_with('!');

                    match command_buffer_args[0] {
                        "q" | "q!" => {
                            if self.document.is_dirty() && !force {
                                self.status_message = StatusMessage::from(
                                    "File has unsaved changes. Use :wq to save and quit, or :q! to quit without saving.".to_string(),
                                );
                            } else {
                                self.should_quit = true;
                            }
                        }
                        "w" => {
                            if command_buffer_args.len() > 1 {
                                self.document.file_name = Some(command_buffer_args[1].to_string());
                            }
                            self.save();
                        }
                        "wq" => {
                            if command_buffer_args.len() > 1 {
                                self.document.file_name = Some(command_buffer_args[1].to_string());
                            }
                            self.save();
                            self.should_quit = true;
                        }
                        _ => {
                            self.status_message = StatusMessage::from(format!(
                                "Unrecognized command: {}",
                                command_buffer_args[0]
                            ))
                        }
                    }
                    self.mode = Mode::Normal;
                    self.command_buffer.clear();
                }
                Key::Char(c) => {
                    self.command_buffer.push(c);
                    self.status_message = StatusMessage::from(format!(":{}", self.command_buffer));
                }
                _ => (),
            },
        }

        self.scroll();
        Ok(())
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            self.status_message = StatusMessage::from("No filename provided".to_string());
            return;
        }
        self.document.save().unwrap();
        self.status_message = StatusMessage::from(format!(
            "Wrote to \"{}\"",
            self.document.file_name.as_ref().unwrap()
        ));
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
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
        let terminal_height = self.terminal.size().height as usize;
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
            Key::Left | Key::Char('h') | Key::Backspace => {
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
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            Key::Home | Key::Char('0') => x = 0,
            Key::End | Key::Char('$') => x = width,
            _ => (),
        }

        // If the cursor is at the end of a line, it should stay there when the
        // user presses the down arrow key.
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
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row) {
        let terminal_width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(terminal_width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
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
        let width = self.terminal.size().width as usize;
        let mut file_name = "[No Name]".to_string();
        let dirty_indicator = if self.document.is_dirty() { " [+]" } else { "" };
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!("{}{}", file_name, dirty_indicator,);

        let mode_indicator: String = format!(" [ {} ] ", self.mode.to_string());

        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(
                &" ".repeat(
                    width
                        .saturating_sub(len)
                        .saturating_sub(mode_indicator.len()),
                ),
            );
        }
        status = format!("{status}{mode_indicator}{line_indicator}");
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
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }
}

fn die(_e: std::io::Error) {
    Terminal::clear_screen();
    exit(0);
}
