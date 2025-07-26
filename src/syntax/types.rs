use std::fmt::{Debug, Display};

/// Represents the category of custom field types that can be referenced in schema definitions.
///
/// Custom field types are user-defined types that extend beyond the built-in core types.
/// They allow for complex data modeling by referencing other schema elements.
#[derive(Debug, Clone, PartialEq)]
pub enum CustomFieldType {
    /// References another object/entity in the schema, establishing a relationship
    /// between entities. Used for foreign key references and complex nested structures.
    Object,
    /// References an enumeration type defined elsewhere in the schema.
    /// Enums provide a way to define a fixed set of possible values for a field.
    Enum,
}

/// Represents the fundamental built-in data types supported by the schema system.
///
/// These are the primitive types that can be used directly in field definitions
/// without requiring custom type definitions. Each core type maps to common
/// data types found across programming languages and databases.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum CoreType {
    /// UTF-8 text string type. Maps to String/varchar in most target languages.
    String,
    /// 64-bit signed integer. Provides large numeric range for IDs and counters.
    Int64,
    /// 32-bit signed integer. Standard integer type for most numeric fields.
    Int32,
    /// 64-bit floating point number. Used for decimal and scientific calculations.
    Float64,
    /// Boolean true/false value. Maps to bool/boolean in target languages.
    Boolean,
    /// Date and time timestamp. Represents a specific point in time.
    DateTime,
    /// Universally Unique Identifier. 128-bit identifier for unique entity references.
    Uuid,
    /// Byte array
    Bytes,
}
impl CoreType {
    /// Parses a string literal into a CoreType enum variant.
    ///
    /// This function is used during schema parsing to convert type names
    /// from the input schema into their corresponding enum variants.
    ///
    /// # Arguments
    /// * `s` - The string representation of the type (e.g., "string", "int64")
    ///
    /// # Returns
    /// * `Some(CoreType)` if the string matches a valid core type
    /// * `None` if the string doesn't match any known core type
    ///
    /// # Examples
    /// ```
    /// assert_eq!(CoreType::from_string("string"), Some(CoreType::String));
    /// assert_eq!(CoreType::from_string("invalid"), None);
    /// ```
    pub fn from_string(s: &str) -> Option<CoreType> {
        Some(match s {
            "string" => Self::String,
            "int64" => Self::Int64,
            "int32" => Self::Int32,
            "float64" => Self::Float64,
            "boolean" => Self::Boolean,
            "datetime" => Self::DateTime,
            "uuid" => Self::Uuid,
            "bytes" => Self::Bytes,
            _ => return None,
        })
    }
}
impl Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::String => "string".to_string(),
            Self::Int64 => "int64".to_string(),
            Self::Int32 => "int32".to_string(),
            Self::Float64 => "float64".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::DateTime => "datetime".to_string(),
            Self::Uuid => "uuid".to_string(),
            Self::Bytes => "bytes".to_string(),
        };
        write!(f, "{res}")
    }
}

/// Represents the complete type system for fields in schema definitions.
///
/// FieldType unifies both built-in core types and user-defined custom types,
/// providing a comprehensive type system for schema modeling. Every field
/// in an object must have a FieldType that defines its data representation.
#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
    /// A built-in primitive type (string, int, boolean, etc.)
    Core(CoreType),
    /// A custom user-defined type with its name and category.
    /// The String contains the type name, and CustomFieldType indicates
    /// whether it's an Object reference or Enum reference.
    Custom(String, CustomFieldType),
}
impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Core(s) => {
                write!(f, "{s}")
            }
            FieldType::Custom(s, _) => {
                write!(f, "{s}")
            }
        }
    }
}
impl FieldType {
    /// Attempts to parse a string into a FieldType, checking only core types.
    ///
    /// This function tries to match the input string against known core types.
    /// If no core type matches, it returns None. Custom types are resolved
    /// later during the type resolution phase of parsing.
    ///
    /// # Arguments
    /// * `s` - The string representation of the type
    ///
    /// # Returns
    /// * `Some(FieldType::Core(_))` if the string matches a core type
    /// * `None` if the string doesn't match any core type
    pub fn from_string(s: &str) -> Option<FieldType> {
        CoreType::from_string(s).map(FieldType::Core)
    }
}
