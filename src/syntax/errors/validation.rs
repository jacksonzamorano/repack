use super::{FieldValidationError, LanguageValidationError, ObjectValidationError};

#[derive(Debug)]
pub enum ValidationError {
    Object(ObjectValidationError),
    Field(FieldValidationError),
    Language(LanguageValidationError),
    CircularDependancy(String, String),
    MissingParent(String, String),
    JoinFieldUnresolvable(String, String),
    RefFieldUnresolvable(String, String),
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
            ValidationError::MissingParent(child, parent) => {
                format!("Expected to find parent named {} on {}, but it wasn't found.", parent, child)
            }
            ValidationError::JoinFieldUnresolvable(obj, field) => {
                format!("Could not resolve from field {} on object {}.", obj, field)
            }
            ValidationError::RefFieldUnresolvable(obj, field) => {
                format!("Could not resolve ref field {} on object {}.", obj, field)
            }
        }
    }
}
