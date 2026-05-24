use crate::server::{
    auth::{hash_password, normalize_username, now_timestamp, validate_password},
    entities::{
        club, club_group, club_membership, group_trainer, team, team_player, training_plan,
        training_plan_template, training_template, user,
    },
    training_plan_templates::{
        create_training_plan_model, create_training_plan_template_model,
        create_training_template_model, TrainingPlanDraft, TrainingTemplateDraft,
    },
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use std::env;
use time::{Duration, OffsetDateTime};

const SEED_DEV_DATA: &str = "SEED_DEV_DATA";
const DEV_PASSWORD: &str = "Testpasswort123";
const CLUB_NAME: &str = "KC Musterstadt";
const GROUP_JUGEND: &str = "Jugend";
const GROUP_ERWACHSENE: &str = "Erwachsene";
const TEAM_JUGEND_A: &str = "Jugend A";
const TEAM_JUGEND_B: &str = "Jugend B";
const TEAM_HERREN_1: &str = "Herren 1";
const TEAM_DAMEN_1: &str = "Damen 1";
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

    let jugend_technik = ensure_training_template(
        db,
        TrainingTemplateSeed {
            club_id: club.id,
            group_id: jugend.id,
            title: "Jugend Technikserie",
            description: "Fokus auf Anlauf, Rhythmus und ruhige Kugelabgabe.",
            number_of_throws: Some(60),
            target_score: Some(520),
            standing_pins: None,
            clear_pins: Some(false),
            created_by_user_id: trainer_jugend.id,
        },
    )
    .await?;
    let jugend_abräumen = ensure_training_template(
        db,
        TrainingTemplateSeed {
            club_id: club.id,
            group_id: jugend.id,
            title: "Jugend Abräumen 3 Bilder",
            description: "Abräumblock mit wechselnden Bildern und kurzer Besprechung nach jeder Serie.",
            number_of_throws: Some(36),
            target_score: Some(140),
            standing_pins: Some(&[1, 2, 5, 8]),
            clear_pins: Some(true),
            created_by_user_id: trainer_jugend.id,
        },
    )
    .await?;
    let erwachsene_volle = ensure_training_template(
        db,
        TrainingTemplateSeed {
            club_id: club.id,
            group_id: erwachsene.id,
            title: "Volle 120 Wurf",
            description: "Wettkampfnaher Block mit Serienrhythmus und kurzer Pausensteuerung.",
            number_of_throws: Some(120),
            target_score: Some(860),
            standing_pins: None,
            clear_pins: Some(false),
            created_by_user_id: trainer_erwachsene.id,
        },
    )
    .await?;
    let erwachsene_bilder = ensure_training_template(
        db,
        TrainingTemplateSeed {
            club_id: club.id,
            group_id: erwachsene.id,
            title: "Abräumen schwierige Bilder",
            description: "Präzisionsübungen auf wechselnde Bilder im Abräumen.",
            number_of_throws: Some(48),
            target_score: Some(180),
            standing_pins: Some(&[3, 6, 7, 9]),
            clear_pins: Some(true),
            created_by_user_id: trainer_erwachsene.id,
        },
    )
    .await?;

    let today = OffsetDateTime::from_unix_timestamp(now)
        .map_err(db_error)?
        .date();
    let jugend_plan_day = (today + Duration::days(1))
        .format(&time::format_description::parse("[year]-[month]-[day]").map_err(db_error)?)
        .map_err(db_error)?;
    let erwachsene_plan_day = (today + Duration::days(3))
        .format(&time::format_description::parse("[year]-[month]-[day]").map_err(db_error)?)
        .map_err(db_error)?;

    let jugend_plan = ensure_training_plan(
        db,
        TrainingPlanSeed {
            club_id: club.id,
            group_id: jugend.id,
            title: "Jugend Freitagstraining",
            day: &jugend_plan_day,
            note: "Technik zuerst, danach Abräumen in kurzen Blöcken.",
            trainer_user_id: Some(trainer_jugend.id),
            created_by_user_id: trainer_jugend.id,
        },
    )
    .await?;
    let erwachsene_plan = ensure_training_plan(
        db,
        TrainingPlanSeed {
            club_id: club.id,
            group_id: erwachsene.id,
            title: "Erwachsene Wettkampfblock",
            day: &erwachsene_plan_day,
            note: "Wettkampfnaher Aufbau mit Abschluss auf schwierige Bilder.",
            trainer_user_id: Some(trainer_erwachsene.id),
            created_by_user_id: trainer_erwachsene.id,
        },
    )
    .await?;

    ensure_training_plan_template(db, jugend_plan.id, jugend_technik.id).await?;
    ensure_training_plan_template(db, jugend_plan.id, jugend_abräumen.id).await?;
    ensure_training_plan_template(db, erwachsene_plan.id, erwachsene_volle.id).await?;
    ensure_training_plan_template(db, erwachsene_plan.id, erwachsene_bilder.id).await?;

    println!(
        "Entwicklungsdaten wurden sichergestellt. Test-Login: trainer.jugend / {}",
        DEV_PASSWORD
    );

    Ok(())
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

struct TrainingTemplateSeed<'a> {
    club_id: i32,
    group_id: i32,
    title: &'a str,
    description: &'a str,
    number_of_throws: Option<i32>,
    target_score: Option<i32>,
    standing_pins: Option<&'a [u8]>,
    clear_pins: Option<bool>,
    created_by_user_id: i32,
}

