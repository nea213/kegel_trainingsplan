use crate::server::{
    auth::{hash_password, normalize_username, now_timestamp, validate_password},
    entities::user,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::env;

const BOOTSTRAP_ADMIN_USERNAME: &str = "BOOTSTRAP_ADMIN_USERNAME";
const BOOTSTRAP_ADMIN_PASSWORD: &str = "BOOTSTRAP_ADMIN_PASSWORD";

pub async fn ensure_system_admin(db: &DatabaseConnection) -> Result<(), DbErr> {
    if system_admin_exists(db).await? {
        return Ok(());
    }

    let Some((username, password)) = bootstrap_credentials()? else {
        return Ok(());
    };

    let username = normalize_username(&username).map_err(db_error)?;
    validate_password(&password).map_err(db_error)?;

    if user::Entity::find()
        .filter(user::Column::Username.eq(username.clone()))
        .one(db)
        .await?
        .is_some()
    {
        return Err(DbErr::Custom(format!(
            "Bootstrap-System-Admin konnte nicht angelegt werden: Benutzername '{username}' existiert bereits."
        )));
    }

    let now = now_timestamp();
    let created_username = username.clone();
    let password_hash = hash_password(&password).map_err(db_error)?;

    user::ActiveModel {
        username: Set(username),
        password_hash: Set(password_hash),
        theme_mode: Set("system".to_string()),
        is_system_admin: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    println!(
        "Bootstrap-System-Admin wurde erfolgreich erstellt: {}",
        created_username
    );

    Ok(())
}

async fn system_admin_exists(db: &DatabaseConnection) -> Result<bool, DbErr> {
    Ok(user::Entity::find()
        .filter(user::Column::IsSystemAdmin.eq(true))
        .one(db)
        .await?
        .is_some())
}

fn bootstrap_credentials() -> Result<Option<(String, String)>, DbErr> {
    let username = env::var(BOOTSTRAP_ADMIN_USERNAME).ok();
    let password = env::var(BOOTSTRAP_ADMIN_PASSWORD).ok();

    match (username, password) {
        (Some(username), Some(password)) => Ok(Some((username, password))),
        (None, None) => Ok(None),
        _ => Err(DbErr::Custom(format!(
            "Bootstrap-System-Admin ist unvollstaendig konfiguriert. Setze entweder beide Variablen '{BOOTSTRAP_ADMIN_USERNAME}' und '{BOOTSTRAP_ADMIN_PASSWORD}' oder keine von beiden."
        ))),
    }
}

fn db_error(error: impl std::fmt::Display) -> DbErr {
    DbErr::Custom(error.to_string())
}
