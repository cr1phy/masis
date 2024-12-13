pub use sea_orm_migration::prelude::*;

mod migrate13122024;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(migrate13122024::Migration)]
    }
}
