pub struct QueryArg {
    pub name: String,
    pub typ: String,
}
pub struct Query {
    pub name: String,
    pub args: Vec<QueryArg>,
    pub contents: Vec<String>,
}
