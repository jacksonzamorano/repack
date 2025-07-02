use crate::syntax::FieldReferenceKind;

use super::{
    CustomFieldType, Enum, FieldType, FileContents, Object, ObjectJoin, ObjectType, Output,
    RepackError, RepackErrorKind, Snippet, Token, dependancies::graph_valid, language,
};

/// Represents the complete parsed schema with all defined entities and configurations.
///
/// ParseResult contains all the parsed elements from a schema file, including objects,
/// enums, output configurations, and blueprint dependencies. This structure serves as
/// the primary input for code generation and validation processes.
#[derive(Debug)]
pub struct ParseResult {
    /// All parsed object definitions (records, structs, synthetics)
    pub objects: Vec<Object>,
    /// Output configuration definitions specifying target languages and settings
    pub languages: Vec<Output>,
    /// All parsed enumeration definitions
    pub enums: Vec<Enum>,
    /// List of external blueprint files to be loaded for code generation
    pub include_blueprints: Vec<String>,
}

impl ParseResult {
    /// Parses the complete schema from tokenized file contents.
    ///
    /// This method performs the complete parsing pipeline:
    /// 1. Parses all top-level definitions (objects, enums, outputs, imports)
    /// 2. Expands snippet inclusions into objects
    /// 3. Resolves dependency ordering for objects
    /// 4. Resolves all field type references and relationships
    /// 5. Validates the complete schema for consistency
    ///
    /// # Arguments
    /// * `contents` - The tokenized file contents to parse
    ///
    /// # Returns
    /// * `Ok(ParseResult)` if parsing succeeds with a valid schema
    /// * `Err(Vec<RepackError>)` if any validation or parsing errors occur
    pub fn from_contents(mut contents: FileContents) -> Result<ParseResult, Vec<RepackError>> {
        let mut errors = Vec::<RepackError>::new();

        let mut objects = Vec::new();
        let mut snippets = Vec::new();
        let mut languages = Vec::new();
        let mut enums = Vec::new();
        let mut include_blueprints = Vec::new();

        while let Some(token) = contents.next() {
            match *token {
                Token::RecordType => {
                    objects.push(Object::read_from_contents(
                        ObjectType::Record,
                        &mut contents,
                    ));
                }
                Token::StructType => {
                    objects.push(Object::read_from_contents(
                        ObjectType::Struct,
                        &mut contents,
                    ));
                }
                Token::SyntheticType => {
                    objects.push(Object::read_from_contents(
                        ObjectType::Synthetic,
                        &mut contents,
                    ));
                }
                Token::EnumType => {
                    enums.push(Enum::read_from_contents(&mut contents));
                }
                Token::SnippetType => {
                    snippets.push(Snippet::read_from_contents(&mut contents));
                }
                Token::OutputType => {
                    if let Some(language) = language::Output::from_contents(&mut contents) {
                        languages.push(language);
                    }
                }
                Token::Import => {
                    if let Some(Token::Literal(path)) = contents.take() {
                        contents.add_relative(&path);
                    }
                }
                Token::Blueprint => {
                    if let Some(Token::Literal(path)) = contents.take() {
                        include_blueprints.push(path);
                    }
                }
                _ => {}
            }
        }

        // Expand all snippets.
        // This is important to do before dependancy checks
        // because snippets could introduce deps.
        let mut object_snip_idx = 0;
        while object_snip_idx < objects.len() {
            let mut snip_offset = 0;
            let mut snip_idx = 0;
            while snip_idx < objects[object_snip_idx].use_snippets.iter().len() {
                let snip_name = &objects[object_snip_idx].use_snippets[snip_idx];
                let snippet = snippets
                    .iter()
                    .find(|snip| snip.name == *snip_name)
                    .ok_or_else(|| {
                        vec![RepackError::from_obj_with_msg(
                            RepackErrorKind::SnippetNotFound,
                            &objects[object_snip_idx],
                            snip_name.to_string(),
                        )]
                    })?;
                let snippet_fields = snippet.fields.clone();
                for s in snippet_fields.into_iter() {
                    objects[object_snip_idx].fields.insert(snip_offset, s);
                    snip_offset += 1;
                }
                let mut snippet_fns = snippet.functions.clone();
                objects[object_snip_idx].functions.append(&mut snippet_fns);
                snip_idx += 1;
            }
            object_snip_idx += 1;
        }

        // Rearrange all objects in dependancy order
        // for simple resolution.
        let mut i = 0;
        while i < objects.len() {
            let mut found_issue = false;
            'dep_search: for dependancy in objects[i].depends_on() {
                let mut x = i;
                while x < objects.len() {
                    if objects[x].name == dependancy {
                        found_issue = true;
                        break 'dep_search;
                    }
                    x += 1;
                }
            }
            if found_issue {
                let dep = objects.remove(i);
                objects.push(dep);
                i = 0
            } else {
                i += 1;
            }
        }

