use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(pk_uuid(Account::Id))
                    .col(string_len_uniq(Account::Username, 128))
                    .col(string_uniq(Account::Email))
                    .col(string(Account::Password))
                    .col(timestamp(Account::DateOfRegistration))
                    .col(timestamp(Account::TimeOfLastOnline))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
    Username,
    Email,
    Password,
    DateOfRegistration,
    TimeOfLastOnline,
}
