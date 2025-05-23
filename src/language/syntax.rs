use std::process::exit;
use super::{FileContents, Object, ObjectType, Token};

#[derive(Debug)]
pub struct ParseResult {
    pub objects: Vec<Object>,
}

impl ParseResult {
    pub fn from_contents(mut contents: FileContents) -> ParseResult {
        let mut objects = Vec::new();

        while let Some(token) = contents.next() {
            match *token {
                Token::RecordType => {
                    objects.push(Object::read_from_contents(
                        ObjectType::Record,
                        &mut contents,
                    ));
                }
                _ => {}
            }
        }

        ParseResult { objects }
    }

    pub fn validate(&self, should_exit: bool) {
        let mut has_errors = false;
        for object in &self.objects {
            if let Some(errors) = object.errors(self) {
                has_errors = true;
                for error in errors {
                    println!("{}", error.to_string());
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
