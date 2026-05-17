use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::dashboard::ClubMembershipSummary;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAssignmentInput {
    pub club_id: i32,
    pub team_id: i32,
    pub user_id: i32,
}

#[post("/api/club-memberships/list")]
pub async fn list_club_members(club_id: i32) -> Result<Vec<ClubMembershipSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::club_memberships::list_club_members(club_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = club_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/club-memberships/unassigned")]
pub async fn list_unassigned_club_members(club_id: i32) -> Result<Vec<ClubMembershipSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::club_memberships::list_unassigned_club_members(club_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = club_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/club-memberships/assign-player")]
pub async fn assign_player_to_team(input: PlayerAssignmentInput) -> Result<()> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::club_memberships::assign_player_to_team(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
