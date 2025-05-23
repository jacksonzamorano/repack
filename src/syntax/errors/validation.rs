use super::{FieldValidationError, LanguageValidationError, ObjectValidationError};

pub enum ValidationError {
    Object(ObjectValidationError),
    Field(FieldValidationError),
    Language(LanguageValidationError),
    CircularDependancy(String, String),
}
impl ValidationError {
    pub fn message(self) -> String {
        match self {
            ValidationError::Object(err) => err.message(),
            ValidationError::Field(err) => err.message(),
            ValidationError::Language(err) => err.message(),
            ValidationError::CircularDependancy(c1, c2) => {
                format!("Circular dependancy between {} and {}", c1, c2)
            }
        }
    }
}