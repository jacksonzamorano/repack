use super::{FileContents, Token};

#[derive(Debug, Clone)]
pub struct FieldFunction {
    pub namespace: String,
    pub name: String,
    pub args: Vec<String>,
}
impl FieldFunction {
    pub fn from_contents(namespace: String, contents: &mut FileContents) -> Option<FieldFunction> {
        if contents.take()? != Token::Colon {
            return None;
        }
        let Some(Token::Literal(name)) = contents.take() else {
            return None;
        };
        let mut args = Vec::<String>::new();
        if *contents.peek()? == Token::OpenParen {
            contents.skip();
            // has args
            let mut buf = String::new();
            loop {
                let Some(tok) = contents.take() else { break };
                match tok {
                    Token::Comma => {
                        args.push(buf);
                        buf = String::new();
                    }
                    Token::CloseParen => {
                        args.push(buf);
                        break;
                    }
                    Token::Literal(text) => {
                        buf.push_str(&text);
                    }
                    _ => {}
                };
            }
        }

        Some(FieldFunction {
            namespace,
            name,
            args,
        })
    }
}
