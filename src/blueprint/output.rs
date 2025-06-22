use std::path::PathBuf;

use crate::{
    blueprint::{BlueprintArrayVariable, BlueprintFieldVariable, BlueprintRecordVariable},
    syntax::{Enum, FieldType, Object, ObjectType, Output, ParseResult},
};

use super::{
    Blueprint, BlueprintCaseVariable, BlueprintEnumVariable, BlueprintError,
    BlueprintGlobalVariable, TemplateDefineSection,
};

const BLUEPRINT_VERSION: &'static str = "v1.0.0";

pub struct BlueprintBuilder<'a> {
    pub blueprint: &'a Blueprint,
    pub config: &'a Output,
    pub objects: Vec<&'a Object>,
    pub enums: Vec<&'a Enum>,
    pub path: PathBuf,
}
impl<'a> BlueprintBuilder<'a> {
    pub fn from_result(
        result: &'a ParseResult,
        config: &'a Output,
        blueprint: &'a Blueprint,
    ) -> BlueprintBuilder<'a> {
        let objects = result
            .objects
            .iter()
            .filter(|x| {
                if config.categories.is_empty() {
                    return true;
                }
                x.categories.iter().any(|y| config.categories.contains(y))
            })
            .collect::<Vec<_>>();
        let enums = result
            .enums
            .iter()
            .filter(|x| {
                if config.categories.is_empty() {
                    return true;
                }
                x.categories.iter().any(|y| config.categories.contains(y))
            })
            .collect::<Vec<_>>();

        let mut path = std::env::current_dir().unwrap();
        if let Some(loc) = &config.location {
            path.push(loc);
        }

        BlueprintBuilder {
            blueprint,
            config,
            objects,
            enums,
            path,
        }
    }

    pub fn build(self) -> Result<(), BlueprintError> {
        // let mut imports: Vec<String> = Vec::new();
        let mut output = String::new();
        for e in self.enums {
            let Some(case_template) = self.blueprint.sections.get(&TemplateDefineSection::Case)
            else {
                return Err(BlueprintError::EnumsNotSupported);
            };
            let Some(case_split_template) = self
                .blueprint
                .sections
                .get(&TemplateDefineSection::CaseSplitter)
            else {
                return Err(BlueprintError::EnumsNotSupported);
            };
            let Some(base_template) = self
                .blueprint
                .sections
                .get(&TemplateDefineSection::EnumBase)
            else {
                return Err(BlueprintError::EnumsNotSupported);
            };

            let mut cases = String::new();
            for (idx, c) in e.options.iter().enumerate() {
                for token in case_template {
                    cases.push_str(token.render(|var| match var.as_case() {
                        BlueprintCaseVariable::Name => c,
                        BlueprintCaseVariable::Value => c,
                    }));
                }
                if idx + 1 < e.options.len() {
                    for token in case_split_template {
                        cases.push_str(token.render(|var| match var.as_global() {
                            BlueprintGlobalVariable::Version => BLUEPRINT_VERSION,
                        }));
                    }
                    cases.push('\n');
                }
            }

            for token in base_template {
                output.push_str(token.render(|var| match var.as_enum() {
                    BlueprintEnumVariable::Name => &e.name,
                    BlueprintEnumVariable::Cases => &cases,
                }));
            }
            output.push_str("\n\n");
        }

        for obj in self.objects {
            match obj.object_type {
                ObjectType::Record => {
                    let Some(field_template) =
                        self.blueprint.sections.get(&TemplateDefineSection::Field)
                    else {
                        return Err(BlueprintError::RecordsNotSupported);
                    };
                    let Some(field_optional_template) = self
                        .blueprint
                        .sections
                        .get(&TemplateDefineSection::FieldOptional)
                    else {
                        return Err(BlueprintError::RecordsNotSupported);
                    };
                    let Some(fields_split_template) = self
                        .blueprint
                        .sections
                        .get(&TemplateDefineSection::FieldSplitter)
                    else {
                        return Err(BlueprintError::RecordsNotSupported);
                    };
                    let Some(base_template) = self
                        .blueprint
                        .sections
                        .get(&TemplateDefineSection::RecordBase)
                    else {
                        return Err(BlueprintError::RecordsNotSupported);
                    };

                    let mut fields = String::new();
                    for (idx, c) in obj.fields.iter().enumerate() {
                        let type_raw = match c.field_type() {
                            FieldType::Core(core_type) => {
                                self.blueprint.types.get(core_type).ok_or_else(|| {
                                    BlueprintError::TypeNotSupported(core_type.to_string())
                                })?
                            }
                            FieldType::Custom(x, _) => x,
                        };
                        let mut new_type = String::new();
                        if c.array {
                            let Some(array_template) = &self.blueprint.array else {
                                return Err(BlueprintError::ArraysNotSupported)
                            };
                            for token in array_template {
                                new_type.push_str(token.render(|var| match var.as_array() {
                                    BlueprintArrayVariable::Type => &type_raw
                                }));
                            }
                        } else {
                            new_type = type_raw.to_string();
                        }

                        if c.optional {
                            for token in field_optional_template {
                                fields.push_str(token.render(|var| match var.as_field() {
                                    BlueprintFieldVariable::Name => &c.name,
                                    BlueprintFieldVariable::Type => &new_type,
                                }));
                            }
                        } else {
                            for token in field_template {
                                fields.push_str(token.render(|var| match var.as_field() {
                                    BlueprintFieldVariable::Name => &c.name,
                                    BlueprintFieldVariable::Type => &new_type,
                                }));
                            }
                        }
                        if idx + 1 < obj.fields.len() {
                            for token in fields_split_template {
                                fields.push_str(token.render(|var| match var.as_global() {
                                    BlueprintGlobalVariable::Version => BLUEPRINT_VERSION,
                                }));
                            }
                            fields.push('\n');
                        }
                    }

                    for token in base_template {
                        output.push_str(token.render(|var| match var.as_record() {
                            BlueprintRecordVariable::Name => &obj.name,
                            BlueprintRecordVariable::Fields => &fields,
                            BlueprintRecordVariable::Table => &obj.table(),
                        }));
                    }
                }
                ObjectType::Struct => {}
            }
            output.push_str("\n\n");
        }

        println!("{}", output);

        return Ok(());
    }
}