struct TrainingPlanSeed<'a> {
    club_id: i32,
    group_id: i32,
    title: &'a str,
    day: &'a str,
    note: &'a str,
    trainer_user_id: Option<i32>,
    created_by_user_id: i32,
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

async fn ensure_training_template(
    db: &DatabaseConnection,
    seed: TrainingTemplateSeed<'_>,
) -> Result<training_template::Model, DbErr> {
    if let Some(existing) = training_template::Entity::find()
        .filter(training_template::Column::GroupId.eq(seed.group_id))
        .filter(training_template::Column::Title.eq(seed.title))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    let model = create_training_template_model(
        TrainingTemplateDraft {
            club_id: seed.club_id,
            group_id: seed.group_id,
            title: seed.title.to_string(),
            description: seed.description.to_string(),
            number_of_throws: seed.number_of_throws,
            target_score: seed.target_score,
            standing_pins: seed.standing_pins.map(|pins| pins.to_vec()),
            clear_pins: seed.clear_pins,
        },
        seed.created_by_user_id,
    )
    .map_err(db_error)?;

    model.insert(db).await
}

async fn ensure_training_plan(
    db: &DatabaseConnection,
    seed: TrainingPlanSeed<'_>,
) -> Result<training_plan::Model, DbErr> {
    if let Some(existing) = training_plan::Entity::find()
        .filter(training_plan::Column::GroupId.eq(seed.group_id))
        .filter(training_plan::Column::Title.eq(seed.title))
        .filter(training_plan::Column::Day.eq(seed.day))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    let model = create_training_plan_model(
        TrainingPlanDraft {
            club_id: seed.club_id,
            group_id: seed.group_id,
            title: seed.title.to_string(),
            day: seed.day.to_string(),
            note: seed.note.to_string(),
            trainer_user_id: seed.trainer_user_id,
        },
        seed.created_by_user_id,
    )
    .map_err(db_error)?;

    model.insert(db).await
}

async fn ensure_training_plan_template(
    db: &DatabaseConnection,
    training_plan_id: i32,
    training_template_id: i32,
) -> Result<(), DbErr> {
    if training_plan_template::Entity::find()
        .filter(training_plan_template::Column::TrainingPlanId.eq(training_plan_id))
        .filter(training_plan_template::Column::TrainingTemplateId.eq(training_template_id))
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    create_training_plan_template_model(training_plan_id, training_template_id)
        .insert(db)
        .await?;

    Ok(())
}

fn db_error(error: impl std::fmt::Display) -> DbErr {
    DbErr::Custom(error.to_string())
}
