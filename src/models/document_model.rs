use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = documents)]
pub struct Document {
    pub id: Uuid,
    #[validate(length(max = 150, message = "Title is too long"))]
    pub title: String,
    pub description: Option<String>,
    pub file_url: String,
    #[validate(length(max = 50, message = "Type is too long"))]
    pub document_type: Option<String>,
    pub shared_at: NaiveDateTime,
}
