use config::{Config, ConfigError, Environment, File};
use serde_derive::Deserialize;
use std::env;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct DBEnvConfig {
  pub url: String,
  pub database: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Database {
  development: DBEnvConfig,
  production: DBEnvConfig,
  testing: DBEnvConfig,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
  debug: bool,
  database: Database,
}

impl Settings {
  pub fn build() -> Result<Config, ConfigError> {
    let env_name = env::var("ENV").unwrap_or_else(|_| "development".into());
    let config_dir = Path::new(
      &env::current_exe().expect("Cannot query current directory").parent().expect("Fail!")
    ).join("config");

    // This tmp madness is caused by https://github.com/rust-lang/rust/issues/15023
    let tmp = config_dir.join("default.yml");
    let default_config_path = File::with_name(tmp.to_str().expect("default config failed"));

    let env_filename = config_dir.join(format!("{}.yml", &env_name));
    let env_config_file_path = File::with_name(env_filename.to_str().expect("hello"));

    let tmp = config_dir.join("database.yml");
    let database_config = File::with_name(tmp.to_str().expect(""));

    let tmp = config_dir.join("local.yml");
    let local_config_file = File::with_name(tmp.to_str().expect(""));

    Config::builder()
      .add_source(default_config_path)
      .add_source(env_config_file_path.required(true))
      .add_source(database_config.required(true))
      // Add in a local configuration file
      // This file shouldn't be checked in to git
      .add_source(local_config_file.required(false))

      // // You may also programmatically change settings
      // .set_override("database.url", "postgres://")?
      .set_override("env", String::from(&env_name))?
      // Add in settings from the environment (with a prefix of APP)
      // Eg.. `SPEND_LENS_DEBUG=1 ./target/app` would set the `debug` key
      .add_source(Environment::with_prefix("spend_lens"))

      .build()

    // You can deserialize (and thus freeze) the entire configuration as
    // s.try_deserialize()
  }
}
