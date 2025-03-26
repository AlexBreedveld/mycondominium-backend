use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = documents)]
pub struct DocumentModel {
    pub id: Uuid,
    #[validate(length(max = 150, message = "Title is too long"))]
    pub title: String,
    pub description: Option<String>,
    pub file_url: String,
    #[validate(length(max = 50, message = "Type is too long"))]
    pub document_type: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
