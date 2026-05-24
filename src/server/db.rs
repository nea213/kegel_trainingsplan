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
        CREATE TABLE IF NOT EXISTS training_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            club_id INTEGER NOT NULL,
            group_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            number_of_throws INTEGER NULL,
            target_score INTEGER NULL,
            standing_pins_mask INTEGER NULL,
            clear_pins BOOLEAN NULL,
            created_by_user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (club_id) REFERENCES clubs(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES club_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS training_plans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            club_id INTEGER NOT NULL,
            group_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            day TEXT NOT NULL,
            note TEXT NOT NULL DEFAULT '',
            trainer_user_id INTEGER NULL,
            created_by_user_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (club_id) REFERENCES clubs(id) ON DELETE CASCADE,
            FOREIGN KEY (group_id) REFERENCES club_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (trainer_user_id) REFERENCES users(id) ON DELETE SET NULL,
            FOREIGN KEY (created_by_user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#
        .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        r#"
        CREATE TABLE IF NOT EXISTS training_plan_templates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            training_plan_id INTEGER NOT NULL,
            training_template_id INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (training_plan_id) REFERENCES training_plans(id) ON DELETE CASCADE,
            FOREIGN KEY (training_template_id) REFERENCES training_templates(id) ON DELETE CASCADE,
            UNIQUE (training_plan_id, training_template_id)
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
        "CREATE INDEX IF NOT EXISTS idx_training_templates_club_id ON training_templates (club_id);"
            .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_templates_group_id ON training_templates (group_id);"
            .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_plans_club_id ON training_plans (club_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_plans_group_id ON training_plans (group_id);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_plans_day ON training_plans (day);".to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_plans_trainer_user_id ON training_plans (trainer_user_id);"
            .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_plan_templates_plan_id ON training_plan_templates (training_plan_id);"
            .to_string(),
    ))
    .await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "CREATE INDEX IF NOT EXISTS idx_training_plan_templates_template_id ON training_plan_templates (training_template_id);"
            .to_string(),
    ))
    .await?;

    ensure_user_theme_mode_column(db).await?;
    ensure_user_is_system_admin_column(db).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ensure_schema;
    use crate::server::entities::{
        club, club_group, training_plan, training_plan_template, training_template, user,
    };
    use sea_orm::{
        ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Database, DatabaseConnection,
        EntityTrait, Statement,
    };

    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should connect");
        ensure_schema(&db).await.expect("schema should be created");
        db
    }

    async fn seed_scope(db: &DatabaseConnection) -> (i32, i32, i32, i32) {
        let now = 1_i64;
        let owner = user::ActiveModel {
            username: Set("owner".to_string()),
            password_hash: Set("hash".to_string()),
            theme_mode: Set("system".to_string()),
            is_system_admin: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .expect("user insert should succeed");

        let trainer = user::ActiveModel {
            username: Set("trainer".to_string()),
            password_hash: Set("hash".to_string()),
            theme_mode: Set("system".to_string()),
            is_system_admin: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .expect("trainer insert should succeed");

        let club = club::ActiveModel {
            name: Set("Verein".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .expect("club insert should succeed");

        let group = club_group::ActiveModel {
            club_id: Set(club.id),
            name: Set("Gruppe".to_string()),
            sort_order: Set(0),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .expect("group insert should succeed");

        (owner.id, trainer.id, club.id, group.id)
    }

    #[tokio::test]
    async fn creates_schema_from_empty_sqlite_database() {
        let db = setup_db().await;

        let result = db
            .query_one(Statement::from_string(
                db.get_database_backend(),
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'training_templates';"
                    .to_string(),
            ))
            .await
            .expect("query should succeed");

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn duplicate_template_assignment_is_rejected() {
        let db = setup_db().await;
        let (owner_id, trainer_id, club_id, group_id) = seed_scope(&db).await;
        let now = 1_i64;

        let template = training_template::ActiveModel {
            club_id: Set(club_id),
            group_id: Set(group_id),
            title: Set("Vorlage".to_string()),
            description: Set(String::new()),
            number_of_throws: Set(None),
            target_score: Set(None),
            standing_pins_mask: Set(None),
            clear_pins: Set(None),
            created_by_user_id: Set(owner_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("template insert should succeed");

        let plan = training_plan::ActiveModel {
            club_id: Set(club_id),
            group_id: Set(group_id),
            title: Set("Plan".to_string()),
            day: Set("2026-05-23".to_string()),
            note: Set(String::new()),
            trainer_user_id: Set(Some(trainer_id)),
            created_by_user_id: Set(owner_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("plan insert should succeed");

        training_plan_template::ActiveModel {
            training_plan_id: Set(plan.id),
            training_template_id: Set(template.id),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("first join insert should succeed");

        let duplicate = training_plan_template::ActiveModel {
            training_plan_id: Set(plan.id),
            training_template_id: Set(template.id),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await;

        assert!(duplicate.is_err());
    }

    #[tokio::test]
    async fn one_template_can_be_linked_to_multiple_plans() {
        let db = setup_db().await;
        let (owner_id, trainer_id, club_id, group_id) = seed_scope(&db).await;
        let now = 1_i64;

        let template = training_template::ActiveModel {
            club_id: Set(club_id),
            group_id: Set(group_id),
            title: Set("Vorlage".to_string()),
            description: Set(String::new()),
            number_of_throws: Set(None),
            target_score: Set(None),
            standing_pins_mask: Set(None),
            clear_pins: Set(None),
            created_by_user_id: Set(owner_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("template insert should succeed");

        let first_plan = training_plan::ActiveModel {
            club_id: Set(club_id),
            group_id: Set(group_id),
            title: Set("Plan A".to_string()),
            day: Set("2026-05-23".to_string()),
            note: Set(String::new()),
            trainer_user_id: Set(Some(trainer_id)),
            created_by_user_id: Set(owner_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("first plan insert should succeed");

        let second_plan = training_plan::ActiveModel {
            club_id: Set(club_id),
            group_id: Set(group_id),
            title: Set("Plan B".to_string()),
            day: Set("2026-05-24".to_string()),
            note: Set(String::new()),
            trainer_user_id: Set(Some(trainer_id)),
            created_by_user_id: Set(owner_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("second plan insert should succeed");

        training_plan_template::ActiveModel {
            training_plan_id: Set(first_plan.id),
            training_template_id: Set(template.id),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("first join insert should succeed");

        training_plan_template::ActiveModel {
            training_plan_id: Set(second_plan.id),
            training_template_id: Set(template.id),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("second join insert should succeed");

        let links = training_plan_template::Entity::find()
            .all(&db)
            .await
            .expect("query should succeed");

        assert_eq!(links.len(), 2);
    }
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
