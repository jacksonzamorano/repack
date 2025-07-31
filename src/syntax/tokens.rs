/// Represents the lexical tokens that can appear in repack schema files.
///
/// Token defines all the symbols, keywords, and constructs that the parser
/// recognizes during lexical analysis. These tokens form the building blocks
/// of the schema syntax and are used throughout the parsing process.
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
    Hat,

    Literal(String),
    OutputType,
    RecordType,
    SyntheticType,
    StructType,
    SnippetType,
    EnumType,
    Ref,
    From,
    As,
    Where,
    Import,
    With,
    Blueprint,
    Convert,
    Configuration,
    Instance,
    Select,
    Insert,
    Update,
    Delete,
}
impl Token {
    /// Converts a single byte character into a Token if it matches a known symbol.
    ///
    /// This method handles the recognition of single-character tokens like
    /// parentheses, brackets, operators, and punctuation marks during tokenization.
    ///
    /// # Arguments
    /// * `byte` - The byte character to convert
    ///
    /// # Returns
    /// * `Some(Token)` if the byte matches a recognized symbol
    /// * `None` if the byte is not a recognized single-character token
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
            b'^' => Some(Token::Hat),
            _ => None,
        }
    }
    /// Converts a string into a Token, checking for keywords first.
    ///
    /// This method recognizes schema keywords (like "record", "enum", "output")
    /// and converts them to their corresponding token types. If the string
    /// doesn't match any keyword, it's treated as a literal identifier.
    ///
    /// # Arguments
    /// * `string` - The string to convert to a token
    ///
    /// # Returns
    /// A Token representing either a keyword or a literal string
    pub fn from_string(string: &str) -> Token {
        match string.trim() {
            "output" => Token::OutputType,
            "record" => Token::RecordType,
            "synthetic" => Token::SyntheticType,
            "struct" => Token::StructType,
            "from" => Token::From,
            "ref" => Token::Ref,
            "as" => Token::As,
            "where" => Token::Where,
            "import" => Token::Import,
            "snippet" => Token::SnippetType,
            "enum" => Token::EnumType,
            "with" => Token::With,
            "conversion" => Token::Convert,
            "blueprint" => Token::Blueprint,
            "configuration" => Token::Configuration,
            "instance" => Token::Instance,
            "select" => Token::Select,
            "insert" => Token::Insert,
            "update" => Token::Update,
            "delete" => Token::Delete,

            _ => Token::Literal(string.trim().to_string()),
        }
    }
}
