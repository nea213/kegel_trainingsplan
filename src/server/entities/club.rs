use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "clubs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::club_membership::Entity")]
    ClubMembership,
    #[sea_orm(has_many = "super::club_group::Entity")]
    ClubGroup,
    #[sea_orm(has_many = "super::invitation::Entity")]
    Invitation,
    #[sea_orm(has_many = "super::team::Entity")]
    Team,
    #[sea_orm(has_many = "super::training_plan::Entity")]
    TrainingPlan,
    #[sea_orm(has_many = "super::training_template::Entity")]
    TrainingTemplate,
}

impl Related<super::club_membership::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClubMembership.def()
    }
}

impl Related<super::club_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ClubGroup.def()
    }
}

impl Related<super::invitation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Invitation.def()
    }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Team.def()
    }
}

impl Related<super::training_plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrainingPlan.def()
    }
}

impl Related<super::training_template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TrainingTemplate.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
