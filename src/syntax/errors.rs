use super::{Field, Object, Output};

#[derive(Debug)]
#[repr(u32)]
pub enum RepackErrorKind {
    CannotWriteFile,
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
    TypeNotResolved,
}
impl RepackErrorKind {
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::CannotWriteFile => {
                "Cannot write to a requested file."
            }
            Self::UnsupportedFieldType => {
                "This builder doesn't support"
            }
            Self::ObjectNotIncluded => {
                "The following object was required but not a part of this output:"
            }
            RepackErrorKind::CircularDependancy => {
                "This definition creates a circular dependancy with:"
            }
            RepackErrorKind::UnknownLanguage => "This language isn't available.",
            RepackErrorKind::RefFieldUnresolvable => "Could not resolve the 'ref' reference.",
            RepackErrorKind::JoinFieldUnresolvable => "Could not resolve the 'from' reference.",
            RepackErrorKind::ParentObjectDoesNotExist => "Parent object couldn't be found:",
            RepackErrorKind::TableNameNotAllowed => "Table name isn't allowed in this context.",
            RepackErrorKind::NoTableName => "Table name is required in this context.",
            RepackErrorKind::CannotReuse => "Reuse is not available in this context.",
            RepackErrorKind::CannotInherit => "Inherit is not available in this context.",
            RepackErrorKind::NoFields => "No fields were found in this object.",
            RepackErrorKind::ManyNotAllowed => "Command 'many' is not valid in this context.",
            RepackErrorKind::PrimaryKeyOptional => "Primary keys cannot be optional.",
            RepackErrorKind::CustomTypeNotAllowed => {
                "Custom types are not available in this context."
            }
            RepackErrorKind::TypeNotResolved => "This type couldn't be resolved.",
        }
    }
}

impl RepackError {
    pub fn into_string(self) -> String {
        let msg = self.error.as_string();
        let loc = match (self.lang_name, self.obj_name, self.field_name) {
            (Some(lang), _, _) => format!(" ({})", lang),
            (_, None, None) => String::new(),
            (_, Some(obj_name), None) => format!(" ({})", obj_name),
            (_, Some(obj_name), Some(field_name)) => format!(" ({}.{})", obj_name, field_name),
            (_, None, Some(field_name)) => format!(" (_.{})", field_name),
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
