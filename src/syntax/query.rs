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
        Ok(QueryArg { name, typ })
    }
}

#[derive(Debug, Clone)]
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

        Ok(Query {
            name,
            args,
            contents,
            ret_type,
        })
    }

    pub fn render(
        &self,
        strct: &RepackStruct,
        other_structs: &[RepackStruct],
    ) -> Result<String, RepackError> {
        let mut output = String::new();

        let mut pos_args: Vec<String> = Vec::new();

        let mut buf = String::new();
        let mut iter = self.contents.chars();
        let mut ct = true;
        let mut last_c = ' ';
        loop {
            if let Some(c) = iter.next() {
                if c.is_alphabetic() || c == '_' || c == '$' {
                    buf.push(c);
                    continue;
                }
                if !(c.is_alphabetic() || c == '_' || c == '$') && !buf.starts_with('$') {
                    output.push_str(&buf);
                    output.push(c);
                    buf.clear();
                    continue;
                }
                last_c = c;
            } else {
                ct = false
            };
            if buf.len() < 2 {
                if !ct {
                    break;
                }
                continue;
            }
            if !buf.starts_with("$") {
                output.push_str(&buf);
                break;
            }
            // We know it's a variable - let's interpolate
            let result = match &buf[1..] {
                "fields" => {
                    let mut field_strings = Vec::<String>::new();
                    for field in &strct.fields {
                        if let Some(location) = &field.field_location {
                            let table = if location.location == "super" {
                                strct.table_name.as_ref().unwrap()
                            } else {
                                &location.location
                            };
                            field_strings
                                .push(format!("{}.{} AS {}", table, location.field, field.name))
                        } else {
                            if let Some(alias) = field.function("db", "as") {
                                let def = String::new();
                                field_strings.push(format!(
                                    "{} AS {}",
                                    alias.args.first().unwrap_or(&def),
                                    field.name
                                ))
                            } else {
                                field_strings.push(format!(
                                    "{}.{} AS {}",
                                    strct.table_name.as_ref().unwrap(),
                                    field.name,
                                    field.name
                                ))
                            }
                        }
                    }
                    Some(field_strings.join(", "))
                }
                "locations" => {
                    let mut locations = Vec::<String>::new();
                    locations.push(strct.table_name.clone().unwrap());
                    for join in &strct.joins {
                        let mut join_string = String::new();
                        let mut template_string_iter = join.contents.chars();
                        let mut join_string_temp = String::new();
                        let mut join_ct = true;
                        let mut last_char = ' ';
                        loop {
                            if let Some(tc) = template_string_iter.next() {
                                if tc == '$' || !join_string_temp.is_empty() {
                                    if tc != ' ' && tc != '.' {
                                        join_string_temp.push(tc);
                                        continue;
                                    }
                                    last_char = tc;
                                } else {
                                    join_string.push(tc);
                                    continue;
                                }
                            } else {
                                join_ct = false
                            }

                            if join_string_temp.len() > 1 {
                                let replace = match &join_string_temp[1..] {
                                    "name" => {
                                        let fe = other_structs
                                            .iter()
                                            .find(|x| x.name == join.foreign_entity)
                                            .unwrap();
                                        // ^ This is safe to unwrap because we've already done the
                                        // checking.
                                        Some(format!(
                                            "{} {}",
                                            fe.table_name.clone().unwrap(),
                                            join.name
                                        ))
                                    }
                                    "super" => Some(strct.table_name.clone().unwrap()),
                                    tn => {
                                        if tn == join.name {
                                            Some(tn.to_string())
                                        } else {
                                            None
                                        }
                                    }
                                };
                                join_string_temp.clear();
                                join_string.push_str(&replace.unwrap());
                                if !join_ct {
                                    break;
                                }
                                join_string.push(last_char);
                            }

                            if !join_ct {
                                break;
                            }
                        }
                        locations.push(join_string);
                    }
                    Some(locations.join(" "))
                }
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
                            Some(format!("${}", idx + 1))
                        } else {
                            pos_args.push(arg.name.clone());
                            let idx = pos_args.len();
                            Some(format!("${idx}"))
                        }
                    } else {
                        Some(format!("[err: {val}]"))
                    }
                }
            };
            buf.clear();

            output.push_str(&result.unwrap());
            if !ct {
                break;
            }
            output.push(last_c);
        }
        output.push(';');

        Ok(output)
    }
}

#[derive(Debug)]
pub enum AutoQueryType {
    Insert,
    Update,
}

#[derive(Debug)]
pub struct AutoQuery {
    pub query_type: AutoQueryType,
    pub name: String,
    pub args: Vec<String>,
    pub ret_type: QueryReturn,
}

impl AutoQuery {
    pub fn parse(
        query_type: AutoQueryType,
        obj_name: &str,
        reader: &mut FileContents,
    ) -> Result<AutoQuery, RepackError> {
        let name = reader.take_literal().ok_or_else(|| {
            RepackError::global(RepackErrorKind::QueryInvalidSyntax, obj_name.to_string())
        })?;
        let mut args = Vec::<String>::new();
        let mut ret_type = QueryReturn::None;
        if matches!(reader.peek(), Some(Token::OpenParen)) {
            loop {
                match reader.peek() {
                    Some(Token::Literal(_)) => {
                        args.push(reader.take_literal().unwrap());
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

        Ok(AutoQuery {
            query_type,
            name,
            args,
            ret_type,
        })
    }

    pub fn into_query(&self, strct: &RepackStruct) -> Result<Query, RepackError> {
        Ok(match self.query_type {
            AutoQueryType::Insert => {
                let mut args = Vec::<QueryArg>::new();
                let mut output = "WITH $table AS (INSERT INTO $table (".to_string();
                let mut query_interpolate = String::new();
                for (idx, selected_field) in self.args.iter().enumerate() {
                    let Some(matching_field) =
                        strct.fields.iter().find(|x| x.name == *selected_field)
                    else {
                        panic!("Field not found in struct.");
                    };
                    output.push_str(&selected_field);
                    query_interpolate.push_str(&format!("$__{}", selected_field));

                    args.push(QueryArg {
                        name: format!("__{}", selected_field),
                        typ: matching_field.field_type.as_ref().unwrap().to_string(),
                    });
                    if idx + 1 != self.args.len() {
                        query_interpolate.push_str(", ");
                        output.push_str(", ");
                    }
                }
                output.push_str(&format!(
                    ") VALUES ({}) RETURNING *) AS {} SELECT $fields FROM $locations",
                    query_interpolate,
                    strct.table_name.as_ref().unwrap()
                ));
                Query {
                    args,
                    name: self.name.clone(),
                    ret_type: self.ret_type.clone(),
                    contents: output,
                }
            }
            _ => Query {
                args: Vec::new(),
                name: self.name.clone(),
                ret_type: self.ret_type.clone(),
                contents: String::new(),
            },
        })
    }
}
