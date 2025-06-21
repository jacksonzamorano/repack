use std::iter::Peekable;

use crate::profiles::{FlyContext, FlyToken};

pub struct TemplatedLanguageReader<'a> {
    pub reader: Peekable<std::slice::Iter<'a, u8>>,
}
impl<'a> TemplatedLanguageReader<'a> {
    pub fn next(&mut self) -> Option<String> {
        let mut temp = String::new();
        while let Some(next) = self.reader.next() {
            if next.is_ascii_whitespace() {
                if !temp.is_empty() {
                    return Some(temp);
                }
            } else {
                temp.push(*next as char);
            }
        }

        None
    }
    pub fn next_token(&mut self) -> Option<FlyToken> {
        self.next()
            .map(|x| FlyToken::from_string(x, &FlyContext::Global))
    }
    pub fn read_line(&mut self) -> Option<String> {
        let mut temp_token = String::new();
        while let Some(next) = self.reader.next() {
            if *next == b'\n' || *next == b'\r' {
                if !temp_token.is_empty() {
                    return Some(temp_token);
                }
            } else {
                temp_token.push(*next as char);
            }
        }
        return None;
    }

    pub fn read_line_tokens(&mut self) -> Vec<FlyToken> {
        let mut tokens = Vec::new();
        let mut temp_token = String::new();
        while let Some(next) = self.reader.next() {
            if next.is_ascii_whitespace() && !temp_token.is_empty() {
                tokens.push(FlyToken::from_string(temp_token, &FlyContext::Global));
                temp_token = String::new();
            } else if *next == b'\n' || *next == b'\r' {
                if !tokens.is_empty() {
                    break;
                }
            } else {
                temp_token.push(*next as char);
            }
        }

        if !temp_token.is_empty() {
            tokens.push(FlyToken::from_string(temp_token, &FlyContext::Global));
        }

        return tokens;
    }

    pub fn read_block(&mut self, context: &FlyContext) -> Vec<FlyToken> {
        let mut tokens = vec![];
        let mut temp_token = String::new();

        let mut dash_ct = 0usize;
        while let Some(next_token) = self.reader.next() {
            if *next_token == b'-' {
                dash_ct += 1;
                if dash_ct == 3 {
                    break;
                }
            } else if dash_ct > 0 {
                while dash_ct != 0 {
                    temp_token += "-";
                    dash_ct -= 1;
                }
            } else if next_token.is_ascii_whitespace() {
                tokens.push(FlyToken::from_string(temp_token, &context));
                temp_token = String::new();
            } else {
                temp_token.push(*next_token as char);
            }
        }

        return tokens;
    }
}
