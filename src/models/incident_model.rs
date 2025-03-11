use super::prelude::*;

#[derive(Debug, Queryable, Identifiable, Validate)]
#[diesel(table_name = incidents)]
pub struct Incident {
    pub id: Uuid,
    pub resident_id: Option<Uuid>,
    pub description: String,
    #[validate(length(max = 20, message = "Status is too long"))]
    pub status: String,
    pub report_date: NaiveDateTime,
    pub resolution_date: Option<NaiveDateTime>,
    pub notes: Option<String>,
}