use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum CustomFieldType {
    Object,
    Enum,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
    String,
    Int64,
    Int32,
    Float64,
    Boolean,
    DateTime,
    Uuid,
    Custom(String, CustomFieldType),
    FutureType,
}
impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            FieldType::String => "string".to_string(),
            FieldType::Int64 => "int64".to_string(),
            FieldType::Int32 => "int32".to_string(),
            FieldType::Float64 => "float64".to_string(),
            FieldType::Boolean => "boolean".to_string(),
            FieldType::DateTime => "datetime".to_string(),
            FieldType::Uuid => "uuid".to_string(),
            FieldType::Custom(s, _) => s.clone(),
            FieldType::FutureType => "FUTURE TYPE".to_string(),
        };
        write!(f, "{}", res)
    }
}
impl FieldType {
    pub fn from_string(s: &str) -> Option<FieldType> {
        Some(match s {
            "string" => FieldType::String,
            "int64" => FieldType::Int64,
            "int32" => FieldType::Int32,
            "float64" => FieldType::Float64,
            "boolean" => FieldType::Boolean,
            "datetime" => FieldType::DateTime,
            "uuid" => FieldType::Uuid,
            "___" => FieldType::FutureType,
            _ => return None,
        })
    }
}
