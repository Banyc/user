use auth::password::Password;
use clap::Parser;
use ryzz::Database;
use user::{ryzz::sqlite_write, User};

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
    let db = Database::new(&cli.db).await?;
    let user = User {
        username: cli.username.into(),
        password: Password::generate(&cli.password),
    };
    sqlite_write(&db, &user).await?;
    Ok(())
}
