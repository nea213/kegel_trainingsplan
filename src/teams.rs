use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamSummary {
    pub id: i32,
    pub club_id: i32,
    pub group_id: i32,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateTeamInput {
    pub club_id: i32,
    pub group_id: i32,
    pub name: String,
    pub sort_order: i32,
}

#[post("/api/teams/create")]
pub async fn create_team(input: CreateTeamInput) -> Result<TeamSummary> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::teams::create(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/teams/list")]
pub async fn list_teams_for_group(group_id: i32) -> Result<Vec<TeamSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::teams::list_for_group(group_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = group_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
