use std::collections::HashMap;

use crate::syntax::{Field, Object, Output, ParseResult};

use super::{
    Blueprint, BlueprintError, FlyToken, SectionContent, SnippetMainTokenName, SnippetReference,
    SnippetSecondaryTokenName,
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
    fn from_field(
        field: &'a Field,
        config: &'a Blueprint,
        is_last: bool,
    ) -> Result<Self, BlueprintError> {
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
        flags.insert("sep", !is_last);

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

    fn render_snippet<'b>(
        &self,
        content: &'b [FlyToken],
        context: BlueprintExecutionContext<'b>,
    ) -> String {
        let mut value = String::new();
        let mut index = 0;
        while index < content.len() {
            let c = &content[index];
            match c {
                FlyToken::Literal(lit_val) => value.push_str(&lit_val),
                FlyToken::Snippet(snip) => {
                    let mut starting_at = index;
                    let mut end_index = starting_at + 1;
                    let mut embed_count = 0;
                    if !snip.is_ended {
                        while end_index < content.len() {
                            let in_block = &content[end_index];
                            match &in_block {
                                FlyToken::SnippetEnd(end_name) if *end_name == snip.main_token => {
                                    embed_count -= 1;
                                    if embed_count == 0 {
                                        break;
                                    }
                                    end_index += 1;
                                }
                                FlyToken::Snippet(embedded)
                                    if embedded.main_token == snip.main_token =>
                                {
                                    embed_count += 1;
                                    end_index += 1;
                                }
                                _ => {
                                    end_index += 1;
                                }
                            }
                        }
                    }
                }
                _ => {}
            };
            index += 1;
        }
        return value;
    }

    fn render_section<'b>(
        &self,
        content: SnippetReference<'b>,
        context: BlueprintExecutionContext<'b>,
    ) -> String {
        let mut value = String::new();
        match content.main_token() {
            SnippetMainTokenName::Each => {
                match content.secondary_token() {
                    SnippetSecondaryTokenName::Object => {
                        for obj in &self.parse_result.objects {
                            value.push_str(&self.render_snippet(content.contents, BlueprintExecutionContext::from_object(&obj)));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        };

        return String::new();
    }

    pub fn build(self) -> Result<(), BlueprintError> {
        let mut output = String::new();
        for section in &self.blueprint.sections {
            match section.0.0 {
                SnippetMainTokenName::Each | SnippetMainTokenName::If => {
                    output.push_str(&self.render_section(
                        SnippetReference::from_content(section.1),
                        BlueprintExecutionContext::new(),
                    ));
                }
                _ => {}
            }
        }

        return Ok(());
    }
}
