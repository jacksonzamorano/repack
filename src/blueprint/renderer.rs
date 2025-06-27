use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fmt::Debug,
    fs::{self},
};

use crate::syntax::{
    Enum, Field, FieldReferenceKind, FieldType, Object, Output, ParseResult, RepackError,
    RepackErrorKind,
};

use super::{
    Blueprint, FlyToken, SnippetMainTokenName, SnippetReference, SnippetSecondaryTokenName,
};

#[derive(Debug, Clone)]
struct BlueprintExecutionContext<'a> {
    variables: HashMap<String, Option<&'a String>>,
    flags: HashMap<&'a str, bool>,
    object: Option<&'a Object>,
    field: Option<&'a Field>,
    enm: Option<&'a Enum>,
}
impl<'a> BlueprintExecutionContext<'a> {
    fn new() -> BlueprintExecutionContext<'a> {
        BlueprintExecutionContext {
            variables: HashMap::new(),
            flags: HashMap::new(),
            object: None,
            field: None,
            enm: None,
        }
    }
    fn with_object(&self, obj: &'a Object) -> Self {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();
        variables.insert("name".to_string(), Some(&obj.name));
        variables.insert("table_name".to_string(), obj.table_name.as_ref());
        flags.insert(
            "record",
            matches!(obj.object_type, crate::syntax::ObjectType::Record),
        );

        Self {
            variables,
            flags,
            object: Some(obj),
            field: None,
            enm: None,
        }
    }
    fn with_field(
        &self,
        obj: &'a Object,
        field: &'a Field,
        blueprint: &'a Blueprint,
        config: &Output,
        is_last: bool,
    ) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        let resolved_type = match field.field_type() {
            FieldType::Core(typ) => {
                blueprint
                    .utilities
                    .get(&(
                        SnippetMainTokenName::TypeDef,
                        SnippetSecondaryTokenName::from_type(typ),
                    ))
                    .ok_or_else(|| {
                        RepackError::from_lang_with_obj_field_msg(
                            RepackErrorKind::TypeNotSupported,
                            config,
                            obj,
                            field,
                            typ.to_string(),
                        )
                    })?
            }
            FieldType::Custom(typ, _) => typ,
        };

        variables.insert("name".to_string(), Some(&field.name));
        variables.insert("type".to_string(), Some(resolved_type));
        flags.insert("optional", field.optional);
        flags.insert("sep", !is_last);

        Ok(Self {
            variables,
            flags,
            object: Some(obj),
            field: Some(field),
            enm: None,
        })
    }
    fn with_enum(&self, enm: &'a Enum) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), Some(&enm.name));
        Ok(Self {
            variables,
            flags: HashMap::new(),
            object: None,
            field: None,
            enm: Some(enm),
        })
    }
    fn with_enum_case(&self, val: &'a String, is_last: bool) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        variables.insert("name".to_string(), Some(val));
        variables.insert("value".to_string(), Some(val));
        flags.insert("sep", !is_last);

        Ok(Self {
            variables,
            flags,
            object: None,
            field: None,
            enm: None,
        })
    }
}

trait TokenConsumer {
    fn set_file_name(&mut self, filename: &str);
    fn write(&mut self, value: &dyn AsRef<str>);
}
#[derive(Default)]
struct BlueprintBuildResult {
    contents: HashMap<String, String>,
    current_file_name: Option<String>,
}
impl TokenConsumer for BlueprintBuildResult {
    fn set_file_name(&mut self, filename: &str) {
        self.current_file_name = Some(filename.to_string());
    }
    fn write(&mut self, value: &dyn AsRef<str>) {
        if let Some(file) = &self.current_file_name {
            if let Some(current) = self.contents.get_mut(file) {
                current.push_str(value.as_ref());
            } else {
                self.contents
                    .insert(file.to_string(), value.as_ref().to_string());
            }
        }
    }
}
impl TokenConsumer for HashSet<String> {
    fn set_file_name(&mut self, filename: &str) {
        self.insert(filename.to_string());
    }
    fn write(&mut self, _value: &dyn AsRef<str>) {}
}
impl TokenConsumer for String {
    fn set_file_name(&mut self, _filename: &str) {}
    fn write(&mut self, value: &dyn AsRef<str>) {
        self.push_str(value.as_ref());
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
        &mut self,
        content: &'b [FlyToken],
        context: &'b BlueprintExecutionContext<'b>,
        writer: &'b mut dyn TokenConsumer,
    ) -> Result<(), RepackError> {
        let mut index = 0;
        while index < content.len() {
            let c = &content[index];
            match c {
                FlyToken::Literal(lit_val) => {
                    let val = lit_val.to_string();
                    // while val.starts_with('\n') {
                    //     val.remove(0);
                    // }
                    writer.write(&val);
                    index += 1;
                }
                FlyToken::Snippet(snip) => {
                    index += 1;
                    let starting_at = index;
                    let mut embed_count = 1;
                    if !snip.autoclose {
                        index += 1;
                        while index < content.len() {
                            let in_block = &content[index];
                            match &in_block {
                                FlyToken::Close(close) => {
                                    if *close == snip.main_token {
                                        embed_count -= 1;
                                        if embed_count == 0 {
                                            break;
                                        }
                                    }
                                }
                                FlyToken::Snippet(embedded)
                                    if embedded.main_token == snip.main_token =>
                                {
                                    embed_count += 1;
                                }
                                _ => {}
                            }
                            index += 1;
                        }
                    }
                    self.render_snippet(
                        SnippetReference {
                            details: snip,
                            contents: &content[starting_at..index],
                        },
                        context,
                        writer,
                    )?;
                }
                _ => {
                    index += 1;
                }
            };
        }
        Ok(())
    }

