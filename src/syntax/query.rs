use super::{CoreType, FileContents, Token};


#[derive(Debug)]
pub struct QueryArg {
    pub name: String,
    pub arg_type: CoreType,
}

#[derive(Debug)]
pub struct Query {
    pub name: String,
    pub args: Vec<QueryArg>,
    pub query_val: String,
}
impl Query {
    pub fn from_contents(contents: &mut FileContents) -> Option<Query> {
        let name = match contents.take() {
            Some(Token::Literal(val)) => val,
            _ => return None,
        };
        let mut query_val = String::new();
        let mut args = Vec::new();

        loop {
            match contents.take() {
                Some(Token::Equals) => match contents.take() {
                    Some(Token::Literal(val)) => query_val = val,
                    _ => return None,
                },
                Some(Token::Literal(arg_name)) => {
                    if !matches!(contents.take(), Some(Token::Colon)) {
                        return None;
                    }
                    let typ_str = match contents.take() {
                        Some(Token::Literal(val)) => val,
                        _ => return None,
                    };
                    let typ = match CoreType::from_string(&typ_str) {
                        Some(val) => val,
                        _ => return None,
                    };
                    args.push(QueryArg {
                        name: arg_name,
                        arg_type: typ,
                    });
                }
                _ => break,
            }
        }

        if let Some(Token::Literal(query)) = contents.take() {
            query_val = query;
        }

        Some(Query {
            name,
            args,
            query_val,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::{Object, ObjectType};

    use super::*;

    #[test]
    fn basic_parse() {
        let query_string = "User @users #model {
                !base
                last_login datetime?
                name string
                email string
                user_type UserType
                subscription_id string?

                query SelectFromEmail email:string = \"SELECT %fields FROM %table_name WHERE %email = $email\"
            }
        ";
        let mut contents = FileContents::empty();
        contents.add_string(&query_string);
        let obj = Object::read_from_contents(ObjectType::Record, &mut contents);
        assert_eq!(obj.queries.len(), 1);
        let query = obj.queries.first().unwrap();
        assert_eq!(query.name, "SelectFromEmail");
        dbg!(obj.render_query(query));
    }
}
