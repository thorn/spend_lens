pub mod settings;

use settings::Settings;
pub use config::{Config, ConfigError};

pub fn config() -> Result<Config, ConfigError> {
  Settings::new()
}