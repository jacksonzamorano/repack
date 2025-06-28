use std::collections::HashSet;

use super::{
    CustomFieldType, Field, FieldType, FileContents, ObjectFunction, RepackError, RepackErrorKind,
    Token, field::FieldReferenceKind,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    Record,
    Synthetic,
    Struct,
}

#[derive(Debug, Clone)]
pub struct ObjectJoin {
    pub join_name: String,
    pub local_field: String,
    pub condition: String,
    pub foreign_entity: String,
    pub foreign_field: String,
}

#[derive(Debug)]
pub struct Object {
    pub object_type: ObjectType,
    pub name: String,
    pub fields: Vec<Field>,
    pub inherits: Option<String>,
    pub categories: Vec<String>,
    pub table_name: Option<String>,
    pub reuse_all: bool,
    pub reuse_exclude: Vec<String>,
    pub reuse_include: Vec<String>,
    pub use_snippets: Vec<String>,
    pub functions: Vec<ObjectFunction>,
    pub joins: Vec<ObjectJoin>,
}
impl Object {
    pub fn read_from_contents(typ: ObjectType, contents: &mut FileContents) -> Object {
        let mut object_type = typ;
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
                    if matches!(object_type, ObjectType::Record) {
                        inherits = match contents.next() {
                            Some(Token::Literal(lit)) => Some(lit.to_string()),
                            _ => None,
                        };
                    }
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

        if inherits.is_some() {
            object_type = ObjectType::Synthetic;
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
                        });
                    } else if obj_2_name == "self" {
                        joins.push(ObjectJoin {
                            join_name,
                            local_field: obj_2_field,
                            condition: "=".to_string(),
                            foreign_entity: obj_1_name,
                            foreign_field: obj_1_field,
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
            object_type,
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

    pub fn errors(&self) -> Option<Vec<RepackError>> {
        let mut errors = Vec::new();
        if self.object_type == ObjectType::Record {
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
                _ => {
                    continue;
                }
            }
        }
        dependencies.into_iter().collect()
    }

    pub fn functions_in_namespace(&self, ns: &str) -> Vec<&ObjectFunction> {
        self.functions
            .iter()
            .filter(|x| x.namespace == *ns)
            .collect()
    }
}
