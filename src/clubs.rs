use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{group_trainers::AssignedTrainer, groups::GroupSummary, invitations::InvitationSummary, team_players::AssignedPlayer, teams::TeamSummary};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubSummary {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubGroupWithTeams {
    pub group: GroupSummary,
    pub invitations: Vec<InvitationSummary>,
    pub trainers: Vec<AssignedTrainer>,
    pub teams: Vec<TeamWithPlayers>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamWithPlayers {
    pub team: TeamSummary,
    pub players: Vec<AssignedPlayer>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubMemberOption {
    pub user_id: i32,
    pub username: String,
    pub is_assigned_to_team: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubDetail {
    pub club: ClubSummary,
    pub groups: Vec<ClubGroupWithTeams>,
    pub club_members: Vec<ClubMemberOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateClubInput {
    pub name: String,
}

#[post("/api/clubs/create")]
pub async fn create_club(input: CreateClubInput) -> Result<ClubSummary> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::clubs::create(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/clubs/list")]
pub async fn list_clubs() -> Result<Vec<ClubSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::clubs::list()
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/clubs/detail")]
pub async fn get_club_detail(club_id: i32) -> Result<ClubDetail> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::clubs::detail(club_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = club_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
