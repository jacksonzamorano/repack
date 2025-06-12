pub fn type_to_rust(field_type: &crate::syntax::FieldType) -> Option<String> {
    match field_type {
        crate::syntax::FieldType::Boolean => Some("bool".to_string()),
        crate::syntax::FieldType::Int32 => Some("i32".to_string()),
        crate::syntax::FieldType::Int64 => Some("i64".to_string()),
        crate::syntax::FieldType::String => Some("String".to_string()),
        crate::syntax::FieldType::Float64 => Some("f64".to_string()),
        crate::syntax::FieldType::Custom(name, _) => Some(name.to_string()),
        crate::syntax::FieldType::DateTime => Some("DateTime<Utc>".to_string()),
        crate::syntax::FieldType::Uuid => Some("Uuid".to_string()),
        _ => None,
    }
}

pub fn camel_to_upper(val: &str) -> String {
    let mut out = String::new();
    let mut last_under = true;
    for v in val.chars() {
        if v != '_' {
            if last_under {
                out.push(v.to_ascii_uppercase());
            } else {
                out.push(v);
            }
            last_under = false
        } else {
            last_under = true
        }
    }
    return out;
}
