use crate::server::{
    auth::{hash_password, normalize_username, now_timestamp, validate_password},
    entities::{
        club, club_group, club_membership, group_trainer, team, team_player, training_session, user,
    },
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use std::env;

const SEED_DEV_DATA: &str = "SEED_DEV_DATA";
const DEV_PASSWORD: &str = "Testpasswort123";
const CLUB_NAME: &str = "KC Musterstadt";
const GROUP_JUGEND: &str = "Jugend";
const GROUP_ERWACHSENE: &str = "Erwachsene";
const TEAM_JUGEND_A: &str = "Jugend A";
const TEAM_JUGEND_B: &str = "Jugend B";
const TEAM_HERREN_1: &str = "Herren 1";
const TEAM_DAMEN_1: &str = "Damen 1";
const STATUS_PLANNED: &str = "planned";

pub async fn seed_dev_data(db: &DatabaseConnection) -> Result<(), DbErr> {
    if !seed_dev_data_enabled() {
        return Ok(());
    }

    let now = now_timestamp();

    let club = ensure_club(db, CLUB_NAME, now).await?;
    let jugend = ensure_group(db, club.id, GROUP_JUGEND, 0, now).await?;
    let erwachsene = ensure_group(db, club.id, GROUP_ERWACHSENE, 1, now).await?;

    let jugend_a = ensure_team(db, club.id, jugend.id, TEAM_JUGEND_A, 0, now).await?;
    let jugend_b = ensure_team(db, club.id, jugend.id, TEAM_JUGEND_B, 1, now).await?;
    let herren_1 = ensure_team(db, club.id, erwachsene.id, TEAM_HERREN_1, 0, now).await?;
    let damen_1 = ensure_team(db, club.id, erwachsene.id, TEAM_DAMEN_1, 1, now).await?;

    let trainer_jugend = ensure_user(db, "trainer.jugend", DEV_PASSWORD, false, now).await?;
    let trainer_erwachsene = ensure_user(db, "trainer.erwachsene", DEV_PASSWORD, false, now).await?;
    let spieler_anna = ensure_user(db, "anna", DEV_PASSWORD, false, now).await?;
    let spieler_ben = ensure_user(db, "ben", DEV_PASSWORD, false, now).await?;
    let spieler_clara = ensure_user(db, "clara", DEV_PASSWORD, false, now).await?;
    let spieler_david = ensure_user(db, "david", DEV_PASSWORD, false, now).await?;
    let spieler_ella = ensure_user(db, "ella", DEV_PASSWORD, false, now).await?;
    let spieler_finn = ensure_user(db, "finn", DEV_PASSWORD, false, now).await?;

    for member in [
        trainer_jugend.id,
        trainer_erwachsene.id,
        spieler_anna.id,
        spieler_ben.id,
        spieler_clara.id,
        spieler_david.id,
        spieler_ella.id,
        spieler_finn.id,
    ] {
        ensure_club_membership(db, club.id, member, now).await?;
    }

    ensure_group_trainer(db, jugend.id, trainer_jugend.id, now).await?;
    ensure_group_trainer(db, erwachsene.id, trainer_erwachsene.id, now).await?;

    ensure_team_player(db, jugend_a.id, spieler_anna.id, now).await?;
    ensure_team_player(db, jugend_a.id, spieler_ben.id, now).await?;
    ensure_team_player(db, jugend_b.id, spieler_clara.id, now).await?;
    ensure_team_player(db, jugend_b.id, spieler_david.id, now).await?;
    ensure_team_player(db, herren_1.id, spieler_finn.id, now).await?;
    ensure_team_player(db, damen_1.id, spieler_ella.id, now).await?;

    ensure_training_session(
        db,
        TrainingSeed {
            club_id: club.id,
            group_id: jugend.id,
            team_id: None,
            title: "Jugendtechnik am Freitag",
            description: "Grundlagentraining für Anlauf, Rhythmus und Räumspiel.",
            location: "Kegelhalle Musterstadt",
            start_at: now + 86_400,
            end_at: now + 93_600,
            created_by_user_id: trainer_jugend.id,
        },
        now,
    )
    .await?;
    ensure_training_session(
        db,
        TrainingSeed {
            club_id: club.id,
            group_id: jugend.id,
            team_id: Some(jugend_a.id),
            title: "Jugend A Wurfserie",
            description: "Serien über die Vollen mit Fokus auf saubere Ausführung.",
            location: "Bahn 1 und 2",
            start_at: now + 172_800,
            end_at: now + 180_000,
            created_by_user_id: trainer_jugend.id,
        },
        now,
    )
    .await?;
    ensure_training_session(
        db,
        TrainingSeed {
            club_id: club.id,
            group_id: erwachsene.id,
            team_id: None,
            title: "Abendtraining Erwachsene",
            description: "Gemeinsames Training mit Ausdauerblock und Abräumen.",
            location: "Kegelhalle Musterstadt",
            start_at: now + 259_200,
            end_at: now + 266_400,
            created_by_user_id: trainer_erwachsene.id,
        },
        now,
    )
    .await?;
    ensure_training_session(
        db,
        TrainingSeed {
            club_id: club.id,
            group_id: erwachsene.id,
            team_id: Some(herren_1.id),
            title: "Herren 1 Wettkampfvorbereitung",
            description: "Wettkampfnahes Training über 120 Wurf mit Pausenrhythmus.",
            location: "Bahn 3 und 4",
            start_at: now + 345_600,
            end_at: now + 352_800,
            created_by_user_id: trainer_erwachsene.id,
        },
        now,
    )
    .await?;
    ensure_training_session(
        db,
        TrainingSeed {
            club_id: club.id,
            group_id: erwachsene.id,
            team_id: Some(damen_1.id),
            title: "Damen 1 Präzisionstraining",
            description: "Präzisionsübungen auf wechselnde Bilder im Abräumen.",
            location: "Bahn 5 und 6",
            start_at: now + 432_000,
            end_at: now + 439_200,
            created_by_user_id: trainer_erwachsene.id,
        },
        now,
    )
    .await?;

    println!(
        "Entwicklungsdaten wurden sichergestellt. Test-Login: trainer.jugend / {}",
        DEV_PASSWORD
    );

    Ok(())
}

struct TrainingSeed<'a> {
    club_id: i32,
    group_id: i32,
    team_id: Option<i32>,
    title: &'a str,
    description: &'a str,
    location: &'a str,
    start_at: i64,
    end_at: i64,
    created_by_user_id: i32,
}

