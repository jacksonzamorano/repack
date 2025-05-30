use crate::syntax::RepackError;
use super::OutputDescription;

pub trait OutputBuilder {
    fn build(&self, description: &mut OutputDescription) -> Result<(), RepackError>;
}
