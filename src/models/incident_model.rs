use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = incidents)]
pub struct Incident {
    pub id: Uuid,
    pub resident_id: Option<Uuid>,
    pub description: String,
    pub status: String,                      // max_length 20
    pub report_date: NaiveDateTime,
    pub resolution_date: Option<NaiveDateTime>,
    pub notes: Option<String>,
}