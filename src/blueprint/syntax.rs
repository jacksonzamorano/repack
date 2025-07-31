#[derive(Debug, Clone)]
pub enum BlueprintToken {
    Literal(String),
    Snippet(BlueprintSnippetDetails),
    Close(String)
}

#[derive(Debug, Clone, Default)]
pub struct BlueprintSnippetDetails {
    pub main_token: String,
    pub secondary_token: String,
    pub contents: String,
    pub autoclose: bool,
}
