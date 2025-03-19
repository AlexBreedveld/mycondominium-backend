use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
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
