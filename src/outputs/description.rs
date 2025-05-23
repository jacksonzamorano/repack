use crate::syntax::{Object, Output, ParseResult};

pub struct OutputDescription<'a> {
    pub objects: &'a Vec<Object>,
    pub output: &'a Output,
}

impl<'a> OutputDescription<'a> {
    pub fn new(result: &'a ParseResult, output: &'a Output) -> Self {
        Self {
            objects: &result.objects,
            output: output,
        }
    }
}