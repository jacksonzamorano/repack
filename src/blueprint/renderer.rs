use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fs::{self},
};

use crate::syntax::{FieldReferenceKind, Output, ParseResult, RepackError, RepackErrorKind};

use super::{
    Blueprint, BlueprintExecutionContext, FlyToken, SnippetMainTokenName, SnippetReference,
    SnippetSecondaryTokenName, TokenConsumer,
};

/// Represents different types of content that can be written to output files.
///
/// DeliveryUnit allows the rendering system to handle both regular text content
/// and special placeholders like import statements that need to be processed
/// and positioned correctly in the final output.
enum DeliveryUnit {
    /// Regular text content to be written directly to the output file
    Text(String),
    /// Placeholder for import statements that will be inserted at this position
    Imports,
}

/// Accumulates the results of blueprint rendering for multiple output files.
///
/// BlueprintBuildResult collects all content generated during the rendering process,
/// organizing it by filename and managing imports separately so they can be
/// properly positioned in the final output files.
#[derive(Default)]
struct BlueprintBuildResult {
    /// Map of filenames to their ordered content units (text and import placeholders)
    contents: HashMap<String, Vec<DeliveryUnit>>,
    /// Map of filenames to their sets of import statements
    imports: HashMap<String, HashSet<String>>,
    /// The currently active output file for new content
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

/// Orchestrates the code generation process using a blueprint and parsed schema.
///
/// BlueprintRenderer takes a parsed schema, a target language blueprint, and output
/// configuration to generate source code files. It processes blueprint templates,
/// handles variable substitution, manages file output, and coordinates the entire
/// code generation workflow.
pub struct BlueprintRenderer<'a> {
    /// The blueprint defining the target language templates and rules
    pub blueprint: &'a Blueprint,
    /// The parsed schema containing objects, enums, and their relationships
    pub parse_result: &'a ParseResult,
    /// Output configuration specifying target location, categories, and options
    pub config: &'a Output,
}
impl<'a> BlueprintRenderer<'a> {
    /// Creates a new BlueprintRenderer with the necessary components for code generation.
    ///
    /// # Arguments
    /// * `parse_result` - The parsed schema data containing objects and enums
    /// * `blueprint` - The blueprint defining how to generate code for the target language
    /// * `config` - Output configuration specifying target settings and options
    ///
    /// # Returns
    /// A new BlueprintRenderer ready to generate code
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
                        if index > content.len() {
                            // NOT FOUND!
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::SnippetNotClosed,
                                self.config,
                                snip.main_token.to_string(),
                            ));
                        }
                    }
                    match self.render_snippet(
                        SnippetReference {
                            details: snip,
                            contents: &content[starting_at..index],
                        },
                        context,
                        writer,
                    ) {
                        Err(mut e) => {
                            e.add_to_stack(&snip);
                            return Err(e);
                        }
                        _ => {}
                    }
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
            SnippetMainTokenName::Each | SnippetMainTokenName::Eachr => {
                let rev = matches!(content.main_token(), SnippetMainTokenName::Eachr);
                let iter_options: Vec<_> = match content.secondary_token() {
                    SnippetSecondaryTokenName::Object => self
                        .parse_result
                        .included_objects(&self.config.categories, &self.config.exclude)
                        .into_iter()
                        .map(|x| Ok(context.with_object(x)))
                        .collect(),
                    SnippetSecondaryTokenName::Field => {
                        let Some(obj) = context.object else {
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::CannotCreateContext,
                                self.config,
                                "field in non-object context.".to_string(),
                            ));
                        };
                        obj.fields
                            .iter()
                            .map(|field| {
                                context.with_field(obj, field, self.blueprint, self.config, writer)
                            })
                            .collect()
                    }
                    SnippetSecondaryTokenName::Join => {
                        let Some(obj) = context.object else {
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::CannotCreateContext,
                                self.config,
                                "join in non-object context.".to_string(),
                            ));
                        };
                        obj.joins
                            .iter()
                            .map(|j| {
                                let foreign_entity = self
                                    .parse_result
                                    .objects
                                    .iter()
                                    .find(|x| match &x.table_name {
                                        Some(val) if *val == j.foreign_entity => true,
                                        _ => false,
                                    })
                                    .ok_or_else(|| {
                                        RepackError::from_lang_with_msg(
                                            RepackErrorKind::CannotCreateContext,
                                            self.config,
                                            "join where foreign entity can't be resolved."
                                                .to_string(),
                                        )
                                    })?;
                                context.with_join(obj, j, foreign_entity)
                            })
                            .collect()
                    }
                    SnippetSecondaryTokenName::Enum => self
                        .parse_result
                        .included_enums(&self.config.categories, &self.config.exclude)
                        .iter()
                        .map(|enm| context.with_enum(enm))
                        .collect(),
                    SnippetSecondaryTokenName::Case => {
                        let Some(enm) = context.enm else {
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::CannotCreateContext,
                                self.config,
                                "case in non-enum context.".to_string(),
                            ));
                        };
                        enm.options
                            .iter()
                            .map(|case| context.with_enum_case(enm, case))
                            .collect()
                    }
                    SnippetSecondaryTokenName::Arg => {
                        let Some(args) = context.func_args else {
                            return Err(RepackError::from_lang_with_msg(
                                RepackErrorKind::CannotCreateContext,
                                self.config,
                                "args in non-func context".to_string(),
                            ));
                        };
                        args.iter().map(|x| context.with_func_arg(&x)).collect()
                    }
                    _ => {
                        return Err(RepackError::from_lang_with_msg(
                            RepackErrorKind::VariableNotInScope,
                            self.config,
                            content.details.secondary_token.to_string(),
                        ));
                    }
                };
                let len = iter_options.len();
                if !rev {
                    for (idx, ctx) in iter_options.into_iter().enumerate() {
                        let mut ctx = ctx?;
                        ctx.flags.insert("sep", idx + 1 < len);
                        self.render_tokens(content.contents, &ctx, writer)?;
                    }
                } else {
                    for (idx, ctx) in iter_options.into_iter().rev().enumerate() {
                        let mut ctx = ctx?;
                        ctx.flags.insert("sep", idx + 1 < len);
                        self.render_tokens(content.contents, &ctx, writer)?;
                    }
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
                    for matched_fn in field
                        .functions_in_namespace(namespace)
                        .iter()
                        .filter(|func| func.name == name)
                    {
                        let updated_context = context.with_func_args(&matched_fn.args)?;
                        self.render_tokens(content.contents, &updated_context, writer)?;
                    }
                }
                if let Some(obj) = context.object {
                    for matched_fn in obj
                        .functions_in_namespace(namespace)
                        .iter()
                        .filter(|func| func.name == name)
                    {
                        let updated_context = context.with_func_args(&matched_fn.args)?;
                        self.render_tokens(content.contents, &updated_context, writer)?;
                    }
                }
            }
            SnippetMainTokenName::Nfunc => {
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
                    if !field
                        .functions_in_namespace(namespace)
                        .iter()
                        .any(|func| func.name == name)
                    {
                        self.render_tokens(content.contents, context, writer)?;
                    }
                }
                if let Some(obj) = context.object {
                    if !obj
                        .functions_in_namespace(namespace)
                        .iter()
                        .any(|func| func.name == name)
                    {
                        self.render_tokens(content.contents, context, writer)?;
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
                } else {
                    return Err(RepackError::from_lang_with_msg(
                        RepackErrorKind::UnknownLink,
                        &self.config,
                        content.details.secondary_token.to_string(),
                    ));
                }
            }
            SnippetMainTokenName::Break => {
                writer.write(&"\n");
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
                            "split_period_first" => {
                                res = res.split(".").next().unwrap().to_string()
                            }
                            "split_period_last" => res = res.split(".").last().unwrap().to_string(),
                            "split_dash_first" => res = res.split("-").next().unwrap().to_string(),
                            "split_dash_last" => res = res.split("-").last().unwrap().to_string(),
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

    /// Executes the complete code generation process and writes output files.
    ///
    /// This method processes the blueprint templates with the parsed schema data,
    /// generates all target source code files, handles import management, and
    /// writes the final files to the configured output location.
    ///
    /// # Returns
    /// * `Ok(())` if code generation completes successfully
    /// * `Err(RepackError)` if any step in the generation process fails
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

    /// Removes all previously generated files from the output directory.
    ///
    /// This method identifies which files would be generated by the current
    /// configuration and removes them from the output directory. Useful for
    /// cleaning up before regeneration or removing outdated generated code.
    ///
    /// # Returns
    /// * `Ok(())` if cleanup completes successfully
    /// * `Err(RepackError)` if files cannot be removed
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
