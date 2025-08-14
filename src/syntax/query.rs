use super::{FileContents, RepackError, RepackErrorKind, RepackStruct, Token};

#[derive(Debug)]
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
        let typ = reader.take_literal().ok_or_else(|| {
            RepackError::global(
                RepackErrorKind::QueryArgInvalidSyntax,
                query_name.to_string(),
            )
        })?;
        return Ok(QueryArg { name, typ });
    }
}

#[derive(Debug)]
pub enum QueryReturn {
    None,
    One,
    Many,
}

#[derive(Debug)]
pub struct Query {
    pub name: String,
    pub args: Vec<QueryArg>,
    pub contents: String,
    pub ret_type: QueryReturn,
}
impl Query {
    pub fn parse(obj_name: &str, reader: &mut FileContents) -> Result<Query, RepackError> {
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
                    Some(Token::CloseParen) => {
                        reader.skip();
                        break;
                    }
                    _ => {
                        reader.skip();
                    }
                }
            }
        }
        if !matches!(reader.take(), Some(Token::Equal)) {
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

    pub fn render(&self, strct: &RepackStruct) -> Result<String, RepackError> {
        let mut output = String::new();

        let mut pos_args: Vec<String> = Vec::new();

        let mut buf = String::new();
        let mut iter = self.contents.chars();
        while let Some(c) = iter.next() {
            if c != ' ' {
                buf.push(c);
                continue;
            }
            if c == ' ' && !buf.starts_with('$') {
                output.push_str(&buf);
                output.push(' ');
                buf.clear();
            }
            // We know it's a variable - let's interpolate
            let result = match &buf[1..] {
                "table" => strct.table_name.clone(),
                val => {
                    if let Some(field) = strct.fields.iter().find(|x| x.name == val) {
                        if let Some(location) = &field.field_location {
                            let table = if location.location == "super" {
                                strct.table_name.as_ref().unwrap()
                            } else {
                                &location.location
                            };
                            Some(format!("{}.{}", table, location.field))
                        } else {
                            Some(format!(
                                "{}.{}",
                                strct.table_name.as_ref().unwrap(),
                                field.name
                            ))
                        }
                    } else if let Some(arg) = self.args.iter().find(|x| x.name == val) {
                        if let Some(idx) = pos_args.iter().position(|x| *x == arg.name) {
                            Some(format!("${}", idx))
                        } else {
                            pos_args.push(arg.name.clone());
                            let idx = pos_args.len() - 1;
                            Some(format!("${}", idx))
                        }
                    } else {
                        None
                    }
                }
            };

            output.push_str(&result.unwrap());
        }

        return Ok(output);
    }
}
