use auth::session::IdContext;
use ryzz::*;

use crate::User;

#[table("user")]
struct DbUser {
    #[ryzz(pk)]
    username: String,
    password: String,
}

pub async fn sqlite_write(db: &Database, user: &User) -> Result<(), ryzz::Error> {
    let users = DbUser::table(db).await?;
    let user = DbUser {
        username: user.username.to_string(),
        password: ron::to_string(&user.password).expect("encode password"),
    };
    db.insert(users).values(user)?.rows_affected().await?;
    Ok(())
}

/// Read validated users from a sqlite database
#[derive(Debug, Clone)]
pub struct SqliteUserSource {
    db: Database,
    users: DbUserTable,
}
impl SqliteUserSource {
    pub async fn new(db: Database) -> Result<Self, ryzz::Error> {
        let users = DbUser::table(&db).await?;
        Ok(Self { db, users })
    }

    pub async fn id(&self, cx: &IdContext<'_>) -> Option<User> {
        let user: DbUser = self.db.select(()).from(self.users).first().await.ok()?;
        let user = User {
            username: user.username.into(),
            password: ron::from_str(&user.password).expect("decode password"),
        };
        if !user.password.matches(cx.password) {
            return None;
        }
        Some(user)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use auth::password::Password;

    use super::*;

    #[tokio::test]
    async fn test_db_ops() {
        let db_name = "test_db_ops.db";
        let db = Database::new(db_name).await.unwrap();
        let cx = IdContext {
            username: "foo",
            password: "bar",
        };
        let u = User {
            username: cx.username.into(),
            password: Password::generate(cx.password),
        };
        sqlite_write(&db, &u).await.unwrap();
        let reader = SqliteUserSource::new(db).await.unwrap();
        let r_u = reader.id(&cx).await.unwrap();
        assert_eq!(u.username, r_u.username);
        tokio::fs::remove_file(db_name).await.unwrap();
        tokio::fs::remove_file(Path::new(db_name).with_extension("db-shm"))
            .await
            .unwrap();
        tokio::fs::remove_file(Path::new(db_name).with_extension("db-wal"))
            .await
            .unwrap();
    }
}
