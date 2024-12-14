use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;

pub fn create_jwt(sub: &str, secret: &str) -> String {
    let claims = Claims {
        sub: sub.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}

#[derive(Debug, Serialize)]
struct Claims {
    sub: String,
}
