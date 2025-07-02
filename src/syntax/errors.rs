use crate::blueprint::SnippetDetails;

use super::{Field, Object, Output};

/// Enumeration of all possible error types that can occur during schema processing.
/// 
/// Each error kind represents a specific category of validation, parsing, or generation
/// error. The u32 representation provides unique error codes for debugging and logging.
/// Error codes are used in formatted error messages as E0001, E0002, etc.
#[derive(Debug)]
#[repr(u32)]
pub enum RepackErrorKind {
    CircularDependancy,
    RefFieldUnresolvable,
    JoinFieldUnresolvable,
    ParentObjectDoesNotExist,
    TableNameNotAllowed,
    NoTableName,
    CannotReuse,
    CannotInherit,
    NoFields,
    ManyNotAllowed,
    CustomTypeNotAllowed,
    CustomTypeNotDefined,
    TypeNotResolved,
    SnippetNotFound,
    DuplicateFieldNames,
    UnknownExplicitJoin,
    JoinObjectNotFound,
    JoinFieldNotFound,
    JoinNoTableName,
    CannotCreateContext,
    FunctionInvalidSyntax,
    TypeNotSupported,
    CannotRead,
    CannotWrite,
    SnippetNotClosed,
    VariableNotInScope,
    InvalidVariableModifier,
    UnknownLink,
}
impl RepackErrorKind {
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::CircularDependancy => "This definition creates a circular dependancy with:",
            Self::RefFieldUnresolvable => "Could not resolve the 'ref' reference:",
            Self::JoinFieldUnresolvable => "Could not resolve the 'from' reference:",
            Self::ParentObjectDoesNotExist => "Parent object couldn't be found:",
            Self::TableNameNotAllowed => "Table name isn't allowed in this context.",
            Self::NoTableName => "Table name is required in this context.",
            Self::CannotReuse => "Reuse is not available in this context.",
            Self::CannotInherit => "Inherit is not available in this context.",
            Self::NoFields => "No fields were found in this object.",
            Self::ManyNotAllowed => "Arrays aren't allowed in this context.",
            Self::CustomTypeNotAllowed => "Custom types are not available in this context.",
            Self::TypeNotSupported => "Type is not allowed:",
            Self::CustomTypeNotDefined => "The custom type cannot be resolved:",
            Self::TypeNotResolved => "This type couldn't be resolved.",
            Self::SnippetNotFound => "Expected to use snippet, but it couldn't be found:",
            Self::DuplicateFieldNames => "A field already exists with this name.",
            Self::UnknownExplicitJoin => "Unknown explicit join:",
            Self::JoinObjectNotFound => "Tried to join but the object was not found:",
            Self::JoinFieldNotFound => "Tried to join but the field was not found:",
            Self::JoinNoTableName => "Table name is required in a join, this entity does not: ",
            Self::CannotCreateContext => "Cannot create a context:",
            Self::FunctionInvalidSyntax => "Function syntax is not vaild:",
            Self::CannotRead => "Cannot read the file:",
            Self::CannotWrite => "Cannot write the file:",
            Self::SnippetNotClosed => "Block was not closed:",
            Self::VariableNotInScope => "Variable was not found in scope:",
            Self::InvalidVariableModifier => "Unknown variable modifier specified:",
            Self::UnknownLink => "Requested import but no link was defined for ",
        }
    }
}

impl RepackError {
    /// Converts the error into a formatted string message for display.
    /// 
    /// This method creates a comprehensive error message that includes:
    /// - Error code (E0001 format)
    /// - Context location (language -> object.field)
    /// - Error description and details
    /// - Stack trace for nested errors
    /// 
    /// # Returns
    /// A formatted string suitable for console output or logging
    pub fn into_string(self) -> String {
        let msg = self.error.as_string();
        let loc = match (self.lang_name, self.obj_name, self.field_name) {
            (Some(lang), None, None) => format!(" ({})", lang),
            (None, Some(obj_name), None) => format!(" ({})", obj_name),
            (None, Some(obj_name), Some(field_name)) => format!(" ({}.{})", obj_name, field_name),
            (Some(lang), Some(obj_name), None) => format!(" ({} -> {})", lang, obj_name),
            (Some(lang), Some(obj_name), Some(field_name)) => {
                format!(" ({} -> {}.{})", lang, obj_name, field_name)
            }
            (_, _, _) => String::new(),
        };

        let details = self.error_details.unwrap_or_default();
        let stack = if self.stack.is_empty() {
            String::new()
        } else {
            format!("\n\n--- Context: ---\n{}", self.stack.join("\n"))
        };
        format!("[E{:04}]{} {} {}{}", self.error as u32, loc, msg, details, stack)
    }
}

