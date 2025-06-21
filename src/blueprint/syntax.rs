use crate::blueprint::BlueprintCommand;

#[derive(Debug, Clone)]
pub enum BlueprintToken {
    LiteralRun(String),
    Command(BlueprintCommand),
    Variable(BlueprintContextualizedVariable),
    Colon,
    Period,
    Space,
    NewLine,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenAngle,
    CloseAngle,
}

impl BlueprintToken {
    pub fn from_char(val: char) -> Option<BlueprintToken> {
        Some(match val {
            ':' => BlueprintToken::Colon,
            '.' => BlueprintToken::Period,
            '\n' | '\r' => BlueprintToken::NewLine,
            ' ' => BlueprintToken::Space,
            '<' => Self::OpenAngle,
            '>' => Self::CloseAngle,
            '[' => Self::OpenBracket,
            ']' => Self::CloseBracket,
            '{' => Self::OpenBrace,
            '}' => Self::CloseBrace,
            _ => return None,
        })
    }
    pub fn from_string(val: String, context: &BlueprintContext) -> BlueprintToken {
        if val.starts_with('#') {
            if let Some(command) = BlueprintCommand::from_language(&val[1..val.len()]) {
                return BlueprintToken::Command(command);
            }
        } else if val.starts_with('$') {
            if let Some(var) = BlueprintContextualizedVariable::from_string(&val[1..val.len()], context) {
                return BlueprintToken::Variable(var);
            }
        }
        return BlueprintToken::LiteralRun(val);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BlueprintContext {
    Global,
    Array,
    Optional,
    Record,
    Struct,
    Enum,
    Field,
    Case,
}

#[derive(Debug, Clone)]
pub enum BlueprintContextualizedVariable {
    Global(BlueprintGlobalVariable),
    Array(BlueprintArrayVariable),
    Optional(BlueprintOptionalVariable),
    Record(BlueprintRecordVariable),
    Struct(BlueprintStructVariable),
    Enum(BlueprintEnumVariable),
    Field(BlueprintFieldVariable),
    Case(BlueprintCaseVariable),
}

impl BlueprintContextualizedVariable {
    fn from_string(val: &str, context: &BlueprintContext) -> Option<BlueprintContextualizedVariable> {
        match context {
            BlueprintContext::Global => BlueprintGlobalVariable::from_string(val).map(|x| Self::Global(x)),
            BlueprintContext::Array => BlueprintArrayVariable::from_string(val).map(|x| Self::Array(x)),
            BlueprintContext::Optional => {
                BlueprintOptionalVariable::from_string(val).map(|x| Self::Optional(x))
            }
            BlueprintContext::Record => BlueprintRecordVariable::from_string(val).map(|x| Self::Record(x)),
            BlueprintContext::Struct => BlueprintStructVariable::from_string(val).map(|x| Self::Struct(x)),
            BlueprintContext::Enum => BlueprintEnumVariable::from_string(val).map(|x| Self::Enum(x)),
            BlueprintContext::Field => BlueprintFieldVariable::from_string(val).map(|x| Self::Field(x)),
            BlueprintContext::Case => BlueprintCaseVariable::from_string(val).map(|x| Self::Case(x)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlueprintGlobalVariable {
    Version,
}
impl BlueprintGlobalVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "version" => Self::Version,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum BlueprintRecordVariable {
    Name,
    Table,
    Fields,
}
impl BlueprintRecordVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "table" => Self::Table,
            "fields" => Self::Fields,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum BlueprintOptionalVariable {
    Type,
}
impl BlueprintOptionalVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "type" => Self::Type,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum BlueprintArrayVariable {
    Type,
}
impl BlueprintArrayVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "type" => Self::Type,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum BlueprintStructVariable {
    Name,
    Fields,
}
impl BlueprintStructVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "fields" => Self::Fields,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum BlueprintEnumVariable {
    Name,
    Cases,
}
impl BlueprintEnumVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "cases" => Self::Cases,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum BlueprintCaseVariable {
    Name,
    Value,
}
impl BlueprintCaseVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "value" => Self::Value,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum BlueprintFieldVariable {
    Name,
    Type,
}
impl BlueprintFieldVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "type" => Self::Type,
            _ => return None,
        })
    }
}
