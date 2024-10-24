mod jobs;

use anyhow::Result;
use apalis::prelude::*;
use apalis::{layers::TraceLayer, postgres::PostgresStorage};
use jobs::{send_email, Email};
use std::time::Duration;

async fn produce_jobs(storage: &PostgresStorage<Email>) -> Result<()> {
  // The programmatic way
  let mut storage = storage.clone();
  for index in 0..1000 {
    storage
      .push(Email {
        to: format!("test{}@example.com", index),
        text: "Test background job from apalis".to_string(),
        subject: "Background email job".to_string(),
      })
      .await?;
  }
  // The sql way
  tracing::info!("You can also add jobs via sql query, run this: \n Select apalis.push_job('apalis::Email', json_build_object('subject', 'Test apalis', 'to', 'test1@example.com', 'text', 'Lorem Ipsum'));");
  Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
  std::env::set_var("RUST_LOG", "debug,sqlx::query=error");
  tracing_subscriber::fmt::init();
  let database_url = std::env::var("DATABASE_URL").expect("Must specify path to db");

  let pg = PostgresStorage::connect(database_url).await?;
  pg.setup()
    .await
    .expect("unable to run migrations for postgres");

  produce_jobs(&pg).await?;

  Monitor::new()
    .register_with_count(5, move |c| {
      WorkerBuilder::new(format!("tasty-orange-{c}"))
        .layer(TraceLayer::new())
        .with_storage_config(pg.clone(), |cfg| {
          cfg
            // Set the buffer size to 100 ( Pick 100 jobs per query)
            .buffer_size(100)
            // Lower the fetch interval because postgres is waiting for notifications
            .fetch_interval(Duration::from_secs(1))
        })
        .build_fn(send_email)
    })
    .run()
    .await?;
  Ok(())
}
