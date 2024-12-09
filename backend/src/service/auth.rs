use actix_web::{http::StatusCode, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use jwt_simple::prelude::*;
use sea_orm::{prelude::*, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use uuid::Uuid;

use ::entity::{account, prelude::*}; // Предполагается, что `entity` - это модуль с ORM-сущностями.

#[derive(Debug, Clone, Deserialize)]
pub struct AccountRegistrationForm {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountLoginForm {
    pub email: String,
    pub password: String,
}

// Функция для обработки логина
pub async fn handle_login(
    db: &DatabaseConnection,
    form: &AccountLoginForm,
) -> Result<String, &'static str> {
    let user = Account::find()
        .filter(account::Column::Email.eq(form.email.clone()))
        .one(db)
        .await
        .map_err(|_| "Database error")?;

    if let Some(user) = user {
        if verify(&form.password, &user.password).unwrap_or(false) {
            Ok(generate_jwt(&user.id))
        } else {
            Err("Invalid password")
        }
    } else {
        Err("User not found")
    }
}

// Функция для обработки регистрации
pub async fn handle_registration(
    db: &DatabaseConnection,
    form: &AccountRegistrationForm,
) -> Result<(), &'static str> {
    let password_hash = hash(&form.password, DEFAULT_COST)
        .map_err(|_| "Failed to hash password")?;

    let new_user = account::ActiveModel {
        id: Set(Uuid::now_v7()),
        username: Set(form.username.clone()),
        email: Set(form.email.clone()),
        password: Set(password_hash),
        date_of_registration: Set(Utc::now().naive_utc()),
        time_of_last_online: Set(Utc::now().naive_utc()),
    };

    Account::insert(new_user)
        .exec(db)
        .await
        .map(|user| println!("{:#?}", user))
        .map_err(|_| "Failed to insert user")
}

// Функция для генерации JWT
fn generate_jwt(user_id: &Uuid) -> String {
    let key = HS256Key::generate();
    let claims = Claims::create(Duration::from_hours(24))
        .with_subject(user_id.to_string());
    key.authenticate(claims).expect("Failed to generate token")
}

