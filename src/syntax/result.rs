use super::{
    FieldValidationError, FileContents, Object, ObjectType, Output, Token, ValidationError,
    dependancies::graph_valid, language,
};
use std::process::exit;

#[derive(Debug)]
pub struct ParseResult {
    pub objects: Vec<Object>,
    pub languages: Vec<Output>,
}

impl ParseResult {
    pub fn from_contents(mut contents: FileContents) -> Result<ParseResult, ValidationError> {
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
                }
                Token::OutputType => {
                    if let Some(language) = language::Output::from_contents(&mut contents) {
                        languages.push(language);
                    }
                }
                _ => {}
            }
        }

        let mut object_idx: usize = 0;
        while object_idx < objects.len() {
            let mut field_idx: usize = 0;
            while field_idx < objects[object_idx].fields.len() {
                if !objects[object_idx].fields[field_idx]
                    .field_type
                    .unresolved()
                {
                    field_idx += 1;
                    continue;
                }

                if let Some(reference) = &objects[object_idx].fields[field_idx].reference {
                    let referenced_object = &objects
                        .iter()
                        .find(|obj| obj.name == reference.object_name)
                        .ok_or(ValidationError::Field(FieldValidationError::new(
                            super::FieldValidationErrorType::InvalidRefObject,
                            &objects[object_idx],
                            &objects[object_idx].fields[field_idx],
                        )))?;
                    let referenced_field = referenced_object
                        .fields
                        .iter()
                        .find(|f| f.name == reference.field_name)
                        .ok_or(ValidationError::Field(FieldValidationError::new(
                            super::FieldValidationErrorType::InvalidRefField,
                            &objects[object_idx],
                            &objects[object_idx].fields[field_idx],
                        )))?;

                    let field_type = referenced_field.field_type.clone();
                    let optional = referenced_field.optional;
                    objects[object_idx].fields[field_idx].field_type = field_type;
                    objects[object_idx].fields[field_idx].optional = optional;
                }
                if let Some(reference) = &objects[object_idx].fields[field_idx].from {
                    let referenced_object = &objects
                        .iter()
                        .find(|obj| obj.name == reference.object_name)
                        .ok_or(ValidationError::Field(FieldValidationError::new(
                            super::FieldValidationErrorType::InvalidRefObject,
                            &objects[object_idx],
                            &objects[object_idx].fields[field_idx],
                        )))?;
                    let referenced_field = referenced_object
                        .fields
                        .iter()
                        .find(|f| f.name == reference.field_name)
                        .ok_or(ValidationError::Field(FieldValidationError::new(
                            super::FieldValidationErrorType::InvalidRefField,
                            &objects[object_idx],
                            &objects[object_idx].fields[field_idx],
                        )))?;

                    let field_type = referenced_field.field_type.clone();
                    let optional = referenced_field.optional;
                    objects[object_idx].fields[field_idx].field_type = field_type;
                    objects[object_idx].fields[field_idx].optional = optional;
                }
                field_idx += 1;
            }
            object_idx += 1;
        }

        Ok(ParseResult { objects, languages })
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
        if let Err(e) = graph_valid(&self.objects) {
            has_errors = true;
            println!("[ERROR] {}", e.message());
        }
        if has_errors {
            println!("Compilation failed.");
            if should_exit {
                exit(-1);
            }
        }
    }
}
