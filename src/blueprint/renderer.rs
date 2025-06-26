use std::collections::HashMap;

use crate::syntax::{Field, Object, Output, ParseResult};

use super::{
    Blueprint, BlueprintError, SectionContent, SnippetMainTokenName, SnippetSecondaryTokenName,
};

struct BlueprintExecutionContext<'a> {
    variables: HashMap<&'a str, Option<&'a String>>,
    flags: HashMap<&'a str, bool>,
    object: Option<&'a Object>,
    field: Option<&'a Field>,
}
impl<'a> BlueprintExecutionContext<'a> {
    fn new() -> BlueprintExecutionContext<'a> {
        BlueprintExecutionContext {
            variables: HashMap::new(),
            flags: HashMap::new(),
            object: None,
            field: None,
        }
    }
    fn from_object(obj: &'a Object) -> Self {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();
        variables.insert("name", Some(&obj.name));
        variables.insert("table_name", obj.table_name.as_ref());
        flags.insert(
            "record",
            matches!(obj.object_type, crate::syntax::ObjectType::Record),
        );

        Self {
            variables,
            flags,
            object: Some(obj),
            field: None,
        }
    }
    fn from_field(field: &'a Field, config: &'a Blueprint) -> Result<Self, BlueprintError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        let resolved_type = &config
            .sections
            .get(&(
                SnippetMainTokenName::TypeDef,
                SnippetSecondaryTokenName::from_type(field.field_type()),
            ))
            .ok_or_else(|| BlueprintError::CouldNotCreateContext("type not available"))?
            .literal_string_value;

        variables.insert("name", Some(&field.name));
        variables.insert("type", Some(&resolved_type));
        flags.insert("optional", field.optional);

        Ok(Self {
            variables,
            flags,
            field: Some(field),
            object: None,
        })
    }
}

pub struct BlueprintRenderer<'a> {
    pub blueprint: &'a Blueprint,
    pub parse_result: &'a ParseResult,
    pub config: &'a Output,
}
impl<'a> BlueprintRenderer<'a> {
    pub fn new(
        parse_result: &'a ParseResult,
        blueprint: &'a Blueprint,
        config: &'a Output,
    ) -> BlueprintRenderer<'a> {
        return BlueprintRenderer {
            parse_result,
            blueprint,
            config,
        };
    }

    fn render_section<'b>(
        content: SectionContent,
        context: BlueprintExecutionContext<'b>,
    ) -> String {
        return String::new();
    }

    pub fn build(self) -> Result<(), BlueprintError> {
        for section in &self.blueprint.sections {
            match section.0.0 {
                SnippetMainTokenName::Each => {}
                _ => {}
            }
        }

        return Ok(());
    }
}
