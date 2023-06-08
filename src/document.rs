use termion::color;

use crate::Position;
use crate::Row;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
}

// open with overriden file_name
impl From<&str> for Document {
    fn from(s: &str) -> Self {
        Self {
            rows: Vec::new(),
            file_name: if s.is_empty() {
                Default::default()
            } else {
                Some(s.to_string())
            },
            dirty: true,
        }
    }
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
            dirty: false,
        })
    }

    pub fn save_as(&mut self, filename: Option<&&str>) -> Result<String, Error> {
        if self.is_empty() && !self.is_dirty() {
            return Err(Error::new(std::io::ErrorKind::Other, "Document is empty"));
        }
        let prev_name = self.file_name.clone();
        match filename {
            Some(filename) => {
                if let Some(file_name) = &self.file_name {
                    // if the file name is the same as the current file name
                    // then just save the file
                    if filename == file_name {
                        self.dirty = false;
                        self.save()
                    } else {
                        self.file_name = Some(filename.to_string()); // change the file name TEMPORARILY
                        match self.save() {
                            // then save the file
                            Ok(message) => {
                                self.file_name = prev_name; // change the file name back
                                Ok(message)
                            }
                            Err(e) => {
                                self.file_name = prev_name; // change the file name back
                                Err(e)
                            }
                        }
                    }
                } else {
                    self.file_name = Some(filename.to_string());
                    self.dirty = false;
                    self.save()
                }
            }

            None => match self.save() {
                Ok(message) => {
                    self.dirty = false;
                    Ok(message)
                }
                Err(e) => Err(e),
            },
        }
    }

    pub fn save(&mut self) -> Result<String, Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            Ok(format!(
                "\"{}\" {}L, {}B written",
                file_name,
                self.len(),
                self.size_in_bytes()
            ))
        } else {
            Err(Error::new(
                std::io::ErrorKind::Other,
                "Document has no file name",
            ))
        }
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.rows.len() {
            return;
        }
        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            #[allow(clippy::indexing_slicing)]
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
        }
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.rows.len();
        if at.y >= len {
            return;
        }
        self.dirty = true;
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }

    pub fn insert_newline(&mut self, at: &Position) {
        match Ord::cmp(&at.y, &self.rows.len()) {
            std::cmp::Ordering::Less => {
                #[allow(clippy::indexing_slicing)]
                let new_row = self.rows[at.y].split(at.x);
                #[allow(clippy::integer_arithmetic)]
                self.rows.insert(at.y + 1, new_row);
            }
            std::cmp::Ordering::Equal => {
                self.rows.push(Row::default());
            }
            std::cmp::Ordering::Greater => {}
        }
    }

    pub fn find(&self, query: &str, after: &Position) -> Option<Position> {
        let mut x = after.x;
        for (y, row) in self.rows.iter().enumerate().skip(after.y) {
            if let Some(x) = row.find(query, x) {
                return Some(Position { x, y });
            }
            x = 0;
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn size_in_bytes(&self) -> usize {
        let mut size = 0;
        for row in &self.rows {
            size += row.len() + 1;
        }
        size
    }
}