fn seed_dev_data_enabled() -> bool {
    env::var(SEED_DEV_DATA)
        .ok()
        .and_then(|value| match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(false)
}

async fn ensure_user(
    db: &DatabaseConnection,
    username: &str,
    password: &str,
    is_system_admin: bool,
    now: i64,
) -> Result<user::Model, DbErr> {
    let username = normalize_username(username).map_err(db_error)?;

    if let Some(existing) = user::Entity::find()
        .filter(user::Column::Username.eq(username.clone()))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    validate_password(password).map_err(db_error)?;
    let password_hash = hash_password(password).map_err(db_error)?;

    user::ActiveModel {
        username: Set(username),
        password_hash: Set(password_hash),
        theme_mode: Set("system".to_string()),
        is_system_admin: Set(is_system_admin),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
}

async fn ensure_club(db: &DatabaseConnection, name: &str, now: i64) -> Result<club::Model, DbErr> {
    if let Some(existing) = club::Entity::find()
        .filter(club::Column::Name.eq(name))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    club::ActiveModel {
        name: Set(name.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
}

async fn ensure_group(
    db: &DatabaseConnection,
    club_id: i32,
    name: &str,
    sort_order: i32,
    now: i64,
) -> Result<club_group::Model, DbErr> {
    if let Some(existing) = club_group::Entity::find()
        .filter(club_group::Column::ClubId.eq(club_id))
        .filter(club_group::Column::Name.eq(name))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    club_group::ActiveModel {
        club_id: Set(club_id),
        name: Set(name.to_string()),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
}

async fn ensure_team(
    db: &DatabaseConnection,
    club_id: i32,
    group_id: i32,
    name: &str,
    sort_order: i32,
    now: i64,
) -> Result<team::Model, DbErr> {
    if let Some(existing) = team::Entity::find()
        .filter(team::Column::GroupId.eq(group_id))
        .filter(team::Column::Name.eq(name))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    team::ActiveModel {
        club_id: Set(club_id),
        group_id: Set(group_id),
        name: Set(name.to_string()),
        sort_order: Set(sort_order),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
}

async fn ensure_club_membership(
    db: &DatabaseConnection,
    club_id: i32,
    user_id: i32,
    now: i64,
) -> Result<(), DbErr> {
    if club_membership::Entity::find()
        .filter(club_membership::Column::ClubId.eq(club_id))
        .filter(club_membership::Column::UserId.eq(user_id))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    club_membership::ActiveModel {
        club_id: Set(club_id),
        user_id: Set(user_id),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn ensure_group_trainer(
    db: &DatabaseConnection,
    group_id: i32,
    user_id: i32,
    now: i64,
) -> Result<(), DbErr> {
    if group_trainer::Entity::find()
        .filter(group_trainer::Column::GroupId.eq(group_id))
        .filter(group_trainer::Column::UserId.eq(user_id))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    group_trainer::ActiveModel {
        group_id: Set(group_id),
        user_id: Set(user_id),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn ensure_team_player(
    db: &DatabaseConnection,
    team_id: i32,
    user_id: i32,
    now: i64,
) -> Result<(), DbErr> {
    if team_player::Entity::find()
        .filter(team_player::Column::TeamId.eq(team_id))
        .filter(team_player::Column::UserId.eq(user_id))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    team_player::ActiveModel {
        team_id: Set(team_id),
        user_id: Set(user_id),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

async fn ensure_training_session(
    db: &DatabaseConnection,
    seed: TrainingSeed<'_>,
    now: i64,
) -> Result<(), DbErr> {
    if training_session::Entity::find()
        .filter(training_session::Column::GroupId.eq(seed.group_id))
        .filter(training_session::Column::Title.eq(seed.title))
        .filter(training_session::Column::StartAt.eq(seed.start_at))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    training_session::ActiveModel {
        club_id: Set(seed.club_id),
        group_id: Set(seed.group_id),
        team_id: Set(seed.team_id),
        title: Set(seed.title.to_string()),
        description: Set(seed.description.to_string()),
        location: Set(seed.location.to_string()),
        start_at: Set(seed.start_at),
        end_at: Set(seed.end_at),
        status: Set(STATUS_PLANNED.to_string()),
        created_by_user_id: Set(seed.created_by_user_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?;

    Ok(())
}

fn db_error(error: impl std::fmt::Display) -> DbErr {
    DbErr::Custom(error.to_string())
}
