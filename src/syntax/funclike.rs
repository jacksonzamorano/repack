use super::{FieldType, FileContents, Token};

pub trait FunctionCommands {
    fn from_string(value: &str, reader: &mut FileContents) -> Option<Self>
    where
        Self: Sized;
}

pub struct FunctionDefinition<T: FunctionCommands> {
    pub name: String,
    pub args: Vec<FunctionArgument>,
    pub commands: Vec<T>,
}
impl<T: FunctionCommands> FunctionDefinition<T> {
    fn from_contents(reader: &mut FileContents) -> Option<Self> {
        let Some(Token::Literal(name)) = reader.take() else {
            return None;
        };
        let mut args = Vec::new();
        // Skip until open paren
        loop {
            if matches!(reader.take(), Some(Token::OpenParen)) {
                break;
            }
        }
        // Read arguments
        loop {
            let Some(tokn) = reader.take() else {
                break;
            };
            match tokn {
                Token::Literal(arg_name) => {
                    let Some(Token::Colon) = reader.take() else {
                        return None;
                    };
                    let Some(Token::Literal(arg_typ_str)) = reader.take() else {
                        return None;
                    };
                    args.push(FunctionArgument {
                        name: arg_name,
                        arg_type: FieldType::from_string(&arg_typ_str),
                        arg_type_string: arg_typ_str,
                    });
                }
                Token::CloseParen => break,
                _ => {}
            }
        }

        let mut commands = Vec::new();
        // Skip until function body
        loop {
            if matches!(reader.take(), Some(Token::OpenBrace)) {
                break;
            }
        }
        // Read commands
        loop {
            let Some(tokn) = reader.take() else {
                break;
            };
            match tokn {
                Token::Literal(val) => {
                    if let Some(cmd) = T::from_string(&val, reader) {
                        commands.push(cmd);
                    }
                }
                Token::CloseBrace => break,
                _ => {}
            }
        }

        return Some(FunctionDefinition {
            name,
            args,
            commands,
        });
    }
}

pub struct FunctionArgument {
    pub name: String,
    pub arg_type: Option<FieldType>,
    pub arg_type_string: String,
}
