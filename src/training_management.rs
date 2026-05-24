use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedTrainingTemplateSummary {
    pub id: i32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingTemplateSummary {
    pub id: i32,
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub description: String,
    pub number_of_throws: Option<i32>,
    pub target_score: Option<i32>,
    pub standing_pins: Option<Vec<u8>>,
    pub clear_pins: Option<bool>,
    pub created_by_user_id: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainingPlanSummary {
    pub id: i32,
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub day: String,
    pub note: String,
    pub trainer_user_id: Option<i32>,
    pub trainer_username: Option<String>,
    pub created_by_user_id: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub templates: Vec<LinkedTrainingTemplateSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTrainingTemplateInput {
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub description: String,
    pub number_of_throws: Option<i32>,
    pub target_score: Option<i32>,
    pub standing_pins: Option<Vec<u8>>,
    pub clear_pins: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateTrainingTemplateInput {
    pub template_id: i32,
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub description: String,
    pub number_of_throws: Option<i32>,
    pub target_score: Option<i32>,
    pub standing_pins: Option<Vec<u8>>,
    pub clear_pins: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTrainingPlanInput {
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub day: String,
    pub note: String,
    pub trainer_user_id: Option<i32>,
    pub template_ids: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateTrainingPlanInput {
    pub plan_id: i32,
    pub club_id: i32,
    pub group_id: i32,
    pub title: String,
    pub day: String,
    pub note: String,
    pub trainer_user_id: Option<i32>,
    pub template_ids: Vec<i32>,
}

#[post("/api/training/templates/list")]
pub async fn list_training_templates(group_id: i32) -> Result<Vec<TrainingTemplateSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::list_templates(group_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = group_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/templates/create")]
pub async fn create_training_template(
    input: CreateTrainingTemplateInput,
) -> Result<TrainingTemplateSummary> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::create_template(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/templates/update")]
pub async fn update_training_template(
    input: UpdateTrainingTemplateInput,
) -> Result<TrainingTemplateSummary> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::update_template(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/templates/delete")]
pub async fn delete_training_template(template_id: i32) -> Result<()> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::delete_template(template_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = template_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/plans/list")]
pub async fn list_training_plans(group_id: i32) -> Result<Vec<TrainingPlanSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::list_plans(group_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = group_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/plans/create")]
pub async fn create_training_plan(input: CreateTrainingPlanInput) -> Result<TrainingPlanSummary> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::create_plan(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/plans/update")]
pub async fn update_training_plan(input: UpdateTrainingPlanInput) -> Result<TrainingPlanSummary> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::update_plan(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/training/plans/delete")]
pub async fn delete_training_plan(plan_id: i32) -> Result<()> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::training_management::delete_plan(plan_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = plan_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
