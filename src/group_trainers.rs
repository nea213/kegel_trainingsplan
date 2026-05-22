use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignedTrainer {
    pub user_id: i32,
    pub username: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignGroupTrainerInput {
    pub group_id: i32,
    pub user_id: i32,
}

#[post("/api/group-trainers/list")]
pub async fn list_group_trainers(group_id: i32) -> Result<Vec<AssignedTrainer>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::group_trainers::list(group_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = group_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/group-trainers/assign")]
pub async fn assign_group_trainer(input: AssignGroupTrainerInput) -> Result<AssignedTrainer> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::group_trainers::assign(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/group-trainers/remove")]
pub async fn remove_group_trainer(group_id: i32, user_id: i32) -> Result<()> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::group_trainers::remove(group_id, user_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = (group_id, user_id);
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
