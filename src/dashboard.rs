use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{auth::PublicUser, teams::TeamSummary};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedGroupSummary {
    pub group_id: i32,
    pub club_id: i32,
    pub club_name: String,
    pub group_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubMembershipSummary {
    pub club_id: i32,
    pub club_name: String,
    pub user_id: i32,
    pub username: String,
    pub is_assigned_to_team: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerContext {
    pub user: PublicUser,
    pub managed_groups: Vec<ManagedGroupSummary>,
    pub member_clubs: Vec<(i32, String)>,
    pub teams: Vec<TeamSummary>,
    pub awaiting_assignment_clubs: Vec<(i32, String)>,
}

#[post("/api/dashboard/context")]
pub async fn get_dashboard_context() -> Result<ViewerContext> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::dashboard::context()
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
