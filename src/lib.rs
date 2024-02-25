use std::sync::Arc;

use auth::password::Password;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ryzz")]
pub mod ryzz;
#[cfg(feature = "sqlx")]
pub mod sqlx;

pub struct User {
    pub username: Arc<str>,
    pub password: Password,
    pub role: Role,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Standard,
    Guest,
}
