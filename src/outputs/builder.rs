use crate::syntax::{Field, Object};

use super::OutputDescription;

pub struct OutputBuilderFieldError {
    pub object_name: String,
    pub field_name: String,
    pub field_type: String,
}
impl OutputBuilderFieldError {
    pub fn new(object: &Object, field: &Field) -> Self {
        OutputBuilderFieldError {
            object_name: object.name.clone(),
            field_name: field.name.clone(),
            field_type: field.field_type().to_string(),
        }
    }
}

pub enum OutputBuilderError {
    CannotOpenFile,
    UnsupportedFieldType(OutputBuilderFieldError),
    ObjectNotIncluded(String),
}

impl OutputBuilderError {
    pub fn description(&self) -> String {
        match self {
            OutputBuilderError::CannotOpenFile => "Cannot open file.".to_string(),
            OutputBuilderError::UnsupportedFieldType(err) => {
                format!("{}.{} requests type {} but this output doesn't support it.", err.object_name, err.field_name, err.field_type)
            },
            OutputBuilderError::ObjectNotIncluded(obj) => {
                format!("Expected to find {} in the output but it wasn't included.", obj)
            }
        }
    }
}

pub trait OutputBuilder {
    fn build(&self, description: &mut OutputDescription) -> Result<(), OutputBuilderError>;
}
