use std::collections::HashSet;

use super::{
    Field, FieldType, FileContents, ObjectFunction, RepackError, RepackErrorKind, Token,
    query::Query,
};

#[derive(Debug)]
pub struct RepackStructJoin {
    pub name: String,
    pub contents: String,
    pub foreign_entity: String,
}
impl RepackStructJoin {
    pub fn parse(contents: &mut FileContents) -> Result<RepackStructJoin, RepackError> {
        if !matches!(contents.take(), Some(Token::OpenParen)) {
            return Err(RepackError::global(
                RepackErrorKind::SyntaxError,
                "Expected to get a join descriptor.".to_string(),
            ));
        }
        let Some(name) = contents.take_literal() else {
            return Err(RepackError::global(
                RepackErrorKind::SyntaxError,
                "Expected to get a join name.".to_string(),
            ));
        };
        let Some(foreign_entity) = contents.take_literal() else {
            return Err(RepackError::global(
                RepackErrorKind::SyntaxError,
                "Expected to get a join foreign entity.".to_string(),
            ));
        };
        if !matches!(contents.take(), Some(Token::CloseParen)) {
            return Err(RepackError::global(
                RepackErrorKind::SyntaxError,
                "Expected to close the join descriptor.".to_string(),
            ));
        }
        if !matches!(contents.take(), Some(Token::Equal)) {
            return Err(RepackError::global(
                RepackErrorKind::SyntaxError,
                "Expected an '=' to mark the start of a join predicate.".to_string(),
            ));
        }
        let Some(contents) = contents.take_literal() else {
            return Err(RepackError::global(
                RepackErrorKind::SyntaxError,
                "Expected to get a join query.".to_string(),
            ));
        };

        Ok(RepackStructJoin {
            name,
            contents,
            foreign_entity,
        })
    }
}

