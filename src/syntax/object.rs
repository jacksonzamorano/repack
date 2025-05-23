use super::{
    Field, FieldCommand, FieldType, FieldValidationError, FieldValidationErrorType, FileContents,
    ObjectValidationError, ObjectValidationErrorType, ParseResult, Token, ValidationError,
};

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectType {
    Record,
    Struct,
}

#[derive(Debug)]
pub struct ObjectJoin {
    pub object_name: String,
    pub join_name: String,
    pub local_field: String,
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
    pub joins: Vec<ObjectJoin>,
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
        let mut joins: Vec<ObjectJoin> = Vec::new();

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
                Token::Plus => {
                    let Some(Token::Literal(val)) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Period) = contents.next() else {
                        continue;
                    };
                    let Some(Token::Literal(field)) = contents.take() else {
                        continue;
                    };
                    let mut alias = None;
                    if let Some(Token::As) = contents.next() {
                        if let Some(Token::Literal(lit)) = contents.take() {
                            alias = Some(lit.to_string());
                        }
                    }
                    let field = Field::from_join(val, field, alias);
                    fields.push(field);
                }
                Token::Ampersand => {
                    let Some(Token::Literal(foreign_object)) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Ampersand) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Literal(join_name)) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Where) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Literal(local_field)) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Equals) = contents.take() else {
                        continue;
                    };
                    let Some(Token::Literal(foreign_field)) = contents.take() else {
                        continue;
                    };
                    joins.push(ObjectJoin {
                        object_name: foreign_object,
                        join_name: join_name,
                        local_field: local_field,
                        foreign_field: foreign_field,
                    });
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
            joins,
        }
    }

    pub fn table(&self) -> &String {
        self.table_name.as_ref().unwrap()
    }

    fn field_error(&self, error: FieldValidationErrorType, field: &Field) -> ValidationError {
        ValidationError::Field(FieldValidationError::new(error, self, field))
    }

    fn object_error(&self, error: ObjectValidationErrorType) -> ValidationError {
        ValidationError::Object(ObjectValidationError::new(error, self))
    }

    pub fn errors(&self, result: &ParseResult) -> Option<Vec<ValidationError>> {
        let mut errors = Vec::new();
        if self.object_type == ObjectType::Record {
            for field in &self.fields {
                if let FieldType::Custom(_) = &field.field_type {
                    errors
                        .push(self.field_error(FieldValidationErrorType::CustomNotAllowed, field));
                }
                if field.optional && field.commands.contains(&FieldCommand::PrimaryKey) {
                    errors.push(
                        self.field_error(FieldValidationErrorType::PrimaryKeyOptional, field),
                    );
                }
                if field.commands.contains(&FieldCommand::Many) {
                    errors.push(self.field_error(FieldValidationErrorType::ManyNotAllowed, field));
                }
            }
            if self.table_name.is_none() {
                errors.push(self.object_error(ObjectValidationErrorType::TableNameRequired));
            }
            if self.fields.is_empty() {
                errors.push(self.object_error(ObjectValidationErrorType::NoFields));
            }
        } else if self.object_type == ObjectType::Struct {
            if self.inherits.is_some() {
                errors.push(self.object_error(ObjectValidationErrorType::CannotInherit));
            }
            if self.reuse_all {
                errors.push(self.object_error(ObjectValidationErrorType::CannotReuse));
            }
            if !self.reuse_exclude.is_empty() {
                errors.push(self.object_error(ObjectValidationErrorType::CannotReuse));
            }
            if self.table_name.is_some() {
                errors.push(self.object_error(ObjectValidationErrorType::TableNameNotAllowed));
            }
        }
        for field in &self.fields {
            match &field.field_type {
                FieldType::Custom(object_name) => {
                    if !result.objects.iter().any(|o| o.name == *object_name) {
                        errors.push(
                            self.field_error(FieldValidationErrorType::CustomTypeNotFound, field),
                        );
                    }
                }
                _ => {}
            }
        }
        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }

    pub fn depends_on(&self) -> Vec<String> {
        let mut dependencies = Vec::new();
        for field in &self.fields {
            if let Some(reference) = &field.reference {
                if !dependencies.contains(&reference.object_name) {
                    dependencies.push(reference.object_name.clone());
                }
            }
        }
        for join in &self.joins {
            if !dependencies.contains(&join.object_name) {
                dependencies.push(join.object_name.clone());
            }
        }
        dependencies
    }
}
