use std::{fs, io::Read, path::PathBuf, process::exit};

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
        if filename.ends_with("*") {
            path.push(filename);
            path.pop();
            let Ok(mut folder_contents) = fs::read_dir(&path) else {
                println!(
                    "[EXIT] Unable to load requested folder '{}'",
                    path.to_str().unwrap()
                );
                exit(5);
            };
            while let Some(Ok(file)) = folder_contents.next() {
                let path = file.path();
                if let Some(extension) = path.extension() {
                    if extension == "repack" {
                        self.add(path.to_str().unwrap());
                    }
                }
            }
        } else {
            path.push(filename);
            self.add(path.to_str().unwrap())
        }
    }

    pub fn add(&mut self, filename: &str) {
        let Ok(mut file) = std::fs::File::open(filename) else {
            println!("[EXIT] Unable to load requested file '{}'", filename);
            exit(5);
        };
        let mut contents = vec![];
        _ = file.read_to_end(&mut contents);

        let mut iter = contents.into_iter().peekable();

        let mut buf: String = String::new();
        let mut in_comment = false;
        let mut in_quote = false;
        loop {
            let Some(byte) = iter.next() else {
                break;
            };
            if byte == b'"' {
                if in_quote {
                    self.contents.push(Token::Literal(buf));
                    buf = String::new();
                } else if !buf.is_empty() {
                    let token = Token::from_string(&buf);
                    self.contents.push(token);
                }
                in_quote = !in_quote;
                continue;
            }
            if in_quote {
                buf.push(byte as char);
            } else {
                if byte == b'/' {
                    if let Some(next_byte) = iter.peek() {
                        if *next_byte == b'/' {
                            in_comment = true;
                            continue;
                        }
                    }
                }
                if !in_comment {
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
                                self.contents.push(Token::from_string(&buf));
                                buf.clear();
                            }
                        }
                    }
                } else if byte == b'\n' || byte == b'\r' {
                    in_comment = false;
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
