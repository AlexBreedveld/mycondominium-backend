use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = documents)]
pub struct Document {
    pub id: Uuid,
    pub title: String,                       // max_length 150
    pub description: Option<String>,
    pub file_url: String,
    pub document_type: Option<String>,       // max_length 50
    pub shared_at: NaiveDateTime,
}
