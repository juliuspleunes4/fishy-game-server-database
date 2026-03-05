use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Claims are encoded in the JWT.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: usize, // Expiration time (as a timestamp)
}

pub fn generate_jwt(user_id: Uuid, secret_key: &str) -> Result<String, String> {
    // calculate experation time.
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Invalid time")
        .timestamp() as usize;

    let claims = Claims {
        user_id: user_id.to_string(),
        exp: expiration,
    };

    // generate jwt
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )
    .map_err(|_| "JWT creation failed".to_string())
}
