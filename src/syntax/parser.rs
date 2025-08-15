use std::{env, fs, io::Read, path::PathBuf, process::exit};

use super::Token;

/// Represents the tokenized contents of a schema file with parsing state.
///
/// FileContents handles the reading, tokenization, and parsing of repack schema files.
/// It maintains the current parsing position and provides methods for token consumption
/// during the parsing process. Supports both single files and directory wildcards.
pub struct FileContents {
    /// The tokenized representation of the file contents
    pub contents: Vec<Token>,
    /// The root directory path for resolving relative file inclusions
    pub root: String,
    /// Current parsing position in the token stream
    pub index: usize,
}

impl FileContents {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        FileContents {
            contents: Vec::new(),
            root: env::current_dir().unwrap().to_str().unwrap().to_string(),
            index: 0,
        }
    }
    /// Creates a new FileContents by reading and tokenizing the specified file.
    ///
    /// This method reads the file, extracts its directory as the root path,
    /// and tokenizes the contents for parsing. The root path is used for
    /// resolving relative file inclusions.
    ///
    /// # Arguments
    /// * `filename` - Path to the schema file to read
    ///
    /// # Returns
    /// A new FileContents instance ready for parsing
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

    /// Adds additional file contents relative to the root directory.
    ///
    /// This method supports both individual files and directory wildcards (ending with *).
    /// For wildcards, it reads all .repack files in the specified directory.
    /// Used for processing include directives in schema files.
    ///
    /// # Arguments
    /// * `filename` - Relative path to file or directory pattern to include
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

    /// Reads and tokenizes a specific file, appending its tokens to the contents.
    ///
    /// This method handles the low-level file reading and tokenization process,
    /// including comment parsing, string literal handling, and token recognition.
    /// The tokenization process respects quoted strings and line comments (//).
    ///
    /// # Arguments
    /// * `filename` - Absolute path to the file to read and tokenize
    pub fn add(&mut self, filename: &str) {
        let Ok(mut file) = std::fs::File::open(filename) else {
            println!("[EXIT] Unable to load requested file '{filename}'");
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

    /// Returns the current token without advancing the parsing position.
    ///
    /// Used for lookahead parsing to make decisions based on upcoming tokens
    /// without consuming them from the stream.
    ///
    /// # Returns
    /// * `Some(&Token)` if there are more tokens to parse
    /// * `None` if the end of the token stream has been reached
    pub fn peek(&self) -> Option<&Token> {
        if self.index < self.contents.len() {
            Some(&self.contents[self.index])
        } else {
            None
        }
    }

    /// Returns the current token and advances the parsing position by one.
    ///
    /// This is the primary method for consuming tokens during parsing.
    /// Returns a reference to the token, which is suitable for pattern matching.
    ///
    /// # Returns
    /// * `Some(&Token)` if a token was consumed
    /// * `None` if the end of the token stream has been reached
    pub fn next(&mut self) -> Option<&Token> {
        if self.index < self.contents.len() {
            let token = self.contents.get(self.index)?;
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Returns an owned copy of the current token and advances parsing position.
    ///
    /// Similar to `next()` but returns an owned Token instead of a reference.
    /// Useful when the token needs to be stored or moved rather than just examined.
    ///
    /// # Returns
    /// * `Some(Token)` if a token was consumed
    /// * `None` if the end of the token stream has been reached
    pub fn take(&mut self) -> Option<Token> {
        if self.index < self.contents.len() {
            let token = self.contents.get(self.index)?;
            self.index += 1;
            Some(token.clone())
        } else {
            None
        }
    }

    pub fn take_literal(&mut self) -> Option<String> {
        match self.take() {
            Some(Token::Literal(val)) => Some(val),
            _ => None,
        }
    }

    pub fn take_colon(&mut self) -> bool {
        matches!(self.take(), Some(Token::Colon))
    }

    pub fn peek_equals(&mut self) -> bool {
        matches!(self.peek(), Some(Token::Equal))
    }

    /// Advances the parsing position by one without returning the token.
    ///
    /// Used when a token needs to be consumed but its value is not needed,
    /// such as skipping expected punctuation like parentheses or dots.
    pub fn skip(&mut self) {
        self.index += 1;
    }

    #[allow(dead_code)]
    pub fn add_string(&mut self, string: &str) {
        let contents = string.bytes();

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
}
