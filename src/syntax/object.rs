use std::collections::HashSet;

use super::{
    Field, FieldType, FileContents, ParseResult, RepackError, RepackErrorKind, Token,
    field::FieldReferenceKind,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    Record,
    Struct,
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
    pub use_snippets: Vec<String>,
}
impl Object {
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
        let mut use_snippets = Vec::new();

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

        'cmd: while let Some(token) = contents.next() {
            match token {
                Token::CloseBrace => {
                    break 'cmd;
                }
                Token::Literal(lit) => {
                    if let Some(field) = Field::from_contents(lit.to_string(), contents) {
                        fields.push(field);
                    }
                }
                Token::Star => {
                    reuse_all = true;
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
            categories,
            use_snippets,
        }
    }

    pub fn table(&self) -> &String {
        self.table_name.as_ref().unwrap()
    }

    pub fn errors(&self, result: &ParseResult) -> Option<Vec<RepackError>> {
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
                if let FieldType::Custom(_) = field_type {
                    errors.push(RepackError::from_field(
                        RepackErrorKind::CustomTypeNotAllowed,
                        self,
                        field,
                    ));
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
        for field in &self.fields {
            let Some(field_type) = &field.field_type else {
                errors.push(RepackError::from_field(
                    RepackErrorKind::TypeNotResolved,
                    self,
                    field,
                ));
                continue;
            };
            if let FieldType::Custom(object_name) = field_type {
                if !result.objects.iter().any(|o| o.name == *object_name) {
                    errors.push(RepackError::from_field_with_msg(
                        RepackErrorKind::CustomTypeNotDefined,
                        self,
                        field,
                        object_name.to_string(),
                    ));
                }
            }
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
                FieldReferenceKind::JoinData(join_name) => {
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
        // for join in &self.joins {
        //     if !dependencies.contains(&join.object_name) {
        //         dependencies.push(join.object_name.clone());
        //     }
        // }
        dependencies.into_iter().collect()
    }
}
