use crate::{
    dashboard::{ManagedGroupSummary, ViewerContext},
    server::{
        db,
        entities::{club, club_membership, club_group, group_trainer, team, team_player},
        permissions,
    },
    teams::TeamSummary,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use std::collections::{BTreeMap, BTreeSet};

pub async fn context() -> Result<ViewerContext, String> {
    let user = permissions::require_authenticated_user().await?;
    let db = db::connection().await.map_err(db_error)?;

    let managed_groups = group_trainer::Entity::find()
        .filter(group_trainer::Column::UserId.eq(user.id))
        .all(db)
        .await
        .map_err(db_error)?;

    let group_ids = managed_groups.iter().map(|membership| membership.group_id).collect::<Vec<_>>();
    let groups = if group_ids.is_empty() {
        Vec::new()
    } else {
        club_group::Entity::find()
            .filter(club_group::Column::Id.is_in(group_ids))
            .order_by_asc(club_group::Column::Name)
            .all(db)
            .await
            .map_err(db_error)?
    };

    let club_ids = groups.iter().map(|group| group.club_id).collect::<BTreeSet<_>>();
    let membership_clubs = club_membership::Entity::find()
        .filter(club_membership::Column::UserId.eq(user.id))
        .all(db)
        .await
        .map_err(db_error)?;
    let membership_club_ids = membership_clubs.iter().map(|membership| membership.club_id).collect::<BTreeSet<_>>();
    let all_club_ids = club_ids.union(&membership_club_ids).copied().collect::<Vec<_>>();

    let clubs = if all_club_ids.is_empty() {
        Vec::new()
    } else {
        club::Entity::find()
            .filter(club::Column::Id.is_in(all_club_ids))
            .order_by_asc(club::Column::Name)
            .all(db)
            .await
            .map_err(db_error)?
    };
    let club_names = clubs
        .iter()
        .map(|club| (club.id, club.name.clone()))
        .collect::<BTreeMap<_, _>>();

    let managed_groups = groups
        .into_iter()
        .map(|group| ManagedGroupSummary {
            group_id: group.id,
            club_id: group.club_id,
            club_name: club_names.get(&group.club_id).cloned().unwrap_or_else(|| "Unbekannter Verein".to_string()),
            group_name: group.name,
        })
        .collect::<Vec<_>>();

    let player_memberships = team_player::Entity::find()
        .filter(team_player::Column::UserId.eq(user.id))
        .all(db)
        .await
        .map_err(db_error)?;
    let team_ids = player_memberships.iter().map(|membership| membership.team_id).collect::<Vec<_>>();
    let teams = if team_ids.is_empty() {
        Vec::new()
    } else {
        team::Entity::find()
            .filter(team::Column::Id.is_in(team_ids))
            .order_by_asc(team::Column::Name)
            .all(db)
            .await
            .map_err(db_error)?
            .into_iter()
            .map(|team| TeamSummary {
                id: team.id,
                club_id: team.club_id,
                group_id: team.group_id,
                name: team.name,
                sort_order: team.sort_order,
            })
            .collect::<Vec<_>>()
    };

    let team_club_ids = teams.iter().map(|team| team.club_id).collect::<BTreeSet<_>>();
    let awaiting_assignment_clubs = membership_clubs
        .into_iter()
        .filter(|membership| !team_club_ids.contains(&membership.club_id))
        .filter_map(|membership| {
            club_names
                .get(&membership.club_id)
                .cloned()
                .map(|name| (membership.club_id, name))
        })
        .collect::<Vec<_>>();

    let member_clubs = club_names
        .into_iter()
        .filter(|(club_id, _)| membership_club_ids.contains(club_id))
        .collect::<Vec<_>>();

    Ok(ViewerContext {
        user,
        managed_groups,
        member_clubs,
        teams,
        awaiting_assignment_clubs,
    })
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
