use crate::server::{bootstrap, seed};
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
        bootstrap::ensure_system_admin(&db).await?;
        seed::seed_dev_data(&db).await?;

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
            theme_mode TEXT NOT NULL DEFAULT 'system',
            is_system_admin BOOLEAN NOT NULL DEFAULT 0,
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
        r#"
        CREATE TABLE IF NOT EXISTS clubs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
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
        CREATE TABLE IF NOT EXISTS club_groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            club_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (club_id) REFERENCES clubs(id) ON DELETE CASCADE,
            UNIQUE (club_id, name)
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS teams (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            club_id INTEGER NOT NULL,
            group_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (club_id) REFERENCES clubs(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES club_groups(id) ON DELETE CASCADE,
            UNIQUE (group_id, name)
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS group_trainers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (group_id) REFERENCES club_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE (group_id, user_id)
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS team_players (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            team_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE (team_id, user_id)
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS club_memberships (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            club_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (club_id) REFERENCES clubs(id) ON DELETE CASCADE,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            UNIQUE (club_id, user_id)
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS invitations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code_hash TEXT NOT NULL UNIQUE,
            club_id INTEGER NOT NULL,
            group_id INTEGER NULL,
            team_id INTEGER NULL,
            target_role TEXT NOT NULL,
            created_by_user_id INTEGER NOT NULL,
            expires_at INTEGER NOT NULL,
            used_at INTEGER NULL,
            used_by_user_id INTEGER NULL,
            revoked_at INTEGER NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (club_id) REFERENCES clubs(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES club_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (used_by_user_id) REFERENCES users(id) ON DELETE SET NULL
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS training_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            club_id INTEGER NOT NULL,
            group_id INTEGER NOT NULL,
            team_id INTEGER NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            location TEXT NOT NULL DEFAULT '',
            start_at INTEGER NOT NULL,
            end_at INTEGER NOT NULL,
            status TEXT NOT NULL,
            created_by_user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (club_id) REFERENCES clubs(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES club_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (team_id) REFERENCES teams(id) ON DELETE SET NULL,
            FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE CASCADE
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

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_club_groups_club_id ON club_groups (club_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_teams_club_id ON teams (club_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_teams_group_id ON teams (group_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_group_trainers_group_id ON group_trainers (group_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_group_trainers_user_id ON group_trainers (user_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_team_players_team_id ON team_players (team_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_team_players_user_id ON team_players (user_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_club_memberships_club_id ON club_memberships (club_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_club_memberships_user_id ON club_memberships (user_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_invitations_club_id ON invitations (club_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_invitations_group_id ON invitations (group_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_invitations_created_by_user_id ON invitations (created_by_user_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_invitations_expires_at ON invitations (expires_at);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_sessions_group_id ON training_sessions (group_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_sessions_team_id ON training_sessions (team_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_sessions_start_at ON training_sessions (start_at);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_sessions_created_by_user_id ON training_sessions (created_by_user_id);".to_string(),
    ))
    .await?;

    ensure_user_theme_mode_column(db).await?;
    ensure_user_is_system_admin_column(db).await?;

    Ok(())
}

async fn ensure_user_theme_mode_column(db: &DatabaseConnection) -> Result<(), DbErr> {
    let statement = Statement::from_string(
        db.get_database_backend(),
        "PRAGMA table_info(users);".to_string(),
    );
    let columns = db.query_all(statement).await?;

    let has_theme_mode = columns.iter().any(|row| {
        row.try_get::<String>("", "name")
            .map(|name| name == "theme_mode")
            .unwrap_or(false)
    });

    if !has_theme_mode {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "ALTER TABLE users ADD COLUMN theme_mode TEXT NOT NULL DEFAULT 'system';".to_string(),
        ))
        .await?;
    }

    Ok(())
}

async fn ensure_user_is_system_admin_column(db: &DatabaseConnection) -> Result<(), DbErr> {
    let statement = Statement::from_string(
        db.get_database_backend(),
        "PRAGMA table_info(users);".to_string(),
    );
    let columns = db.query_all(statement).await?;

    let has_is_system_admin = columns.iter().any(|row| {
        row.try_get::<String>("", "name")
            .map(|name| name == "is_system_admin")
            .unwrap_or(false)
    });

    if !has_is_system_admin {
        db.execute(Statement::from_string(
            db.get_database_backend(),
            "ALTER TABLE users ADD COLUMN is_system_admin BOOLEAN NOT NULL DEFAULT 0;".to_string(),
        ))
        .await?;
    }

    Ok(())
}

fn io_error(error: std::io::Error) -> DbErr {
    DbErr::Custom(error.to_string())
}
