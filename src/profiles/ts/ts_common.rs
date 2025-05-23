use crate::outputs::OutputDescription;

pub fn make_index(desc: &OutputDescription) -> bool {
    desc.bool("make_index", false)
}

pub fn type_to_ts(field_type: &crate::syntax::FieldType) -> Option<String> {
    match field_type {
        crate::syntax::FieldType::Boolean => Some("boolean".to_string()),
        crate::syntax::FieldType::Int32 => Some("number".to_string()),
        crate::syntax::FieldType::Int64 => Some("number".to_string()),
        crate::syntax::FieldType::String => Some("string".to_string()),
        crate::syntax::FieldType::Float64 => Some("number".to_string()),
        crate::syntax::FieldType::DateTime => Some("Date".to_string()),
        crate::syntax::FieldType::Custom(name) => Some(name.to_string()),
        _ => None,
    }
}