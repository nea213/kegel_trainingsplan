use crate::{
    clubs::{
        ClubDetail, ClubGroupWithTeams, ClubMemberOption, ClubSummary, CreateClubInput,
        TeamWithPlayers,
    },
    group_trainers::AssignedTrainer,
    groups::GroupSummary,
    invitations::InvitationSummary,
    server::{
        auth::now_timestamp,
        db,
        entities::{
            club, club_group, club_membership, group_trainer, invitation, team, team_player, user,
        },
        permissions,
    },
    team_players::AssignedPlayer,
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

    let invitations = invitation::Entity::find()
        .filter(invitation::Column::ClubId.eq(club_id))
        .filter(invitation::Column::RevokedAt.is_null())
        .filter(invitation::Column::UsedAt.is_null())
        .order_by_desc(invitation::Column::CreatedAt)
        .all(db)
        .await
        .map_err(db_error)?;

    let club_memberships = club_membership::Entity::find()
        .filter(club_membership::Column::ClubId.eq(club_id))
        .order_by_asc(club_membership::Column::CreatedAt)
        .all(db)
        .await
        .map_err(db_error)?;

    let group_ids = groups.iter().map(|group| group.id).collect::<Vec<_>>();
    let team_ids = teams.iter().map(|team| team.id).collect::<Vec<_>>();

    let group_trainers = if group_ids.is_empty() {
        Vec::new()
    } else {
        group_trainer::Entity::find()
            .filter(group_trainer::Column::GroupId.is_in(group_ids.clone()))
            .order_by_asc(group_trainer::Column::CreatedAt)
            .all(db)
            .await
            .map_err(db_error)?
    };

    let team_players = if team_ids.is_empty() {
        Vec::new()
    } else {
        team_player::Entity::find()
            .filter(team_player::Column::TeamId.is_in(team_ids.clone()))
            .order_by_asc(team_player::Column::CreatedAt)
            .all(db)
            .await
            .map_err(db_error)?
    };

    let assigned_user_ids = team_players
        .iter()
        .map(|membership| membership.user_id)
        .collect::<std::collections::BTreeSet<_>>();

    let user_ids = group_trainers
        .iter()
        .map(|membership| membership.user_id)
        .chain(team_players.iter().map(|membership| membership.user_id))
        .chain(club_memberships.iter().map(|membership| membership.user_id))
        .collect::<Vec<_>>();

    let users = if user_ids.is_empty() {
        Vec::new()
    } else {
        user::Entity::find()
            .filter(user::Column::Id.is_in(user_ids))
            .all(db)
            .await
            .map_err(db_error)?
    };

    let mut users_by_id = HashMap::new();
    for user in users {
        users_by_id.insert(user.id, user);
    }

    let mut trainers_by_group = HashMap::<i32, Vec<AssignedTrainer>>::new();
    for membership in group_trainers {
        let found_user = users_by_id
            .get(&membership.user_id)
            .cloned()
            .ok_or_else(|| "Ein zugewiesener Trainer wurde nicht gefunden.".to_string())?;
        trainers_by_group
            .entry(membership.group_id)
            .or_default()
            .push(assigned_trainer(found_user));
    }

    let mut players_by_team = HashMap::<i32, Vec<AssignedPlayer>>::new();
    for membership in team_players {
        let found_user = users_by_id
            .get(&membership.user_id)
            .cloned()
            .ok_or_else(|| "Ein zugewiesener Spieler wurde nicht gefunden.".to_string())?;
        players_by_team
            .entry(membership.team_id)
            .or_default()
            .push(assigned_player(found_user));
    }

    let mut teams_by_group = HashMap::<i32, Vec<TeamWithPlayers>>::new();
    for team in teams {
        teams_by_group
            .entry(team.group_id)
            .or_default()
            .push(TeamWithPlayers {
                players: players_by_team.remove(&team.id).unwrap_or_default(),
                team: team_summary(team),
            });
    }

    let mut invitations_by_group = HashMap::<i32, Vec<InvitationSummary>>::new();
    for invitation in invitations {
        if let Some(group_id) = invitation.group_id {
            invitations_by_group
                .entry(group_id)
                .or_default()
                .push(invitation_summary(invitation));
        }
    }

    let groups = groups
        .into_iter()
        .map(|group| ClubGroupWithTeams {
            group: group_summary(group.clone()),
            invitations: invitations_by_group.remove(&group.id).unwrap_or_default(),
            trainers: trainers_by_group.remove(&group.id).unwrap_or_default(),
            teams: teams_by_group.remove(&group.id).unwrap_or_default(),
        })
        .collect();

    let club_members = club_memberships
        .into_iter()
        .filter_map(|membership| {
            let found_user = users_by_id.get(&membership.user_id)?;
            Some(ClubMemberOption {
                user_id: found_user.id,
                username: found_user.username.clone(),
                is_assigned_to_team: assigned_user_ids.contains(&found_user.id),
            })
        })
        .collect();

    Ok(ClubDetail {
        club: club_summary(club_model),
        groups,
        club_members,
    })
}

fn normalize_name(label: &str, value: &str) -> Result<String, String> {
    let name = value.trim();

    if name.len() < 2 {
        return Err(format!("{label} muss mindestens 2 Zeichen lang sein."));
    }

    if name.len() > MAX_CLUB_NAME_LEN {
        return Err(format!("{label} darf höchstens {MAX_CLUB_NAME_LEN} Zeichen lang sein."));
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

fn assigned_trainer(user: user::Model) -> AssignedTrainer {
    AssignedTrainer {
        user_id: user.id,
        username: user.username,
    }
}

fn assigned_player(user: user::Model) -> AssignedPlayer {
    AssignedPlayer {
        user_id: user.id,
        username: user.username,
    }
}

fn invitation_summary(invitation: invitation::Model) -> InvitationSummary {
    InvitationSummary {
        id: invitation.id,
        club_id: invitation.club_id,
        group_id: invitation.group_id,
        role: match invitation.target_role.trim().to_ascii_lowercase().as_str() {
            "trainer" => crate::invitations::InvitationRole::Trainer,
            _ => crate::invitations::InvitationRole::Player,
        },
        expires_at: invitation.expires_at,
        revoked_at: invitation.revoked_at,
        used_at: invitation.used_at,
    }
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
