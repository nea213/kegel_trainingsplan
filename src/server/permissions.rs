use crate::{auth::PublicUser, server::auth};

pub async fn require_authenticated_user() -> Result<PublicUser, String> {
    auth::current_user()
        .await?
        .ok_or_else(|| "Nicht angemeldet.".to_string())
}

pub async fn require_system_admin() -> Result<PublicUser, String> {
    let user = require_authenticated_user().await?;

    if !user.is_system_admin {
        return Err("Nur System-Admins duerfen diesen Bereich verwalten.".to_string());
    }

    Ok(user)
}
