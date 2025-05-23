use crate::syntax::Output;

use super::OutputDescription;

pub enum OutputBuilderError {
    CannotOpenFile,
}

impl ToString for OutputBuilderError {
    fn to_string(&self) -> String {
        match self {
            OutputBuilderError::CannotOpenFile => "Cannot open file.".to_string(),
        }
    }
}

pub trait OutputBuilder {
    fn build(&self, output: &Output, description: &mut OutputDescription) -> Result<(), OutputBuilderError>;
}