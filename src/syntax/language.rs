use super::{FileContents, RepackError, Token};
use std::collections::HashMap;

/// Represents an output configuration that specifies code generation targets.
///
/// Output defines how and where code should be generated for a specific target language
/// or format. Each output maps to a blueprint that handles the actual code generation.
#[derive(Debug)]
pub struct Output {
    /// The blueprint profile name (e.g., "rust", "typescript", "postgres")
    /// This must match a built-in blueprint or loaded external blueprint
    pub profile: String,
    /// Optional directory path where generated files should be written
    /// If None, files are written to the current directory
    pub location: Option<String>,
    /// List of categories to include in this output (e.g., "#model", "#api")
    /// Only objects/enums tagged with these categories will be generated
    pub categories: Vec<String>,
    /// Additional options passed to the blueprint for customization
    /// Used for blueprint-specific configuration like package names
    pub options: HashMap<String, String>,
    /// List of categories to explicitly exclude from this output
    /// Objects tagged with these categories will not be generated
    pub exclude: Vec<String>,
}
impl Output {
    /// Parses an Output definition from the input file contents.
    ///
    /// This method reads output configuration syntax and constructs an Output instance
    /// with its blueprint profile, location, categories, and options. It handles the
    /// syntax: `output profile @location #category1 #category2 { options }`
    ///
    /// # Arguments
    /// * `contents` - Mutable reference to the file contents being parsed
    ///
    /// # Returns
    /// An Output instance with all parsed configuration options
    pub fn from_contents(contents: &mut FileContents) -> Option<Output> {
        let Some(name_opt) = contents.next() else {
            panic!("Read record type, expected a name but got end of file.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!("Read record type, expected a name but got {name_opt:?}");
        };
        let output_language = name_ref.to_string();
        let mut location = None;
        let mut options = HashMap::new();
        let mut categories = Vec::new();
        let mut exclude = Vec::new();

        let mut empty = false;
        while let Some(token) = contents.next() {
            match token {
                Token::At => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        location = Some(lit.to_string());
                    }
                }
                Token::Pound => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        categories.push(lit.to_string());
                    }
                }
                Token::OpenBrace => {
                    break;
                }
                Token::Semicolon => {
                    empty = true;
                    break;
                }
                _ => {}
            }
        }

        if !empty {
            while let Some(token) = contents.next() {
                match token {
                    Token::Literal(lit) => {
                        let key = lit.to_string();
                        let value = match contents.next() {
                            Some(Token::Literal(lit)) => lit.to_string(),
                            _ => {
                                continue;
                            }
                        };
                        options.insert(key, value);
                    }
                    Token::CloseBrace => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        Some(Output {
            profile: output_language,
            location,
            categories,
            exclude,
            options,
        })
    }

    pub fn errors(&self) -> Vec<RepackError> {
        
        // if OutputProfile::from_keyword(&self.profile).is_none() {
        //     errors.push(RepackError::from_lang(
        //         RepackErrorKind::UnknownLanguage,
        //         self,
        //     ));
        // }
        Vec::new()
    }
}
