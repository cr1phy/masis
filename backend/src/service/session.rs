use chrono::Utc;
use entity::{prelude::Session, session};
use sea_orm::{DbConn, EntityTrait, Set};
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiErrorKind},
    utils::jwt::create_jwt,
};

pub async fn create_session(
    account_id: Uuid,
    device_name: String,
    ip_address: String,
    secret: &str,
    conn: &DbConn,
) -> Result<String, ApiError> {
    let session_id = Uuid::now_v7();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(365);

    let token = create_jwt(&session_id.to_string(), secret);

    let session = session::ActiveModel {
        id: Set(session_id),
        account_id: Set(account_id),
        device_name: Set(device_name),
        ip: Set(ip_address),
        created_at: Set(now.naive_utc()),
        expires_at: Set(expires_at.naive_utc()),
        token: Set(token.clone()),
    };

    Session::insert(session)
        .exec(conn)
        .await
        .map(|_| token)
        .map_err(|_| ApiError::new(ApiErrorKind::InternalServerError))
}
