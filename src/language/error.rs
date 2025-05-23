use super::{Field, Object};

pub enum ValidationError {
    Object(ObjectValidationError),
    Field(FieldValidationError),
}
impl ValidationError {
    pub fn to_string(self) -> String {
        match self {
            ValidationError::Object(err) => err.to_string(),
            ValidationError::Field(err) => err.to_string(),
        }
    }
}

pub enum ObjectValidationErrorType {
    CannotInherit = 1,
    CannotReuse,
    TableNameRequired,
    TableNameNotAllowed,
    NoFields,
}
pub enum FieldValidationErrorType {
    InvalidRefObject = 1,
    InvalidRefField = 2,
    CustomNotAllowed,
    ManyNotAllowed,
    PrimaryKeyOptional,
}

pub struct ObjectValidationError {
    error_type: ObjectValidationErrorType,
    object_name: String,
}
impl ObjectValidationError {
    pub fn new(error_type: ObjectValidationErrorType, object: &Object) -> Self {
        Self {
            error_type,
            object_name: object.name.clone(),
        }
    }

    pub fn to_string(self) -> String {
        let err = match self.error_type {
            ObjectValidationErrorType::CannotInherit => {
                format!("Struct types don't support inheritance.")
            }
            ObjectValidationErrorType::CannotReuse => {
                format!("Reuse is not allowed in this context.")
            }
            ObjectValidationErrorType::TableNameRequired => {
                format!("Records must have a table name.")
            }
            ObjectValidationErrorType::TableNameNotAllowed => {
                format!("Structs cannot have a table name.")
            }
            ObjectValidationErrorType::NoFields => {
                format!("Records must have at least one field.")
            }
        };

        return format!("[OE{:04}] {}: {}", self.error_type as u8, self.object_name, err);
    }
}
pub struct FieldValidationError {
    error_type: FieldValidationErrorType,
    object_name: String,
    field_name: String,
}
impl FieldValidationError {
    pub fn new(error_type: FieldValidationErrorType, object: &Object, field: &Field) -> Self {
        Self {
            error_type,
            object_name: object.name.clone(),
            field_name: field.name.clone(),
        }
    }

    pub fn to_string(self) -> String {
        let err = match self.error_type {
            FieldValidationErrorType::InvalidRefField => {
                format!("Reference field doesn't exist in the object.")
            },
            FieldValidationErrorType::InvalidRefObject => {
                format!("Reference object doesn't exist.")
            }
            FieldValidationErrorType::CustomNotAllowed => {
                format!("Custom types are not allowed in this context.")
            }
            FieldValidationErrorType::PrimaryKeyOptional => {
                format!("Primary key cannot be optional.")
            }
            FieldValidationErrorType::ManyNotAllowed => {
                format!("Many-to-many relationships are not allowed in this context.")
            }
        };

        return format!("[FE{:04}] {}.{}: {}", self.error_type as u8, self.object_name, self.field_name, err);
    }
}