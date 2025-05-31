use super::{Field, FileContents, Token};

#[derive(Debug)]
pub struct Snippet {
    pub name: String,
    pub fields: Vec<Field>,
}

impl Snippet {
    pub fn read_from_contents(contents: &mut FileContents) -> Snippet {
        let Some(name_opt) = contents.next() else {
            panic!("Read record type, expected a name but got end of file.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!("Read record type, expected a name but got {:?}", name_opt);
        };
        let name = name_ref.to_string();
        let mut fields = Vec::new();

        while let Some(next) = contents.take() {
            if next == Token::OpenBrace {
                break;
            }
        }

        'cmd: while let Some(token) = contents.next() {
            match token {
                Token::CloseBrace => {
                    break 'cmd;
                }
                Token::Literal(lit) => {
                    if let Some(field) = Field::from_contents(lit.to_string(), contents) {
                        fields.push(field);
                    }
                }
                _ => {}
            }
        }

        Snippet { name, fields }
    }
}
