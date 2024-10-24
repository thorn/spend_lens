use apalis::prelude::{Job, JobContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Email {
  pub to: String,
  pub subject: String,
  pub text: String,
}

impl Job for Email {
  const NAME: &'static str = "apalis::Email";
}

pub async fn send_email(job: Email, _ctx: JobContext) {
  tracing::info!("Attempting to send email to {}", job.to);
}