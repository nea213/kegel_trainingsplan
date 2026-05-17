use crate::{
    server::{
        auth::now_timestamp,
        db,
        entities::{club, club_group, team, team_player, training_session},
        permissions,
    },
    training::{CreateTrainingSessionInput, TrainingSessionSummary},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder,
};
use std::collections::{BTreeMap, BTreeSet};
use time::{Date, Month, PrimitiveDateTime, Time};

const MAX_TITLE_LEN: usize = 120;
const MAX_LOCATION_LEN: usize = 120;
const MAX_DESCRIPTION_LEN: usize = 2_000;

pub async fn create(input: CreateTrainingSessionInput) -> Result<TrainingSessionSummary, String> {
    let actor = permissions::require_group_manager(input.group_id).await?;
    let title = normalize_required_text("Der Trainingstitel", &input.title, 2, MAX_TITLE_LEN)?;
    let description = normalize_optional_text("Die Beschreibung", &input.description, MAX_DESCRIPTION_LEN)?;
    let location = normalize_optional_text("Der Ort", &input.location, MAX_LOCATION_LEN)?;
    let start_at = parse_datetime_input("Der Startzeitpunkt", &input.start_at)?;
    let end_at = parse_datetime_input("Der Endzeitpunkt", &input.end_at)?;

    if end_at < start_at {
        return Err("Der Endzeitpunkt darf nicht vor dem Startzeitpunkt liegen.".to_string());
    }

    let db = db::connection().await.map_err(db_error)?;
    let club = club::Entity::find_by_id(input.club_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Zielverein wurde nicht gefunden.".to_string())?;
    let group = club_group::Entity::find_by_id(input.group_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Zielgruppe wurde nicht gefunden.".to_string())?;

    if group.club_id != input.club_id {
        return Err("Die Zielgruppe gehoert nicht zum ausgewaehlten Verein.".to_string());
    }

    let team_name = match input.team_id {
        Some(team_id) => {
            let team = team::Entity::find_by_id(team_id)
                .one(db)
                .await
                .map_err(db_error)?
                .ok_or_else(|| "Die Zielmannschaft wurde nicht gefunden.".to_string())?;

            if team.group_id != input.group_id || team.club_id != input.club_id {
                return Err(
                    "Die Zielmannschaft muss zur ausgewaehlten Gruppe und zum ausgewaehlten Verein gehoeren."
                        .to_string(),
                );
            }

            Some(team.name)
        }
        None => None,
    };

    let now = now_timestamp();
    let created = training_session::ActiveModel {
        club_id: Set(input.club_id),
        group_id: Set(input.group_id),
        team_id: Set(input.team_id),
        title: Set(title),
        description: Set(description),
        location: Set(location),
        start_at: Set(start_at),
        end_at: Set(end_at),
        status: Set("planned".to_string()),
        created_by_user_id: Set(actor.id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(training_summary(
        created,
        &club.name,
        &group.name,
        team_name.as_deref(),
    ))
}

pub async fn list_for_group(group_id: i32) -> Result<Vec<TrainingSessionSummary>, String> {
    permissions::require_group_manager(group_id).await?;
    let db = db::connection().await.map_err(db_error)?;

    let items = training_session::Entity::find()
        .filter(training_session::Column::GroupId.eq(group_id))
        .order_by_asc(training_session::Column::StartAt)
        .order_by_asc(training_session::Column::Title)
        .all(db)
        .await
        .map_err(db_error)?;

    enrich_training_summaries(db, items).await
}

pub async fn list_for_viewer() -> Result<Vec<TrainingSessionSummary>, String> {
    let user = permissions::require_authenticated_user().await?;
    let db = db::connection().await.map_err(db_error)?;

    let memberships = team_player::Entity::find()
        .filter(team_player::Column::UserId.eq(user.id))
        .all(db)
        .await
        .map_err(db_error)?;
    let team_ids = memberships.iter().map(|membership| membership.team_id).collect::<Vec<_>>();
    if team_ids.is_empty() {
        return Ok(Vec::new());
    }

    let teams = team::Entity::find()
        .filter(team::Column::Id.is_in(team_ids.clone()))
        .all(db)
        .await
        .map_err(db_error)?;
    let group_ids = teams.iter().map(|team| team.group_id).collect::<BTreeSet<_>>();
    if group_ids.is_empty() {
        return Ok(Vec::new());
    }

    let items = training_session::Entity::find()
        .filter(training_session::Column::EndAt.gte(now_timestamp()))
        .filter(
            Condition::any()
                .add(
                    Condition::all()
                        .add(training_session::Column::GroupId.is_in(group_ids.into_iter().collect::<Vec<_>>()))
                        .add(training_session::Column::TeamId.is_null()),
                )
                .add(training_session::Column::TeamId.is_in(team_ids)),
        )
        .order_by_asc(training_session::Column::StartAt)
        .order_by_asc(training_session::Column::Title)
        .all(db)
        .await
        .map_err(db_error)?;

    enrich_training_summaries(db, items).await
}

async fn enrich_training_summaries(
    db: &DatabaseConnection,
    items: Vec<training_session::Model>,
) -> Result<Vec<TrainingSessionSummary>, String> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let club_ids = items.iter().map(|item| item.club_id).collect::<BTreeSet<_>>();
    let group_ids = items.iter().map(|item| item.group_id).collect::<BTreeSet<_>>();
    let team_ids = items.iter().filter_map(|item| item.team_id).collect::<BTreeSet<_>>();

    let clubs = club::Entity::find()
        .filter(club::Column::Id.is_in(club_ids.into_iter().collect::<Vec<_>>()))
        .all(db)
        .await
        .map_err(db_error)?;
    let groups = club_group::Entity::find()
        .filter(club_group::Column::Id.is_in(group_ids.into_iter().collect::<Vec<_>>()))
        .all(db)
        .await
        .map_err(db_error)?;
    let teams = if team_ids.is_empty() {
        Vec::new()
    } else {
        team::Entity::find()
            .filter(team::Column::Id.is_in(team_ids.into_iter().collect::<Vec<_>>()))
            .all(db)
            .await
            .map_err(db_error)?
    };

    let club_names = clubs.into_iter().map(|club| (club.id, club.name)).collect::<BTreeMap<_, _>>();
    let group_names = groups.into_iter().map(|group| (group.id, group.name)).collect::<BTreeMap<_, _>>();
    let team_names = teams.into_iter().map(|team| (team.id, team.name)).collect::<BTreeMap<_, _>>();

    Ok(items
        .into_iter()
        .map(|item| TrainingSessionSummary {
            id: item.id,
            club_id: item.club_id,
            club_name: club_names
                .get(&item.club_id)
                .cloned()
                .unwrap_or_else(|| "Unbekannter Verein".to_string()),
            group_id: item.group_id,
            group_name: group_names
                .get(&item.group_id)
                .cloned()
                .unwrap_or_else(|| "Unbekannte Gruppe".to_string()),
            team_id: item.team_id,
            team_name: item.team_id.and_then(|team_id| team_names.get(&team_id).cloned()),
            title: item.title,
            description: item.description,
            location: item.location,
            start_at: item.start_at,
            end_at: item.end_at,
            status: item.status,
            created_by_user_id: item.created_by_user_id,
        })
        .collect())
}

fn normalize_required_text(label: &str, value: &str, min_len: usize, max_len: usize) -> Result<String, String> {
    let value = value.trim();

    if value.len() < min_len {
        return Err(format!("{label} muss mindestens {min_len} Zeichen lang sein."));
    }

    if value.len() > max_len {
        return Err(format!("{label} darf hoechstens {max_len} Zeichen lang sein."));
    }

    Ok(value.to_string())
}

fn normalize_optional_text(label: &str, value: &str, max_len: usize) -> Result<String, String> {
    let value = value.trim();

    if value.len() > max_len {
        return Err(format!("{label} darf hoechstens {max_len} Zeichen lang sein."));
    }

    Ok(value.to_string())
}

fn parse_datetime_input(label: &str, value: &str) -> Result<i64, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label} ist erforderlich."));
    }

    let (date_part, time_part) = value
        .split_once('T')
        .or_else(|| value.split_once(' '))
        .ok_or_else(|| format!("{label} muss im Format JJJJ-MM-TTTHH:MM angegeben werden."))?;

    let mut date_parts = date_part.split('-');
    let year = parse_i32_part(date_parts.next(), label, "Jahr")?;
    let month = parse_u8_part(date_parts.next(), label, "Monat")?;
    let day = parse_u8_part(date_parts.next(), label, "Tag")?;
    if date_parts.next().is_some() {
        return Err(format!("{label} enthaelt ein ungueltiges Datum."));
    }

    let mut time_parts = time_part.split(':');
    let hour = parse_u8_part(time_parts.next(), label, "Stunde")?;
    let minute = parse_u8_part(time_parts.next(), label, "Minute")?;
    let second = match time_parts.next() {
        Some(second) if !second.is_empty() => second
            .split('.')
            .next()
            .ok_or_else(|| format!("{label} enthaelt ungueltige Sekunden."))?
            .parse::<u8>()
            .map_err(|_| format!("{label} enthaelt ungueltige Sekunden."))?,
        _ => 0,
    };

    let month = Month::try_from(month).map_err(|_| format!("{label} enthaelt einen ungueltigen Monat."))?;
    let date = Date::from_calendar_date(year, month, day)
        .map_err(|_| format!("{label} enthaelt ein ungueltiges Datum."))?;
    let time = Time::from_hms(hour, minute, second)
        .map_err(|_| format!("{label} enthaelt eine ungueltige Uhrzeit."))?;

    Ok(PrimitiveDateTime::new(date, time).assume_utc().unix_timestamp())
}

fn parse_i32_part(value: Option<&str>, label: &str, field: &str) -> Result<i32, String> {
    value
        .ok_or_else(|| format!("{label} enthaelt kein gueltiges {field}."))?
        .parse::<i32>()
        .map_err(|_| format!("{label} enthaelt kein gueltiges {field}."))
}

fn parse_u8_part(value: Option<&str>, label: &str, field: &str) -> Result<u8, String> {
    value
        .ok_or_else(|| format!("{label} enthaelt keine gueltige {field}."))?
        .parse::<u8>()
        .map_err(|_| format!("{label} enthaelt keine gueltige {field}."))
}

fn training_summary(
    item: training_session::Model,
    club_name: &str,
    group_name: &str,
    team_name: Option<&str>,
) -> TrainingSessionSummary {
    TrainingSessionSummary {
        id: item.id,
        club_id: item.club_id,
        club_name: club_name.to_string(),
        group_id: item.group_id,
        group_name: group_name.to_string(),
        team_id: item.team_id,
        team_name: team_name.map(ToString::to_string),
        title: item.title,
        description: item.description,
        location: item.location,
        start_at: item.start_at,
        end_at: item.end_at,
        status: item.status,
        created_by_user_id: item.created_by_user_id,
    }
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
