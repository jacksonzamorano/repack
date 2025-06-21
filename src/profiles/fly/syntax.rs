use crate::profiles::TemplateToken;

#[derive(Debug, Clone)]
pub struct FlyToken {
    pub value: String,
    pub token_type: FlyTokenType,
}

impl FlyToken {
    pub fn from_string(val: String, context: &FlyContext) -> FlyToken {
        if val.starts_with('#') {
            if let Some(command) = TemplateToken::from_language(&val[1..val.len()]) {
                return FlyToken {
                    token_type: FlyTokenType::Command(command),
                    value: val,
                };
            }
        } else if val.starts_with('$') {
            if let Some(var) = FlyContextualizedVariable::from_string(&val[1..val.len()], context) {
                return FlyToken {
                    token_type: FlyTokenType::Variable(var),
                    value: val,
                };
            }
        }
        FlyToken {
            token_type: FlyTokenType::LiteralRun,
            value: val,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FlyTokenType {
    LiteralRun,
    Command(TemplateToken),
    Variable(FlyContextualizedVariable),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FlyContext {
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
pub enum FlyContextualizedVariable {
    Global(FlyGlobalVariable),
    Array(FlyArrayVariable),
    Optional(FlyOptionalVariable),
    Record(FlyRecordVariable),
    Struct(FlyStructVariable),
    Enum(FlyEnumVariable),
    Field(FlyFieldVariable),
    Case(FlyCaseVariable),
}

impl FlyContextualizedVariable {
    fn from_string(val: &str, context: &FlyContext) -> Option<FlyContextualizedVariable> {
        match context {
            FlyContext::Global => FlyGlobalVariable::from_string(val).map(|x| Self::Global(x)),
            FlyContext::Array => FlyArrayVariable::from_string(val).map(|x| Self::Array(x)),
            FlyContext::Optional => {
                FlyOptionalVariable::from_string(val).map(|x| Self::Optional(x))
            }
            FlyContext::Record => FlyRecordVariable::from_string(val).map(|x| Self::Record(x)),
            FlyContext::Struct => FlyStructVariable::from_string(val).map(|x| Self::Struct(x)),
            FlyContext::Enum => FlyEnumVariable::from_string(val).map(|x| Self::Enum(x)),
            FlyContext::Field => FlyFieldVariable::from_string(val).map(|x| Self::Field(x)),
            FlyContext::Case => FlyCaseVariable::from_string(val).map(|x| Self::Case(x)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FlyGlobalVariable {
    Version,
}
impl FlyGlobalVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "version" => Self::Version,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum FlyRecordVariable {
    Name,
    Table,
    Fields,
}
impl FlyRecordVariable {
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
pub enum FlyOptionalVariable {
    Type,
}
impl FlyOptionalVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "type" => Self::Type,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum FlyArrayVariable {
    Type,
}
impl FlyArrayVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "type" => Self::Type,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum FlyStructVariable {
    Name,
    Fields,
}
impl FlyStructVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "fields" => Self::Fields,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum FlyEnumVariable {
    Name,
    Cases,
}
impl FlyEnumVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "cases" => Self::Cases,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum FlyCaseVariable {
    Name,
    Value,
}
impl FlyCaseVariable {
    fn from_string(val: &str) -> Option<Self> {
        Some(match val {
            "name" => Self::Name,
            "value" => Self::Value,
            _ => return None,
        })
    }
}
#[derive(Debug, Clone)]
pub enum FlyFieldVariable {
    Name,
    Type,
}
impl FlyFieldVariable {
    fn from_string(val: &str) -> Option<Self> {
        dbg!(&val);
        Some(match val {
            "name" => Self::Name,
            "type" => Self::Type,
            _ => return None,
        })
    }
}
