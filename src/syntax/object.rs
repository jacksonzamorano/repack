use std::collections::HashSet;

use super::{
    CustomFieldType, Field, FieldType, FileContents, ObjectFunction, RepackError, RepackErrorKind,
    Token, field::FieldReferenceKind,
};

/// Defines the different categories of objects that can be defined in a schema.
///
/// Each object type has different capabilities, constraints, and code generation
/// behaviors. The type determines how the object can be used and what features
/// are available during code generation.
#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    /// A database-backed entity that represents a table or collection.
    /// Records support database operations, table mapping, and persistence features.
    /// They typically map to database tables and support CRUD operations.
    Record,
    /// A derived object that inherits from a Record but adds additional computed fields.
    /// Synthetic objects extend existing Records with additional functionality
    /// while maintaining the relationship to the parent Record.
    Synthetic,
    /// A simple data structure without database backing.
    /// Structs are used for data transfer objects, API models, and in-memory
    /// data structures that don't require persistence.
    Struct,
}

/// Represents a relationship join between two objects in the schema.
///
/// ObjectJoin defines how objects are related to each other, specifying
/// the local and foreign fields that establish the relationship. This is
/// used for generating proper foreign key relationships and join queries
/// in the target code.
#[derive(Debug, Clone)]
pub struct ObjectJoin {
    /// The name identifier for this join relationship.
    /// Used in code generation to create meaningful method and variable names.
    pub join_name: String,
    /// The field name in the current object that participates in the join.
    pub local_field: String,
    /// The join condition operator (typically "=" for equality joins).
    pub condition: String,
    /// The name of the target object/entity being joined to.
    pub foreign_entity: String,
    /// The name of the target table being joined to.
    pub foreign_table: Option<String>,
    /// The field name in the foreign entity that participates in the join.
    pub foreign_field: String,
}

