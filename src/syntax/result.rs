use std::process::exit;
use super::{language, FileContents, Output, Object, ObjectType, Token};

#[derive(Debug)]
pub struct ParseResult {
    pub objects: Vec<Object>,
    pub languages: Vec<Output>,
}

impl ParseResult {
    pub fn from_contents(mut contents: FileContents) -> ParseResult {
        let mut objects = Vec::new();
        let mut languages = Vec::new();

        while let Some(token) = contents.next() {
            match *token {
                Token::RecordType => {
                    objects.push(Object::read_from_contents(
                        ObjectType::Record,
                        &mut contents,
                    ));
                }
                Token::StructType => {
                    objects.push(Object::read_from_contents(
                        ObjectType::Struct,
                        &mut contents,
                    ));
                },
                Token::OutputType => {
                    if let Some(language) = language::Output::from_contents(&mut contents) {
                        languages.push(language);
                    }
                }
                _ => {}
            }
        }

        ParseResult { objects, languages }
    }

    pub fn validate(&self, should_exit: bool) {
        let mut has_errors = false;
        for object in &self.objects {
            if let Some(errors) = object.errors(self) {
                has_errors = true;
                for error in errors {
                    println!("{}", error.message());
                }
            }
        }
        for language in &self.languages {
            let errors = language.errors();
            if !errors.is_empty() {
                has_errors = true;
                for error in errors {
                    println!("{}", error.message());
                }
            }
        }
        if has_errors {
            println!("Compilation failed.");
            if should_exit {
                exit(-1);
            }
        }
    }
}
