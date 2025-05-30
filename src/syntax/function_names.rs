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

pub const FUNCTION_DEFAULT_VALUE: &str = "default";
