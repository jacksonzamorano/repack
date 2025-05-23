use super::{Field, Object};
impl ObjectValidationError {
    pub fn message(self) -> String {
        let err = match self.error_type {
            ObjectValidationErrorType::CannotInherit => {
                "Struct types don't support inheritance.".to_string()
            }
            ObjectValidationErrorType::CannotReuse => {
                "Reuse is not allowed in this context.".to_string()
            }
            ObjectValidationErrorType::TableNameRequired => {
                "Records must have a table name.".to_string()
            }
            ObjectValidationErrorType::TableNameNotAllowed => {
                "Structs cannot have a table name.".to_string()
            }
            ObjectValidationErrorType::NoFields => {
                "Records must have at least one field.".to_string()
            }
        };

        format!(
            "[OE{:04}] {}: {}",
            self.error_type as u8, self.object_name, err
        )
    }
}

impl FieldValidationError {
    fn message(self) -> String {
        let err = match self.error_type {
            FieldValidationErrorType::InvalidRefField => {
                "Reference field doesn't exist in the object.".to_string()
            }
            FieldValidationErrorType::InvalidRefObject => {
                "Reference object doesn't exist.".to_string()
            }
            FieldValidationErrorType::CustomNotAllowed => {
                "Custom types are not allowed in this context.".to_string()
            }
            FieldValidationErrorType::PrimaryKeyOptional => {
                "Primary key cannot be optional.".to_string()
            }
            FieldValidationErrorType::ManyNotAllowed => {
                "Many-to-many relationships are not allowed in this context.".to_string()
            }
        };

        format!(
            "[FE{:04}] {}.{}: {}",
            self.error_type as u8, self.object_name, self.field_name, err
        )
    }
}

pub enum ValidationError {
    Object(ObjectValidationError),
    Field(FieldValidationError),
}
impl ValidationError {
    pub fn message(self) -> String {
        match self {
            ValidationError::Object(err) => err.message(),
            ValidationError::Field(err) => err.message(),
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
}
