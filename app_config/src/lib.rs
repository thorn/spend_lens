pub mod settings;

pub use settings::Settings;
pub use config::{Config, ConfigError};

pub fn build_config() -> Result<Config, ConfigError> {
  Settings::build()
}