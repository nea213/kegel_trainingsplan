use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InvitationRole {
    Trainer,
    Player,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateInvitationInput {
    pub club_id: i32,
    pub group_id: Option<i32>,
    pub role: InvitationRole,
    pub expires_in_days: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatedInvitation {
    pub invitation: InvitationSummary,
    pub plain_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvitationSummary {
    pub id: i32,
    pub club_id: i32,
    pub group_id: Option<i32>,
    pub role: InvitationRole,
    pub expires_at: i64,
    pub revoked_at: Option<i64>,
    pub used_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvitationRegistrationInput {
    pub invitation_code: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvitationPreview {
    pub club_id: i32,
    pub club_name: String,
    pub group_id: Option<i32>,
    pub group_name: Option<String>,
    pub role: InvitationRole,
}

#[post("/api/invitations/create")]
pub async fn create_invitation(input: CreateInvitationInput) -> Result<CreatedInvitation> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::invitations::create(input)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = input;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/invitations/list")]
pub async fn list_invitations(club_id: i32, group_id: Option<i32>) -> Result<Vec<InvitationSummary>> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::invitations::list(club_id, group_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = (club_id, group_id);
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/invitations/revoke")]
pub async fn revoke_invitation(invitation_id: i32) -> Result<()> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::invitations::revoke(invitation_id)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = invitation_id;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}

#[post("/api/invitations/preview")]
pub async fn preview_invitation(code: String) -> Result<InvitationPreview> {
    #[cfg(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        return Ok(crate::server::invitations::preview(&code)
            .await
            .map_err(ServerFnError::new)?);
    }

    #[cfg(not(all(feature = "server", any(target_os = "linux", target_os = "macos", target_os = "windows"))))]
    {
        let _ = code;
        Err(ServerFnError::new("The server feature is not enabled."))
    }
}
