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
                    .col(binary(Account::Password))
                    .col(timestamp(Account::DateOfRegistration))
                    .col(timestamp(Account::TimeOfLastOnline))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(pk_uuid(Session::Id))
                    .col(uuid_uniq(Session::AccountId))
                    .col(string(Session::DeviceName))
                    .col(string(Session::IP))
                    .col(timestamp(Session::CreatedAt))
                    .col(timestamp(Session::ExpiresAt))
                    .col(string(Session::Token))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await?;
        Ok(())
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

#[derive(DeriveIden)]
enum Session {
    Table,
    Id,
    AccountId,
    DeviceName,
    IP,
    CreatedAt,
    ExpiresAt,
    Token,
}
