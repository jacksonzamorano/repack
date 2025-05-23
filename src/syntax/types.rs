use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum FieldType {
    String,
    Int64,
    Int32,
    Float64,
    Boolean,
    DateTime,
    Ref(String, String),
    Custom(String)
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
            FieldType::Ref(object_name, field_name) => format!("{}:{}", object_name, field_name),
            FieldType::Custom(s) => s.clone()
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
            _ => FieldType::Custom(s.to_string())
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FieldCommand {
    Default,
    Many,
    PrimaryKey,
    Increment,
    Cascade,
}
impl  FieldCommand {
    pub fn string(&self) -> String {
        match self {
            FieldCommand::Default => "Default Value".to_string(),
            FieldCommand::Many => "Many".to_string(),
            FieldCommand::PrimaryKey => "Primary Key".to_string(),
            FieldCommand::Increment => "Auto Increment".to_string(),
            FieldCommand::Cascade => "Cascade".to_string(),
        }
    }
}
impl FieldCommand {
    pub fn from_string(s: &str) -> Option<FieldCommand> {
        match s {
            "default" => Some(FieldCommand::Default),
            "many" => Some(FieldCommand::Many),
            "pk" => Some(FieldCommand::PrimaryKey),
            "increment" => Some(FieldCommand::Increment),
            "cascade" => Some(FieldCommand::Cascade),
            _ => None
        }
    }
}