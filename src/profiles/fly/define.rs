use crate::profiles::{FlyContext, FlyContextualizedVariable};

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

    pub fn context(&self) -> &'static FlyContext {
        match self {
            Self::RecordBase => &FlyContext::Record,
            Self::StructBase => &FlyContext::Struct,
            Self::EnumBase => &FlyContext::Enum,
            Self::RecordField | Self::StructField => &FlyContext::Field,
            Self::EnumCase => &FlyContext::Case,
            Self::RecordFieldSplitter | Self::StructFieldSplitter | Self::EnumCaseSplitter => {
                &FlyContext::Global
            }
        }
    }
}