    fn render_snippet<'b>(
        &mut self,
        content: SnippetReference<'b>,
        context: &'b BlueprintExecutionContext<'b>,
        writer: &'b mut dyn TokenConsumer,
    ) -> Result<(), RepackError> {
        match content.main_token() {
            SnippetMainTokenName::File => {
                let mut file_name = content.details.contents.clone();
                if file_name.is_empty() {
                    self.render_tokens(content.contents, context, &mut file_name)?;
                }
                writer.set_file_name(&file_name);
            }
            SnippetMainTokenName::Each
            | SnippetMainTokenName::Eachr
            | SnippetMainTokenName::EachInline => {
                let rev = matches!(content.main_token(), SnippetMainTokenName::Eachr);
                let inline = matches!(content.main_token(), SnippetMainTokenName::EachInline);
                if !inline {
                    writer.write(&"\n");
                }
                match content.secondary_token() {
                    SnippetSecondaryTokenName::Object => {
                        let mut objects_included: Vec<_> = self
                            .parse_result
                            .objects
                            .iter()
                            .filter(|x| {
                                self.config
                                    .categories
                                    .iter()
                                    .any(|cat| x.categories.contains(cat))
                                    && !self.config.exclude.contains(&x.name)
                            })
                            .collect();
                        if rev {
                            objects_included.reverse();
                        }
                        for obj in objects_included {
                            self.render_tokens(
                                content.contents,
                                &context.with_object(obj),
                                writer,
                            )?;
                            if !inline {
                                writer.write(&"\n");
                            }
                        }
                    }
                    SnippetSecondaryTokenName::Field => {
                        let Some(obj) = context.object else {
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::CannotCreateContext,
                                self.config,
                                "field in non-object context.".to_string(),
                            ));
                        };
                        let mut fields = obj.fields.iter().collect::<Vec<_>>();
                        if rev {
                            fields.reverse();
                        }
                        for (idx, field) in fields.iter().enumerate() {
                            self.render_tokens(
                                content.contents,
                                &context.with_field(
                                    obj,
                                    field,
                                    self.blueprint,
                                    self.config,
                                    idx + 1 == obj.fields.len(),
                                )?,
                                writer,
                            )?;
                            if !inline {
                                writer.write(&"\n");
                            }
                        }
                    }
                    SnippetSecondaryTokenName::Enum => {
                        let mut enums_included = self
                            .parse_result
                            .enums
                            .iter()
                            .filter(|x| {
                                self.config
                                    .categories
                                    .iter()
                                    .any(|cat| x.categories.contains(cat))
                                    && !self.config.exclude.contains(&x.name)
                            })
                            .collect::<Vec<_>>();
                        if rev {
                            enums_included.reverse();
                        }
                        for enm in enums_included {
                            self.render_tokens(content.contents, &context.with_enum(enm)?, writer)?;
                            if !inline {
                                writer.write(&"\n");
                            }
                        }
                    }
                    SnippetSecondaryTokenName::Case => {
                        let Some(enm) = context.enm else {
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::CannotCreateContext,
                                self.config,
                                "case in non-enum context.".to_string(),
                            ));
                        };
                        let mut cases = enm.options.iter().collect::<Vec<_>>();
                        if rev {
                            cases.reverse();
                        }
                        for (idx, case) in cases.iter().enumerate() {
                            self.render_tokens(
                                content.contents,
                                &context.with_enum_case(case, idx + 1 == enm.options.len())?,
                                writer,
                            )?;
                            if !inline {
                                writer.write(&"\n");
                            }
                        }
                    }
                    _ => {}
                }
            }
            SnippetMainTokenName::If => {
                let token = &content.details.secondary_token;

                if context.flags.get(token.as_str()).copied().unwrap_or(false) {
                    writer.write(&content.details.contents);
                    self.render_tokens(content.contents, context, writer)?;
                }
            }
            SnippetMainTokenName::Ifn => {
                let token = &content.details.secondary_token;

                if !context.flags.get(token.as_str()).copied().unwrap_or(false) {
                    writer.write(&content.details.contents);
                    self.render_tokens(content.contents, context, writer)?;
                }
            }
            SnippetMainTokenName::Func => {
                let mut parts = content.details.secondary_token.split(".");
                let namespace = parts.next().ok_or_else(|| {
                    RepackError::from_lang_with_msg(
                        RepackErrorKind::FunctionInvalidSyntax,
                        self.config,
                        content.details.secondary_token.clone(),
                    )
                })?;
                let name = parts.next().ok_or_else(|| {
                    RepackError::from_lang_with_msg(
                        RepackErrorKind::FunctionInvalidSyntax,
                        self.config,
                        content.details.secondary_token.clone(),
                    )
                })?;
                if let Some(field) = context.field {
                    if let Some(matched_fn) = field
                        .functions_in_namespace(namespace)
                        .iter()
                        .find(|func| func.name == name)
                    {
                        let mut updated_context = context.clone();
                        for (idx, arg) in matched_fn.args.iter().enumerate() {
                            updated_context.variables.insert(idx.to_string(), Some(arg));
                        }

                        self.render_tokens(content.contents, &updated_context, writer)?;
                    }
                }
            }
            SnippetMainTokenName::Join => {
                if let Some(field) = context.field {
                    if let Some(obj) = context.object {
                        let mut u_context = context.clone();
                        u_context
                            .variables
                            .insert("local_entity".to_string(), obj.table_name.as_ref());
                        match &field.location.reference {
                            FieldReferenceKind::Local => {}
                            FieldReferenceKind::ImplicitJoin(entity_name) => {
                                u_context
                                    .variables
                                    .insert("foreign_entity".to_string(), Some(entity_name));
                                u_context.variables.insert(
                                    "foreign_field".to_string(),
                                    Some(&field.location.name),
                                );

                                self.render_tokens(content.contents, &u_context, writer)?;
                            }
                            FieldReferenceKind::FieldType(entity_name) => {
                                u_context
                                    .variables
                                    .insert("foreign_entity".to_string(), Some(entity_name));
                                u_context.variables.insert(
                                    "foreign_field".to_string(),
                                    Some(&field.location.name),
                                );
                                self.render_tokens(content.contents, &u_context, writer)?;
                            }
                            FieldReferenceKind::ExplicitJoin(join_name) => {
                                u_context
                                    .variables
                                    .insert("foreign_entity".to_string(), Some(join_name));
                                u_context.variables.insert(
                                    "foreign_field".to_string(),
                                    Some(&field.location.name),
                                );
                                self.render_tokens(content.contents, &u_context, writer)?;
                            }
                        };
                    }
                }
            }
            SnippetMainTokenName::Ref => {
                if let Some(field) = context.field {
                    if let Some(obj) = context.object {
                        let mut u_context = context.clone();
                        u_context
                            .variables
                            .insert("local_entity".to_string(), obj.table_name.as_ref());
                        if let FieldReferenceKind::FieldType(entity_name) =
                            &field.location.reference
                        {
                            u_context
                                .variables
                                .insert("foreign_entity".to_string(), Some(entity_name));
                            u_context
                                .variables
                                .insert("foreign_field".to_string(), Some(&field.location.name));
                            self.render_tokens(content.contents, &u_context, writer)?;
                        };
                    }
                }
            }
            SnippetMainTokenName::Variable(var) => {
                if let Some(Some(var_val)) = context.variables.get(var.as_str()) {
                    writer.write(&var_val);
                }
            }
            _ => {}
        };

        Ok(())
    }

    pub fn build(&mut self) -> Result<(), RepackError> {
        let mut files = BlueprintBuildResult::default();
        _ = &self.render_tokens(
            &self.blueprint.tokens,
            &BlueprintExecutionContext::new(),
            &mut files,
        )?;
        let mut path = current_dir().unwrap();
        if let Some(loc) = &self.config.location {
            path.push(loc);
        }
        _ = fs::create_dir_all(&path);
        for f in &files.contents {
            let mut file = path.clone();
            file.push(f.0);
            fs::write(file, f.1).map_err(|_| {
                RepackError::from_lang_with_msg(
                    RepackErrorKind::CannotWrite,
                    self.config,
                    f.0.to_string(),
                )
            })?;
        }
        Ok(())
    }

    pub fn clean(&mut self) -> Result<(), RepackError> {
        let mut files = HashSet::<String>::new();
        self.render_tokens(
            &self.blueprint.tokens,
            &BlueprintExecutionContext::new(),
            &mut files,
        )?;
        let mut path = current_dir().unwrap();
        if let Some(loc) = &self.config.location {
            path.push(loc);
        }
        _ = fs::create_dir_all(&path);
        for f in &files {
            let mut file = path.clone();
            file.push(f);
            fs::remove_file(file).map_err(|_| {
                RepackError::from_lang_with_msg(
                    RepackErrorKind::CannotWrite,
                    self.config,
                    f.to_string(),
                )
            })?;
        }

        // Will not delete if dir is not empty.
        _ = fs::remove_dir(&path);

        Ok(())
    }
}
