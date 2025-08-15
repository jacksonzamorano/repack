use crate::blueprint::BlueprintSnippetDetails;

use super::{Field, Output, RepackStruct};

/// Enumeration of all possible error types that can occur during schema processing.
///
/// Each error kind represents a specific category of validation, parsing, or generation
/// error. The u32 representation provides unique error codes for debugging and logging.
/// Error codes are used in formatted error messages as E0001, E0002, etc.
#[derive(Debug)]
#[repr(u32)]
pub enum RepackErrorKind {
    CircularDependancy,
    ParentObjectDoesNotExist,
    CustomTypeNotDefined,
    TypeNotResolved,
    SnippetNotFound,
    DuplicateFieldNames,
    CannotCreateContext,
    FunctionInvalidSyntax,
    TypeNotSupported,
    CannotRead,
    CannotWrite,
    SnippetNotClosed,
    UnknownSnippet,
    VariableNotInScope,
    InvalidVariableModifier,
    UnknownLink,
    UnknownObject,
    QueryArgInvalidSyntax,
    QueryInvalidSyntax,
    InvalidSuper,
    FieldNotOnSuper,
    InvalidJoin,
    FieldNotOnJoin,
    SyntaxError,
    UnknownError,
}
impl Default for RepackErrorKind {
    fn default() -> Self {
        Self::UnknownError
    }
}
impl RepackErrorKind {
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::CircularDependancy => "This definition creates a circular dependancy with:",
            Self::ParentObjectDoesNotExist => "Parent object couldn't be found:",
            Self::TypeNotSupported => "Type is not allowed:",
            Self::CustomTypeNotDefined => "The custom type cannot be resolved:",
            Self::TypeNotResolved => "This type couldn't be resolved.",
            Self::SnippetNotFound => "Expected to use snippet, but it couldn't be found:",
            Self::DuplicateFieldNames => "A field already exists with this name.",
            Self::CannotCreateContext => "Cannot create a context:",
            Self::FunctionInvalidSyntax => "Function syntax is not vaild:",
            Self::CannotRead => "Cannot read the file:",
            Self::CannotWrite => "Cannot write the file:",
            Self::SnippetNotClosed => "Block was not closed:",
            Self::VariableNotInScope => "Variable was not found in scope:",
            Self::InvalidVariableModifier => "Unknown variable modifier specified:",
            Self::UnknownSnippet => "Specified snippet does not exist:",
            Self::UnknownLink => "Requested import but no link was defined for ",
            Self::UnknownObject => {
                "Attempted to resolve this dependancy but the object couldn't be found: "
            }
            Self::UnknownError => "An unknown error occured.",
            Self::SyntaxError => "Error when parsing ",
            Self::QueryInvalidSyntax => "Invalid query syntax.",
            Self::QueryArgInvalidSyntax => "Invalid query argument syntax.",
            Self::InvalidSuper => "Cannot use super without an inheritance.",
            Self::FieldNotOnSuper => "Field does not exist in this super.",
            Self::InvalidJoin => "Joined entity not found.",
            Self::FieldNotOnJoin => "Field does not exist in this join.",
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
        let details = self.error_details.unwrap_or_default();
        let stack = if self.stack.is_empty() {
            String::new()
        } else {
            format!("\n\n--- Context: ---\n{}", self.stack.join("\n"))
        };
        format!(
            "[E{:04}]{} {} {}{}",
            self.error as u32, self.specifier, msg, details, stack
        )
    }
}

/// Represents a complete error with context information for debugging.
///
/// RepackError combines an error type with contextual information about where
/// the error occurred (language, object, field) and provides detailed error
/// messages with stack traces for complex nested errors.
#[derive(Debug, Default)]
pub struct RepackError {
    /// The specific type/category of error that occurred
    pub error: RepackErrorKind,
    /// The location the error occured.
    pub specifier: String,
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
            error_details: Some(msg),
            stack: Vec::new(),
            ..Default::default()
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
    #[allow(dead_code)]
    pub fn from_obj(error: RepackErrorKind, obj: &RepackStruct) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({})", obj.name),
            stack: Vec::new(),
            ..Default::default()
        }
    }

    pub fn from_obj_with_msg(
        error: RepackErrorKind,
        obj: &RepackStruct,
        msg: String,
    ) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({})", obj.name),
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
    pub fn from_field(error: RepackErrorKind, obj: &RepackStruct, field: &Field) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({}.{})", obj.name, field.name),
            stack: Vec::new(),
            ..Default::default()
        }
    }

    pub fn from_field_with_msg(
        error: RepackErrorKind,
        obj: &RepackStruct,
        field: &Field,
        msg: String,
    ) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({}.{})", obj.name, field.name),
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_lang(error: RepackErrorKind, lang: &Output) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({})", lang.profile),
            stack: Vec::new(),
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn from_lang_with_obj(
        error: RepackErrorKind,
        lang: &Output,
        obj: &RepackStruct,
    ) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({} -> {})", lang.profile, obj.name),
            stack: Vec::new(),
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn from_lang_with_obj_msg(
        error: RepackErrorKind,
        lang: &Output,
        obj: &RepackStruct,
        msg: String,
    ) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({} -> {})", lang.profile, obj.name),
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    pub fn from_lang_with_obj_field_msg(
        error: RepackErrorKind,
        lang: &Output,
        obj: &RepackStruct,
        field: &Field,
        msg: String,
    ) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({} -> {}.{})", lang.profile, obj.name, field.name),
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    pub fn from_lang_with_msg(error: RepackErrorKind, lang: &Output, msg: String) -> RepackError {
        RepackError {
            error,
            specifier: format!(" ({})", lang.profile),
            error_details: Some(msg),
            stack: Vec::new(),
        }
    }

    pub fn add_to_stack(&mut self, snip: &BlueprintSnippetDetails) {
        self.stack
            .push(format!("\t- {} {}", snip.main_token, snip.secondary_token));
    }
}
