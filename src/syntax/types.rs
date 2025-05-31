use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
    String,
    Int64,
    Int32,
    Float64,
    Boolean,
    DateTime,
    Custom(String),
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
            FieldType::Custom(s) => s.clone(),
            FieldType::FutureType => "FUTURE TYPE".to_string(),
        };
        write!(f, "{}", res)
    }
}
impl FieldType {
    pub fn from_string(s: &str) -> FieldType {
        match s {
            "string" => FieldType::String,
            "int64" => FieldType::Int64,
            "int32" => FieldType::Int32,
            "float64" => FieldType::Float64,
            "boolean" => FieldType::Boolean,
            "datetime" => FieldType::DateTime,
            "___" => FieldType::FutureType,
            _ => FieldType::Custom(s.to_string()),
        }
    }
}
