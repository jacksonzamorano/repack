use super::{
    FileContents, ObjectFunctionName, FunctionNamespace, Object, Output, RepackError, RepackErrorKind, Token
};

#[derive(Debug, Clone)]
pub struct ObjectFunction {
    pub namespace: FunctionNamespace,
    pub name: ObjectFunctionName,
    pub args: Vec<String>,
}
impl ObjectFunction {
    pub fn arg(
        &self,
        output: &Output,
        obj: &Object,
        i: usize,
    ) -> Result<&String, RepackError> {
        self.args
            .get(i)
            .ok_or(RepackError::from_lang_with_obj_msg(
                RepackErrorKind::ExpectedArgument,
                output,
                obj,
                i.to_string(),
            ))
    }
    pub fn from_contents(namespace: String, contents: &mut FileContents) -> Option<ObjectFunction> {
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

        Some(ObjectFunction {
            namespace: FunctionNamespace::from_string(&namespace),
            name: ObjectFunctionName::from_string(&name),
            args,
        })
    }
}
