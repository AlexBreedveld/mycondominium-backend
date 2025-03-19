use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = common_areas)]
pub struct CommonAreaModel {
    pub id: Uuid,
    #[validate(length(max = 100, message = "Name is too long"))]
    pub name: String,
    pub description: Option<String>,
    pub community_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
}
