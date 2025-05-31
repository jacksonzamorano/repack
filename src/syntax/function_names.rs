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
pub enum FunctionName {
    Default,
    Generated,
    Identity,
    PrimaryKey,
    Unique,
    Cascade,
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
            "cascade" => Self::Cascade,
            _ => Self::Custom(val.to_string())
        }
    }
}
