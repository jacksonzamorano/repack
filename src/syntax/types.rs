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
}
impl  FieldCommand {
    pub fn string(&self) -> String {
        match self {
            FieldCommand::Default => "Default Value".to_string(),
            FieldCommand::Many => "Many".to_string(),
            FieldCommand::PrimaryKey => "Primary Key".to_string(),
            FieldCommand::Increment => "Auto Increment".to_string(),
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
            _ => None
        }
    }
}