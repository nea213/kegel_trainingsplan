use crate::{auth::PublicUser, server::{auth, db, entities::{club_group, group_trainer, team, team_player}}};
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
    require_group_manager(group_id).await
}

pub async fn require_invitation_manager(club_id: i32, group_id: Option<i32>) -> Result<PublicUser, String> {
    let user = require_authenticated_user().await?;

    if user.is_system_admin {
        return Ok(user);
    }

    let Some(group_id) = group_id else {
        return Err("Nur System-Admins duerfen vereinsweite Spielereinladungen erzeugen.".to_string());
    };

    let db = db::connection().await.map_err(db_error)?;
    let group = club_group::Entity::find_by_id(group_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Zielgruppe wurde nicht gefunden.".to_string())?;

    if group.club_id != club_id {
        return Err("Die Zielgruppe gehoert nicht zum ausgewaehlten Verein.".to_string());
    }

    if is_group_trainer(user.id, group_id).await? {
        return Ok(user);
    }

    Err("Nur Trainer dieser Gruppe oder System-Admins duerfen Einladungen erzeugen.".to_string())
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

pub async fn is_club_trainer(user_id: i32, club_id: i32) -> Result<bool, String> {
    let db = db::connection().await.map_err(db_error)?;
    let group_ids = club_group::Entity::find()
        .filter(club_group::Column::ClubId.eq(club_id))
        .all(db)
        .await
        .map_err(db_error)?
        .into_iter()
        .map(|group| group.id)
        .collect::<Vec<_>>();

    if group_ids.is_empty() {
        return Ok(false);
    }

    group_trainer::Entity::find()
        .filter(group_trainer::Column::UserId.eq(user_id))
        .filter(group_trainer::Column::GroupId.is_in(group_ids))
        .one(db)
        .await
        .map(|membership| membership.is_some())
        .map_err(db_error)
}

pub async fn require_club_manager(club_id: i32) -> Result<PublicUser, String> {
    let user = require_authenticated_user().await?;
    if user.is_system_admin || is_club_trainer(user.id, club_id).await? {
        return Ok(user);
    }

    Err("Nur Trainer mit Gruppen in diesem Verein oder System-Admins duerfen diesen Bereich verwalten.".to_string())
}

pub async fn can_manage_group(user_id: i32, group_id: i32) -> Result<bool, String> {
    let user = auth::current_user().await?.ok_or_else(|| "Nicht angemeldet.".to_string())?;
    if user.id != user_id {
        return Ok(false);
    }

    if user.is_system_admin {
        return Ok(true);
    }

    is_group_trainer(user_id, group_id).await
}

pub async fn require_group_manager(group_id: i32) -> Result<PublicUser, String> {
    let user = require_authenticated_user().await?;
    if user.is_system_admin || is_group_trainer(user.id, group_id).await? {
        return Ok(user);
    }

    Err("Nur Trainer dieser Gruppe oder System-Admins duerfen diesen Bereich verwalten.".to_string())
}

pub async fn can_manage_team(user_id: i32, team_id: i32) -> Result<bool, String> {
    let user = auth::current_user().await?.ok_or_else(|| "Nicht angemeldet.".to_string())?;
    if user.id != user_id {
        return Ok(false);
    }

    if user.is_system_admin {
        return Ok(true);
    }

    let db = db::connection().await.map_err(db_error)?;
    let Some(team) = team::Entity::find_by_id(team_id).one(db).await.map_err(db_error)? else {
        return Ok(false);
    };

    is_group_trainer(user_id, team.group_id).await
}

pub async fn require_team_manager(team_id: i32) -> Result<PublicUser, String> {
    let user = require_authenticated_user().await?;
    if user.is_system_admin {
        return Ok(user);
    }

    let db = db::connection().await.map_err(db_error)?;
    let team = team::Entity::find_by_id(team_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Zielmannschaft wurde nicht gefunden.".to_string())?;

    if is_group_trainer(user.id, team.group_id).await? {
        return Ok(user);
    }

    Err("Nur Trainer dieser Gruppe oder System-Admins duerfen diese Mannschaft verwalten.".to_string())
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
