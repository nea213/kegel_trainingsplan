use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use time::{Month, OffsetDateTime};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingSessionSummary {
    pub id: i32,
    pub club_id: i32,
    pub club_name: String,
    pub group_id: i32,
    pub group_name: String,
    pub team_id: Option<i32>,
    pub team_name: Option<String>,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_at: i64,
    pub end_at: i64,
    pub status: String,
    pub created_by_user_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTrainingSessionInput {
    pub club_id: i32,
    pub group_id: i32,
    pub team_id: Option<i32>,
    pub title: String,
    pub description: String,
    pub location: String,
    pub start_at: String,
    pub end_at: String,
}

#[post("/api/training/create")]
pub async fn create_training_session(input: CreateTrainingSessionInput) -> Result<TrainingSessionSummary> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training::create(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/group")]
pub async fn list_group_training_sessions(group_id: i32) -> Result<Vec<TrainingSessionSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training::list_for_group(group_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = group_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/mine")]
pub async fn list_my_training_sessions() -> Result<Vec<TrainingSessionSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training::list_for_viewer()
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

pub fn format_training_range(start_at: i64, end_at: i64) -> String {
    format!("{} bis {}", format_timestamp_label(start_at), format_timestamp_label(end_at))
}

pub fn training_scope_label(training: &TrainingSessionSummary) -> String {
    match &training.team_name {
        Some(team_name) => format!("{} | {}", training.group_name, team_name),
        None => format!("{} | ganze Gruppe", training.group_name),
    }
}

pub fn format_timestamp_label(timestamp: i64) -> String {
    let Ok(date_time) = OffsetDateTime::from_unix_timestamp(timestamp) else {
        return "Unbekannte Zeit".to_string();
    };

    format!(
        "{:02}.{:02}.{} {:02}:{:02}",
        date_time.day(),
        month_number(date_time.month()),
        date_time.year(),
        date_time.hour(),
        date_time.minute(),
    )
}

fn month_number(month: Month) -> u8 {
    match month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    }
}
