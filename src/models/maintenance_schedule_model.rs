use super::prelude::*;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = maintenance_schedules)]
pub struct MaintenanceSchedule {
    pub id: Uuid,
    pub description: String,
    pub scheduled_date: NaiveDateTime,
    pub status: String,                      // max_length 20
    pub details: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}