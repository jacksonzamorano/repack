use super::{FileContents, RepackError, RepackErrorKind, Token};
use crate::profiles::OutputProfile;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Output {
    pub profile: String,
    pub location: Option<String>,
    pub categories: Vec<String>,
    pub options: HashMap<String, String>,
    pub exclude: Vec<String>,
}
impl Output {
    pub fn from_contents(contents: &mut FileContents) -> Option<Output> {
        let Some(name_opt) = contents.next() else {
            panic!("Read record type, expected a name but got end of file.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!("Read record type, expected a name but got {:?}", name_opt);
        };
        let output_language = name_ref.to_string();
        let mut location = None;
        let mut options = HashMap::new();
        let mut categories = Vec::new();
        let mut exclude = Vec::new();

        let mut empty = false;
        while let Some(token) = contents.next() {
            match token {
                Token::At => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        location = Some(lit.to_string());
                    }
                }
                Token::Pound => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        categories.push(lit.to_string());
                    }
                }
                Token::OpenBrace => {
                    break;
                }
                Token::Semicolon => {
                    empty = true;
                    break;
                }
                _ => {}
            }
        }

        if !empty {
            while let Some(token) = contents.next() {
                match token {
                    Token::Minus => {
                        if let Some(Token::Literal(lit)) = contents.next() {
                            exclude.push(lit.to_string());
                        }
                    }
                    Token::Literal(lit) => {
                        let key = lit.to_string();
                        let value = match contents.next() {
                            Some(Token::Literal(lit)) => lit.to_string(),
                            _ => {
                                continue;
                            }
                        };
                        options.insert(key, value);
                    }
                    Token::CloseBrace => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        Some(Output {
            profile: output_language,
            location,
            categories,
            exclude,
            options,
        })
    }

    pub fn errors(&self) -> Vec<RepackError> {
        let mut errors = Vec::new();
        if OutputProfile::from_keyword(&self.profile).is_none() {
            errors.push(RepackError::from_lang(
                RepackErrorKind::UnknownLanguage,
                self,
            ));
        }
        errors
    }
}
