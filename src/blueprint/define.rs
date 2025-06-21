use crate::blueprint::{BlueprintContext};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TemplateDefineSection {
    RecordBase,
    RecordField,
    RecordFieldSplitter,
    StructBase,
    StructField,
    StructFieldSplitter,
    EnumBase,
    EnumCase,
    EnumCaseSplitter,
}
impl TemplateDefineSection {
    pub fn from_language(val: &str) -> Option<Self> {
        Some(match val {
            "record.base" => Self::RecordBase,
            "record.field" => Self::RecordField,
            "record.field.splitter" => Self::RecordFieldSplitter,
            "struct.base" => Self::StructBase,
            "struct.field" => Self::StructField,
            "struct.field.splitter" => Self::StructFieldSplitter,
            "enum.base" => Self::EnumBase,
            "enum.case" => Self::EnumCase,
            "enum.case.splitter" => Self::EnumCaseSplitter,
            _ => return None,
        })
    }

    pub fn context(&self) -> &'static BlueprintContext {
        match self {
            Self::RecordBase => &BlueprintContext::Record,
            Self::StructBase => &BlueprintContext::Struct,
            Self::EnumBase => &BlueprintContext::Enum,
            Self::RecordField | Self::StructField => &BlueprintContext::Field,
            Self::EnumCase => &BlueprintContext::Case,
            Self::RecordFieldSplitter | Self::StructFieldSplitter | Self::EnumCaseSplitter => {
                &BlueprintContext::Global
            }
        }
    }
}
