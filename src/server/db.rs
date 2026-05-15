use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbErr, Statement};
use std::{env, fs, path::PathBuf};
use tokio::sync::OnceCell;

static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn connection() -> Result<&'static DatabaseConnection, DbErr> {
    DB.get_or_try_init(|| async {
        let database_url = database_url()?;
        ensure_database_directory(&database_url)?;

        let db = Database::connect(&database_url).await?;
        ensure_schema(&db).await?;

        Ok(db)
    })
    .await
}

fn database_url() -> Result<String, DbErr> {
    if let Ok(url) = env::var("DATABASE_URL") {
        return Ok(url);
    }

    let path = PathBuf::from("data").join("kegel-trainingsplan.sqlite");
    let normalized = path.to_string_lossy().replace('\\', "/");

    Ok(format!("sqlite://{normalized}?mode=rwc"))
}

fn ensure_database_directory(database_url: &str) -> Result<(), DbErr> {
    if !database_url.starts_with("sqlite://") {
        return Ok(());
    }

    let path_part = database_url
        .trim_start_matches("sqlite://")
        .split('?')
        .next()
        .unwrap_or_default();

    if path_part.is_empty() || path_part == ":memory:" {
        return Ok(());
    }

    let path = PathBuf::from(path_part);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io_error)?;
    }

    Ok(())
}

async fn ensure_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "PRAGMA foreign_keys = ON;".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions (expires_at);".to_string(),
    ))
    .await?;

    Ok(())
}

fn io_error(error: std::io::Error) -> DbErr {
    DbErr::Custom(error.to_string())
}
