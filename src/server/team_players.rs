use crate::{
    server::{
        auth::now_timestamp,
        db,
        entities::{club_membership, team, team_player, user},
        permissions,
    },
    team_players::{AssignTeamPlayerInput, AssignedPlayer},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};

pub async fn list(team_id: i32) -> Result<Vec<AssignedPlayer>, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    let memberships = team_player::Entity::find()
        .filter(team_player::Column::TeamId.eq(team_id))
        .order_by_asc(team_player::Column::CreatedAt)
        .all(db)
        .await
        .map_err(db_error)?;

    let mut players = Vec::with_capacity(memberships.len());
    for membership in memberships {
        let found_user = user::Entity::find_by_id(membership.user_id)
            .one(db)
            .await
            .map_err(db_error)?
            .ok_or_else(|| "Ein zugewiesener Spieler wurde nicht gefunden.".to_string())?;

        players.push(assigned_player(found_user));
    }

    Ok(players)
}

pub async fn assign(input: AssignTeamPlayerInput) -> Result<AssignedPlayer, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    let team_model = team::Entity::find_by_id(input.team_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Zielmannschaft wurde nicht gefunden.".to_string())?;

    let found_user = user::Entity::find_by_id(input.user_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die ausgewählte Person wurde nicht gefunden.".to_string())?;

    let is_club_member = club_membership::Entity::find()
        .filter(club_membership::Column::ClubId.eq(team_model.club_id))
        .filter(club_membership::Column::UserId.eq(found_user.id))
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if !is_club_member {
        return Err("Die ausgewählte Person ist kein Mitglied dieses Vereins.".to_string());
    }

    let already_assigned = team_player::Entity::find()
        .filter(team_player::Column::UserId.eq(found_user.id))
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if already_assigned {
        return Err("Diese Person ist bereits einer Mannschaft zugewiesen.".to_string());
    }

    team_player::ActiveModel {
        team_id: Set(input.team_id),
        user_id: Set(found_user.id),
        created_at: Set(now_timestamp()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(assigned_player(found_user))
}

pub async fn remove(team_id: i32, user_id: i32) -> Result<(), String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    let membership = team_player::Entity::find()
        .filter(team_player::Column::TeamId.eq(team_id))
        .filter(team_player::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Dieser Spieler ist der Mannschaft nicht zugewiesen.".to_string())?;

    team_player::Entity::delete_by_id(membership.id)
        .exec(db)
        .await
        .map_err(db_error)?;

    Ok(())
}

fn assigned_player(user: user::Model) -> AssignedPlayer {
    AssignedPlayer {
        user_id: user.id,
        username: user.username,
    }
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
