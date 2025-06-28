use super::{FileContents, Token};

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub categories: Vec<String>,
    pub options: Vec<String>,
}
impl Enum {
    pub fn read_from_contents(contents: &mut FileContents) -> Enum {
        let Some(name_opt) = contents.next() else {
            panic!("Read enum name, expected a name but got end of file.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!("Read enum name, expected a name but got {:?}", name_opt);
        };
        let name = name_ref.to_string();
        let mut options = Vec::new();
        let mut categories = Vec::new();

        'header: while let Some(token) = contents.next() {
            match token {
                Token::Pound => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        categories.push(lit.to_string());
                    }
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
                Token::Literal(lit) => {
                    options.push(lit);
                }
                _ => {}
            }
        }

        Enum {
            name,
            categories,
            options,
        }
    }

    pub fn cases(&self, reverse: bool) -> Vec<&String> {
        if reverse {
            self.categories.iter().rev().collect()
        } else {
            self.categories.iter().collect()
        }
    }
}