/// Represents a complete error with context information for debugging.
/// 
/// RepackError combines an error type with contextual information about where
/// the error occurred (language, object, field) and provides detailed error
/// messages with stack traces for complex nested errors.
#[derive(Debug)]
pub struct RepackError {
    /// The specific type/category of error that occurred
    pub error: RepackErrorKind,
    /// The target language/profile being processed when the error occurred
    pub lang_name: Option<String>,
    /// The object name where the error occurred
    pub obj_name: Option<String>,
    /// The field name where the error occurred (if applicable)
    pub field_name: Option<String>,
    /// Additional details or context about the error
    pub error_details: Option<String>,
    /// Stack trace for nested processing contexts (e.g., snippet processing)
    pub stack: Vec<String>,
}

impl RepackError {
    /// Creates a global error without specific object or field context.
    /// 
    /// Used for system-level errors like file I/O issues or blueprint loading problems.
    /// 
    /// # Arguments
    /// * `error` - The type of error that occurred
    /// * `msg` - Detailed error message
    pub fn global(error: RepackErrorKind, msg: String) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: None,
            field_name: None,
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }
    /// Creates an error associated with a specific object.
    /// 
    /// Used for object-level validation errors like missing table names
    /// or inheritance issues.
    /// 
    /// # Arguments
    /// * `error` - The type of error that occurred
    /// * `obj` - The object where the error was found
    pub fn from_obj(error: RepackErrorKind, obj: &Object) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: Some(obj.name.to_string()),
            field_name: None,
            error_details: None,
            stack: Vec::new(),
        }
    }

    pub fn from_obj_with_msg(error: RepackErrorKind, obj: &Object, msg: String) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: Some(obj.name.to_string()),
            field_name: None,
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    /// Creates an error associated with a specific field in an object.
    /// 
    /// Used for field-level validation errors like type resolution failures
    /// or invalid field configurations.
    /// 
    /// # Arguments
    /// * `error` - The type of error that occurred
    /// * `obj` - The object containing the problematic field
    /// * `field` - The field where the error was found
    pub fn from_field(error: RepackErrorKind, obj: &Object, field: &Field) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: Some(obj.name.to_string()),
            field_name: Some(field.name.to_string()),
            error_details: None,
            stack: Vec::new(),
        }
    }

    pub fn from_field_with_msg(
        error: RepackErrorKind,
        obj: &Object,
        field: &Field,
        msg: String,
    ) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: Some(obj.name.to_string()),
            field_name: Some(field.name.to_string()),
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_lang(error: RepackErrorKind, lang: &Output) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: None,
            field_name: None,
            error_details: None,
            stack: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_lang_with_obj(error: RepackErrorKind, lang: &Output, obj: &Object) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: Some(obj.name.to_string()),
            field_name: None,
            error_details: None,
            stack: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_lang_with_obj_msg(
        error: RepackErrorKind,
        lang: &Output,
        obj: &Object,
        msg: String,
    ) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: Some(obj.name.to_string()),
            field_name: None,
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    pub fn from_lang_with_obj_field_msg(
        error: RepackErrorKind,
        lang: &Output,
        obj: &Object,
        field: &Field,
        msg: String,
    ) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: Some(obj.name.to_string()),
            field_name: Some(field.name.to_string()),
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    pub fn from_lang_with_msg(error: RepackErrorKind, lang: &Output, msg: String) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: None,
            field_name: None,
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    pub fn add_to_stack(&mut self, snip: &SnippetDetails) {
        self.stack
            .push(format!("\t- {} {}", snip.main_token, snip.secondary_token));
    }
}
