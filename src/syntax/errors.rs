use super::{Field, Object, Output};

#[derive(Debug)]
#[repr(u32)]
pub enum RepackErrorKind {
    CannotWriteFile,
    UnsupportedObjectType,
    UnsupportedFieldType,
    ObjectNotIncluded,
    UnknownLanguage,
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
    PrimaryKeyOptional,
    CustomTypeNotAllowed,
    CustomTypeNotDefined,
    TypeNotResolved,
    ExpectedReference,
}
impl RepackErrorKind {
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::UnsupportedObjectType => "This object isn't supported by this builder.",
            Self::CannotWriteFile => "Cannot write to a requested file.",
            Self::UnsupportedFieldType => "This builder doesn't support",
            Self::ObjectNotIncluded => {
                "The following object was required but not a part of this output:"
            }
            Self::CircularDependancy => {
                "This definition creates a circular dependancy with:"
            }
            Self::UnknownLanguage => "Repack doesn't recognize this language. Make sure you are running the latest version.",
            Self::RefFieldUnresolvable => "Could not resolve the 'ref' reference:",
            Self::JoinFieldUnresolvable => "Could not resolve the 'from' reference:",
            Self::ParentObjectDoesNotExist => "Parent object couldn't be found:",
            Self::TableNameNotAllowed => "Table name isn't allowed in this context.",
            Self::NoTableName => "Table name is required in this context.",
            Self::CannotReuse => "Reuse is not available in this context.",
            Self::CannotInherit => "Inherit is not available in this context.",
            Self::NoFields => "No fields were found in this object.",
            Self::ManyNotAllowed => "Command 'many' is not valid in this context.",
            Self::PrimaryKeyOptional => "Primary keys cannot be optional.",
            Self::CustomTypeNotAllowed => {
                "Custom types are not available in this context."
            }
            Self::CustomTypeNotDefined => "The custom type cannot be resolved:",
            Self::ExpectedReference => {
                "Expected a reference but got a local or join field."
            },
            Self::TypeNotResolved => "This type couldn't be resolved.",
        }
    }
}

impl RepackError {
    pub fn into_string(self) -> String {
        let msg = self.error.as_string();
        let loc = match (self.lang_name, self.obj_name, self.field_name) {
            (Some(lang), None, None) => format!(" ({})", lang),
            (None, Some(obj_name), None) => format!(" ({})", obj_name),
            (None, Some(obj_name), Some(field_name)) => format!(" ({}.{})", obj_name, field_name),
            (Some(lang), Some(obj_name), None) => format!(" ({} -> {})", lang, obj_name),
            (Some(lang), Some(obj_name), Some(field_name)) => format!(" ({} -> {}.{})", lang, obj_name, field_name),
            (_, _, _) => String::new(),
        };

        let details = self.error_details.unwrap_or(String::new());

        format!("[E{:04}]{} {} {}", self.error as u32, loc, msg, details)
    }
}

#[derive(Debug)]
pub struct RepackError {
    pub error: RepackErrorKind,
    pub lang_name: Option<String>,
    pub obj_name: Option<String>,
    pub field_name: Option<String>,
    pub error_details: Option<String>,
}

impl RepackError {
    pub fn from_obj(error: RepackErrorKind, obj: &Object) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: Some(obj.name.to_string()),
            field_name: None,
            error_details: None,
        }
    }

    pub fn from_obj_with_msg(error: RepackErrorKind, obj: &Object, msg: String) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: Some(obj.name.to_string()),
            field_name: None,
            error_details: Some(msg),
        }
    }

    pub fn from_field(error: RepackErrorKind, obj: &Object, field: &Field) -> RepackError {
        RepackError {
            error,
            lang_name: None,
            obj_name: Some(obj.name.to_string()),
            field_name: Some(field.name.to_string()),
            error_details: None,
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
        }
    }

    pub fn from_lang(error: RepackErrorKind, lang: &Output) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: None,
            field_name: None,
            error_details: None,
        }
    }

    pub fn from_lang_with_obj(error: RepackErrorKind, lang: &Output, obj: &Object) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: Some(obj.name.to_string()),
            field_name: None,
            error_details: None,
        }
    }

    pub fn from_lang_with_msg(error: RepackErrorKind, lang: &Output, msg: String) -> RepackError {
        RepackError {
            error,
            lang_name: Some(lang.profile.clone()),
            obj_name: None,
            field_name: None,
            error_details: Some(msg),
        }
    }
}
