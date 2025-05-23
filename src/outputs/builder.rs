use crate::syntax::Output;

use super::OutputDescription;

pub enum OutputBuilderError {
}

pub trait OutputBuilder {
    fn build(output: &Output, description: &mut OutputDescription) -> Result<(), OutputBuilderError>;
}