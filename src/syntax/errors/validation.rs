use super::{FieldValidationError, LanguageValidationError, ObjectValidationError};

pub enum ValidationError {
    Object(ObjectValidationError),
    Field(FieldValidationError),
    Language(LanguageValidationError)
}
impl ValidationError {
    pub fn message(self) -> String {
        match self {
            ValidationError::Object(err) => err.message(),
            ValidationError::Field(err) => err.message(),
            ValidationError::Language(err) => err.message(),
        }
    }
}