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
            if *next == b'$' {
                if let Some(maybe_open_brace) = self.reader.next() {
                    if *maybe_open_brace == b'{' {
                        if let Some(maybe_backslash) = self.reader.peek() {
                            if **maybe_backslash == b'/' {
                                _ = self.reader.next();
                                for in_block_read in self.reader.by_ref() {
                                    if *in_block_read == b'}' {
                                        break;
                                    } else {
                                        temp.push(*in_block_read as char);
                                    }
                                }
                                return Some(FlyToken::SnippetEnd(temp));
                            }
                        }

                        let mut sd = SnippetDetails::default();

                        while let Some(in_block_read) = self.reader.next() {
                            match *in_block_read as char {
                                ' ' | '}' => {
                                    if sd.main_token.is_empty() {
                                        sd.main_token = temp;
                                    } else if sd.secondary_token.is_empty() {
                                        sd.secondary_token = temp;
                                    } else {
                                        sd.contents.push_str(&temp);
                                        if *in_block_read == b' ' {
                                            sd.contents.push(' ');
                                        }
                                    }
                                    temp = String::new();
                                    if *in_block_read == b'}' {
                                        break;
                                    }
                                }
                                ':' => {
                                    sd.secondary_token = temp;
                                    temp = String::new();
                                }
                                '/' => {
                                    if matches!(self.reader.peek(), Some(b'}')) {
                                        sd.is_ended = true;
                                    }
                                }
                                _ => {
                                    temp.push(*in_block_read as char);
                                }
                            }
                        }

                        return Some(FlyToken::Snippet(sd));
                    }
                    temp.push('{');
                }
                temp.push('$');
            }
            if next.is_ascii_whitespace() && !temp.is_empty() {
                return Some(FlyToken::Literal(temp));
            } else {
                temp.push(*next as char);
                if matches!(self.reader.peek(), Some(b'$')) {
                    return Some(FlyToken::Literal(temp));
                }
            }
        }

        None
    }
}
