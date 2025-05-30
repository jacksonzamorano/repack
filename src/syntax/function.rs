use super::{
    Field, FileContents, FunctionNamespace, Object, Output, RepackError, RepackErrorKind, Token,
};

#[derive(Debug, Clone)]
pub struct FieldFunction {
    pub namespace: FunctionNamespace,
    pub name: String,
    pub args: Vec<String>,
}
impl FieldFunction {
    pub fn arg(
        &self,
        output: &Output,
        obj: &Object,
        field: &Field,
        i: usize,
    ) -> Result<&String, RepackError> {
        self.args
            .get(i)
            .ok_or(RepackError::from_lang_with_obj_field_msg(
                RepackErrorKind::ExpectedArgument,
                output,
                obj,
                field,
                i.to_string(),
            ))
    }
    pub fn from_contents(namespace: String, contents: &mut FileContents) -> Option<FieldFunction> {
        if contents.take()? != Token::Colon {
            return None;
        }
        let Some(Token::Literal(name)) = contents.take() else {
            return None;
        };
        let mut args = Vec::<String>::new();
        contents.take(); // skip (
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

        Some(FieldFunction {
            namespace: FunctionNamespace::from_string(&namespace),
            name,
            args,
        })
    }
}
