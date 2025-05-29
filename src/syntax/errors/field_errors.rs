use crate::syntax::{Field, Object};

#[derive(Debug)]
pub enum FieldValidationErrorType {
    CustomNotAllowed = 1,
    ManyNotAllowed,
    PrimaryKeyOptional,
    TypeNotResolved,
}

impl FieldValidationError {
    pub fn message(self) -> String {
        let err = match self.error_type {
            FieldValidationErrorType::CustomNotAllowed => {
                "Custom types are not allowed in this context.".to_string()
            }
            FieldValidationErrorType::PrimaryKeyOptional => {
                "Primary key cannot be optional.".to_string()
            }
            FieldValidationErrorType::ManyNotAllowed => {
                "Many-to-many relationships are not allowed in this context.".to_string()
            }
            FieldValidationErrorType::TypeNotResolved => {
                "Type could not be resolved.".to_string()
            }
        };

        format!(
            "[FE{:04}] {}.{}: {}",
            self.error_type as u8, self.object_name, self.field_name, err
        )
    }
}


#[derive(Debug)]
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
