use super::{Field, FileContents, ObjectFunction, Token, RepackError, RepackErrorKind};

#[derive(Debug)]
pub struct Snippet {
    pub name: String,
    pub fields: Vec<Field>,
    pub functions: Vec<ObjectFunction>,
}

impl Snippet {
    pub fn read_from_contents(contents: &mut FileContents) -> Result<Snippet, RepackError> {
        let Some(name_opt) = contents.next() else {
            return Err(RepackError::global(
                RepackErrorKind::ParseIncomplete,
                "snippet name".to_string()
            ));
        };
        let Token::Literal(name_ref) = name_opt else {
            return Err(RepackError::global(
                RepackErrorKind::ParseIncomplete,
                format!("{name_opt:?}")
            ));
        };
        let name = name_ref.to_string();
        let mut fields = Vec::new();
        let mut functions = Vec::new();

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
                    if let Some(next) = contents.peek() {
                        if *next == Token::Colon {
                            if let Some(func) =
                                ObjectFunction::from_contents(lit.to_string(), contents)
                            {
                                functions.push(func);
                            }
                        } else if let Some(field) = Field::from_contents(lit.to_string(), contents) {
                            fields.push(field);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Snippet { name, fields, functions })
    }
}
