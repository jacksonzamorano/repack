use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fs::{self},
};

use crate::syntax::{
    Enum, Field, FieldReferenceKind, FieldType, Object, ObjectJoin, Output, ParseResult,
    RepackError, RepackErrorKind,
};

use super::{
    Blueprint, FlyToken, SnippetMainTokenName, SnippetReference, SnippetSecondaryTokenName,
};

#[derive(Debug, Clone)]
struct BlueprintExecutionContext<'a> {
    variables: HashMap<String, String>,
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
        variables.insert("name".to_string(), obj.name.to_string());
        if let Some(tn) = obj.table_name.as_ref() {
            variables.insert("table_name".to_string(), tn.to_string());
        }
        flags.insert(
            "record",
            matches!(obj.object_type, crate::syntax::ObjectType::Record),
        );
        flags.insert("syn", obj.inherits.is_some());

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
        writer: &mut dyn TokenConsumer,
        is_last: bool,
    ) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        let resolved_type = match field.field_type() {
            FieldType::Core(typ) => {
                if let Some(link) = blueprint.links.get(&typ.to_string()) {
                    writer.import(link.to_string());
                }

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
            FieldType::Custom(typ, _) => {
                if let Some(link) = blueprint.links.get("custom") {
                    writer.import(link.replace("$", typ))
                }
                typ
            }
        };

        let (name, loc) = match &field.location.reference {
            FieldReferenceKind::Local | FieldReferenceKind::FieldType(_) => (
                field.name.clone(),
                obj.table_name
                    .as_ref()
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
            ),
            FieldReferenceKind::ImplicitJoin(local_field_name) => (
                field.location.name.clone(),
                format!("j_{}", local_field_name),
            ),
            FieldReferenceKind::ExplicitJoin(jn) => (field.location.name.clone(), jn.to_string()),
        };
        variables.insert("ref_entity".to_string(), loc);
        variables.insert("ref_field".to_string(), name);
        variables.insert("object_name".to_string(), obj.name.to_string());
        variables.insert("name".to_string(), field.name.to_string());
        variables.insert("type".to_string(), resolved_type.to_string());
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
    fn with_join(
        &self,
        obj: &'a Object,
        join: &'a ObjectJoin,
        is_last: bool,
    ) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        variables.insert("name".to_string(), join.join_name.to_string());
        if let Some(tn) = obj.table_name.as_ref() {
            variables.insert("local_entity".to_string(), tn.to_string());
        }
        variables.insert("local_field".to_string(), join.local_field.to_string());
        variables.insert("ref_field".to_string(), join.foreign_field.to_string());
        variables.insert("ref_entity".to_string(), join.foreign_entity.to_string());
        variables.insert("condition".to_string(), join.condition.to_string());

        flags.insert("sep", is_last);

        Ok(Self {
            variables,
            flags,
            object: Some(obj),
            field: None,
            enm: None,
        })
    }

    fn with_enum(&self, enm: &'a Enum) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), enm.name.to_string());
        Ok(Self {
            variables,
            flags: HashMap::new(),
            object: None,
            field: None,
            enm: Some(enm),
        })
    }
    fn with_enum_case(
        &self,
        enm: &'a Enum,
        val: &'a String,
        is_last: bool,
    ) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        let mut flags = HashMap::new();

        variables.insert("enum_name".to_string(), enm.name.to_string());
        variables.insert("name".to_string(), val.to_string());
        variables.insert("value".to_string(), val.to_string());
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
    fn import_point(&mut self);
    fn write(&mut self, value: &dyn AsRef<str>);
    fn import(&mut self, value: String);
}
enum DeliveryUnit {
    Text(String),
    Imports,
}

