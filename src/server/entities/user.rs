use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub theme_mode: String,
    pub is_system_admin: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::club_membership::Entity")]
    ClubMembership,
    #[sea_orm(has_many = "super::group_trainer::Entity")]
    GroupTrainer,
    #[sea_orm(has_many = "super::invitation::Entity")]
    Invitation,
    #[sea_orm(has_many = "super::session::Entity")]
    Session,
    #[sea_orm(has_many = "super::team_player::Entity")]
    TeamPlayer,
}

impl Related<super::club_membership::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClubMembership.def()
    }
}

impl Related<super::group_trainer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupTrainer.def()
    }
}

impl Related<super::invitation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invitation.def()
    }
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Session.def()
    }
}

impl Related<super::team_player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TeamPlayer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
