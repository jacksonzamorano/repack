use std::{collections::HashMap, hash::Hash};

use crate::syntax::{Enum, Field, FieldType, Object, Output, ParseResult};

use super::{
    Blueprint, BlueprintError, FlyToken, SnippetMainTokenName, SnippetReference,
    SnippetSecondaryTokenName,
};

#[derive(Debug)]
struct BlueprintExecutionContext<'a> {
    variables: HashMap<&'a str, Option<&'a String>>,
    flags: HashMap<&'a str, bool>,
    object: Option<&'a Object>,
    enm: Option<&'a Enum>,
}
impl<'a> BlueprintExecutionContext<'a> {
    fn new() -> BlueprintExecutionContext<'a> {
        BlueprintExecutionContext {
            variables: HashMap::new(),
            flags: HashMap::new(),
            object: None,
            enm: None,
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
            enm: None,
        }
    }
    fn from_field(
        field: &'a Field,
        config: &'a Blueprint,
        is_last: bool,
    ) -> Result<Self, BlueprintError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        let resolved_type = match field.field_type() {
            FieldType::Core(typ) => {
                &config
                    .sections
                    .get(&(
                        SnippetMainTokenName::TypeDef,
                        SnippetSecondaryTokenName::from_type(typ),
                    ))
                    .ok_or_else(|| {
                        BlueprintError::TypeNotSupported(field.field_type().to_string())
                    })?
                    .literal_string_value
            }
            FieldType::Custom(typ, _) => typ,
        };

        variables.insert("name", Some(&field.name));
        variables.insert("type", Some(resolved_type));
        flags.insert("optional", field.optional);
        flags.insert("sep", !is_last);

        Ok(Self {
            variables,
            flags,
            object: None,
            enm: None,
        })
    }
    fn from_enum(enm: &'a Enum) -> Result<Self, BlueprintError> {
        let mut variables = HashMap::new();
        variables.insert("name", Some(&enm.name));
        Ok(Self {
            variables,
            flags: HashMap::new(),
            object: None,
            enm: Some(enm),
        })
    }
    fn from_enum_case(val: &'a String, is_last: bool) -> Result<Self, BlueprintError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        variables.insert("name", Some(val));
        variables.insert("value", Some(val));
        flags.insert("sep", !is_last);

        Ok(Self {
            variables,
            flags,
            object: None,
            enm: None,
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
        BlueprintRenderer {
            parse_result,
            blueprint,
            config,
        }
    }

    fn render_tokens<'b>(
        &self,
        content: &'b [FlyToken],
        context: &'b BlueprintExecutionContext<'b>,
    ) -> Result<String, BlueprintError> {
        let mut value = String::new();
        let mut index = 0;
        while index < content.len() {
            let c = &content[index];
            match c {
                FlyToken::Literal(lit_val) => {
                    value.push_str(lit_val);
                    index += 1;
                }
                FlyToken::Snippet(snip) => {
                    index += 1;
                    let starting_at = index;
                    let mut embed_count = 1;
                    if !snip.is_ended {
                        index += 1;
                        while index < content.len() {
                            let in_block = &content[index];
                            match &in_block {
                                FlyToken::SnippetEnd(end_name) if *end_name == snip.main_token => {
                                    embed_count -= 1;
                                    if embed_count == 0 {
                                        break;
                                    }
                                    index += 1;
                                }
                                FlyToken::Snippet(embedded)
                                    if embedded.main_token == snip.main_token =>
                                {
                                    embed_count += 1;
                                    index += 1;
                                }
                                _ => {
                                    index += 1;
                                }
                            }
                        }
                    }
                    value.push_str(&self.render_snippet(
                        SnippetReference {
                            details: snip,
                            contents: &content[starting_at..index],
                        },
                        context,
                    )?);
                }
                _ => {
                    index += 1;
                }
            };
        }
        Ok(value)
    }

    fn render_snippet<'b>(
        &self,
        content: SnippetReference<'b>,
        context: &'b BlueprintExecutionContext<'b>,
    ) -> Result<String, BlueprintError> {
        let mut value = String::new();
        match content.main_token() {
            SnippetMainTokenName::Each => {
                let mut splitter = content.details.contents.to_string();
                if splitter == "\\n" {
                    splitter = "\n".to_string()
                }
                match content.secondary_token() {
                    SnippetSecondaryTokenName::Object => {
                        for obj in &self.parse_result.objects {
                            value.push_str(&self.render_tokens(
                                content.contents,
                                &BlueprintExecutionContext::from_object(obj),
                            )?);
                            value.push_str(&splitter);
                        }
                    }
                    SnippetSecondaryTokenName::Field => {
                        let Some(obj) = context.object else {
                            return Err(BlueprintError::CouldNotCreateContext(
                                "tried to create a field in a non-object context",
                            ));
                        };
                        for (idx, field) in obj.fields.iter().enumerate() {
                            value.push_str(&self.render_tokens(
                                content.contents,
                                &BlueprintExecutionContext::from_field(
                                    field,
                                    self.blueprint,
                                    idx + 1 == obj.fields.len(),
                                )?,
                            )?);
                            value.push_str(&splitter);
                        }
                    }
                    SnippetSecondaryTokenName::Enum => {
                        for enm in &self.parse_result.enums {
                            value.push_str(&self.render_tokens(
                                content.contents,
                                &BlueprintExecutionContext::from_enum(enm)?,
                            )?);
                            value.push_str(&splitter);
                        }
                    }
                    SnippetSecondaryTokenName::Case => {
                        let Some(enm) = context.enm else {
                            return Err(BlueprintError::CouldNotCreateContext(
                                "tried to create a case in a non-enum context",
                            ));
                        };
                        for (idx, case) in enm.options.iter().enumerate() {
                            value.push_str(&self.render_tokens(
                                content.contents,
                                &BlueprintExecutionContext::from_enum_case(
                                    case,
                                    idx + 1 == enm.options.len(),
                                )?,
                            )?);
                            value.push_str(&splitter);
                        }
                    }
                    _ => {}
                }
            }
            SnippetMainTokenName::If => {
                let token = &content.details.secondary_token;

                if context.flags.get(token.as_str()).copied().unwrap_or(false) {
                    value.push_str(&content.details.contents);
                    value.push_str(&self.render_tokens(content.contents, context)?);
                }
            }
            SnippetMainTokenName::Variable(var) => {
                if let Some(Some(var_val)) = context.variables.get(var.as_str()) {
                    value.push_str(var_val);
                }
            }
            _ => {}
        };

        Ok(value)
    }

    pub fn build(self) -> Result<(), BlueprintError> {
        let mut output = String::new();
        for section in &self.blueprint.sections {
            match section.0.0 {
                SnippetMainTokenName::Each | SnippetMainTokenName::If => {
                    output.push_str(&self.render_snippet(
                        SnippetReference::from_content(section.1),
                        &BlueprintExecutionContext::new(),
                    )?);
                }
                _ => {}
            }
        }

        println!("{}", output);

        Ok(())
    }
}
