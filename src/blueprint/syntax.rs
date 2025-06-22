use crate::{blueprint::BlueprintCommand, syntax::Enum};

#[derive(Debug, Clone)]
pub enum BlueprintToken {
    LiteralRun(String),
    Command(BlueprintCommand),
    Variable(BlueprintContextualizedVariable),
    Colon,
    Indent,
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
            '\t' => BlueprintToken::Indent,
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
            if let Some(var) =
                BlueprintContextualizedVariable::from_string(&val[1..val.len()], context)
            {
                return BlueprintToken::Variable(var);
            }
        }
        return BlueprintToken::LiteralRun(val);
    }
    pub fn render<'a, F>(&'a self, handle: F) -> &'a str
    where
        F: Fn(&BlueprintContextualizedVariable) -> &'a str,
    {
        match self {
            Self::LiteralRun(lit) => lit,
            Self::Colon => ":",
            Self::Period => ".",
            Self::Space => " ",
            Self::NewLine => "\n",
            Self::OpenBrace => "{",
            Self::CloseBrace => "}",
            Self::OpenBracket => "[",
            Self::CloseBracket => "]",
            Self::OpenAngle => "<",
            Self::CloseAngle => ">",
            Self::Indent => "\t",
            Self::Variable(var) => {
                handle(var)
            },
            Self::Command(_) => {
                panic!("Commands should never be rendered. This is a compiler error on our end.")
            }
        }
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
    pub fn as_array(&self) -> &BlueprintArrayVariable {
        match self {
            Self::Array(val) => val,
            _ => panic!("Cannot render this variable type in 'array' context!")
        }
    }
    pub fn as_global(&self) -> &BlueprintGlobalVariable {
        match self {
            Self::Global(val) => val,
            _ => panic!("Cannot render this variable type in 'global' context!")
        }
    }
    pub fn as_enum(&self) -> &BlueprintEnumVariable {
        match self {
            Self::Enum(val) => val,
            _ => panic!("Cannot render this variable type in 'enum' context!")
        }
    }
    pub fn as_case(&self) -> &BlueprintCaseVariable {
        match self {
            Self::Case(val) => val,
            _ => panic!("Cannot render this variable type in 'case' context!")
        }
    }
    pub fn as_field(&self) -> &BlueprintFieldVariable {
        match self {
            Self::Field(val) => val,
            _ => panic!("Cannot render this variable type in 'field' context!")
        }
    }
    pub fn as_record(&self) -> &BlueprintRecordVariable {
        match self {
            Self::Record(val) => val,
            _ => panic!("Cannot render this variable type in 'record' context!")
        }
    }
    fn from_string(
        val: &str,
        context: &BlueprintContext,
    ) -> Option<BlueprintContextualizedVariable> {
        match context {
            BlueprintContext::Global => {
                BlueprintGlobalVariable::from_string(val).map(|x| Self::Global(x))
            }
            BlueprintContext::Array => {
                BlueprintArrayVariable::from_string(val).map(|x| Self::Array(x))
            }
            BlueprintContext::Optional => {
                BlueprintOptionalVariable::from_string(val).map(|x| Self::Optional(x))
            }
            BlueprintContext::Record => {
                BlueprintRecordVariable::from_string(val).map(|x| Self::Record(x))
            }
            BlueprintContext::Struct => {
                BlueprintStructVariable::from_string(val).map(|x| Self::Struct(x))
            }
            BlueprintContext::Enum => {
                BlueprintEnumVariable::from_string(val).map(|x| Self::Enum(x))
            }
            BlueprintContext::Field => {
                BlueprintFieldVariable::from_string(val).map(|x| Self::Field(x))
            }
            BlueprintContext::Case => {
                BlueprintCaseVariable::from_string(val).map(|x| Self::Case(x))
            }
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
