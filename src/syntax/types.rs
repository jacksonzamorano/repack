use std::fmt::{Debug, Display};

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
    Uuid,
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
impl Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            Self::String => "string".to_string(),
            Self::Int64 => "int64".to_string(),
            Self::Int32 => "int32".to_string(),
            Self::Float64 => "float64".to_string(),
            Self::Boolean => "boolean".to_string(),
            Self::DateTime => "datetime".to_string(),
            Self::Uuid => "uuid".to_string(),
        };
        write!(f, "{}", res)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldType {
    Core(CoreType),
    Custom(String, CustomFieldType),
}
impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Core(s) => {
                write!(f, "{}", s)
            }
            FieldType::Custom(s, _) => {
                write!(f, "{}", s)
            }
        }
    }
}
impl FieldType {
    pub fn from_string(s: &str) -> Option<FieldType> {
        CoreType::from_string(s).map(|x| FieldType::Core(x))
    }
}
