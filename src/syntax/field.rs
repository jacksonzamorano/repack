use super::{FieldCommand, FieldType, FileContents, Token};

#[derive(Debug, Clone)]
pub struct FieldLocation {
    pub reference: FieldReferenceKind,
    pub name: String, // Could be name
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[repr(i32)]
pub enum FieldReferenceKind {
    Local = 1,
    FieldType(String) = 2,
    JoinData(String) = 3,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String, // Could be name or aliased field name
    pub location: FieldLocation,
    pub field_type: Option<FieldType>,
    pub optional: bool,
    pub commands: Vec<FieldCommand>,
}
impl Field {
    /// Only safe for use in profile/output code.
    pub fn field_type(&self) -> &FieldType {
        self.field_type.as_ref().unwrap()
    }
    pub fn from_contents(name: String, contents: &mut FileContents) -> Option<Field> {
        let type_token = contents.take()?;
        let field_type_loc: (Option<FieldType>, FieldLocation) = match type_token {
            Token::Literal(literal) => (
                Some(FieldType::from_string(&literal)),
                FieldLocation {
                    reference: FieldReferenceKind::Local,
                    name: literal,
                },
            ),
            Token::From => {
                contents.skip(); // Skip (
                let Some(Token::Literal(entity_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip .
                let Some(Token::Literal(field_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip )
                (
                    None,
                    FieldLocation {
                        reference: FieldReferenceKind::JoinData(entity_name),
                        name: field_name,
                    },
                )
            }
            Token::Ref => {
                contents.skip(); // Skip (
                let Some(Token::Literal(entity_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip .
                let Some(Token::Literal(field_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip )
                (
                    None,
                    FieldLocation {
                        reference: FieldReferenceKind::FieldType(entity_name),
                        name: field_name,
                    },
                )
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
            field_type: field_type_loc.0,
            location: field_type_loc.1,
            optional,
            commands,
        })
    }
}
