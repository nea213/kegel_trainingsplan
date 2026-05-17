use crate::{
    groups::{CreateGroupInput, GroupSummary},
    server::{
        auth::now_timestamp,
        db,
        entities::{club, club_group},
        permissions,
    },
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};

const MAX_GROUP_NAME_LEN: usize = 64;

pub async fn create(input: CreateGroupInput) -> Result<GroupSummary, String> {
    permissions::require_system_admin().await?;
    let name = normalize_name("Der Gruppenname", &input.name)?;
    validate_sort_order(input.sort_order)?;
    let db = db::connection().await.map_err(db_error)?;

    let club_exists = club::Entity::find_by_id(input.club_id)
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if !club_exists {
        return Err("Der Zielverein wurde nicht gefunden.".to_string());
    }

    let now = now_timestamp();
    let created = club_group::ActiveModel {
        club_id: Set(input.club_id),
        name: Set(name),
        sort_order: Set(input.sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(group_summary(created))
}

pub async fn list(club_id: i32) -> Result<Vec<GroupSummary>, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    club_group::Entity::find()
        .filter(club_group::Column::ClubId.eq(club_id))
        .order_by_asc(club_group::Column::SortOrder)
        .order_by_asc(club_group::Column::Name)
        .all(db)
        .await
        .map(|groups| groups.into_iter().map(group_summary).collect())
        .map_err(db_error)
}

fn normalize_name(label: &str, value: &str) -> Result<String, String> {
    let name = value.trim();

    if name.len() < 2 {
        return Err(format!("{label} muss mindestens 2 Zeichen lang sein."));
    }

    if name.len() > MAX_GROUP_NAME_LEN {
        return Err(format!("{label} darf hoechstens {MAX_GROUP_NAME_LEN} Zeichen lang sein."));
    }

    Ok(name.to_string())
}

fn validate_sort_order(sort_order: i32) -> Result<(), String> {
    if sort_order < 0 {
        return Err("Die Sortierung darf nicht negativ sein.".to_string());
    }

    Ok(())
}

fn group_summary(group: club_group::Model) -> GroupSummary {
    GroupSummary {
        id: group.id,
        club_id: group.club_id,
        name: group.name,
        sort_order: group.sort_order,
    }
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
