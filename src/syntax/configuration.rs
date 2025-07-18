use super::{FileContents, Token};

#[derive(Debug)]
pub struct Configuration {
    pub name: String,
    pub fields: Vec<ConfigurationField>,
}
#[derive(Debug)]
pub struct ConfigurationField {
    pub name: String,
}

impl Configuration {
    pub fn read_from_contents(contents: &mut FileContents) -> Configuration {
        let Some(name_opt) = contents.next() else {
            panic!("Could not find a name for this configuration.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!(
                "Started configuration, expected a name but got {:?}",
                name_opt
            );
        };
        let name = name_ref.to_string();
        let mut fields = Vec::new();

        while let Some(next) = contents.take() {
            if next == Token::OpenBrace {
                break;
            }
        }

        'cmd: while let Some(token) = contents.take() {
            match token {
                Token::CloseBrace => {
                    break 'cmd;
                }
                Token::Literal(lit) => {
                    let field = ConfigurationField { name: lit };
                    fields.push(field);
                }
                _ => {}
            }
        }

        Configuration { name, fields }
    }
}
