use crate::{outputs::OutputDescription, syntax::FieldType};

pub fn make_index(desc: &OutputDescription) -> bool {
    desc.bool("make_index", false)
}

pub fn type_to_ts(field_type: &crate::syntax::FieldType) -> Option<String> {
    match field_type {
        FieldType::Boolean => Some("boolean".to_string()),
        FieldType::Int32 => Some("number".to_string()),
        FieldType::Int64 => Some("number".to_string()),
        FieldType::String => Some("string".to_string()),
        FieldType::Float64 => Some("number".to_string()),
        FieldType::DateTime => Some("Date".to_string()),
        FieldType::Custom(name) => Some(name.to_string()),
        _ => None,
    }
}
