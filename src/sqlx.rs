use auth::session::IdContext;

use crate::User;

pub async fn sqlite_write<'db>(
    executor: impl sqlx::Acquire<'db, Database = sqlx::Sqlite>,
    user: &User,
) -> Result<(), sqlx::Error> {
    let mut executor = executor.acquire().await?;

    sqlx::query(
        r#"create table if not exists user (
username text primary key,
password text not null,
role     text not null
)"#,
    )
    .execute(&mut *executor)
    .await?;

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
    .await?;

    Ok(())
}

async fn sqlite_read<'db>(
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

/// Read user from sqlite
#[derive(Debug, Clone)]
pub struct SqliteUserSource {
    pool: sqlx::SqlitePool,
}
impl SqliteUserSource {
    pub async fn new(url: &str) -> Self {
        let pool = sqlx::SqlitePool::connect(url)
            .await
            .expect("sqlite pool connect");
        Self { pool }
    }

    pub async fn id(&self, cx: &IdContext<'_>) -> Option<User> {
        let user = sqlite_read(&self.pool, cx.username).await?;
        if !user.password.matches(cx.password) {
            return None;
        }
        Some(user)
    }
}

#[cfg(test)]
mod tests {
    use auth::password::Password;
    use sqlx::{Connection, SqliteConnection};

    use crate::Role;

    use super::*;

    #[tokio::test]
    async fn test_db_ops() {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
        let u = User {
            username: "foo".into(),
            password: Password::generate("bar"),
            role: Role::Standard,
        };
        sqlite_write(&mut conn, &u).await.unwrap();
        let r_u = sqlite_read(&mut conn, "foo").await.unwrap();
        assert_eq!(u.username, r_u.username);
    }
}
