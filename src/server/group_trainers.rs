use crate::{
    group_trainers::{AssignGroupTrainerInput, AssignedTrainer},
    server::{
        auth::now_timestamp,
        db,
        entities::{club_group, club_membership, group_trainer, user},
        permissions,
    },
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};

pub async fn list(group_id: i32) -> Result<Vec<AssignedTrainer>, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    let memberships = group_trainer::Entity::find()
        .filter(group_trainer::Column::GroupId.eq(group_id))
        .order_by_asc(group_trainer::Column::CreatedAt)
        .all(db)
        .await
        .map_err(db_error)?;

    let mut trainers = Vec::with_capacity(memberships.len());
    for membership in memberships {
        let found_user = user::Entity::find_by_id(membership.user_id)
            .one(db)
            .await
            .map_err(db_error)?
            .ok_or_else(|| "Ein zugewiesener Trainer wurde nicht gefunden.".to_string())?;

        trainers.push(assigned_trainer(found_user));
    }

    Ok(trainers)
}

pub async fn assign(input: AssignGroupTrainerInput) -> Result<AssignedTrainer, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    let group = club_group::Entity::find_by_id(input.group_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Zielgruppe wurde nicht gefunden.".to_string())?;

    let found_user = user::Entity::find_by_id(input.user_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die ausgewählte Person wurde nicht gefunden.".to_string())?;

    let is_club_member = club_membership::Entity::find()
        .filter(club_membership::Column::ClubId.eq(group.club_id))
        .filter(club_membership::Column::UserId.eq(found_user.id))
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if !is_club_member {
        return Err("Die ausgewählte Person ist kein Mitglied dieses Vereins.".to_string());
    }

    let already_assigned = group_trainer::Entity::find()
        .filter(group_trainer::Column::GroupId.eq(input.group_id))
        .filter(group_trainer::Column::UserId.eq(found_user.id))
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if already_assigned {
        return Err("Dieser Benutzer ist bereits Trainer dieser Gruppe.".to_string());
    }

    group_trainer::ActiveModel {
        group_id: Set(input.group_id),
        user_id: Set(found_user.id),
        created_at: Set(now_timestamp()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(assigned_trainer(found_user))
}

pub async fn remove(group_id: i32, user_id: i32) -> Result<(), String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    let membership = group_trainer::Entity::find()
        .filter(group_trainer::Column::GroupId.eq(group_id))
        .filter(group_trainer::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Dieser Trainer ist der Gruppe nicht zugewiesen.".to_string())?;

    group_trainer::Entity::delete_by_id(membership.id)
        .exec(db)
        .await
        .map_err(db_error)?;

    Ok(())
}

fn assigned_trainer(user: user::Model) -> AssignedTrainer {
    AssignedTrainer {
        user_id: user.id,
        username: user.username,
    }
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
