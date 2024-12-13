async fn create_session(
    account_id: Uuid,
    device_name: Option<String>,
    ip_address: Option<String>,
    secret: &str,
) -> Result<Session, ApiError> {
    let session_id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(30); // Сессия на 30 дней

    // Создаем JWT токен
    let token = create_jwt(&session_id.to_string(), secret);

    let session = Session {
        id: session_id,
        account_id,
        device_name,
        ip_address,
        created_at: now,
        expires_at,
        token: token.clone(),
    };

    // TODO: Сохранить сессию в базе данных
    save_session_to_db(&session).await?;

    Ok(session)
}
