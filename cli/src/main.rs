use auth::password::Password;
use clap::Parser;
use sqlx::Connection;
use user::{sqlx::sqlite_write, User};

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[clap(long)]
    db: String,
    #[clap(short, long)]
    username: String,
    #[clap(short, long)]
    password: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut db = sqlx::SqliteConnection::connect(&cli.db).await?;
    let user = User {
        username: cli.username.into(),
        password: Password::generate(&cli.password),
    };
    sqlite_write(&mut db, &user).await?;
    Ok(())
}
