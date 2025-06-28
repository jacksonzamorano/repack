use super::{FieldFunction, FieldType, FileContents, Token};

/// Describes where a field's data comes from and how it should be accessed.
/// 
/// FieldLocation combines the reference type (local, foreign object, join)
/// with the specific field name, providing complete information about
/// how to locate and access the field's data during code generation.
#[derive(Debug, Clone)]
pub struct FieldLocation {
    /// The type of reference this field makes (local, object reference, join)
    pub reference: FieldReferenceKind,
    /// The specific field name being referenced
    /// For local fields: the field name in the current object
    /// For joins: the field name in the joined object
    pub name: String,
}

/// Defines the different ways a field can reference data in the schema.
/// 
/// This enum categorizes how fields access their data, from simple local fields
/// to complex cross-object references and joins. The numeric values provide
/// ordering for dependency resolution and code generation sequencing.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[repr(i32)]
pub enum FieldReferenceKind {
    /// Field is defined locally in the current object (value = 1)
    Local = 1,
    /// Field references a field from another object type (value = 2)
    /// String contains the name of the target object
    FieldType(String) = 2,
    /// Field is accessed through an implicit join relationship (value = 3)
    /// String contains the name of the join field in the current object
    ImplicitJoin(String) = 3,
    /// Field is accessed through an explicitly defined join (value = 4)
    /// String contains the name of the explicit join definition
    ExplicitJoin(String) = 4,
}

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
    pub field_type_string: Option<String>,
    /// Describes where this field's data comes from (local, join, reference)
    pub location: FieldLocation,
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
    /// Returns the resolved field type, panicking if not yet resolved.
    /// 
    /// This method should only be called after the type resolution phase
    /// has completed. It's primarily used in blueprint rendering and
    /// code generation where types are guaranteed to be resolved.
    /// 
    /// # Panics
    /// Panics if the field type has not been resolved yet
    /// 
    /// # Returns
    /// A reference to the resolved FieldType
    pub fn field_type(&self) -> &FieldType {
        self.field_type.as_ref().unwrap()
    }

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
    /// - Implicit joins: `field_name from(join_field.target_field)`
    /// - Explicit joins: `field_name with(join_name.field_name)`
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
        let field_type_loc: (Option<FieldType>, Option<String>, FieldLocation) = match type_token {
            Token::Literal(literal) => (
                FieldType::from_string(&literal),
                Some(literal.clone()),
                FieldLocation {
                    reference: FieldReferenceKind::Local,
                    name: name.clone(),
                },
            ),
            Token::From => {
                contents.skip(); // Skip (
                let Some(Token::Literal(join_field_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip .
                let Some(Token::Literal(field_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip )
                (
                    None,
                    None,
                    FieldLocation {
                        reference: FieldReferenceKind::ImplicitJoin(join_field_name),
                        name: field_name,
                    },
                )
            }
            Token::Ref => {
                contents.skip(); // Skip (
                let Some(Token::Literal(entity_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip .
                let Some(Token::Literal(field_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip )
                (
                    None,
                    None,
                    FieldLocation {
                        reference: FieldReferenceKind::FieldType(entity_name),
                        name: field_name,
                    },
                )
            }
            Token::With => {
                contents.skip(); // Skip (
                let Some(Token::Literal(join_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip .
                let Some(Token::Literal(field_name)) = contents.take() else {
                    return None;
                };
                contents.skip(); // Skip )
                (
                    None,
                    None,
                    FieldLocation {
                        reference: FieldReferenceKind::ExplicitJoin(join_name),
                        name: field_name,
                    },
                )
            }
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
            location: field_type_loc.2,
            optional,
            array: is_many,
            functions,
        })
    }
}
