use crate::outputs::{OutputBuilderError, OutputDescription};

use super::{FieldCommand, FieldType, FileContents, Object, Token};

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub optional: bool,
    pub commands: Vec<FieldCommand>,
}
impl Field {
    pub fn resolve_type<'a>(
        &'a self,
        object: &Object,
        description: &'a OutputDescription,
    ) -> Result<&'a FieldType, OutputBuilderError> {
        match &self.field_type {
            FieldType::Ref(object_name, field_name) => description
                .field(object, self, object_name, field_name)
                .map(|x| &x.field_type),
            _ => Ok(&self.field_type),
        }
    }
}
impl Field {
    pub fn from_contents(name: String, contents: &mut FileContents) -> Option<Field> {
        let type_token = contents.next()?;
        let field_type: FieldType = match type_token {
            Token::Literal(literal) => FieldType::from_string(literal),
            Token::Ref => {
                let mut object_name = String::new();
                let mut field_name = String::new();
                while let Some(token) = contents.next() {
                    match token {
                        Token::OpenParen => {
                            if let Some(Token::Literal(lit)) = contents.next() {
                                object_name = lit.to_string();
                            }
                        }
                        Token::Period => {
                            if let Some(Token::Literal(lit)) = contents.next() {
                                field_name = lit.to_string();
                                break;
                            }
                        }
                        _ => {
                            return None;
                        }
                    }
                }
                FieldType::Ref(object_name, field_name)
            }
            _ => {
                return None;
            }
        };

        let optional = match contents.peek() {
            Some(Token::Question) => {
                contents.next();
                true
            }
            _ => false,
        };
        let mut commands = Vec::new();

        while let Some(token) = contents.next() {
            match token {
                Token::Pound => {
                    if let Some(Token::Literal(cmd)) = contents.next() {
                        if let Some(command) = FieldCommand::from_string(cmd) {
                            commands.push(command);
                        }
                    }
                }
                Token::NewLine => {
                    break;
                }
                _ => {}
            }
        }

        Some(Field {
            name,
            field_type,
            optional,
            commands,
        })
    }
}
