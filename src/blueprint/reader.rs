use std::iter::Peekable;

use crate::blueprint::FlyToken;

use super::SnippetDetails;

pub struct BlueprintFileReader<'a> {
    pub reader: Peekable<std::slice::Iter<'a, u8>>,
}
impl<'a> BlueprintFileReader<'a> {
    pub fn next(&mut self) -> Option<FlyToken> {
        let mut temp = String::new();

        while let Some(next) = self.reader.next() {
            if *next == b'{' || *next == b'[' {
                let mut sd = SnippetDetails {
                    is_ended: *next == b'{',
                    ..Default::default()
                };
                if matches!(self.reader.peek(), Some(b' ')) {
                    self.reader.next();
                }
                match self.reader.peek() {
                    Some(val) if val.is_ascii_alphanumeric() => {}
                    _ => {
                        if *next == b'{' {
                            return Some(FlyToken::Literal("{".to_string()));
                        }
                        if *next == b'[' {
                            return Some(FlyToken::Literal("[".to_string()));
                        }
                    }
                }

                while let Some(in_block_read) = self.reader.next() {
                    match *in_block_read as char {
                        ' ' | '}' | ']' => {
                            if sd.main_token.is_empty() {
                                sd.main_token = temp;
                            } else if sd.secondary_token.is_empty() {
                                sd.secondary_token = temp;
                            } else {
                                sd.contents.push_str(&temp);
                                if *in_block_read == b' ' {
                                    match self.reader.peek() {
                                        Some(b'}') | Some(b']') => {}
                                        _ => {
                                            sd.contents.push(' ');
                                        }
                                    }
                                }
                            }
                            temp = String::new();
                            if (*in_block_read == b'}' && *next == b'{')
                                || (*in_block_read == b']' && *next == b'[')
                            {
                                if *in_block_read == b']' {
                                    while matches!(self.reader.peek(), Some(b'\n')) {
                                        _ = self.reader.next()
                                    }
                                }
                                if sd.main_token == "end" {
                                    sd.is_ended = true;
                                }
                                break;
                            }
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
            match self.reader.peek() {
                Some(b'[') | Some(b'{') => {
                    return Some(FlyToken::Literal(temp));
                }
                _ => {}
            }
        }

        None
    }
}
