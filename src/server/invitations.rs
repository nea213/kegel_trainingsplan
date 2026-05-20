use crate::{
    invitations::{
        CreateInvitationInput, CreatedInvitation, InvitationPreview, InvitationRole,
        InvitationSummary,
    },
    server::{
        auth::{hash_password, now_timestamp, normalize_username, validate_password},
        db,
        entities::{
            club, club_group, club_membership, group_trainer, invitation, user,
        },
        permissions,
    },
};
use argon2::{Argon2, PasswordHash, PasswordHasher};
use password_hash::{SaltString, PasswordVerifier};
use rand_core::OsRng;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder,
};

const INVITATION_MIN_DAYS: i32 = 1;
const INVITATION_MAX_DAYS: i32 = 30;

pub async fn create(input: CreateInvitationInput) -> Result<CreatedInvitation, String> {
    validate_create_input(&input)?;
    let actor = permissions::require_invitation_manager(input.club_id, input.group_id).await?;

    if input.role == InvitationRole::Trainer && !actor.is_system_admin {
        return Err("Nur System-Admins dürfen Trainer-Einladungen erzeugen.".to_string());
    }

    let db = db::connection().await.map_err(db_error)?;
    validate_scope(&db, &input).await?;

    let plain_code = generate_invitation_code();
    let code_hash = hash_invitation_code(&plain_code)?;
    let now = now_timestamp();
    let expires_at = now + i64::from(input.expires_in_days) * 24 * 60 * 60;

    let created = invitation::ActiveModel {
        code_hash: Set(code_hash),
        club_id: Set(input.club_id),
        group_id: Set(input.group_id),
        team_id: Set(None),
        target_role: Set(role_as_str(input.role).to_string()),
        created_by_user_id: Set(actor.id),
        expires_at: Set(expires_at),
        used_at: Set(None),
        used_by_user_id: Set(None),
        revoked_at: Set(None),
        created_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(CreatedInvitation {
        invitation: invitation_summary(created),
        plain_code,
    })
}

pub async fn list(club_id: i32, group_id: Option<i32>) -> Result<Vec<InvitationSummary>, String> {
    permissions::require_invitation_manager(club_id, group_id).await?;
    let db = db::connection().await.map_err(db_error)?;

    let mut query = invitation::Entity::find().filter(invitation::Column::ClubId.eq(club_id));
    if let Some(group_id) = group_id {
        query = query.filter(invitation::Column::GroupId.eq(group_id));
    }

    query
        .order_by_desc(invitation::Column::CreatedAt)
        .all(db)
        .await
        .map(|items| items.into_iter().map(invitation_summary).collect())
        .map_err(db_error)
}

pub async fn revoke(invitation_id: i32) -> Result<(), String> {
    let db = db::connection().await.map_err(db_error)?;
    let invitation = invitation::Entity::find_by_id(invitation_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Die Einladung wurde nicht gefunden.".to_string())?;

    let actor = permissions::require_invitation_manager(invitation.club_id, invitation.group_id).await?;
    if role_from_str(&invitation.target_role)? == InvitationRole::Trainer && !actor.is_system_admin {
        return Err("Nur System-Admins dürfen Trainer-Einladungen widerrufen.".to_string());
    }

    if invitation.used_at.is_some() {
        return Err("Bereits verwendete Einladungen können nicht widerrufen werden.".to_string());
    }

    let mut active_invitation: invitation::ActiveModel = invitation.into();
    active_invitation.revoked_at = Set(Some(now_timestamp()));
    active_invitation.update(db).await.map_err(db_error)?;

    Ok(())
}

pub async fn preview(code: &str) -> Result<InvitationPreview, String> {
    let db = db::connection().await.map_err(db_error)?;
    let invitation = find_active_invitation_by_code(&db, code).await?;
    let club = club::Entity::find_by_id(invitation.club_id)
        .one(db)
        .await
        .map_err(db_error)?
        .ok_or_else(|| "Der Zielverein wurde nicht gefunden.".to_string())?;

    let group = match invitation.group_id {
        Some(group_id) => Some(
            club_group::Entity::find_by_id(group_id)
                .one(db)
                .await
                .map_err(db_error)?
                .ok_or_else(|| "Die Zielgruppe wurde nicht gefunden.".to_string())?,
        ),
        None => None,
    };

    Ok(InvitationPreview {
        club_id: club.id,
        club_name: club.name,
        group_id: group.as_ref().map(|group| group.id),
        group_name: group.map(|group| group.name),
        role: role_from_str(&invitation.target_role)?,
    })
}

pub async fn register_with_invitation(
    code: &str,
    username: &str,
    password: &str,
) -> Result<user::Model, String> {
    let username = normalize_username(username)?;
    validate_password(password)?;
    let db = db::connection().await.map_err(db_error)?;

    if user::Entity::find()
        .filter(user::Column::Username.eq(username.clone()))
        .one(db)
        .await
        .map_err(db_error)?
        .is_some()
    {
        return Err("Dieser Benutzername ist bereits vergeben.".to_string());
    }

    let invitation = find_active_invitation_by_code(&db, code).await?;
    validate_registration_scope(&db, &invitation).await?;

    let now = now_timestamp();
    let password_hash = hash_password(password)?;

    let created_user = user::ActiveModel {
        username: Set(username),
        password_hash: Set(password_hash),
        theme_mode: Set("system".to_string()),
        is_system_admin: Set(false),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    ensure_club_membership(&db, invitation.club_id, created_user.id).await?;

    if role_from_str(&invitation.target_role)? == InvitationRole::Trainer {
        let group_id = invitation
            .group_id
            .ok_or_else(|| "Trainer-Einladungen müssen an eine Gruppe gebunden sein.".to_string())?;

        group_trainer::ActiveModel {
            group_id: Set(group_id),
            user_id: Set(created_user.id),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(db_error)?;
    }

    let mut active_invitation: invitation::ActiveModel = invitation.into();
    active_invitation.used_at = Set(Some(now));
    active_invitation.used_by_user_id = Set(Some(created_user.id));
    active_invitation.update(db).await.map_err(db_error)?;

    Ok(created_user)
}

fn validate_create_input(input: &CreateInvitationInput) -> Result<(), String> {
    if !(INVITATION_MIN_DAYS..=INVITATION_MAX_DAYS).contains(&input.expires_in_days) {
        return Err(format!(
            "Die Einladung muss zwischen {INVITATION_MIN_DAYS} und {INVITATION_MAX_DAYS} Tagen gültig sein."
        ));
    }

    if input.role == InvitationRole::Trainer && input.group_id.is_none() {
        return Err("Trainer-Einladungen müssen an eine Gruppe gebunden sein.".to_string());
    }

    if input.role == InvitationRole::Player && input.group_id.is_some() {
        return Err("Spieler-Einladungen werden aktuell vereinsweit angelegt, nicht gruppenbezogen.".to_string());
    }

    Ok(())
}

async fn validate_scope(db: &DatabaseConnection, input: &CreateInvitationInput) -> Result<(), String> {
    let club_exists = club::Entity::find_by_id(input.club_id)
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if !club_exists {
        return Err("Der Zielverein wurde nicht gefunden.".to_string());
    }

    if let Some(group_id) = input.group_id {
        let group = club_group::Entity::find_by_id(group_id)
            .one(db)
            .await
            .map_err(db_error)?
            .ok_or_else(|| "Die Zielgruppe wurde nicht gefunden.".to_string())?;

        if group.club_id != input.club_id {
            return Err("Die Zielgruppe gehört nicht zum ausgewählten Verein.".to_string());
        }
    }

    Ok(())
}

async fn validate_registration_scope(
    db: &DatabaseConnection,
    invitation: &invitation::Model,
) -> Result<(), String> {
    let club_exists = club::Entity::find_by_id(invitation.club_id)
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if !club_exists {
        return Err("Der Zielverein wurde nicht gefunden.".to_string());
    }

    if let Some(group_id) = invitation.group_id {
        let group = club_group::Entity::find_by_id(group_id)
            .one(db)
            .await
            .map_err(db_error)?
            .ok_or_else(|| "Die Zielgruppe wurde nicht gefunden.".to_string())?;

        if group.club_id != invitation.club_id {
            return Err("Die Einladung verweist auf eine ungültige Gruppen-Zuordnung.".to_string());
        }
    }

    Ok(())
}

async fn ensure_club_membership(db: &DatabaseConnection, club_id: i32, user_id: i32) -> Result<(), String> {
    let exists = club_membership::Entity::find()
        .filter(club_membership::Column::ClubId.eq(club_id))
        .filter(club_membership::Column::UserId.eq(user_id))
        .one(db)
        .await
        .map_err(db_error)?
        .is_some();

    if exists {
        return Ok(());
    }

    club_membership::ActiveModel {
        club_id: Set(club_id),
        user_id: Set(user_id),
        created_at: Set(now_timestamp()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(db_error)?;

    Ok(())
}

async fn find_active_invitation_by_code(
    db: &DatabaseConnection,
    code: &str,
) -> Result<invitation::Model, String> {
    let code = code.trim();
    if code.len() < 8 {
        return Err("Der Einladungscode ist ungültig.".to_string());
    }

    let invitations = invitation::Entity::find()
        .filter(invitation::Column::RevokedAt.is_null())
        .filter(invitation::Column::UsedAt.is_null())
        .all(db)
        .await
        .map_err(db_error)?;

    let now = now_timestamp();
    for invitation in invitations {
        if invitation.expires_at < now {
            continue;
        }

        if verify_invitation_code(code, &invitation.code_hash)? {
            return Ok(invitation);
        }
    }

    Err("Der Einladungscode ist ungültig oder nicht mehr aktiv.".to_string())
}

fn invitation_summary(invitation: invitation::Model) -> InvitationSummary {
    InvitationSummary {
        id: invitation.id,
        club_id: invitation.club_id,
        group_id: invitation.group_id,
        role: role_from_str(&invitation.target_role).unwrap_or(InvitationRole::Player),
        expires_at: invitation.expires_at,
        revoked_at: invitation.revoked_at,
        used_at: invitation.used_at,
    }
}

fn role_as_str(role: InvitationRole) -> &'static str {
    match role {
        InvitationRole::Trainer => "trainer",
        InvitationRole::Player => "player",
    }
}

fn role_from_str(value: &str) -> Result<InvitationRole, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "trainer" => Ok(InvitationRole::Trainer),
        "player" => Ok(InvitationRole::Player),
        _ => Err("Die Einladung enthält eine ungültige Rolle.".to_string()),
    }
}

fn generate_invitation_code() -> String {
    uuid::Uuid::new_v4().to_string().replace('-', "")
}

fn hash_invitation_code(code: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(code.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| error.to_string())
}

fn verify_invitation_code(code: &str, code_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(code_hash).map_err(|error| error.to_string())?;
    Ok(Argon2::default()
        .verify_password(code.as_bytes(), &parsed_hash)
        .is_ok())
}

fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
