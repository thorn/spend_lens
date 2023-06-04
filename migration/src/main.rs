extern crate core;

use async_std::task::block_on;
use sea_orm_migration::prelude::*;
use sea_orm::{ConnectionTrait, Database, DbBackend, DbErr, Statement};
use app_config::{Config, ConfigError};
use app_config::settings::DBEnvConfig;
use migration::{Migrator, MigratorTrait};
use migration::sea_orm::DatabaseConnection;

use clap::{Parser, Subcommand};

async fn setup_database(database_config: DBEnvConfig, debug: Option<bool>) -> Result<DatabaseConnection, DbErr> {
  let debug = debug.unwrap_or(false);

  let db_url = database_config.url;
  let db_name = database_config.database;
  let db = Database::connect(&db_url).await?;

  if debug { println!("Recreating the database if needed...") }
  let db = match db.get_database_backend() {
    DbBackend::MySql => {
      db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("CREATE DATABASE IF NOT EXISTS `{}`;", &db_name),
      ))
        .await?;

      let url = format!("{}/{}", &db_url, &db_name);
      Database::connect(&url).await?
    }
    DbBackend::Postgres => {
      db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("DROP DATABASE IF EXISTS \"{}\";", &db_name),
      ))
        .await?;
      db.execute(Statement::from_string(
        db.get_database_backend(),
        format!("CREATE DATABASE \"{}\";", &db_name),
      ))
        .await?;

      let url = format!("{}/{}", &db_url, &db_name);
      Database::connect(&url).await?
    }
    DbBackend::Sqlite => db,
  };
  if debug { println!("Database was recreated...")}

  return Ok(db);
}

fn database_config(app_config: Config) -> Result<DBEnvConfig, ConfigError> {
  let env = app_config.get::<String>("env").unwrap_or_else(|err| {
    panic!("Error retrieving environment from config: {:?}", err)
  });

  let key = format!("database.{}", &env);
  app_config.get::<DBEnvConfig>(&key)
}

fn printable_amount(amount: &Option<u32>) -> String {
  match amount {
    Some(amount) => { amount.to_string() }
    None => { String::from("all") }
  }
}

async fn apply_pending_migrations(db: &DatabaseConnection, amount: &Option<u32>, debug: bool) {
  if debug { println!("Running {:} pending migrations...", printable_amount(amount)) }
  Migrator::up(db, *amount).await.expect("Failed to apply pending migrations!")
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  /// Apply pending migrations
  Migrate {
    /// How many pending migrations are run
    amount: Option<u32>
  },
  /// Rollback migrations
  Rollback {
    /// How many migrations have to be rolled back
    amount: Option<u32>
  },
  /// Drop all tables from the database, then reapply all migrations
  Fresh,
  /// Rollback all applied migrations, then reapply all migrations
  Refresh,
  /// Show migration status
  Status,
  /// Rollback all applied migrations
  Reset,
  /// Drop database and then create it
  ResetDb,
  /// Create database unless it already exists
  SetupDb,
  /// Drop database
  DropDb,
}

#[async_std::main]
async fn main() {
  let args = Cli::parse();

  let config = app_config::build_config().unwrap_or_else(|err| {
    panic!("Error retrieving app config: {:?}", err)
  });

  let debug = config.get_bool("debug").unwrap_or(false);
  let database_config = database_config(config)
    .unwrap_or_else(|err| panic!("Error loading database config: {:?}", err));

  let db = block_on(setup_database(database_config, Some(debug)))
    .unwrap_or_else(|err| panic!("Failed to establish database connection: {:?}", err));

  match &args.command {
    Some(Commands::Migrate { amount }) => {
      apply_pending_migrations(&db, amount, debug).await
    }
    Some(Commands::Rollback { amount }) => {
      if debug { println!("Rolling back {:} pending migrations...", printable_amount(amount)) }
      Migrator::down(&db, *amount).await.expect("Failed to rollback migrations!")
    }
    Some(Commands::Fresh) => {
      if debug { println!("Dropping all tables from the database, then reapplying all migrations")}
      Migrator::fresh(&db).await.expect("Failed to make the database fresh!")
    }
    Some(Commands::Refresh) => {
      if debug { println!("Rollback all applied migrations, then reapply all migrations")}
      Migrator::refresh(&db).await.expect("Failed to refresh the database!")
    }
    Some(Commands::Status) => {
      if debug { println!("Showing migration status")}
      Migrator::status(&db).await.expect("Failed to get the database status!")
    }
    Some(Commands::Reset) => {
      if debug { println!("Rolling back all applied migrations")}
      Migrator::status(&db).await.expect("Failed to rollback all applied migrations!")
    }
    Some(Commands::ResetDb) => {
      if debug { println!("Dropping the database and creating it again.")}
      // TODO: implement this
      Migrator::status(&db).await.expect("Failed to reset the database!")
    }
    Some(Commands::SetupDb) => {
      // TODO: Implement this
      println!("'myapp add' was used, name is:")
    }
    Some(Commands::DropDb) => {
      // TODO: Implement this
      println!("'myapp add' was used, name is:")
    }
    None => {
      apply_pending_migrations(&db, &None, debug).await
    }
  }

  if debug { println!("Closing the database connection...")}
  db.close().await.expect("Unable to close db connection!");
}
