use crate::auth::{LoginInput, PublicUser, RegisterInput};
use crate::server::{
    db,
    entities::{session, user},
};
use crate::theme::ThemeMode;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use dioxus_cookie::{self as cookie, CookieOptions, SameSite};
use password_hash::SaltString;
use rand_core::OsRng;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DbErr, EntityTrait, QueryFilter,
};
use std::{env, time::Duration};
use uuid::Uuid;

const SESSION_COOKIE: &str = "kegel_session";
const SESSION_DURATION_SECS: i64 = 60 * 60 * 24 * 7;
const MIN_USERNAME_LEN: usize = 3;
const MIN_PASSWORD_LEN: usize = 8;

pub async fn register(input: RegisterInput) -> Result<PublicUser, String> {
    let username = normalize_username(&input.username)?;
    validate_password(&input.password)?;

    let db = db::connection().await.map_err(db_error)?;
    delete_expired_sessions(db).await.map_err(db_error)?;

    let existing = user::Entity::find()
        .filter(user::Column::Username.eq(username.clone()))
        .one(db)
        .await
        .map_err(db_error)?;

    if existing.is_some() {
        return Err("Dieser Benutzername ist bereits vergeben.".to_string());
    }

    let password_hash = hash_password(&input.password)?;
    let now = now_timestamp();

    let new_user = user::ActiveModel {
        username: Set(username),
        password_hash: Set(password_hash),
        theme_mode: Set(ThemeMode::System.as_str().to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    create_session_for_user(db, &new_user).await
}

pub async fn login(input: LoginInput) -> Result<PublicUser, String> {
    let username = normalize_username(&input.username)?;
    validate_password(&input.password)?;

    let db = db::connection().await.map_err(db_error)?;
    delete_expired_sessions(db).await.map_err(db_error)?;

    let Some(found_user) = user::Entity::find()
        .filter(user::Column::Username.eq(username))
        .one(db)
        .await
        .map_err(db_error)?
    else {
        return Err("Benutzername oder Passwort ist ungültig.".to_string());
    };

    verify_password(&input.password, &found_user.password_hash)?;

    create_session_for_user(db, &found_user).await
}

pub async fn current_user() -> Result<Option<PublicUser>, String> {
    let Some(session_id) = cookie::get_internal(SESSION_COOKIE) else {
        return Ok(None);
    };

    let db = db::connection().await.map_err(db_error)?;
    delete_expired_sessions(db).await.map_err(db_error)?;

    let Some(active_session) = session::Entity::find_by_id(session_id.clone())
        .one(db)
        .await
        .map_err(db_error)?
    else {
        clear_session_cookie()?;
        return Ok(None);
    };

    if active_session.expires_at <= now_timestamp() {
        session::Entity::delete_by_id(session_id)
            .exec(db)
            .await
            .map_err(db_error)?;
        clear_session_cookie()?;
        return Ok(None);
    }

    let user = user::Entity::find_by_id(active_session.user_id)
        .one(db)
        .await
        .map_err(db_error)?
        .map(public_user);

    if user.is_none() {
        clear_session_cookie()?;
    }

    Ok(user)
}

pub async fn logout() -> Result<(), String> {
    let db = db::connection().await.map_err(db_error)?;

    if let Some(session_id) = cookie::get_internal(SESSION_COOKIE) {
        session::Entity::delete_by_id(session_id)
            .exec(db)
            .await
            .map_err(db_error)?;
    }

    clear_session_cookie()
}

pub async fn update_theme_mode(theme_mode: ThemeMode) -> Result<PublicUser, String> {
    let Some(session_id) = cookie::get_internal(SESSION_COOKIE) else {
        return Err("Nicht angemeldet.".to_string());
    };

    let db = db::connection().await.map_err(db_error)?;
    delete_expired_sessions(db).await.map_err(db_error)?;

    let Some(active_session) = session::Entity::find_by_id(session_id)
        .one(db)
        .await
        .map_err(db_error)?
    else {
        clear_session_cookie()?;
        return Err("Nicht angemeldet.".to_string());
    };

    let Some(found_user) = user::Entity::find_by_id(active_session.user_id)
        .one(db)
        .await
        .map_err(db_error)?
    else {
        clear_session_cookie()?;
        return Err("Benutzer wurde nicht gefunden.".to_string());
    };

    let now = now_timestamp();
    let mut active_user: user::ActiveModel = found_user.clone().into();
    active_user.theme_mode = Set(theme_mode.as_str().to_string());
    active_user.updated_at = Set(now);

    let updated_user = active_user.update(db).await.map_err(db_error)?;
    Ok(public_user(updated_user))
}

async fn create_session_for_user(
    db: &sea_orm::DatabaseConnection,
    user: &user::Model,
) -> Result<PublicUser, String> {
    let now = now_timestamp();
    let session_id = Uuid::new_v4().to_string();

    let session = session::ActiveModel {
        id: Set(session_id.clone()),
        user_id: Set(user.id),
        created_at: Set(now),
        expires_at: Set(now + SESSION_DURATION_SECS),
    };

    session.insert(db).await.map_err(db_error)?;

    cookie::set(SESSION_COOKIE, &session_id, &cookie_options()).map_err(cookie_error)?;

    Ok(public_user(user.clone()))
}

async fn delete_expired_sessions(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
    session::Entity::delete_many()
        .filter(session::Column::ExpiresAt.lte(now_timestamp()))
        .exec(db)
        .await?;

    Ok(())
}

fn normalize_username(value: &str) -> Result<String, String> {
    let username = value.trim().to_lowercase();

    if username.len() < MIN_USERNAME_LEN {
        return Err(format!(
            "Der Benutzername muss mindestens {MIN_USERNAME_LEN} Zeichen lang sein."
        ));
    }

    if username.len() > 32 {
        return Err("Der Benutzername darf höchstens 32 Zeichen lang sein.".to_string());
    }

    Ok(username)
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(format!(
            "Das Passwort muss mindestens {MIN_PASSWORD_LEN} Zeichen lang sein."
        ));
    }

    if password.len() > 128 {
        return Err("Das Passwort darf höchstens 128 Zeichen lang sein.".to_string());
    }

    Ok(())
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| error.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), String> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|error| error.to_string())?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Benutzername oder Passwort ist ungültig.".to_string())
}

fn public_user(user: user::Model) -> PublicUser {
    PublicUser {
        id: user.id,
        username: user.username,
        theme_mode: ThemeMode::from_storage(&user.theme_mode),
    }
}

fn cookie_options() -> CookieOptions {
    CookieOptions {
        max_age: Some(Duration::from_secs(SESSION_DURATION_SECS as u64)),
        http_only: true,
        secure: cookie_secure(),
        same_site: SameSite::Lax,
        path: "/".to_string(),
    }
}

fn cookie_secure() -> bool {
    env::var("AUTH_COOKIE_SECURE")
        .ok()
        .and_then(|value| match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(false)
}

fn clear_session_cookie() -> Result<(), String> {
    cookie::clear(SESSION_COOKIE).map_err(cookie_error)
}

fn cookie_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
