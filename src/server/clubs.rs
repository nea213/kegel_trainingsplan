use crate::{
    clubs::{ClubDetail, ClubGroupWithTeams, ClubSummary, CreateClubInput},
    groups::GroupSummary,
    server::{
        auth::now_timestamp,
        db,
        entities::{club, club_group, team},
        permissions,
    },
    teams::TeamSummary,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
};
use std::collections::HashMap;

const MAX_CLUB_NAME_LEN: usize = 64;

pub async fn create(input: CreateClubInput) -> Result<ClubSummary, String> {
    permissions::require_system_admin().await?;
    let name = normalize_name("Der Vereinsname", &input.name)?;
    let now = now_timestamp();
    let db = db::connection().await.map_err(db_error)?;

    let created = club::ActiveModel {
        name: Set(name),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(club_summary(created))
}

pub async fn list() -> Result<Vec<ClubSummary>, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    club::Entity::find()
        .order_by_asc(club::Column::Name)
        .all(db)
        .await
        .map(|clubs| clubs.into_iter().map(club_summary).collect())
        .map_err(db_error)
}

pub async fn detail(club_id: i32) -> Result<ClubDetail, String> {
    permissions::require_system_admin().await?;
    let db = db::connection().await.map_err(db_error)?;

    let club_model = club::Entity::find_by_id(club_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Verein wurde nicht gefunden.".to_string())?;

    let groups = club_group::Entity::find()
        .filter(club_group::Column::ClubId.eq(club_id))
        .order_by_asc(club_group::Column::SortOrder)
        .order_by_asc(club_group::Column::Name)
        .all(db)
        .await
        .map_err(db_error)?;

    let teams = team::Entity::find()
        .filter(team::Column::ClubId.eq(club_id))
        .order_by_asc(team::Column::SortOrder)
        .order_by_asc(team::Column::Name)
        .all(db)
        .await
        .map_err(db_error)?;

    let mut teams_by_group = HashMap::<i32, Vec<TeamSummary>>::new();
    for team in teams {
        teams_by_group
            .entry(team.group_id)
            .or_default()
            .push(team_summary(team));
    }

    let groups = groups
        .into_iter()
        .map(|group| ClubGroupWithTeams {
            group: group_summary(group.clone()),
            teams: teams_by_group.remove(&group.id).unwrap_or_default(),
        })
        .collect();

    Ok(ClubDetail {
        club: club_summary(club_model),
        groups,
    })
}

fn normalize_name(label: &str, value: &str) -> Result<String, String> {
    let name = value.trim();

    if name.len() < 2 {
        return Err(format!("{label} muss mindestens 2 Zeichen lang sein."));
    }

    if name.len() > MAX_CLUB_NAME_LEN {
        return Err(format!("{label} darf hoechstens {MAX_CLUB_NAME_LEN} Zeichen lang sein."));
    }

    Ok(name.to_string())
}

fn club_summary(club: club::Model) -> ClubSummary {
    ClubSummary {
        id: club.id,
        name: club.name,
    }
}

fn group_summary(group: club_group::Model) -> GroupSummary {
    GroupSummary {
        id: group.id,
        club_id: group.club_id,
        name: group.name,
        sort_order: group.sort_order,
    }
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
