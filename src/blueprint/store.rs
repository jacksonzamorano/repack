use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use crate::{
    blueprint::{Blueprint, BlueprintFileReader},
    syntax::{RepackError, RepackErrorKind},
};

/// Embedded core blueprint definitions for built-in language support.
/// 
/// These blueprints are compiled into the binary and provide immediate support
/// for common target languages without requiring external blueprint files.
/// Each blueprint defines the code generation templates and rules for its language.
const CORE_BLUEPRINTS: &[&str] = &[
    include_str!("core/rust.blueprint"),
    include_str!("core/postgres.blueprint"),
    include_str!("core/typescript.blueprint"),
    include_str!("core/go.blueprint"),
    include_str!("core/markdown.blueprint"),
];

/// Central repository for managing and accessing blueprint definitions.
/// 
/// BlueprintStore handles loading, storing, and retrieving blueprints for different
/// target languages. It manages both core built-in blueprints and user-defined
/// external blueprints loaded from files.
pub struct BlueprintStore {
    /// Map of blueprint identifiers to their loaded Blueprint instances
    languages: HashMap<String, Blueprint>,
}
impl BlueprintStore {
    /// Creates a new BlueprintStore with all core blueprints loaded.
    /// 
    /// This constructor initializes the store and loads all embedded core blueprints
    /// (Rust, PostgreSQL, TypeScript, Go) making them immediately available for use.
    /// 
    /// # Returns
    /// * `Ok(BlueprintStore)` if all core blueprints load successfully
    /// * `Err(RepackError)` if any core blueprint fails to parse
    pub fn new() -> Result<BlueprintStore, RepackError> {
        let mut store = BlueprintStore {
            languages: HashMap::new(),
        };

        for core in CORE_BLUEPRINTS {
            store.load_string(core)?
        }

        Ok(store)
    }

    /// Loads a blueprint from an external file and adds it to the store.
    /// 
    /// This method reads a blueprint file from disk, parses it, and adds it to
    /// the available blueprints. The blueprint's ID from the file is used as
    /// the key for later retrieval.
    /// 
    /// # Arguments
    /// * `path` - Path to the blueprint file to load
    /// 
    /// # Returns
    /// * `Ok(())` if the blueprint loads successfully
    /// * `Err(RepackError)` if the file cannot be read or parsed
    pub fn load_file(&mut self, path: &PathBuf) -> Result<(), RepackError> {
        let mut file = File::open(path).map_err(|_| {
            RepackError::global(
                RepackErrorKind::CannotRead,
                path.to_str().unwrap_or("<invalid path>").to_string(),
            )
        })?;
        let mut contents = vec![];
        _ = file.read_to_end(&mut contents);

        let reader = BlueprintFileReader {
            reader: contents.iter().peekable(),
        };

        let lang = Blueprint::new(reader)?;
        self.languages.insert(lang.id.clone(), lang);

        Ok(())
    }

    pub fn load_string(&mut self, contents: &str) -> Result<(), RepackError> {
        let reader = BlueprintFileReader {
            reader: contents.as_bytes().iter().peekable(),
        };
        let lang = Blueprint::new(reader)?;
        self.languages.insert(lang.id.clone(), lang);

        Ok(())
    }

    /// Retrieves a blueprint by its identifier.
    /// 
    /// This method looks up a loaded blueprint by its ID/tag, which is typically
    /// used as the profile name in output configurations.
    /// 
    /// # Arguments
    /// * `tag` - The blueprint identifier to look up
    /// 
    /// # Returns
    /// * `Some(&Blueprint)` if a blueprint with the given ID exists
    /// * `None` if no blueprint with the given ID is found
    pub fn blueprint(&self, tag: &str) -> Option<&Blueprint> {
        self.languages.get(tag)
    }
}
