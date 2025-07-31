use super::{FieldFunction, FieldType, FileContents, Token};

/// Represents a single field definition within an object.
///
/// Field contains all the metadata needed to generate code for a single
/// property or attribute of an object, including its type, location,
/// constraints, and any associated functions or transformations.
#[derive(Debug, Clone)]
pub struct Field {
    /// The field name as it appears in generated code
    /// May be an alias if the field is referenced from another location
    pub name: String,
    /// The original string representation of the field type from the schema
    /// Used for error reporting and debugging; None for computed fields
    pub field_type_string: String,
    /// The resolved type information for this field
    /// None during initial parsing, resolved during type checking phase
    pub field_type: Option<FieldType>,
    /// Whether this field can be null/undefined in the generated code
    pub optional: bool,
    /// Whether this field represents an array/collection of values
    pub array: bool,
    /// Custom functions or transformations applied to this field
    /// Used for computed properties, validation, and formatting
    pub functions: Vec<FieldFunction>,
}
impl Field {
    /// Filters field functions by their namespace.
    ///
    /// Returns all functions defined on this field that belong to the
    /// specified namespace. This allows different blueprints to define
    /// field-specific functions for different target languages.
    ///
    /// # Arguments
    /// * `ns` - The namespace identifier to filter by
    ///
    /// # Returns
    /// A vector of references to functions in the specified namespace
    pub fn functions_in_namespace(&self, ns: &str) -> Vec<&FieldFunction> {
        self.functions
            .iter()
            .filter(|x| x.namespace == *ns)
            .collect()
    }

    /// Parses a Field definition from the input file contents.
    ///
    /// This method reads field definition syntax and constructs a Field instance
    /// with its type, modifiers (optional, array), and any associated functions.
    /// It handles different field reference syntaxes:
    /// - Direct types: `field_name type_name`
    /// - References: `field_name ref(ObjectName.field_name)`
    /// - Direct type references: `field_name Type`
    /// - Foreign key references: `field_name ref(Object.field)`
    ///
    /// # Arguments
    /// * `name` - The field name as parsed from the schema
    /// * `contents` - Mutable reference to the file contents being parsed
    ///
    /// # Returns
    /// * `Some(Field)` if parsing succeeds
    /// * `None` if the field definition is malformed
    pub fn from_contents(name: String, contents: &mut FileContents) -> Option<Field> {
        let type_token = contents.take()?;
        let field_type_loc: (Option<FieldType>, String) = match type_token {
            Token::Literal(literal) => (FieldType::from_string(&literal), literal.clone()),
            _ => {
                return None;
            }
        };

        let is_many = match contents.peek() {
            Some(Token::OpenBracket) => {
                contents.skip();
                match contents.peek() {
                    Some(Token::CloseBracket) => {
                        contents.skip();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        };

        let optional = match contents.peek() {
            Some(Token::Question) => {
                contents.next();
                true
            }
            _ => false,
        };
        let mut functions = Vec::new();

        while let Some(token) = contents.take() {
            match token {
                Token::Literal(name) => {
                    if let Some(func) = FieldFunction::from_contents(name, contents) {
                        functions.push(func);
                    }
                }
                Token::NewLine => {
                    break;
                }
                _ => {}
            }
        }

        Some(Field {
            name,
            field_type: field_type_loc.0,
            field_type_string: field_type_loc.1,
            optional,
            array: is_many,
            functions,
        })
    }
}
