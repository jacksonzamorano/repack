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
        let mut file = std::fs::File::open(filename).expect("Unable to open file");
        let mut contents = vec![];
        _ = file.read_to_end(&mut contents);

        let mut buf: String = String::new();
        for byte in contents {
            match Token::from_byte(byte) {
                Some(token) => {
                    if !buf.is_empty() {
                        self.contents.push(Token::from_string(&buf));
                        buf.clear();
                    }
                    self.contents.push(token);
                }
                None => {
                    if !byte.is_ascii_whitespace() {
                        buf.push(byte as char);
                    } else if !buf.is_empty() {
                        // Handle the buffer content
                        self.contents.push(Token::from_string(&buf));
                        buf.clear();
                    }
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

    pub fn take(&mut self) -> Option<Token> {
        if self.index < self.contents.len() {
            let token = self.contents.get(self.index)?;
            self.index += 1;
            Some(token.clone())
        } else {
            None
        }
    }

    pub fn skip(&mut self) {
        self.index += 1;
    }
}
