//! Support for internationalization.
//!
//! mau uses [Project Fluent](https://projectfluent.org/) for internationalization support.
//! The architecture consists of

pub use mau_i18n::*;

use crate::LanguageError;

/// Initialization function for language maps.
///
/// This has to be a separate trait to make `LanguageMap` object-safe.
pub trait LanguageMapInit {
    /// Initializes the language map.
    ///
    /// Note how this function **must not fail**; a loaded language map should at least have `en-US`
    /// available for error message translations. Failures during loading should not abort the
    /// whole process (Fluent is very lenient when it comes to errors and will load partially broken
    /// files.)
    fn new() -> Self;
}

/// Mapping of language IDs to FTL translation files.
pub trait LanguageMap {
    /// Returns the FTL source code for the language with the given locale code.
    fn get(&self, code: &str) -> Option<&str>;

    /// Loads the language with the given locale code.
    fn load_language(&self, code: &str) -> Result<Language, LanguageError> {
        if let Some(ftl_source) = self.get(code) {
            let language = Language::load(code, ftl_source);
            let language = match language {
                Ok(language) => language,
                Err(error) => {
                    log::error!("error while loading language:");
                    log::error!("{}", error);
                    return Err(LanguageError::InvalidFTL(code.to_string()));
                }
            };
            Ok(language)
        } else {
            Err(LanguageError::NoTranslations(code.to_string()))
        }
    }
}

/// The empty tuple can be used as a language map for testing purposes.
impl LanguageMapInit for () {
    fn new() -> Self {}
}

/// The empty tuple can be used as a language map for testing purposes. For each language it will
/// return an empty FTL file.
impl LanguageMap for () {
    fn get(&self, _code: &str) -> Option<&str> {
        Some("")
    }
}
