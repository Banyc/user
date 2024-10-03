use std::sync::Arc;

use auth::password::Password;

#[cfg(feature = "sqlx")]
pub mod sqlx;

#[derive(Debug, Clone)]
pub struct User {
    pub username: Arc<str>,
    pub password: Password,
}
