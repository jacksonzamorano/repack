#[derive(Debug, Clone, PartialEq)]
pub enum FunctionNamespace {
    Database,
    Custom(String),
}
impl FunctionNamespace {
    pub fn from_string(val: &str) -> FunctionNamespace {
        match val {
            "db" => Self::Database,
            _ => Self::Custom(val.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFunctionName {
    Index,
    Custom(String),
}

impl ObjectFunctionName {
    pub fn from_string(val: &str) -> ObjectFunctionName {
        match val {
            "index" => Self::Index,
            _ => Self::Custom(val.to_string()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FieldFunctionName {
    Default,
    Generated,
    Identity,
    PrimaryKey,
    Unique,
    Cascade,
    Custom(String),
}

impl FieldFunctionName {
    pub fn from_string(val: &str) -> FieldFunctionName {
        match val {
            "default" => Self::Default,
            "generated" => Self::Generated,
            "identity" => Self::Identity,
            "primary_key" => Self::PrimaryKey,
            "unique" => Self::Unique,
            "cascade" => Self::Cascade,
            _ => Self::Custom(val.to_string()),
        }
    }
}
