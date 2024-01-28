use std::sync::Arc;

use auth::password::Password;
use serde::{Deserialize, Serialize};

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

pub async fn sqlite_write<'db>(
    executor: impl sqlx::Acquire<'db, Database = sqlx::Sqlite>,
    user: &User,
) {
    let mut executor = executor.acquire().await.expect("acquire executor");

    sqlx::query(
        r#"create table if not exists user (
username text primary key,
password text not null,
role     text not null
)"#,
    )
    .execute(&mut *executor)
    .await
    .expect("failed to create a table");

    let username = user.username.as_ref();
    let password = ron::to_string(&user.password).expect("encode password");
    let role = ron::to_string(&user.role).expect("encode role");
    sqlx::query(
        r#"insert into user ( username, password, role )
values ( $1, $2, $3 )"#,
    )
    .bind(username)
    .bind(&password)
    .bind(&role)
    .execute(&mut *executor)
    .await
    .expect("failed to insert");
}

pub async fn sqlite_read<'db>(
    executor: impl sqlx::Acquire<'db, Database = sqlx::Sqlite>,
    username: &str,
) -> Option<User> {
    let mut executor = executor.acquire().await.expect("acquire executor");
    let user: DbUser = sqlx::query_as(r#"select * from user where username = $1"#)
        .bind(username)
        .fetch_one(&mut *executor)
        .await
        .ok()?;
    let user = User {
        username: user.username.into(),
        password: ron::from_str(&user.password).expect("decode password"),
        role: ron::from_str(&user.role).expect("decode role"),
    };
    return Some(user);

    #[allow(unused)]
    #[derive(Debug, sqlx::FromRow)]
    struct DbUser {
        pub username: String,
        pub password: String,
        pub role: String,
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{Connection, SqliteConnection};

    use super::*;

    #[tokio::test]
    async fn test_db_ops() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let u = User {
            username: "foo".into(),
            password: Password::generate("bar"),
            role: Role::Standard,
        };
        sqlite_write(&mut conn, &u).await;
        let r_u = sqlite_read(&mut conn, "foo").await.unwrap();
        assert_eq!(u.username, r_u.username);
    }
}
