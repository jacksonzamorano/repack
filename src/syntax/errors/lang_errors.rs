use crate::syntax::Output;

pub enum LanguageValidationErrorType {
    UnknownLanguage = 1,
}

impl LanguageValidationError {
    pub fn message(self) -> String {
        let err = match self.error_type {
            LanguageValidationErrorType::UnknownLanguage => {
                "Unknown language specified.".to_string()
            }
        };

        format!("[LE{:04}] {}: {}", self.error_type as u8, self.language_name, err)
    }
}

pub struct LanguageValidationError {
    error_type: LanguageValidationErrorType,
    language_name: String,
}
impl LanguageValidationError {
    pub fn new(error_type: LanguageValidationErrorType, language: &Output) -> Self {
        Self {
            error_type,
            language_name: language.profile.clone(),
        }
    }
}