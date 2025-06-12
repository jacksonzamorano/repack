#[derive(Debug, Clone, PartialEq)]
pub enum FunctionNamespace {
    Database,
    Usage,
    Custom(String),
}
impl FunctionNamespace {
    pub fn from_string(val: &str) -> FunctionNamespace {
        match val {
            "db" => Self::Database,
            "usage" => Self::Usage,
            _ => Self::Custom(val.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFunctionName {
    Index,
    Temporary,
    Check,
    Custom(String),
}

impl ObjectFunctionName {
    pub fn from_string(val: &str) -> ObjectFunctionName {
        match val {
            "index" => Self::Index,
            "temporary" => Self::Temporary,
            "check" => Self::Check,
            _ => Self::Custom(val.to_string()),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum FieldFunctionName {
    Default,
    Generated,
    GeneratedStored,
    Identity,
    PrimaryKey,
    Unique,
    Cascade,
    Transient,
    Custom(String),
}

impl FieldFunctionName {
    pub fn from_string(val: &str) -> FieldFunctionName {
        match val {
            "default" => Self::Default,
            "generated" => Self::Generated,
            "generated_stored" => Self::GeneratedStored,
            "identity" => Self::Identity,
            "primary_key" => Self::PrimaryKey,
            "unique" => Self::Unique,
            "cascade" => Self::Cascade,
            "transient" => Self::Transient,
            _ => Self::Custom(val.to_string()),
        }
    }
}
