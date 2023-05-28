use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(User::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(User::Id)
              .integer()
              .not_null()
              .auto_increment()
              .primary_key(),
          )
          .col(ColumnDef::new(User::FirstName).string().not_null())
          .col(ColumnDef::new(User::LastName).string().not_null())

          // Timestamps
          .col(ColumnDef::new(User::DeletedAt).date_time().not_null())
          .col(ColumnDef::new(User::UpdatedAt).date_time().not_null())
          .col(ColumnDef::new(User::CreatedAt).date_time().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(User::Table).to_owned())
      .await
  }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum User {
  #[iden="users"]
  Table,
  Id,
  FirstName,
  LastName,
  CreatedAt,
  UpdatedAt,
  DeletedAt,
}