/// Represents a complete object definition in the schema system.
///
/// Object is the core building block of the schema, containing all the metadata
/// needed to generate code for entities, data structures, and their relationships.
/// Each object can have fields, functions, inheritance relationships, and database
/// mappings depending on its type.
#[derive(Debug)]
pub struct RepackStruct {
    /// The unique name identifier for this object used in code generation.
    pub name: String,
    /// The list of fields/properties that belong to this object.
    pub fields: Vec<Field>,
    /// Optional parent object name for inheritance relationships.
    /// Currently unused as inheritance is not yet implemented.
    pub inherits: Option<String>,
    /// Tags/categories for organizing and filtering objects during generation.
    /// Used by blueprints to selectively process certain object types.
    pub categories: Vec<String>,
    /// Optional database table name for objects that map to database tables.
    /// Used by database blueprints like PostgreSQL for table generation.
    pub table_name: Option<String>,
    /// Names of code snippets to include in the generated code.
    /// Snippets provide custom code injection points for specialized logic.
    pub use_snippets: Vec<String>,
    /// Custom functions/methods defined for this object.
    /// These generate additional methods in the target language classes.
    pub functions: Vec<ObjectFunction>,
    pub queries: Vec<Query>,
    pub joins: Vec<RepackStructJoin>,
}
impl RepackStruct {
    /// Parses an Object definition from the input file contents.
    ///
    /// This method reads the schema definition syntax and constructs a complete
    /// Object instance with all its metadata, fields, and relationships.
    /// The parsing handles various tokens like @table_name, :inheritance,
    /// #categories, and field definitions within braces.
    ///
    /// # Arguments
    /// * `typ` - The initial object type (Record, Synthetic, or Struct)
    /// * `contents` - Mutable reference to the file contents being parsed
    ///
    /// # Returns
    /// A fully constructed Object with all parsed metadata and fields
    ///
    /// # Panics
    /// Panics if the expected object name is missing or malformed
    pub fn read_from_contents(contents: &mut FileContents) -> RepackStruct {
        let Some(name_opt) = contents.next() else {
            panic!("Read record type, expected a name but got end of file.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!("Read record type, expected a name but got {name_opt:?}");
        };
        let name = name_ref.to_string();
        let mut fields = Vec::new();
        let mut categories = Vec::new();
        let mut inherits = None;
        let mut table_name = None;
        let mut use_snippets = Vec::new();
        let mut functions = Vec::new();
        let mut queries = Vec::new();
        let mut joins = Vec::new();

        'header: while let Some(token) = contents.next() {
            match token {
                Token::At => {
                    table_name = match contents.next() {
                        Some(Token::Literal(lit)) => Some(lit.to_string()),
                        _ => None,
                    };
                }
                Token::Colon => {
                    inherits = match contents.next() {
                        Some(Token::Literal(lit)) => Some(lit.to_string()),
                        _ => None,
                    };
                }
                Token::Pound => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        categories.push(lit.to_string());
                    }
                }
                Token::OpenBrace => {
                    break 'header;
                }
                _ => {}
            }
        }

        'cmd: while let Some(token) = contents.take() {
            match token {
                Token::CloseBrace => break 'cmd,
                Token::Literal(lit) => {
                    if let Some(next) = contents.peek() {
                        if *next == Token::Colon {
                            if let Some(func) =
                                ObjectFunction::from_contents(lit.to_string(), contents)
                            {
                                functions.push(func);
                            }
                        } else if let Some(field) = Field::from_contents(lit.to_string(), contents) {
                            fields.push(field);
                        } else {
                            panic!("Cannot parse field in {name}");
                        }
                    }
                }
                Token::Join => match RepackStructJoin::parse(contents) {
                    Ok(j) => joins.push(j),
                    Err(e) => panic!("{}", e.into_string()),
                },
                Token::Query => match Query::parse(&name, contents) {
                    Ok(q) => queries.push(q),
                    Err(e) => panic!("{}", e.into_string()),
                },
                Token::Exclamation => {
                    if let Some(Token::Literal(snippet_name)) = contents.take() {
                        use_snippets.push(snippet_name);
                    }
                }
                _ => {}
            }
        }

        RepackStruct {
            name,
            fields,
            inherits,
            table_name,
            categories,
            use_snippets,
            functions,
            queries,
            joins,
        }
    }

    /// Validates the object definition and returns any semantic errors.
    ///
    /// This method performs comprehensive validation of the object based on its type:
    /// - Validates field names are unique within each object
    /// - Ensures all field types are properly resolved
    /// - All objects must have unique field names and resolved field types
    ///
    /// # Returns
    /// * `Some(Vec<RepackError>)` if validation errors are found
    /// * `None` if the object is valid
    pub fn errors(&self) -> Option<Vec<RepackError>> {
        let mut errors = Vec::new();
        let mut field_names = HashSet::new();
        for field in &self.fields {
            if field_names.contains(&field.name) {
                errors.push(RepackError::from_field(
                    RepackErrorKind::DuplicateFieldNames,
                    self,
                    field,
                ));
            } else {
                field_names.insert(field.name.clone());
            }
            if field.field_type.is_none() {
                errors.push(RepackError::from_field(
                    RepackErrorKind::TypeNotResolved,
                    self,
                    field,
                ));
                continue;
            };
        }
        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }

    /// Determines the dependency relationships for this object.
    ///
    /// Analyzes the object's inheritance and field references to identify
    /// which other objects this object depends on. This is crucial for
    /// proper dependency ordering during code generation.
    ///
    /// # Returns
    /// A vector of object names that this object depends on, including:
    /// - Parent objects (via inheritance)
    /// - Referenced objects (via field types)
    /// - Join target objects (via implicit joins)
    pub fn depends_on(&self) -> Vec<String> {
        let mut dependencies = HashSet::new();
        if let Some(inherit) = &self.inherits {
            dependencies.insert(inherit.to_string());
        }
        for field in &self.fields {
            match field.field_type {
                Some(FieldType::Custom(_, super::CustomFieldType::Object)) | None => {
                    dependencies.insert(field.field_type_string.to_string());
                }
                _ => {}
            }
        }
        dependencies.into_iter().collect()
    }

    /// Filters object functions by their namespace.
    ///
    /// Returns all functions defined on this object that belong to the
    /// specified namespace. Namespaces are used to organize functions
    /// by their target language or usage context.
    ///
    /// # Arguments
    /// * `ns` - The namespace identifier to filter by
    ///
    /// # Returns
    /// A vector of references to functions in the specified namespace
    pub fn functions_in_namespace(&self, ns: &str) -> Vec<&ObjectFunction> {
        self.functions
            .iter()
            .filter(|x| x.namespace == *ns)
            .collect()
    }
}
