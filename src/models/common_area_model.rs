use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = common_areas)]
pub struct CommonArea {
    pub id: Uuid,
    pub name: String,       // max_length 100
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
}