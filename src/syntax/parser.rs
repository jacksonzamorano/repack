use std::{io::Read, path::PathBuf};

use super::Token;

pub struct FileContents {
    pub contents: Vec<Token>,
    pub root: String,
    pub index: usize,
}

impl FileContents {
    pub fn new(filename: &str) -> Self {
        let mut path = PathBuf::from(filename);
        path.pop();
        let mut contents = FileContents {
            contents: Vec::new(),
            root: path.to_str().unwrap().to_string(),
            index: 0,
        };
        contents.add(filename);
        contents
    }

    pub fn add_relative(&mut self, filename: &str) {
        let mut path = PathBuf::from(&self.root);
        path.push(filename);
        self.add(path.to_str().unwrap())
    }

    pub fn add(&mut self, filename: &str) {
        let mut file = std::fs::File::open(filename).expect("Unable to open file");
        let mut contents = vec![];
        _ = file.read_to_end(&mut contents);

        let mut buf: String = String::new();
        let mut in_quote: bool = false;
        for byte in contents {
            if byte == b'"' {
                if in_quote {
                    self.contents.push(Token::Literal(buf));
                    buf = String::new();
                } else if !buf.is_empty() {
                    self.contents.push(Token::from_string(&buf));
                }
                in_quote = !in_quote;
                continue;
            }
            if in_quote {
                buf.push(byte as char);
            } else {
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
