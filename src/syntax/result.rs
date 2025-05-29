use crate::syntax::FieldReferenceKind;

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

            if let Some(parent_obj_name) = &objects[object_idx].inherits {
                let Some(parent_obj_idx) =
                    objects.iter().position(|obj| obj.name == *parent_obj_name)
                else {
                    return Err(ValidationError::MissingParent(
                        objects[object_idx].name.to_string(),
                        parent_obj_name.to_string(),
                    ));
                };

                if objects[object_idx].reuse_all {
                    let mut copy = objects[parent_obj_idx].fields.clone();
                    objects[object_idx].fields.append(&mut copy);
                }
                objects[object_idx].table_name = objects[parent_obj_idx].table_name.clone();
            }

            while field_idx < objects[object_idx].fields.len() {
                if objects[object_idx].fields[field_idx].field_type.is_some() {
                    field_idx += 1;
                    continue;
                }

                match &objects[object_idx].fields[field_idx].location.reference {
                    FieldReferenceKind::JoinData(joining_field) => {
                        let referenced_field = &objects[object_idx]
                            .fields
                            .iter()
                            .find(|field| field.name == *joining_field)
                            .ok_or(ValidationError::JoinFieldUnresolvable(
                                objects[object_idx].name.to_string(),
                                objects[object_idx].fields[field_idx].name.to_string(),
                            ))?;
                        let referenced_entity = match &referenced_field.location.reference {
                            FieldReferenceKind::FieldType(entity_name) => objects
                                .iter()
                                .find(|obj| obj.name == *entity_name)
                                .ok_or(ValidationError::JoinFieldUnresolvable(
                                    objects[object_idx].name.to_string(),
                                    objects[object_idx].fields[field_idx].name.to_string(),
                                ))?,
                            _ => {
                                return Err(ValidationError::JoinFieldUnresolvable(
                                    objects[object_idx].name.to_string(),
                                    objects[object_idx].fields[field_idx].name.to_string(),
                                ));
                            }
                        };
                        let referenced_foreign_field = referenced_entity
                            .fields
                            .iter()
                            .find(|field| {
                                field.name == objects[object_idx].fields[field_idx].location.name
                            })
                            .ok_or(ValidationError::JoinFieldUnresolvable(
                                objects[object_idx].name.to_string(),
                                objects[object_idx].fields[field_idx].name.to_string(),
                            ))?;
                        objects[object_idx].fields[field_idx].field_type =
                            referenced_foreign_field.field_type.clone();
                    }
                    FieldReferenceKind::FieldType(joining_entity) => {
                        let referenced_entity = objects
                            .iter()
                            .find(|obj| obj.name == *joining_entity)
                            .ok_or(ValidationError::JoinFieldUnresolvable(
                                objects[object_idx].name.to_string(),
                                objects[object_idx].fields[field_idx].name.to_string(),
                            ))?;
                        let referenced_foreign_field = referenced_entity
                            .fields
                            .iter()
                            .find(|field| {
                                field.name == objects[object_idx].fields[field_idx].location.name
                            })
                            .ok_or(ValidationError::JoinFieldUnresolvable(
                                objects[object_idx].name.to_string(),
                                objects[object_idx].fields[field_idx].name.to_string(),
                            ))?;
                        objects[object_idx].fields[field_idx].field_type =
                            referenced_foreign_field.field_type.clone();
                    }
                    _ => {}
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
