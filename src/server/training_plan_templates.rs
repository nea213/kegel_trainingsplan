#![allow(dead_code)]

use crate::server::{
    auth::now_timestamp,
    entities::{training_plan, training_plan_template, training_template},
};
use sea_orm::ActiveValue::Set;
use std::collections::BTreeSet;
use time::{Date, Month};

const MAX_TITLE_LEN: usize = 120;
const MAX_DESCRIPTION_LEN: usize = 2_000;
const MAX_NOTE_LEN: usize = 2_000;
const MAX_STANDING_PIN: u8 = 9;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TrainingTemplateDraft {
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub description: String,
    pub number_of_throws: Option<i32>,
    pub target_score: Option<i32>,
    pub standing_pins: Option<Vec<u8>>,
    pub clear_pins: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TrainingPlanDraft {
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub day: String,
    pub note: String,
    pub trainer_user_id: Option<i32>,
}

pub(crate) fn create_training_template_model(
    input: TrainingTemplateDraft,
    created_by_user_id: i32,
) -> Result<training_template::ActiveModel, String> {
    // TODO: Define who can create, read, update, and delete training templates.
    let title = normalize_required_text("Der Vorlagentitel", &input.title, 1, MAX_TITLE_LEN)?;
    let description = normalize_optional_text("Die Beschreibung", &input.description, MAX_DESCRIPTION_LEN)?;
    let number_of_throws =
        validate_optional_non_negative("Die Wurfanzahl", input.number_of_throws)?;
    let target_score = validate_optional_non_negative("Die Zielpunktzahl", input.target_score)?;
    let standing_pins_mask = standing_pins_to_mask(input.standing_pins.as_deref())?;
    let now = now_timestamp();

    Ok(training_template::ActiveModel {
        club_id: Set(input.club_id),
        group_id: Set(input.group_id),
        title: Set(title),
        description: Set(description),
        number_of_throws: Set(number_of_throws),
        target_score: Set(target_score),
        standing_pins_mask: Set(standing_pins_mask),
        clear_pins: Set(input.clear_pins),
        created_by_user_id: Set(created_by_user_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    })
}

pub(crate) fn update_training_template_model(
    model: &mut training_template::ActiveModel,
    input: TrainingTemplateDraft,
) -> Result<(), String> {
    // TODO: Define who can create, read, update, and delete training templates.
    model.club_id = Set(input.club_id);
    model.group_id = Set(input.group_id);
    model.title = Set(normalize_required_text(
        "Der Vorlagentitel",
        &input.title,
        1,
        MAX_TITLE_LEN,
    )?);
    model.description = Set(normalize_optional_text(
        "Die Beschreibung",
        &input.description,
        MAX_DESCRIPTION_LEN,
    )?);
    model.number_of_throws = Set(validate_optional_non_negative(
        "Die Wurfanzahl",
        input.number_of_throws,
    )?);
    model.target_score = Set(validate_optional_non_negative(
        "Die Zielpunktzahl",
        input.target_score,
    )?);
    model.standing_pins_mask = Set(standing_pins_to_mask(input.standing_pins.as_deref())?);
    model.clear_pins = Set(input.clear_pins);
    model.updated_at = Set(now_timestamp());

    Ok(())
}

pub(crate) fn create_training_plan_model(
    input: TrainingPlanDraft,
    created_by_user_id: i32,
) -> Result<training_plan::ActiveModel, String> {
    // TODO: Define whether training-plan trainer assignment must be restricted to group trainers.
    let title = normalize_required_text("Der Planname", &input.title, 1, MAX_TITLE_LEN)?;
    let day = validate_day(&input.day)?;
    let note = normalize_optional_text("Die Notiz", &input.note, MAX_NOTE_LEN)?;
    let now = now_timestamp();

    Ok(training_plan::ActiveModel {
        club_id: Set(input.club_id),
        group_id: Set(input.group_id),
        title: Set(title),
        day: Set(day),
        note: Set(note),
        trainer_user_id: Set(input.trainer_user_id),
        created_by_user_id: Set(created_by_user_id),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    })
}

pub(crate) fn update_training_plan_model(
    model: &mut training_plan::ActiveModel,
    input: TrainingPlanDraft,
) -> Result<(), String> {
    // TODO: Define whether training-plan trainer assignment must be restricted to group trainers.
    model.club_id = Set(input.club_id);
    model.group_id = Set(input.group_id);
    model.title = Set(normalize_required_text("Der Planname", &input.title, 1, MAX_TITLE_LEN)?);
    model.day = Set(validate_day(&input.day)?);
    model.note = Set(normalize_optional_text("Die Notiz", &input.note, MAX_NOTE_LEN)?);
    model.trainer_user_id = Set(input.trainer_user_id);
    model.updated_at = Set(now_timestamp());

    Ok(())
}

pub(crate) fn create_training_plan_template_model(
    training_plan_id: i32,
    training_template_id: i32,
) -> training_plan_template::ActiveModel {
    training_plan_template::ActiveModel {
        training_plan_id: Set(training_plan_id),
        training_template_id: Set(training_template_id),
        created_at: Set(now_timestamp()),
        ..Default::default()
    }
}

pub(crate) fn standing_pins_to_mask(pins: Option<&[u8]>) -> Result<Option<i32>, String> {
    let Some(pins) = pins else {
        return Ok(None);
    };

    if pins.is_empty() {
        return Err("Es muss mindestens ein stehender Kegel ausgewählt werden.".to_string());
    }

    let mut seen = BTreeSet::new();
    let mut mask = 0_i32;

    for pin in pins {
        if *pin == 0 || *pin > MAX_STANDING_PIN {
            return Err("Stehende Kegel dürfen nur Werte von 1 bis 9 enthalten.".to_string());
        }

        if !seen.insert(*pin) {
            return Err("Stehende Kegel dürfen nicht doppelt angegeben werden.".to_string());
        }

        mask |= 1_i32 << (pin - 1);
    }

    Ok(Some(mask))
}

pub(crate) fn standing_pins_from_mask(mask: Option<i32>) -> Result<Option<Vec<u8>>, String> {
    let Some(mask) = mask else {
        return Ok(None);
    };

    if mask <= 0 || mask & !((1_i32 << MAX_STANDING_PIN) - 1) != 0 {
        return Err("Die gespeicherten stehenden Kegel sind ungültig.".to_string());
    }

    let pins = (1..=MAX_STANDING_PIN)
        .filter(|pin| mask & (1_i32 << (pin - 1)) != 0)
        .collect::<Vec<_>>();

    Ok(Some(pins))
}

pub(crate) fn validate_day(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("Der Trainingstag ist erforderlich.".to_string());
    }

    let mut parts = value.split('-');
    let year = parse_i32_part(parts.next(), "Der Trainingstag", "Jahr")?;
    let month = parse_u8_part(parts.next(), "Der Trainingstag", "Monat")?;
    let day = parse_u8_part(parts.next(), "Der Trainingstag", "Tag")?;

    if parts.next().is_some() {
        return Err("Der Trainingstag muss im Format JJJJ-MM-TT angegeben werden.".to_string());
    }

    let month = Month::try_from(month)
        .map_err(|_| "Der Trainingstag enthält einen ungültigen Monat.".to_string())?;
    Date::from_calendar_date(year, month, day)
        .map_err(|_| "Der Trainingstag enthält ein ungültiges Datum.".to_string())?;

    Ok(value.to_string())
}

fn normalize_required_text(label: &str, value: &str, min_len: usize, max_len: usize) -> Result<String, String> {
    let value = value.trim();

    if value.len() < min_len {
        return Err(format!("{label} ist erforderlich."));
    }

    if value.len() > max_len {
        return Err(format!("{label} darf höchstens {max_len} Zeichen lang sein."));
    }

    Ok(value.to_string())
}

fn normalize_optional_text(label: &str, value: &str, max_len: usize) -> Result<String, String> {
    let value = value.trim();

    if value.len() > max_len {
        return Err(format!("{label} darf höchstens {max_len} Zeichen lang sein."));
    }

    Ok(value.to_string())
}

fn validate_optional_non_negative(label: &str, value: Option<i32>) -> Result<Option<i32>, String> {
    match value {
        Some(value) if value < 0 => Err(format!("{label} darf nicht negativ sein.")),
        Some(value) => Ok(Some(value)),
        None => Ok(None),
    }
}

fn parse_i32_part(value: Option<&str>, label: &str, field: &str) -> Result<i32, String> {
    value
        .ok_or_else(|| format!("{label} enthält kein gültiges {field}."))?
        .parse::<i32>()
        .map_err(|_| format!("{label} enthält kein gültiges {field}."))
}

fn parse_u8_part(value: Option<&str>, label: &str, field: &str) -> Result<u8, String> {
    value
        .ok_or_else(|| format!("{label} enthält keine gültige {field}."))?
        .parse::<u8>()
        .map_err(|_| format!("{label} enthält keine gültige {field}."))
}

#[cfg(test)]
mod tests {
    use super::{
        standing_pins_from_mask, standing_pins_to_mask, validate_day, TrainingPlanDraft,
        TrainingTemplateDraft,
    };
    use crate::server::training_plan_templates::{
        create_training_plan_model, create_training_template_model,
    };
    use sea_orm::ActiveValue::Set;

    #[test]
    fn standing_pins_round_trip() {
        let mask = standing_pins_to_mask(Some(&[1, 4, 9])).expect("valid mask");
        assert_eq!(mask, Some(265));

        let pins = standing_pins_from_mask(mask).expect("valid pins");
        assert_eq!(pins, Some(vec![1, 4, 9]));
    }

    #[test]
    fn standing_pins_reject_out_of_range() {
        let error = standing_pins_to_mask(Some(&[10])).expect_err("out of range should fail");
        assert!(error.contains("1 bis 9"));
    }

    #[test]
    fn standing_pins_reject_duplicates() {
        let error = standing_pins_to_mask(Some(&[2, 2])).expect_err("duplicates should fail");
        assert!(error.contains("doppelt"));
    }

    #[test]
    fn validate_day_accepts_valid_date() {
        assert_eq!(
            validate_day("2026-05-23").expect("valid day"),
            "2026-05-23".to_string()
        );
    }

    #[test]
    fn validate_day_rejects_invalid_date() {
        let error = validate_day("2026-02-30").expect_err("invalid date should fail");
        assert!(error.contains("ungültiges Datum"));
    }

    #[test]
    fn create_template_model_allows_nullable_fields() {
        let model = create_training_template_model(
            TrainingTemplateDraft {
                club_id: 1,
                group_id: 2,
                title: "Abräum-Spiel".to_string(),
                description: String::new(),
                number_of_throws: None,
                target_score: None,
                standing_pins: None,
                clear_pins: None,
            },
            7,
        )
        .expect("template model should be valid");

        assert_eq!(model.number_of_throws, Set(None));
        assert_eq!(model.target_score, Set(None));
        assert_eq!(model.standing_pins_mask, Set(None));
        assert_eq!(model.clear_pins, Set(None));
    }

    #[test]
    fn create_plan_model_allows_nullable_fields() {
        let model = create_training_plan_model(
            TrainingPlanDraft {
                club_id: 1,
                group_id: 2,
                title: "Dienstag".to_string(),
                day: "2026-05-23".to_string(),
                note: String::new(),
                trainer_user_id: None,
            },
            7,
        )
        .expect("plan model should be valid");

        assert_eq!(model.trainer_user_id, Set(None));
    }
}
