use std::iter::Peekable;

use crate::blueprint::{BlueprintContext, BlueprintToken};

pub struct BlueprintFileReader<'a> {
    pub reader: Peekable<std::slice::Iter<'a, u8>>,
}
impl<'a> BlueprintFileReader<'a> {
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
    pub fn next_token(&mut self) -> Option<BlueprintToken> {
        self.next()
            .map(|x| BlueprintToken::from_string(x, &BlueprintContext::Global))
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

    pub fn read_line_tokens_with_context(&mut self, ctx: &BlueprintContext) -> Vec<BlueprintToken> {
        let mut tokens = Vec::new();
        let mut temp_token = String::new();
        while let Some(next) = self.reader.next() {
            if let Some(individual_token) = BlueprintToken::from_char(*next as char) {
                if !temp_token.is_empty() {
                    tokens.push(BlueprintToken::from_string(temp_token, ctx));
                }
                if matches!(individual_token, BlueprintToken::NewLine) {
                    break;
                }
                tokens.push(individual_token);
                temp_token = String::new();
                continue;
            }
            if next.is_ascii_whitespace() {
                if !temp_token.is_empty() {
                    tokens.push(BlueprintToken::from_string(temp_token, ctx));
                }
                temp_token = String::new();
            } else {
                temp_token.push(*next as char);
            }
        }

        return tokens;
    }

    pub fn read_line_tokens(&mut self) -> Vec<BlueprintToken> {
        self.read_line_tokens_with_context(&BlueprintContext::Global)
    }

    pub fn read_block(&mut self, context: &BlueprintContext) -> Vec<BlueprintToken> {
        let mut tokens = vec![];
        let mut temp_token = String::new();

        let mut dash_ct = 0usize;
        while let Some(next) = self.reader.next() {
            if *next == b'-' {
                dash_ct += 1;
                if dash_ct == 3 {
                    break;
                }
            } else if dash_ct > 0 {
                while dash_ct != 0 {
                    temp_token += "-";
                    dash_ct -= 1;
                }
            }
            if let Some(individual_token) = BlueprintToken::from_char(*next as char) {
                if !temp_token.is_empty() {
                    tokens.push(BlueprintToken::from_string(temp_token, context));
                }
                tokens.push(individual_token);
                temp_token = String::new();
                continue;
            }
            if next.is_ascii_whitespace() {
                if !temp_token.is_empty() {
                    tokens.push(BlueprintToken::from_string(temp_token, context));
                }
                temp_token = String::new();
            } else {
                temp_token.push(*next as char);
            }
        }

        loop {
            if matches!(tokens.last(), Some(BlueprintToken::NewLine)) {
                tokens.pop();
            } else {
                break;
            }
        }

        return tokens;
    }
}
