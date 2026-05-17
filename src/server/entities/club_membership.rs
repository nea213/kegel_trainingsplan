use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "club_memberships")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub club_id: i32,
    pub user_id: i32,
    pub created_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::club::Entity",
        from = "Column::ClubId",
        to = "super::club::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Club,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::club::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Club.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
