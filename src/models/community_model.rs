use super::prelude::*;

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Serialize,
    Deserialize,
    DbOps,
    Clone,
    Debug,
    AsChangeset,
    Validate,
    ToSchema,
)]
#[diesel(table_name = communities)]
pub struct CommunityModel {
    pub id: Uuid,
    #[validate(length(max = 50, message = "Name is too long"))]
    pub name: String,
    #[validate(length(max = 25, message = "Short name is too long"))]
    pub short_name: Option<String>,
    pub address: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct CommunityModelNew {
    #[validate(length(max = 50, message = "Name is too long"))]
    pub name: String,
    #[validate(length(max = 25, message = "Short name is too long"))]
    pub short_name: Option<String>,
    pub address: String,
}