pub use sea_orm_migration::prelude::*;

mod m20230525_114533_add_users;
mod m20230525_114748_add_bill_parse_requests;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230525_114533_add_users::Migration),
            Box::new(m20230525_114748_add_bill_parse_requests::Migration),
        ]
    }
}
