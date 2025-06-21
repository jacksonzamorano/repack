use crate::syntax::CoreType;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum TemplatedLanguageType {
    Type(CoreType),
    Array,
    Optional,
}
