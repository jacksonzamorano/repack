use crate::{
    blueprint::{
        BlueprintContext, BlueprintToken, TemplateDefineSection, Blueprint, BlueprintError,
        BlueprintFileReader,
    },
    syntax::CoreType,
};

#[derive(Debug, Clone)]
pub enum BlueprintCommand {
    Id,
    Name,
    Type,
    Array,
    Optional,
    Define,
}
impl BlueprintCommand {
    pub fn from_language(val: &str) -> Option<BlueprintCommand> {
        Some(match val {
            "id" => Self::Id,
            "name" => Self::Name,
            "type" => Self::Type,
            "array" => Self::Array,
            "optional" => Self::Optional,
            "define" => Self::Define,
            _ => return None,
        })
    }

    pub fn handle(
        self,
        file: &mut BlueprintFileReader,
        lang: &mut Blueprint,
    ) -> Result<(), BlueprintError> {
        match self {
            Self::Id => {
                lang.id = file
                    .next()
                    .ok_or_else(|| BlueprintError::InvalidCommandSyntax(self))?
            }
            Self::Name => {
                lang.name = file
                    .read_line()
                    .ok_or_else(|| BlueprintError::InvalidCommandSyntax(self))?
            }
            Self::Define => {
                let mut matching_sections: Vec<TemplateDefineSection> = Vec::new();
                let remaining_tokens = file.read_line_tokens();
                let mut temp_string = String::new();
                for token in remaining_tokens {
                    match token {
                        BlueprintToken::Period => {
                            temp_string.push_str(".");
                        }
                        BlueprintToken::LiteralRun(lit) => {
                            temp_string.push_str(&lit);
                        }
                        BlueprintToken::Space => {
                            if let Some(parsed) = TemplateDefineSection::from_language(&temp_string)
                            {
                                matching_sections.push(parsed);
                            }
                            temp_string = String::new();
                        }
                        _ => {}
                    }
                }
                if !temp_string.is_empty() {
                    if let Some(parsed) = TemplateDefineSection::from_language(&temp_string) {
                        matching_sections.push(parsed);
                    }
                }

                if matching_sections.is_empty() {
                    return Err(BlueprintError::NoSections);
                }
                let target_context = matching_sections[0].context();
                for t in &matching_sections {
                    if target_context != t.context() {
                        return Err(BlueprintError::InconsistentContexts);
                    }
                }

                let contents = file.read_block(target_context);
                for sec in matching_sections {
                    lang.sections.insert(sec, contents.clone());
                }
            }
            Self::Type => {
                let source_type = file
                    .next()
                    .and_then(|x| CoreType::from_string(&x))
                    .ok_or_else(|| BlueprintError::InvalidCommandSyntax(self.clone()))?;
                let end_type = file
                    .next()
                    .ok_or_else(|| BlueprintError::InvalidCommandSyntax(self.clone()))?;
                lang.types.insert(source_type, end_type);
            }
            Self::Optional => {
                lang.optional = Some(file.read_line_tokens_with_context(&BlueprintContext::Optional));
            }
            Self::Array => {
                lang.array = Some(file.read_line_tokens_with_context(&BlueprintContext::Array));
            }
        }
        Ok(())
    }
}
