use super::{CoreType, FileContents, Token};

pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete
}

pub struct QueryArg {
    pub name: String,
    pub arg_type: CoreType,
}

pub struct Query {
    pub query_type: QueryType,
    pub name: String,
    pub arg_names: Vec<QueryArg>,
    pub contents: String,
}
impl Query {
    fn from_contents(query_type: QueryType, contents: &mut FileContents) -> Option<Query> {
        let name = match contents.next() {
            Some(Token::Literal(val)) => val,
            _ => return None
        };
        let mut arg_names = Vec::new();

        Some(Query { query_type, name, arg_names, contents: String::new() })
    }

}