        // Resolve references and do some error checking.
        let mut object_idx: usize = 0;
        while object_idx < objects.len() {
            let mut field_idx: usize = 0;

            let mut join_idx: usize = 0;
            while join_idx < objects[object_idx].joins.len() {
                if objects[object_idx].joins[join_idx].foreign_table.is_some() {
                    continue;
                }
                let Some(entity) = objects
                    .iter()
                    .find(|x| x.name == objects[object_idx].joins[join_idx].foreign_entity)
                else {
                    errors.push(RepackError::from_obj(
                        RepackErrorKind::JoinFieldUnresolvable,
                        &objects[object_idx],
                    ));
                    join_idx += 1;
                    continue;
                };
                let Some(table_name) = entity.table_name.as_ref() else {
                    errors.push(RepackError::from_obj(
                        RepackErrorKind::JoinNoTableName,
                        &objects[object_idx],
                    ));
                    join_idx += 1;
                    continue;
                };
                objects[object_idx].joins[join_idx].foreign_table = Some(table_name.to_string());
                join_idx += 1;
            }

            if let Some(parent_obj_name) = &objects[object_idx].inherits {
                if !matches!(&objects[object_idx].object_type, ObjectType::Synthetic) {
                    errors.push(RepackError::from_obj(
                        RepackErrorKind::CannotInherit,
                        &objects[object_idx],
                    ));
                    object_idx += 1;
                    continue;
                }
                let Some(parent_obj_idx) =
                    objects.iter().position(|obj| obj.name == *parent_obj_name)
                else {
                    errors.push(RepackError::from_obj_with_msg(
                        RepackErrorKind::ParentObjectDoesNotExist,
                        &objects[object_idx],
                        parent_obj_name.to_string(),
                    ));
                    object_idx += 1;
                    continue;
                };

                let copy = objects[parent_obj_idx].fields.clone();
                if objects[object_idx].reuse_all {
                    for c in copy {
                        if !objects[object_idx].reuse_exclude.contains(&c.name) {
                            objects[object_idx].fields.push(c);
                        }
                    }
                } else {
                    for c in copy {
                        if objects[object_idx].reuse_include.contains(&c.name) {
                            objects[object_idx].fields.push(c);
                        }
                    }
                }
                let mut parent_joins = objects[parent_obj_idx].joins.clone();
                objects[object_idx].joins.append(&mut parent_joins);
                objects[object_idx].table_name = objects[parent_obj_idx].table_name.clone();
                objects[object_idx]
                    .fields
                    .sort_by(|a, b| a.location.reference.cmp(&b.location.reference));
            }

            while field_idx < objects[object_idx].fields.len() {
                if objects[object_idx].fields[field_idx].field_type.is_none() {
                    match &objects[object_idx].fields[field_idx].location.reference {
                        FieldReferenceKind::Local => {
                            if let Some(lookup_name) =
                                &objects[object_idx].fields[field_idx].field_type_string
                            {
                                if objects.iter().any(|obj| obj.name == *lookup_name) {
                                    objects[object_idx].fields[field_idx].field_type =
                                        Some(FieldType::Custom(
                                            lookup_name.clone(),
                                            CustomFieldType::Object,
                                        ));
                                } else if enums.iter().any(|en| en.name == *lookup_name) {
                                    objects[object_idx].fields[field_idx].field_type =
                                        Some(FieldType::Custom(
                                            lookup_name.clone(),
                                            CustomFieldType::Enum,
                                        ));
                                }
                            }
                        }
                        FieldReferenceKind::ImplicitJoin(joining_field) => {
                            let Some(referenced_field) = &objects[object_idx]
                                .fields
                                .iter()
                                .find(|field| field.name == *joining_field)
                            else {
                                errors.push(RepackError::from_field_with_msg(
                                    RepackErrorKind::JoinFieldUnresolvable,
                                    &objects[object_idx],
                                    &objects[object_idx].fields[field_idx],
                                    joining_field.to_string(),
                                ));
                                field_idx += 1;
                                continue;
                            };
                            let referenced_entity = match &referenced_field.location.reference {
                                FieldReferenceKind::FieldType(entity_name) => {
                                    let Some(res) =
                                        objects.iter().find(|obj| obj.name == *entity_name)
                                    else {
                                        errors.push(RepackError::from_field_with_msg(
                                            RepackErrorKind::JoinFieldUnresolvable,
                                            &objects[object_idx],
                                            &objects[object_idx].fields[field_idx],
                                            joining_field.to_string(),
                                        ));
                                        field_idx += 1;
                                        continue;
                                    };
                                    res
                                }
                                _ => {
                                    errors.push(RepackError::from_field_with_msg(
                                        RepackErrorKind::JoinFieldUnresolvable,
                                        &objects[object_idx],
                                        &objects[object_idx].fields[field_idx],
                                        joining_field.to_string(),
                                    ));
                                    field_idx += 1;
                                    continue;
                                }
                            };
                            let Some(referenced_foreign_field) =
                                referenced_entity.fields.iter().find(|field| {
                                    field.name
                                        == objects[object_idx].fields[field_idx].location.name
                                })
                            else {
                                errors.push(RepackError::from_field_with_msg(
                                    RepackErrorKind::JoinFieldUnresolvable,
                                    &objects[object_idx],
                                    &objects[object_idx].fields[field_idx],
                                    joining_field.to_string(),
                                ));
                                field_idx += 1;
                                continue;
                            };
                            objects[object_idx].fields[field_idx].field_type =
                                referenced_foreign_field.field_type.clone();
                        }
                        FieldReferenceKind::FieldType(joining_entity) => {
                            let Some(referenced_entity) =
                                objects.iter().find(|obj| obj.name == *joining_entity)
                            else {
                                errors.push(RepackError::from_field_with_msg(
                                    RepackErrorKind::RefFieldUnresolvable,
                                    &objects[object_idx],
                                    &objects[object_idx].fields[field_idx],
                                    joining_entity.to_string(),
                                ));
                                field_idx += 1;
                                continue;
                            };
                            let Some(referenced_foreign_field) =
                                referenced_entity.fields.iter().find(|field| {
                                    field.name
                                        == objects[object_idx].fields[field_idx].location.name
                                })
                            else {
                                errors.push(RepackError::from_field_with_msg(
                                    RepackErrorKind::RefFieldUnresolvable,
                                    &objects[object_idx],
                                    &objects[object_idx].fields[field_idx],
                                    joining_entity.to_string(),
                                ));
                                field_idx += 1;
                                continue;
                            };
                            let typ = referenced_foreign_field.field_type.clone();
                            if matches!(objects[object_idx].object_type, ObjectType::Synthetic) {
                                let j = ObjectJoin {
                                    join_name: format!(
                                        "j_{}",
                                        objects[object_idx].fields[field_idx].name
                                    ),
                                    local_field: objects[object_idx].fields[field_idx]
                                        .name
                                        .to_string(),
                                    condition: "=".to_string(),
                                    foreign_entity: referenced_entity.name.clone(),
                                    foreign_table: referenced_entity.table_name.clone(),
                                    foreign_field: referenced_foreign_field.name.to_string(),
                                };
                                objects[object_idx].joins.push(j);
                            }
                            objects[object_idx].fields[field_idx].field_type = typ;
                        }
                        FieldReferenceKind::ExplicitJoin(join_name) => {
                            let Some(join) = objects[object_idx]
                                .joins
                                .iter()
                                .find(|x| x.join_name == *join_name)
                            else {
                                errors.push(RepackError::from_field_with_msg(
                                    RepackErrorKind::UnknownExplicitJoin,
                                    &objects[object_idx],
                                    &objects[object_idx].fields[field_idx],
                                    join_name.to_string(),
                                ));
                                field_idx += 1;
                                continue;
                            };
                            let Some(foreign_entity) =
                                objects.iter().find(|x| x.name == *join.foreign_entity)
                            else {
                                errors.push(RepackError::from_field_with_msg(
                                    RepackErrorKind::JoinObjectNotFound,
                                    &objects[object_idx],
                                    &objects[object_idx].fields[field_idx],
                                    join.foreign_entity.to_string(),
                                ));
                                field_idx += 1;
                                continue;
                            };
                            let Some(field) = foreign_entity
                                .fields
                                .iter()
                                .find(|x| x.name == objects[object_idx].fields[field_idx].location.name)
                            else {
                                errors.push(RepackError::from_field_with_msg(
                                    RepackErrorKind::JoinFieldNotFound,
                                    &objects[object_idx],
                                    &objects[object_idx].fields[field_idx],
                                    join.foreign_field.to_string(),
                                ));
                                field_idx += 1;
                                continue;
                            };
                            objects[object_idx].fields[field_idx].field_type =
                                field.field_type.clone();
                        }
                    }
                }

                // Ensure custom types are resolved
                if let Some(FieldType::Custom(object_name, _)) =
                    &objects[object_idx].fields[field_idx].field_type
                {
                    if !objects.iter().any(|o| o.name == *object_name)
                        && !enums.iter().any(|e| e.name == *object_name)
                    {
                        errors.push(RepackError::from_field_with_msg(
                            RepackErrorKind::CustomTypeNotDefined,
                            &objects[object_idx],
                            &objects[object_idx].fields[field_idx],
                            object_name.to_string(),
                        ));
                    }
                }
                field_idx += 1;
            }
            object_idx += 1;
        }

