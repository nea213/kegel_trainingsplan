use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignedPlayer {
    pub user_id: i32,
    pub username: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignTeamPlayerInput {
    pub team_id: i32,
    pub user_id: i32,
}

#[post("/api/team-players/list")]
pub async fn list_team_players(team_id: i32) -> Result<Vec<AssignedPlayer>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::team_players::list(team_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = team_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/team-players/assign")]
pub async fn assign_team_player(input: AssignTeamPlayerInput) -> Result<AssignedPlayer> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::team_players::assign(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/team-players/remove")]
pub async fn remove_team_player(team_id: i32, user_id: i32) -> Result<()> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::team_players::remove(team_id, user_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = (team_id, user_id);
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
