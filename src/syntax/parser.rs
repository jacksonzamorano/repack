use std::io::Read;

use super::Token;

pub struct FileContents {
    pub contents: Vec<Token>,
    pub index: usize,
}

impl FileContents {
    pub fn new() -> Self {
        FileContents {
            contents: Vec::new(),
            index: 0,
        }
    }

    pub fn read(&mut self, filename: &str) {
        let file = std::fs::File::open(filename).expect("Unable to open file");
        let contents = file.bytes();

        let mut buf: String = String::new();
        for byte in contents {
            match byte {
                Ok(b) => {
                    match Token::from_byte(b) {
                        Some(token) => {
                            if !buf.is_empty() {
                                self.contents.push(Token::from_string(&buf));
                                buf.clear();
                            }
                            self.contents.push(token);
                        }
                        None => {
                            if !b.is_ascii_whitespace() {
                                buf.push(b as char);
                            } else if !buf.is_empty() {
                                // Handle the buffer content
                                self.contents.push(Token::from_string(&buf));
                                buf.clear();
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading byte: {}", e);
                }
            }
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        if self.index < self.contents.len() {
            Some(&self.contents[self.index])
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Option<&Token> {
        if self.index < self.contents.len() {
            let token = self.contents.get(self.index)?;
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }
}