use crate::{
    profiles::{
        TemplateDefineSection, TemplatedLanguageReader,
        fly::{TemplatedLanguage, TemplatedLanguageError},
    },
    syntax::CoreType,
};

#[derive(Debug, Clone)]
pub enum TemplateToken {
    Id,
    Name,
    Type,
    Array,
    Optional,
    Define,
}
impl TemplateToken {
    pub fn from_language(val: &str) -> Option<TemplateToken> {
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
        file: &mut TemplatedLanguageReader,
        lang: &mut TemplatedLanguage,
    ) -> Result<(), TemplatedLanguageError> {
        match self {
            Self::Id => {
                lang.id = file
                    .next()
                    .ok_or_else(|| TemplatedLanguageError::InvalidCommandSyntax(self))?
            }
            Self::Name => {
                lang.name = file
                    .read_line()
                    .ok_or_else(|| TemplatedLanguageError::InvalidCommandSyntax(self))?
            }
            Self::Define => {
                let matching_sections = file
                    .read_line_tokens()
                    .into_iter()
                    .filter_map(|x| {
                        dbg!(&x);
                        TemplateDefineSection::from_language(&x.value)
                    })
                    .collect::<Vec<_>>();
                if matching_sections.is_empty() {
                    return Err(TemplatedLanguageError::NoSections);
                }
                let target_context = matching_sections[0].context();
                for t in &matching_sections {
                    if target_context != t.context() {
                        return Err(TemplatedLanguageError::InconsistentContexts);
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
                    .ok_or_else(|| TemplatedLanguageError::InvalidCommandSyntax(self.clone()))?;
                let end_type = file
                    .next()
                    .ok_or_else(|| TemplatedLanguageError::InvalidCommandSyntax(self.clone()))?;
                lang.types.insert(source_type, end_type);
            }
            Self::Optional => {
                lang.optional =
                    Some(file.read_line().ok_or_else(|| {
                        TemplatedLanguageError::InvalidCommandSyntax(self.clone())
                    })?);
            }
            Self::Array => {
                lang.array =
                    Some(file.read_line().ok_or_else(|| {
                        TemplatedLanguageError::InvalidCommandSyntax(self.clone())
                    })?);
            }
        }
        Ok(())
    }
}
