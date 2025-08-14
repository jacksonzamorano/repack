use super::{FileContents, RepackError, RepackErrorKind, Token};

pub struct QueryArg {
    pub name: String,
    pub typ: String,
}
impl QueryArg {
    fn parse(query_name: &str, reader: &mut FileContents) -> Result<QueryArg, RepackError> {
        let name = reader.take_literal().ok_or_else(|| {
            RepackError::global(
                RepackErrorKind::QueryArgInvalidSyntax,
                query_name.to_string(),
            )
        })?;
        if !reader.take_colon() {
            return Err(RepackError::global(
                RepackErrorKind::QueryArgInvalidSyntax,
                name.to_string(),
            ));
        }
        let typ = reader.take_literal().ok_or_else(|| {
            RepackError::global(
                RepackErrorKind::QueryArgInvalidSyntax,
                query_name.to_string(),
            )
        })?;
        return Ok(QueryArg { name, typ });
    }
}

pub enum QueryReturn {
    None,
    One,
    Many,
}

pub struct Query {
    pub name: String,
    pub args: Vec<QueryArg>,
    pub contents: String,
    pub ret_type: QueryReturn,
}
impl Query {
    fn parse(obj_name: &str, reader: &mut FileContents) -> Result<Query, RepackError> {
        let name = reader.take_literal().ok_or_else(|| {
            RepackError::global(RepackErrorKind::QueryInvalidSyntax, obj_name.to_string())
        })?;
        let mut args = Vec::<QueryArg>::new();
        let mut ret_type = QueryReturn::None;
        if matches!(reader.peek(), Some(Token::OpenParen)) {
            loop {
                match reader.peek() {
                    Some(Token::Literal(_)) => {
                        args.push(QueryArg::parse(&name, reader)?);
                    }
                    Some(Token::CloseParen) => break,
                    _ => {}
                }
            }
        }
        if !matches!(reader.peek(), Some(Token::Equal)) {
            return Err(RepackError::global(
                RepackErrorKind::QueryInvalidSyntax,
                obj_name.to_string(),
            ));
        }
        let contents = reader.take_literal().ok_or_else(|| {
            RepackError::global(RepackErrorKind::QueryInvalidSyntax, obj_name.to_string())
        })?;
        if reader.take_colon() {
            match reader.take() {
                Some(Token::One) => ret_type = QueryReturn::One,
                Some(Token::Many) => ret_type = QueryReturn::Many,
                _ => {
                    return Err(RepackError::global(
                        RepackErrorKind::QueryInvalidSyntax,
                        obj_name.to_string(),
                    ));
                }
            }
        }

        return Ok(Query {
            name,
            args,
            contents,
            ret_type,
        });
    }
}