        for object in &objects {
            if let Some(mut errs) = object.errors() {
                errors.append(&mut errs);
            }
        }
        for language in &languages {
            let mut errs = language.errors();
            errors.append(&mut errs);
        }
        if let Err(e) = graph_valid(&objects) {
            errors.push(e)
        }
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ParseResult {
                objects,
                languages,
                enums,
                include_blueprints,
            })
        }
    }

    /// Filters objects based on category inclusion and explicit exclusions.
    ///
    /// This method selects objects for code generation based on the target
    /// configuration's category filters and exclusion lists. Objects without
    /// categories are included by default when no category filter is specified.
    ///
    /// # Arguments
    /// * `categories` - List of categories to include (empty means include all)
    /// * `excludes` - List of object names to explicitly exclude
    ///
    /// # Returns
    /// A vector of object references that match the filtering criteria
    pub fn included_objects(&self, categories: &[String], excludes: &[String]) -> Vec<&Object> {
        self.objects
            .iter()
            .filter(|obj| {
                if obj.categories.is_empty() || categories.is_empty() {
                    return true;
                }
                if excludes.contains(&obj.name) {
                    return false;
                }
                obj.categories.iter().any(|cat| categories.contains(cat))
            })
            .collect()
    }

    /// Filters enums based on category inclusion and explicit exclusions.
    ///
    /// Similar to included_objects, this method selects enums for code generation
    /// based on category matching and exclusion rules.
    ///
    /// # Arguments
    /// * `categories` - List of categories to include (empty means include all)
    /// * `excludes` - List of enum names to explicitly exclude
    ///
    /// # Returns
    /// A vector of enum references that match the filtering criteria
    pub fn included_enums(&self, categories: &[String], excludes: &[String]) -> Vec<&Enum> {
        self.enums
            .iter()
            .filter(|enm| {
                if enm.categories.is_empty() || categories.is_empty() {
                    return true;
                }
                if excludes.contains(&enm.name) {
                    return false;
                }
                enm.categories.iter().any(|cat| categories.contains(cat))
            })
            .collect()
    }
}
