#[derive(Debug, Clone)]
pub enum FlyToken {
    Literal(String),
    Snippet(SnippetDetails),
    SnippetEnd(String)
}

#[derive(Debug, Clone, Default)]
pub struct SnippetDetails {
    pub main_token: String,
    pub secondary_token: String,
    pub contents: String,
    pub is_ended: bool,
}
