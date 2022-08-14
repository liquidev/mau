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
}

/// Defines a module with a thread-local config variable that automatically saves the config file
/// upon write.
///
/// After the module is declared by this macro, all of its inner symbols should be reexported into
/// the current scope if you wish for the module to be public, like so:
/// ```
/// mau::config_module!(MyConfig, tls);
/// pub use tls::*;
/// ```
#[macro_export]
macro_rules! config_module {
    ($T:tt, $modname:tt) => {
        mod $modname {
            use once_cell::sync::OnceCell;
            use std::sync::RwLock;
            use std::sync::RwLockReadGuard;

            use $crate::config::AppConfig;

            use super::$T;

            static CONFIG: OnceCell<RwLock<$T>> = OnceCell::new();

            /// Loads or creates the user config.
            pub fn load_or_create() -> Result<(), $crate::ConfigError> {
                let config = <$T as $crate::config::AppConfig>::load_or_create()?;
                if CONFIG.set(RwLock::new(config)).is_err() {
                    return Err($crate::ConfigError::ConfigIsAlreadyLoaded);
                }
                Ok(())
            }

            /// Saves the user config.
            pub fn save() -> Result<(), $crate::ConfigError> {
                <$T as $crate::config::AppConfig>::save(&config())
            }

            /// Reads from the user config.
            pub fn config() -> RwLockReadGuard<'static, $T> {
                CONFIG
                    .get()
                    .expect("attempt to read config without loading it")
                    .read()
                    .unwrap()
            }

            /// Writes to the user config. After the closure is done running, saves the user config to the disk.
            pub fn write(f: impl FnOnce(&mut $T)) {
                {
                    let mut config = CONFIG
                        .get()
                        .expect("attempt to write config without loading it")
                        .write()
                        .unwrap();
                    f(&mut config);
                }
                match save() {
                    Ok(_) => (),
                    Err(error) => {
                        log::error!("cannot save config: {}", error);
                    }
                }
            }
        }
    };
}

#[allow(unused)]
mod test {
    use super::*;

    #[derive(Deserialize, Serialize)]
    pub struct MyConfig {
        language: String,
        window: Option<WindowConfig>,
    }

    impl Default for MyConfig {
        fn default() -> Self {
            Self {
                language: "en-US".to_string(),
                window: None,
            }
        }
    }

    impl AppConfig for MyConfig {
        fn app_name() -> &'static str {
            "MyApp"
        }

        fn language(&self) -> &str {
            &self.language
        }

        fn window_config(&self) -> &Option<WindowConfig> {
            &self.window
        }
    }

    config_module!(MyConfig, tls);
    use tls::*;
}