/// Represents a complete object definition in the schema system.
///
/// Object is the core building block of the schema, containing all the metadata
/// needed to generate code for entities, data structures, and their relationships.
/// Each object can have fields, functions, inheritance relationships, and database
/// mappings depending on its type.
#[derive(Debug)]
pub struct Object {
    /// The category/type of this object (Record, Synthetic, or Struct).
    pub object_type: ObjectType,
    /// The unique name identifier for this object used in code generation.
    pub name: String,
    /// The list of fields/properties that belong to this object.
    pub fields: Vec<Field>,
    /// Optional parent object name for inheritance relationships.
    /// Only available for Synthetic objects that extend Records.
    pub inherits: Option<String>,
    /// Tags/categories for organizing and filtering objects during generation.
    /// Used by blueprints to selectively process certain object types.
    pub categories: Vec<String>,
    /// Database table name override for Record objects.
    /// If None, the object name is used as the table name.
    pub table_name: Option<String>,
    /// When true, inherits all fields from the parent object.
    /// Used in combination with reuse_exclude to selectively inherit fields.
    pub reuse_all: bool,
    /// List of field names to exclude when reuse_all is true.
    /// Allows fine-grained control over inheritance.
    pub reuse_exclude: Vec<String>,
    /// List of specific field names to include from the parent object.
    /// Alternative to reuse_all for selective inheritance.
    pub reuse_include: Vec<String>,
    /// Names of code snippets to include in the generated code.
    /// Snippets provide custom code injection points for specialized logic.
    pub use_snippets: Vec<String>,
    /// Custom functions/methods defined for this object.
    /// These generate additional methods in the target language classes.
    pub functions: Vec<ObjectFunction>,
    /// Database join relationships to other objects.
    /// Defines how this object relates to other entities in queries.
    pub joins: Vec<ObjectJoin>,
}
impl Object {
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
    pub fn read_from_contents(typ: ObjectType, contents: &mut FileContents) -> Object {
        let Some(name_opt) = contents.next() else {
            panic!("Read record type, expected a name but got end of file.");
        };
        let Token::Literal(name_ref) = name_opt else {
            panic!("Read record type, expected a name but got {:?}", name_opt);
        };
        let name = name_ref.to_string();
        let mut fields = Vec::new();
        let mut categories = Vec::new();
        let mut inherits = None;
        let mut table_name = None;
        let mut reuse_all = false;
        let mut reuse_exclude = Vec::new();
        let mut reuse_include = Vec::new();
        let mut use_snippets = Vec::new();
        let mut functions = Vec::new();
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
                        } else if let Some(field) = Field::from_contents(lit.to_string(), contents)
                        {
                            fields.push(field);
                        }
                    }
                }
                Token::Star => {
                    reuse_all = true;
                }
                Token::Hat => {
                    let Some(Token::Literal(join_name)) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Literal(obj_1_name)) = contents.take() else {
                        continue;
                    };
                    contents.skip(); // Skip .
                    let Some(Token::Literal(obj_1_field)) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Equals) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Literal(obj_2_name)) = contents.take() else {
                        continue;
                    };
                    contents.skip(); // Skip .
                    let Some(Token::Literal(obj_2_field)) = contents.take() else {
                        continue;
                    };

                    if obj_1_name == "self" {
                        joins.push(ObjectJoin {
                            join_name,
                            local_field: obj_1_field,
                            condition: "=".to_string(),
                            foreign_entity: obj_2_name,
                            foreign_field: obj_2_field,
                            foreign_table: None,
                        });
                    } else if obj_2_name == "self" {
                        joins.push(ObjectJoin {
                            join_name,
                            local_field: obj_2_field,
                            condition: "=".to_string(),
                            foreign_entity: obj_1_name,
                            foreign_field: obj_1_field,
                            foreign_table: None,
                        });
                    }
                }
                Token::Plus => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        reuse_include.push(lit.to_string());
                    }
                }
                Token::Minus => {
                    if let Some(Token::Literal(lit)) = contents.next() {
                        reuse_exclude.push(lit.to_string());
                    }
                }
                Token::Exclamation => {
                    if let Some(Token::Literal(snippet_name)) = contents.take() {
                        use_snippets.push(snippet_name);
                    }
                }
                _ => {}
            }
        }

        Object {
            object_type: typ,
            name,
            fields,
            inherits,
            table_name,
            reuse_all,
            reuse_exclude,
            reuse_include,
            categories,
            use_snippets,
            functions,
            joins,
        }
    }

    /// Validates the object definition and returns any semantic errors.
    ///
    /// This method performs comprehensive validation of the object based on its type:
    /// - Records must have table names and cannot have custom object field types
    /// - Structs cannot inherit, reuse fields, or have table names
    /// - All objects must have unique field names and resolved field types
    ///
    /// # Returns
    /// * `Some(Vec<RepackError>)` if validation errors are found
    /// * `None` if the object is valid
    pub fn errors(&self) -> Option<Vec<RepackError>> {
        let mut errors = Vec::new();
        if self.object_type == ObjectType::Record || self.object_type == ObjectType::Synthetic {
            for field in &self.fields {
                let Some(field_type) = &field.field_type else {
                    errors.push(RepackError::from_field(
                        RepackErrorKind::TypeNotResolved,
                        self,
                        field,
                    ));
                    continue;
                };
                if let FieldType::Custom(_, obj_type) = field_type {
                    if *obj_type != CustomFieldType::Enum {
                        errors.push(RepackError::from_field(
                            RepackErrorKind::CustomTypeNotAllowed,
                            self,
                            field,
                        ));
                    }
                }
                if field.array {
                    errors.push(RepackError::from_field(
                        RepackErrorKind::ManyNotAllowed,
                        self,
                        field,
                    ));
                }
            }
            if self.table_name.is_none() {
                errors.push(RepackError::from_obj(RepackErrorKind::NoTableName, self));
            }
            if self.fields.is_empty() {
                errors.push(RepackError::from_obj(RepackErrorKind::NoFields, self));
            }
        } else if self.object_type == ObjectType::Struct {
            if self.inherits.is_some() {
                errors.push(RepackError::from_obj(RepackErrorKind::CannotInherit, self));
            }
            if self.reuse_all {
                errors.push(RepackError::from_obj(RepackErrorKind::CannotReuse, self));
            }
            if !self.reuse_exclude.is_empty() {
                errors.push(RepackError::from_obj(RepackErrorKind::CannotReuse, self));
            }
            if self.table_name.is_some() {
                errors.push(RepackError::from_obj(
                    RepackErrorKind::TableNameNotAllowed,
                    self,
                ));
            }
        }
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
            match &field.location.reference {
                FieldReferenceKind::FieldType(foreign_obj) => {
                    dependencies.insert(foreign_obj.to_string());
                }
                FieldReferenceKind::ImplicitJoin(join_name) => {
                    let Some(ref_field) = self.fields.iter().find(|field| field.name == *join_name)
                    else {
                        continue;
                    };
                    let FieldReferenceKind::FieldType(foreign_obj) = &ref_field.location.reference
                    else {
                        continue;
                    };
                    dependencies.insert(foreign_obj.to_string());
                }
                FieldReferenceKind::ExplicitJoin(join_name) => {
                    let Some(entity) = self.joins.iter().find(|x| x.join_name == *join_name) else {
                        continue;
                    };
                    dependencies.insert(entity.foreign_entity.to_string());
                }
                _ => {
                    continue;
                }
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
