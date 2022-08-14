//! Common configuration options.

use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::ConfigError;

/// Window position and size.
#[derive(Deserialize, Serialize)]
pub struct WindowConfig {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    // NOTE: `maximized` is optional because older versions of NetCanv did not serialize this
    // property, so we need a default to maintain compatibility.
    #[serde(default)]
    pub maximized: bool,
}

/// An application config file.
///
/// mau automatically serializes/deserializes config files from the app directory upon the
/// [`App`][crate::App]'s construction.
pub trait AppConfig: DeserializeOwned + Serialize + Default {
    /// Returns the name of the app.
    ///
    /// This name is used to determine where to save config files.
    fn app_name() -> &'static str;

    /// Returns the language set in the config.
    fn language(&self) -> &str;

    /// Returns the window config.
    fn window_config(&self) -> &Option<WindowConfig>;

    /// Returns a mutable reference to the window config.
    fn window_config_mut(&mut self) -> &mut Option<WindowConfig>;

    /// Returns the path to the application's config directory.
    fn config_dir() -> PathBuf {
        let project_dirs = ProjectDirs::from("", "", Self::app_name())
            .expect("cannot determine the user's home directory");
        project_dirs.config_dir().into()
    }

    /// Returns the path to the `config.toml` file located in the application's config directory.
    fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Loads the `config.toml` file, or creates a fresh one if it doesn't already exist.
    fn load_or_create() -> Result<Self, ConfigError> {
        let config_dir = Self::config_dir();
        let config_file = Self::config_path();
        log::info!("loading config from {:?}", config_file);
        std::fs::create_dir_all(config_dir)?;
        if !config_file.is_file() {
            let config = Self::default();
            config.save()?;
            Ok(config)
        } else {
            let file = std::fs::read_to_string(&config_file)?;
            let config: Self = match toml::from_str(&file) {
                Ok(config) => config,
                Err(error) => {
                    log::error!("error while deserializing config file: {}", error);
                    log::error!("falling back to default config");
                    return Ok(Self::default());
                }
            };
            // Preemptively save the config to the disk if any new keys have been added.
            // I'm not sure if errors should be treated as fatal or not in this case.
            config.save()?;
            Ok(config)
        }
    }

    /// Saves the config file to the application's config directory.
    fn save(&self) -> Result<(), ConfigError> {
        // Assumes that `config_dir` was already created in `load_or_create`.
        let config_file = Self::config_path();
        std::fs::write(&config_file, toml::to_string(self)?)?;
        Ok(())
    }

    /// Writes values to the config and then saves it. This is recommended over using `save()`
    /// manually when you want to modify the config.
    ///
    /// Note that this function, unlike `save()` is infallible. Instead it simply logs the error
    /// on failure.
    fn write(&mut self, f: impl FnOnce(&mut Self)) {
        f(self);
        if let Err(error) = self.save() {
            // TODO: Global error bus
            log::error!("error while saving config: {error}");
        }
    }
}
