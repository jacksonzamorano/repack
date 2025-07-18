use std::collections::HashMap;

use super::{FileContents, Token};

#[derive(Debug)]
pub struct ConfigurationInstance {
    pub name: String,
    pub environment: Option<String>,
    pub configuration: String,
    pub values: HashMap<String, String>,
}

impl ConfigurationInstance {
    pub fn read_from_contents(contents: &mut FileContents) -> ConfigurationInstance {
        let Some(name_opt) = contents.next() else {
            panic!("Could not find a name for this instance.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!("Started instance, expected a name but got {name_opt:?}");
        };
        let name = name_ref.to_string();
        let mut environment: Option<String> = None;
        let mut configuration: Option<String> = None;
        let mut values = HashMap::new();

        'header: while let Some(token) = contents.next() {
            match token {
                Token::At => {
                    environment = match contents.next() {
                        Some(Token::Literal(lit)) => Some(lit.to_string()),
                        _ => None,
                    };
                }
                Token::Colon => {
                    configuration = match contents.next() {
                        Some(Token::Literal(lit)) => Some(lit.to_string()),
                        _ => None,
                    };
                }
                Token::OpenBrace => {
                    break 'header;
                }
                _ => {}
            }
        }

        'cmd: while let Some(token) = contents.take() {
            match token {
                Token::CloseBrace => {
                    break 'cmd;
                }
                Token::Literal(key) => {
                    if let Some(Token::Literal(value)) = contents.take() {
                        values.insert(key, value);
                    }
                }
                _ => {}
            }
        }

        if let Some(configuration) = configuration {
            ConfigurationInstance {
                name,
                environment,
                configuration,
                values,
            }
        } else {
            panic!("Instances must comply with a configuration.")
        }
    }
}
