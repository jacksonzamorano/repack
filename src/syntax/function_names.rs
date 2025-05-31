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

#[derive(Debug, Clone)]
pub enum FunctionName {
    Default,
    Generated,
    Identity,
    PrimaryKey,
    Unique,
    Custom(String)
}

impl FunctionName {
    pub fn from_string(val: &str) -> FunctionName {
        match val {
            "default" => Self::Default,
            "generated" => Self::Generated,
            "identity" => Self::Identity,
            "primary_key" => Self::PrimaryKey,
            "unique" => Self::Unique,
            _ => Self::Custom(val.to_string())
        }
    }
}
