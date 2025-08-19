use super::{FileContents, Token, RepackError, RepackErrorKind};

#[derive(Debug)]
pub struct RepackEnumCase {
    pub name: String,
    pub value: Option<String>,
}

/// Represents an enumeration type definition in the schema.
///
/// Enums define a fixed set of possible values that can be used as field types.
/// They are useful for representing status codes, categories, or any field
/// that should be restricted to a predefined set of values.
#[derive(Debug)]
pub struct RepackEnum {
    /// The unique name identifier for this enumeration
    pub name: String,
    /// Tags/categories for organizing and filtering enums during generation
    pub categories: Vec<String>,
    /// The list of possible values this enum can take
    pub options: Vec<RepackEnumCase>,
}
impl RepackEnum {
    /// Parses an Enum definition from the input file contents.
    ///
    /// This method reads the enum definition syntax and constructs an Enum instance
    /// with its name, categories (marked with #), and the list of possible values
    /// enclosed in braces.
    ///
    /// # Arguments
    /// * `contents` - Mutable reference to the file contents being parsed
    ///
    /// # Returns
    /// A fully constructed Enum with all parsed options and metadata
    ///
    /// # Panics
    /// Panics if the expected enum name is missing or malformed
    pub fn read_from_contents(contents: &mut FileContents) -> Result<RepackEnum, RepackError> {
        let Some(name_opt) = contents.next() else {
            return Err(RepackError::global(
                RepackErrorKind::ParseIncomplete,
                "enum name".to_string()
            ));
        };
        let Token::Literal(name_ref) = name_opt else {
            return Err(RepackError::global(
                RepackErrorKind::ParseIncomplete,
                format!("{name_opt:?}")
            ));
        };
        let name = name_ref.to_string();
        let mut options = Vec::new();
        let mut categories = Vec::new();

        'header: while let Some(token) = contents.next() {
            match token {
                Token::Pound => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        categories.push(lit.to_string());
                    }
                }
                Token::OpenBrace => {
                    break 'header;
                }
                _ => {}
            }
        }

        'cmd: while let Some(token) = contents.take() {
            match token {
                Token::CloseBrace => {
                    break 'cmd;
                }
                Token::Literal(lit) => {
                    let mut cs = RepackEnumCase {
                        name: lit,
                        value: None,
                    };
                    if let Some(Token::Literal(val)) = contents.take() { cs.value = Some(val) }
                    options.push(cs);
                }
                _ => {}
            }
        }

        Ok(RepackEnum {
            name,
            categories,
            options,
        })
    }
}
