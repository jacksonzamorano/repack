use crate::blueprint::{BlueprintContext};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TemplateDefineSection {
    RecordBase,
    StructBase,
    EnumBase,
    Field,
    FieldOptional,
    Case,
    FieldSplitter,
    CaseSplitter,
}
impl TemplateDefineSection {
    pub fn from_language(val: &str) -> Option<Self> {
        Some(match val {
            "record.base" => Self::RecordBase,
            "struct.base" => Self::StructBase,
            "enum.base" => Self::EnumBase,
            "case" => Self::Case,
            "case.splitter" => Self::CaseSplitter,
            "field" => Self::Field,
            "field.optional" => Self::FieldOptional,
            "field.splitter" => Self::FieldSplitter,
            _ => return None,
        })
    }

    pub fn context(&self) -> &'static BlueprintContext {
        match self {
            Self::RecordBase => &BlueprintContext::Record,
            Self::StructBase => &BlueprintContext::Struct,
            Self::EnumBase => &BlueprintContext::Enum,
            Self::Field | Self::FieldOptional => &BlueprintContext::Field,
            Self::Case => &BlueprintContext::Case,
            Self::FieldSplitter | Self::CaseSplitter => {
                &BlueprintContext::Global
            }
        }
    }
}
