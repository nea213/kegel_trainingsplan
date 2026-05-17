use crate::{
    server::{
        auth::now_timestamp,
        db,
        entities::{club_group, team},
        permissions,
    },
    teams::{CreateTeamInput, TeamSummary},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};

const MAX_TEAM_NAME_LEN: usize = 64;

pub async fn create(input: CreateTeamInput) -> Result<TeamSummary, String> {
    permissions::require_system_admin().await?;
    let name = normalize_name("Der Mannschaftsname", &input.name)?;
    validate_sort_order(input.sort_order)?;
    let db = db::connection().await.map_err(db_error)?;

    let group = club_group::Entity::find_by_id(input.group_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Zielgruppe wurde nicht gefunden.".to_string())?;

    if group.club_id != input.club_id {
        return Err(
            "Die Mannschaft kann nur in einer Gruppe des ausgewaehlten Vereins angelegt werden."
                .to_string(),
        );
    }

    let now = now_timestamp();
    let created = team::ActiveModel {
        club_id: Set(input.club_id),
        group_id: Set(input.group_id),
        name: Set(name),
        sort_order: Set(input.sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(team_summary(created))
}

pub async fn list_for_group(group_id: i32) -> Result<Vec<TeamSummary>, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    team::Entity::find()
        .filter(team::Column::GroupId.eq(group_id))
        .order_by_asc(team::Column::SortOrder)
        .order_by_asc(team::Column::Name)
        .all(db)
        .await
        .map(|teams| teams.into_iter().map(team_summary).collect())
        .map_err(db_error)
}

fn normalize_name(label: &str, value: &str) -> Result<String, String> {
    let name = value.trim();

    if name.len() < 2 {
        return Err(format!("{label} muss mindestens 2 Zeichen lang sein."));
    }

    if name.len() > MAX_TEAM_NAME_LEN {
        return Err(format!("{label} darf hoechstens {MAX_TEAM_NAME_LEN} Zeichen lang sein."));
    }

    Ok(name.to_string())
}

fn validate_sort_order(sort_order: i32) -> Result<(), String> {
    if sort_order < 0 {
        return Err("Die Sortierung darf nicht negativ sein.".to_string());
    }

    Ok(())
}

fn team_summary(team: team::Model) -> TeamSummary {
    TeamSummary {
        id: team.id,
        club_id: team.club_id,
        group_id: team.group_id,
        name: team.name,
        sort_order: team.sort_order,
    }
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
