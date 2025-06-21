use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum CustomFieldType {
    Object,
    Enum,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum CoreType {
    String,
    Int64,
    Int32,
    Float64,
    Boolean,
    DateTime,
    Uuid
}
impl CoreType {
    pub fn from_string(s: &str) -> Option<CoreType> {
        Some(match s {
            "string" => Self::String,
            "int64" => Self::Int64,
            "int32" => Self::Int32,
            "float64" => Self::Float64,
            "boolean" => Self::Boolean,
            "datetime" => Self::DateTime,
            "uuid" => Self::Uuid,
            _ => return None,
        })
    }
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
            _ => return None,
        })
    }
}
