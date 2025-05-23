use crate::syntax::Object;


#[derive(Debug)]
pub enum ObjectValidationErrorType {
    CannotInherit = 1,
    CannotReuse,
    TableNameRequired,
    TableNameNotAllowed,
    NoFields,
}

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


#[derive(Debug)]
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