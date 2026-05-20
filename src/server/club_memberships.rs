use crate::{
    club_memberships::PlayerAssignmentInput,
    dashboard::ClubMembershipSummary,
    server::{
        db,
        entities::{club, club_membership, team, team_player, user},
        permissions,
    },
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::{BTreeMap, BTreeSet};

pub async fn list_club_members(club_id: i32) -> Result<Vec<ClubMembershipSummary>, String> {
    permissions::require_system_admin().await?;
    list_members_for_club(club_id, false).await
}

pub async fn list_unassigned_club_members(club_id: i32) -> Result<Vec<ClubMembershipSummary>, String> {
    permissions::require_club_manager(club_id).await?;
    list_members_for_club(club_id, true).await
}

pub async fn assign_player_to_team(input: PlayerAssignmentInput) -> Result<(), String> {
    permissions::require_team_manager(input.team_id).await?;
    let db = db::connection().await.map_err(db_error)?;

    let team = team::Entity::find_by_id(input.team_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Zielmannschaft wurde nicht gefunden.".to_string())?;

    if team.club_id != input.club_id {
        return Err("Die Zielmannschaft gehört nicht zum ausgewählten Verein.".to_string());
    }

    let membership = club_membership::Entity::find()
        .filter(club_membership::Column::ClubId.eq(input.club_id))
        .filter(club_membership::Column::UserId.eq(input.user_id))
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Benutzer ist kein Mitglied dieses Vereins.".to_string())?;

    let already_assigned = team_player::Entity::find()
        .filter(team_player::Column::TeamId.eq(input.team_id))
        .filter(team_player::Column::UserId.eq(input.user_id))
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if already_assigned {
        return Err("Dieser Spieler ist bereits dieser Mannschaft zugewiesen.".to_string());
    }

    team_player::ActiveModel {
        team_id: Set(input.team_id),
        user_id: Set(input.user_id),
        created_at: Set(membership.created_at),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(())
}

async fn list_members_for_club(club_id: i32, only_unassigned: bool) -> Result<Vec<ClubMembershipSummary>, String> {
    let db = db::connection().await.map_err(db_error)?;
    let club = club::Entity::find_by_id(club_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Verein wurde nicht gefunden.".to_string())?;

    let memberships = club_membership::Entity::find()
        .filter(club_membership::Column::ClubId.eq(club_id))
        .order_by_asc(club_membership::Column::CreatedAt)
        .all(db)
        .await
        .map_err(db_error)?;

    let user_ids = memberships.iter().map(|membership| membership.user_id).collect::<Vec<_>>();
    let users = if user_ids.is_empty() {
        Vec::new()
    } else {
        user::Entity::find()
            .filter(user::Column::Id.is_in(user_ids))
            .all(db)
            .await
            .map_err(db_error)?
    };
    let users_by_id = users
        .into_iter()
        .map(|user| (user.id, user))
        .collect::<BTreeMap<_, _>>();

    let assigned_user_ids = team_player::Entity::find()
        .all(db)
        .await
        .map_err(db_error)?
        .into_iter()
        .map(|membership| membership.user_id)
        .collect::<BTreeSet<_>>();

    let members = memberships
        .into_iter()
        .filter_map(|membership| {
            let found_user = users_by_id.get(&membership.user_id)?;
            let is_assigned_to_team = assigned_user_ids.contains(&membership.user_id);

            if only_unassigned && is_assigned_to_team {
                return None;
            }

            Some(ClubMembershipSummary {
                club_id: club.id,
                club_name: club.name.clone(),
                user_id: found_user.id,
                username: found_user.username.clone(),
                is_assigned_to_team,
            })
        })
        .collect();

    Ok(members)
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
