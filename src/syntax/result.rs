use super::{
    CustomFieldType, FieldType, FileContents, Output, RepackEnum, RepackError, RepackErrorKind,
    RepackStruct, Snippet, Token, dependancies::graph_valid, language,
};

/// Represents the complete parsed schema with all defined entities and configurations.
///
/// ParseResult contains all the parsed elements from a schema file, including objects,
/// enums, output configurations, and blueprint dependencies. This structure serves as
/// the primary input for code generation and validation processes.
#[derive(Debug)]
pub struct ParseResult {
    /// All parsed object definitions (structs)
    pub strcts: Vec<RepackStruct>,
    /// Output configuration definitions specifying target languages and settings
    pub languages: Vec<Output>,
    /// All parsed enumeration definitions
    pub enums: Vec<RepackEnum>,
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

        let mut strcts = Vec::new();
        let mut snippets = Vec::new();
        let mut languages = Vec::new();
        let mut enums = Vec::new();
        let mut include_blueprints = Vec::new();

        while let Some(token) = contents.next() {
            match *token {
                Token::StructType => {
                    match RepackStruct::read_from_contents(&mut contents) {
                        Ok(s) => strcts.push(s),
                        Err(e) => return Err(vec![e]),
                    }
                }
                Token::EnumType => {
                    match RepackEnum::read_from_contents(&mut contents) {
                        Ok(e) => enums.push(e),
                        Err(e) => return Err(vec![e]),
                    }
                }
                Token::SnippetType => {
                    match Snippet::read_from_contents(&mut contents) {
                        Ok(s) => snippets.push(s),
                        Err(e) => return Err(vec![e]),
                    }
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
        let mut strct_snip_idx = 0;
        while strct_snip_idx < strcts.len() {
            let mut snip_offset = 0;
            let mut snip_idx = 0;
            while snip_idx < strcts[strct_snip_idx].use_snippets.iter().len() {
                let snip_name = &strcts[strct_snip_idx].use_snippets[snip_idx];
                let snippet = snippets
                    .iter()
                    .find(|snip| snip.name == *snip_name)
                    .ok_or_else(|| {
                        vec![RepackError::from_obj_with_msg(
                            RepackErrorKind::SnippetNotFound,
                            &strcts[strct_snip_idx],
                            snip_name.to_string(),
                        )]
                    })?;
                let snippet_fields = snippet.fields.clone();
                for s in snippet_fields.into_iter() {
                    strcts[strct_snip_idx].fields.insert(snip_offset, s);
                    snip_offset += 1;
                }
                let mut snippet_fns = snippet.functions.clone();
                strcts[strct_snip_idx].functions.append(&mut snippet_fns);
                snip_idx += 1;
            }
            strct_snip_idx += 1;
        }

        // Rearrange all objects in dependancy order
        // for simple resolution.
        let mut i = 0;
        while i < strcts.len() {
            let mut found_issue = false;
            'dep_search: for dependancy in strcts[i].depends_on() {
                let mut x = i;
                while x < strcts.len() {
                    if strcts[x].name == dependancy {
                        found_issue = true;
                        break 'dep_search;
                    }
                    x += 1;
                }
            }
            if found_issue {
                let dep = strcts.remove(i);
                strcts.push(dep);
                i = 0
            } else {
                i += 1;
            }
        }

        // Resolve references and do some error checking.
        let mut object_idx: usize = 0;
        while object_idx < strcts.len() {
            let mut field_idx: usize = 0;

            if let Some(parent_obj_name) = &strcts[object_idx].inherits {
                let Some(parent_obj_idx) =
                    strcts.iter().position(|obj| obj.name == *parent_obj_name)
                else {
                    errors.push(RepackError::from_obj_with_msg(
                        RepackErrorKind::ParentObjectDoesNotExist,
                        &strcts[object_idx],
                        parent_obj_name.to_string(),
                    ));
                    object_idx += 1;
                    continue;
                };
                strcts[object_idx].table_name = strcts[parent_obj_idx].table_name.clone();
            }

            while field_idx < strcts[object_idx].fields.len() {
                if let Some(ext) = &strcts[object_idx].fields[field_idx].field_location {
                    // This comes from a join or a super.
                    if ext.location == "super" {
                        let Some(sup) = &strcts[object_idx].inherits else {
                            errors.push(RepackError::from_field(
                                RepackErrorKind::InvalidSuper,
                                &strcts[object_idx],
                                &strcts[object_idx].fields[field_idx],
                            ));
                            field_idx += 1;
                            continue;
                        };
                        let Some(sup_idx) = strcts.iter().position(|x| x.name == *sup) else {
                            return Err(vec![RepackError::from_field(
                                RepackErrorKind::ParentObjectDoesNotExist,
                                &strcts[object_idx],
                                &strcts[object_idx].fields[field_idx],
                            )]);
                        };
                        let Some(foreign_pos) = &strcts[sup_idx]
                            .fields
                            .iter()
                            .position(|x| x.name == ext.field)
                        else {
                            errors.push(RepackError::from_field(
                                RepackErrorKind::FieldNotOnSuper,
                                &strcts[object_idx],
                                &strcts[object_idx].fields[field_idx],
                            ));
                            field_idx += 1;
                            continue;
                        };
                        strcts[object_idx].fields[field_idx].field_type =
                            strcts[sup_idx].fields[*foreign_pos].field_type.clone();
                    } else {
                        let Some(join_idx) = &strcts[object_idx]
                            .joins
                            .iter()
                            .position(|x| x.name == ext.location)
                        else {
                            errors.push(RepackError::from_field(
                                RepackErrorKind::InvalidJoin,
                                &strcts[object_idx],
                                &strcts[object_idx].fields[field_idx],
                            ));
                            field_idx += 1;
                            continue;
                        };
                        let Some(joined_entity_idx) = &strcts.iter().position(|x| {
                            x.name == strcts[object_idx].joins[*join_idx].foreign_entity
                        }) else {
                            errors.push(RepackError::from_field(
                                RepackErrorKind::InvalidJoin,
                                &strcts[object_idx],
                                &strcts[object_idx].fields[field_idx],
                            ));
                            field_idx += 1;
                            continue;
                        };
                        let Some(joined_field_idx) = &strcts[*joined_entity_idx]
                            .fields
                            .iter()
                            .position(|x| x.name == ext.field)
                        else {
                            errors.push(RepackError::from_field(
                                RepackErrorKind::FieldNotOnJoin,
                                &strcts[object_idx],
                                &strcts[object_idx].fields[field_idx],
                            ));
                            field_idx += 1;
                            continue;
                        };
                        strcts[object_idx].fields[field_idx].field_type =
                            strcts[*joined_entity_idx].fields[*joined_field_idx]
                                .field_type
                                .clone();
                    }
                } else {
                    // This is just a custom type, let's resolve it.
                    let lookup_name = &strcts[object_idx].fields[field_idx].field_type_string;
                    if strcts.iter().any(|obj| obj.name == *lookup_name) {
                        strcts[object_idx].fields[field_idx].field_type = Some(FieldType::Custom(
                            lookup_name.clone(),
                            CustomFieldType::Object,
                        ));
                    } else if enums.iter().any(|en| en.name == *lookup_name) {
                        strcts[object_idx].fields[field_idx].field_type = Some(FieldType::Custom(
                            lookup_name.clone(),
                            CustomFieldType::Enum,
                        ));
                    }
                }
                // Ensure types are resolved
                if let Some(FieldType::Custom(object_name, _)) =
                    &strcts[object_idx].fields[field_idx].field_type
                {
                    if !strcts.iter().any(|o| o.name == *object_name)
                        && !enums.iter().any(|e| e.name == *object_name)
                    {
                        errors.push(RepackError::from_field_with_msg(
                            RepackErrorKind::CustomTypeNotDefined,
                            &strcts[object_idx],
                            &strcts[object_idx].fields[field_idx],
                            object_name.to_string(),
                        ));
                    }
                }
                field_idx += 1;
            }

            let mut autoq_idx = 0;
            while autoq_idx < strcts[object_idx].autoinsertqueries.len() {
                match strcts[object_idx].autoinsertqueries[autoq_idx].to_query(&strcts[object_idx]) {
                    Ok(val) => {
                        strcts[object_idx].queries.push(val);
                    }
                    Err(e) => {
                        errors.push(e)
                    }
                }
                autoq_idx += 1;
            } 
            autoq_idx = 0;
            while autoq_idx < strcts[object_idx].autoupdatequeries.len() {
                match strcts[object_idx].autoupdatequeries[autoq_idx].to_query() {
                    Ok(val) => {
                        strcts[object_idx].queries.push(val);
                    }
                    Err(e) => {
                        errors.push(e)
                    }
                }
                autoq_idx += 1;
            }

            object_idx += 1;
        }

        for object in &strcts {
            if let Some(mut errs) = object.errors() {
                errors.append(&mut errs);
            }
        }
        for language in &languages {
            let mut errs = language.errors();
            errors.append(&mut errs);
        }
        if let Err(e) = graph_valid(&strcts) {
            errors.push(e)
        }
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ParseResult {
                strcts,
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
    pub fn included_strcts(
        &self,
        categories: &[String],
        excludes: &[String],
    ) -> Vec<&RepackStruct> {
        self.strcts
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
    pub fn included_enums(&self, categories: &[String], excludes: &[String]) -> Vec<&RepackEnum> {
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
