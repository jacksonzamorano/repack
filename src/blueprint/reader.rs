use std::iter::Peekable;

use crate::blueprint::FlyToken;

use super::{SnippetDetails, SnippetMainTokenName};

pub struct BlueprintFileReader<'a> {
    pub reader: Peekable<std::slice::Iter<'a, u8>>,
}
impl<'a> BlueprintFileReader<'a> {
    pub fn next(&mut self) -> Option<FlyToken> {
        let mut temp = String::new();

        while let Some(next) = self.reader.next() {
            if temp.is_empty() && *next == b'\n' {
                continue
            }
            if *next == b'[' {
                let mut sd = SnippetDetails::default();
                if matches!(self.reader.peek(), Some(b' ')) {
                    self.reader.next();
                }

                if matches!(self.reader.peek(), Some(b'/')) {
                    self.reader.next();
                    for in_block_read in self.reader.by_ref() {
                        match *in_block_read as char {
                            ']' => return Some(FlyToken::Close(temp)),
                            ' ' => {}
                            _ => {
                                temp.push(*in_block_read as char);
                            }
                        }
                    }
                }

                while let Some(in_block_read) = self.reader.next() {
                    match *in_block_read as char {
                        ' ' => {
                            if sd.main_token.is_empty() {
                                sd.main_token = temp;
                            } else if sd.secondary_token.is_empty() {
                                sd.secondary_token = temp;
                            } else {
                                sd.contents.push_str(&temp);
                                match self.reader.peek() {
                                    Some(b'}') => {}
                                    _ => {
                                        sd.contents.push(' ');
                                    }
                                }
                            }
                            temp = String::new();
                        }
                        ']' => {
                            if sd.main_token.is_empty() {
                                sd.main_token = temp;
                            } else if sd.secondary_token.is_empty() {
                                sd.secondary_token = temp;
                            } else {
                                sd.contents.push_str(&temp);
                            }
                            if let SnippetMainTokenName::Variable(_) = SnippetMainTokenName::from_string(&sd.main_token) { sd.autoclose = true }
                            if !sd.autoclose {
                                while let Some(tok) = self.reader.peek() {
                                    match tok {
                                        b'\n' => _ = self.reader.next(),
                                        _ => break,
                                    }
                                }
                            }
                            break;
                        }
                        ':' if sd.secondary_token.is_empty() => {
                            sd.secondary_token = temp;
                            temp = String::new();
                            if matches!(self.reader.peek(), Some(b' ')) {
                                self.reader.next();
                            }
                        }
                        _ => {
                            temp.push(*in_block_read as char);
                        }
                    }
                }
                return Some(FlyToken::Snippet(sd));
            }
            temp.push(*next as char);
            if matches!(self.reader.peek(), Some(b'[')) {
                while temp.ends_with('\n') {
                    temp.pop();
                }
                // End of a token, just before a block specifier.
                return Some(FlyToken::Literal(temp));
            }
        }

        None
    }
}
