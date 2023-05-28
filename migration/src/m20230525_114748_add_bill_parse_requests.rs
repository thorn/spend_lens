use sea_orm_migration::prelude::*;
use super::m20230525_114533_add_users::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(BillParseRequest::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(BillParseRequest::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(ColumnDef::new(BillParseRequest::Url).string().not_null())
          .col(ColumnDef::new(BillParseRequest::Status).string().not_null())
          .col(ColumnDef::new(BillParseRequest::ProcessedAt).date_time().not_null())
          .col(ColumnDef::new(BillParseRequest::FailedAt).date_time().not_null())
          .col(ColumnDef::new(BillParseRequest::UserId).integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-parse_requests-user_id")
              .from(BillParseRequest::Table, BillParseRequest::UserId)
              .to(User::Table, User::Id),
          )

          // Timestamps
          .col(ColumnDef::new(BillParseRequest::UpdatedAt).date_time().not_null())
          .col(ColumnDef::new(BillParseRequest::CreatedAt).date_time().not_null())
          .to_owned(),
      )
      .await?;

    manager.create_index(
      Index::create()
        .name("idx-bill_parse_request-user_id")
        .table(BillParseRequest::Table)
        .col(BillParseRequest::UserId)
        .to_owned()
    ).await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(BillParseRequest::Table).to_owned())
      .await
  }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum BillParseRequest {
  #[iden = "bill_parse_requests"]
  Table,
  Id,
  Url,
  UserId,
  Status,
  ProcessedAt,
  FailedAt,
  UpdatedAt,
  CreatedAt,
}
