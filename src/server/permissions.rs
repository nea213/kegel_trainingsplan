use crate::{auth::PublicUser, server::{auth, db, entities::{group_trainer, team_player}}};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn require_authenticated_user() -> Result<PublicUser, String> {
    auth::current_user()
        .await?
        .ok_or_else(|| "Nicht angemeldet.".to_string())
}

pub async fn require_system_admin() -> Result<PublicUser, String> {
    let user = require_authenticated_user().await?;

    if !user.is_system_admin {
        return Err("Nur System-Admins duerfen diesen Bereich verwalten.".to_string());
    }

    Ok(user)
}

pub async fn is_group_trainer(user_id: i32, group_id: i32) -> Result<bool, String> {
    let db = db::connection().await.map_err(db_error)?;

    group_trainer::Entity::find()
        .filter(group_trainer::Column::UserId.eq(user_id))
        .filter(group_trainer::Column::GroupId.eq(group_id))
        .one(db)
        .await
        .map(|membership| membership.is_some())
        .map_err(db_error)
}

pub async fn require_group_trainer_or_system_admin(group_id: i32) -> Result<PublicUser, String> {
    let user = require_authenticated_user().await?;

    if user.is_system_admin || is_group_trainer(user.id, group_id).await? {
        return Ok(user);
    }

    Err("Nur Trainer dieser Gruppe oder System-Admins duerfen diesen Bereich verwalten.".to_string())
}

pub async fn is_team_player(user_id: i32, team_id: i32) -> Result<bool, String> {
    let db = db::connection().await.map_err(db_error)?;

    team_player::Entity::find()
        .filter(team_player::Column::UserId.eq(user_id))
        .filter(team_player::Column::TeamId.eq(team_id))
        .one(db)
        .await
        .map(|membership| membership.is_some())
        .map_err(db_error)
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
