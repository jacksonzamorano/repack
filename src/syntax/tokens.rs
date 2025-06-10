#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Period,
    Comma,
    Plus,
    Pound,
    NewLine,
    Question,
    Star,
    Exclamation,
    At,
    Colon,
    Minus,
    Semicolon,
    Ampersand,
    Equals,

    Literal(String),
    OutputType,
    RecordType,
    StructType,
    SnippetType,
    EnumType,
    Ref,
    From,
    As,
    Where,
    Import,
}
impl Token {
    pub fn from_byte(byte: u8) -> Option<Token> {
        match byte {
            b'(' => Some(Token::OpenParen),
            b')' => Some(Token::CloseParen),
            b'[' => Some(Token::OpenBracket),
            b']' => Some(Token::CloseBracket),
            b'{' => Some(Token::OpenBrace),
            b'}' => Some(Token::CloseBrace),
            b'.' => Some(Token::Period),
            b',' => Some(Token::Comma),
            b'+' => Some(Token::Plus),
            b'#' => Some(Token::Pound),
            b'?' => Some(Token::Question),
            b'\n' => Some(Token::NewLine),
            b'*' => Some(Token::Star),
            b'!' => Some(Token::Exclamation),
            b'@' => Some(Token::At),
            b':' => Some(Token::Colon),
            b'-' => Some(Token::Minus),
            b';' => Some(Token::Semicolon),
            b'&' => Some(Token::Ampersand),
            b'=' => Some(Token::Equals),
            _ => None,
        }
    }
    pub fn from_string(string: &str) -> Token {
        match string.trim() {
            "output" => Token::OutputType,
            "record" => Token::RecordType,
            "struct" => Token::StructType,
            "from" => Token::From,
            "ref" => Token::Ref,
            "as" => Token::As,
            "where" => Token::Where,
            "import" => Token::Import,
            "snippet" => Token::SnippetType,
            "enum" => Token::EnumType,

            _ => Token::Literal(string.trim().to_string()),
        }
    }
}
