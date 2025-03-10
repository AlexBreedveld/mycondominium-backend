use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = elections)]
pub struct Election {
    pub id: Uuid,
    pub title: String,                       // max_length 150
    pub description: Option<String>,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
}