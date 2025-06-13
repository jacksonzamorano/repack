use crate::{outputs::OutputDescription, syntax::{Enum, FieldType}};

pub fn make_index(desc: &OutputDescription) -> bool {
    desc.bool("make_index", false)
}

pub fn enum_type(enm: &Enum) -> String {
    format!("export type {} = {};", enm.name, enm.options.iter().map(|x| format!("\"{}\"", x)).collect::<Vec<_>>().join(" | "))
}


pub fn type_to_ts(field_type: &crate::syntax::FieldType) -> Option<String> {
    match field_type {
        FieldType::Boolean => Some("boolean".to_string()),
        FieldType::Int32 => Some("number".to_string()),
        FieldType::Int64 => Some("number".to_string()),
        FieldType::String => Some("string".to_string()),
        FieldType::Float64 => Some("number".to_string()),
        FieldType::DateTime => Some("Date".to_string()),
        FieldType::Uuid => Some("string".to_string()),
        FieldType::Custom(name, _) => Some(name.to_string()),
        _ => None,
    }
}
