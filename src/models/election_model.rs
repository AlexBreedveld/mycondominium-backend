use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = elections)]
pub struct Election {
    pub id: Uuid,
    #[validate(length(max = 150, message = "Title is too long"))]
    pub title: String,
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
}