#[derive(Default)]
struct BlueprintBuildResult {
    contents: HashMap<String, Vec<DeliveryUnit>>,
    imports: HashMap<String, HashSet<String>>,
    current_file_name: Option<String>,
}
impl TokenConsumer for BlueprintBuildResult {
    fn set_file_name(&mut self, filename: &str) {
        self.current_file_name = Some(filename.to_string());
    }
    fn write(&mut self, value: &dyn AsRef<str>) {
        if let Some(file) = &self.current_file_name {
            if let Some(current) = self.contents.get_mut(file) {
                current.push(DeliveryUnit::Text(value.as_ref().to_string()));
            } else {
                self.contents.insert(
                    file.to_string(),
                    vec![DeliveryUnit::Text(value.as_ref().to_string())],
                );
            }
        }
    }
    fn import(&mut self, value: String) {
        if let Some(file) = &self.current_file_name {
            if let Some(current) = self.imports.get_mut(file) {
                current.insert(value);
            } else {
                let mut new = HashSet::new();
                new.insert(value);
                self.imports.insert(file.to_string(), new);
            }
        }
    }
    fn import_point(&mut self) {
        if let Some(file) = &self.current_file_name {
            if let Some(current) = self.contents.get_mut(file) {
                current.push(DeliveryUnit::Imports);
            } else {
                self.contents
                    .insert(file.to_string(), vec![DeliveryUnit::Imports]);
            }
        }
    }
}
impl TokenConsumer for HashSet<String> {
    fn set_file_name(&mut self, filename: &str) {
        self.insert(filename.to_string());
    }
    fn write(&mut self, _value: &dyn AsRef<str>) {}
    fn import(&mut self, _value: String) {}
    fn import_point(&mut self) {}
}
impl TokenConsumer for String {
    fn set_file_name(&mut self, _filename: &str) {}
    fn write(&mut self, value: &dyn AsRef<str>) {
        self.push_str(value.as_ref());
    }
    fn import(&mut self, _value: String) {}
    fn import_point(&mut self) {}
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
                        if index == content.len() {
                            // NOT FOUND!
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::SnippetNotClosed,
                                self.config,
                                snip.main_token.to_string(),
                            ));
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
                            let ctx = &context.with_field(
                                obj,
                                field,
                                self.blueprint,
                                self.config,
                                writer,
                                idx + 1 == obj.fields.len(),
                            )?;
                            self.render_tokens(content.contents, ctx, writer)?;
                            if !inline {
                                writer.write(&"\n");
                            }
                        }
                    }
                    SnippetSecondaryTokenName::Join => {
                        let Some(obj) = context.object else {
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::CannotCreateContext,
                                self.config,
                                "join in non-object context.".to_string(),
                            ));
                        };
                        let mut joins = obj.joins.iter().collect::<Vec<_>>();
                        if rev {
                            joins.reverse();
                        }
                        for (idx, j) in joins.iter().enumerate() {
                            self.render_tokens(
                                content.contents,
                                &context.with_join(obj, j, idx + 1 == obj.fields.len())?,
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
                                &context.with_enum_case(enm, case, idx + 1 == enm.options.len())?,
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
                            updated_context
                                .variables
                                .insert(idx.to_string(), arg.to_string());
                        }

                        self.render_tokens(content.contents, &updated_context, writer)?;
                    }
                }
                if let Some(obj) = context.object {
                    if let Some(matched_fn) = obj
                        .functions_in_namespace(namespace)
                        .iter()
                        .find(|func| func.name == name)
                    {
                        let mut updated_context = context.clone();
                        for (idx, arg) in matched_fn.args.iter().enumerate() {
                            updated_context
                                .variables
                                .insert(idx.to_string(), arg.to_string());
                        }

                        self.render_tokens(content.contents, &updated_context, writer)?;
                    }
                }
            }
            SnippetMainTokenName::Ref => {
                if let Some(field) = context.field {
                    if let Some(obj) = context.object {
                        let mut u_context = context.clone();
                        if let Some(tn) = obj.table_name.as_ref() {
                            u_context
                                .variables
                                .insert("local_entity".to_string(), tn.to_string());
                        }
                        if let FieldReferenceKind::FieldType(entity_name) =
                            &field.location.reference
                        {
                            if let Some(entity) = self
                                .parse_result
                                .objects
                                .iter()
                                .find(|x| x.name == *entity_name)
                            {
                                if let Some(e_tn) = entity.table_name.as_ref() {
                                    u_context
                                        .variables
                                        .insert("foreign_table".to_string(), e_tn.to_string());
                                }
                            }
                            u_context
                                .variables
                                .insert("foreign_entity".to_string(), entity_name.to_string());
                            u_context.variables.insert(
                                "foreign_field".to_string(),
                                field.location.name.to_string(),
                            );
                            self.render_tokens(content.contents, &u_context, writer)?;
                        };
                    }
                }
            }
            SnippetMainTokenName::PlaceImports => {
                writer.import_point();
            }
            SnippetMainTokenName::Import => {
                if let Some(import) = self.blueprint.links.get(&content.details.secondary_token) {
                    writer.import(import.clone());
                }
            }
            SnippetMainTokenName::Variable(var) => {
                let mut components = var.split(".");
                let name = components.next().unwrap();
                if let Some(mut res) = context.variables.get(name).map(|x| x.to_string()) {
                    for transform in components {
                        match transform {
                            "uppercase" => res = res.to_uppercase(),
                            "lowercase" => res = res.to_lowercase(),
                            "titlecase" => {
                                res = res
                                    .split('_')
                                    .map(|x| {
                                        let mut chars = x.chars();
                                        match chars.next() {
                                            None => String::new(),
                                            Some(first) => {
                                                first.to_uppercase().collect::<String>()
                                                    + &chars.as_str().to_lowercase()
                                            }
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join("")
                            }
                            "camelcase" => {
                                res = res
                                    .split('_')
                                    .enumerate()
                                    .map(|(i, x)| {
                                        if i > 0 {
                                            let mut chars = x.chars();
                                            match chars.next() {
                                                None => String::new(),
                                                Some(first) => {
                                                    first.to_uppercase().collect::<String>()
                                                        + &chars.as_str().to_lowercase()
                                                }
                                            }
                                        } else {
                                            x.to_string()
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join("")
                            }
                            _ => {
                                return Err(RepackError::from_lang_with_msg(
                                    RepackErrorKind::InvalidVariableModifier,
                                    self.config,
                                    transform.to_string(),
                                ));
                            }
                        }
                    }
                    writer.write(&res);
                } else {
                    return Err(RepackError::from_lang_with_msg(
                        RepackErrorKind::VariableNotInScope,
                        self.config,
                        name.to_string(),
                    ));
                }
            }
            _ => {}
        };

        Ok(())
    }

    pub fn build(&mut self) -> Result<(), RepackError> {
        let mut files = BlueprintBuildResult::default();
        let mut context = BlueprintExecutionContext::new();
        for opt in &self.config.options {
            context
                .variables
                .insert(opt.0.to_string(), opt.1.to_string());
        }
        _ = &self.render_tokens(&self.blueprint.tokens, &context, &mut files)?;
        let mut path = current_dir().unwrap();
        if let Some(loc) = &self.config.location {
            path.push(loc);
        }
        _ = fs::create_dir_all(&path);
        for f in files.contents {
            let mut file = path.clone();
            file.push(&f.0);

            let mut write_value = String::new();
            for part in f.1 {
                match part {
                    DeliveryUnit::Text(txt) => write_value.push_str(&txt),
                    DeliveryUnit::Imports => {
                        if let Some(imports) = files.imports.remove(&f.0) {
                            write_value.push('\n');
                            for import in imports.into_iter() {
                                write_value.push_str(&import);
                                write_value.push('\n');
                            }
                            write_value.push('\n');
                        }
                    }
                }
            }

            fs::write(file, write_value).map_err(|_| {
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
