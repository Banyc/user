use std::sync::Arc;

use password::Password;

pub mod password;

#[cfg(feature = "sqlx")]
pub mod sqlx;

#[derive(Debug, Clone)]
pub struct User {
    pub username: Arc<str>,
    pub password: Password,
}
