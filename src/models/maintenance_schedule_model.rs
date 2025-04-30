use super::prelude::*;
use crate::models::user_role_model::UserRoleModel;
use crate::services::UserRoles;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::{AsExpression, FromSqlRow, deserialize, serialize};
use validator::ValidateLength;

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    AsChangeset,
    Validate,
    ToSchema,
    DbOps,
)]
#[diesel(table_name = maintenance_schedules)]
pub struct MaintenanceScheduleModel {
    pub id: Uuid,
    pub community_id: Option<Uuid>,
    pub description: String,
    pub scheduled_date: NaiveDateTime,
    pub status: MaintenanceScheduleStatus,
    pub details: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct MaintenanceScheduleModelNew {
    pub community_id: Option<Uuid>,
    pub description: String,
    pub scheduled_date: NaiveDateTime,
    pub status: MaintenanceScheduleStatus,
    pub details: Option<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum MaintenanceScheduleStatus {
    Scheduled,
    Ongoing,
    Completed,
    Immediate,
}

impl MaintenanceScheduleModel {
    pub fn db_read_by_id_matching_community(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        id: uuid::Uuid,
    ) -> diesel::QueryResult<MaintenanceScheduleModel> {
        let maintenance = MaintenanceScheduleModel::db_read_by_id(conn, id)?;

        if user_role.role == UserRoles::Root
            || (user_role.role == UserRoles::Admin
                && user_role.community_id == maintenance.community_id)
            || (user_role.role == UserRoles::Resident
                && user_role.community_id == maintenance.community_id)
        {
            Ok(maintenance)
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }

    pub fn db_count_all_matching_community(
        user_role: UserRoleModel,
        status: MaintenanceScheduleStatus,
        conn: &mut PgConnection,
    ) -> diesel::QueryResult<i64> {
        use crate::schema::maintenance_schedules;
        use diesel::prelude::*;

        // Base query
        let mut query = MaintenanceScheduleModel::table().into_boxed(); // Needed for conditional filters

        // Apply additional filter if role is Admin (not Root)
        match user_role.role {
            UserRoles::Root => query = query.filter(maintenance_schedules::status.eq(status)),
            UserRoles::Admin | UserRoles::Resident => {
                query = query
                    .filter(maintenance_schedules::community_id.eq(user_role.community_id))
                    .filter(maintenance_schedules::status.eq(status));
            }
        }

        // Count data
        query.count().get_result::<i64>(conn)
    }

    pub fn db_read_all_matching_community_by_range(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<MaintenanceScheduleModel>> {
        use crate::schema::maintenance_schedules;
        use diesel::prelude::*;

        // Base query
        let mut query = MaintenanceScheduleModel::table().into_boxed(); // Needed for conditional filters

        // Apply additional filter if role is Admin (not Root)
        match user_role.role {
            UserRoles::Root => { /* No additional filter required for Root */ }
            UserRoles::Admin | UserRoles::Resident => {
                query =
                    query.filter(maintenance_schedules::community_id.eq(user_role.community_id));
            }
            _ => return Err(diesel::result::Error::NotFound), // Early return for unauthorized roles
        }

        // Fetch data with limit/offset (pagination)
        let data_result: Vec<MaintenanceScheduleModel> = query
            .limit(per_page)
            .offset(offset)
            .load::<MaintenanceScheduleModel>(conn)?;

        // Map query result into desired `ResidentModelResult`
        match data_result.length() {
            Some(l) => {
                if l > 0 {
                    Ok(data_result)
                } else {
                    Err(diesel::result::Error::NotFound)
                }
            }
            None => Err(diesel::result::Error::NotFound),
        }
    }
}

impl<DB> ToSql<diesel::sql_types::Text, DB> for MaintenanceScheduleStatus
where
    DB: Backend,
    str: ToSql<diesel::sql_types::Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match self {
            MaintenanceScheduleStatus::Scheduled => "Scheduled",
            MaintenanceScheduleStatus::Ongoing => "Ongoing",
            MaintenanceScheduleStatus::Completed => "Completed",
            MaintenanceScheduleStatus::Immediate => "Immediate",
        };
        s.to_sql(out)
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for MaintenanceScheduleStatus
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match <String as FromSql<diesel::sql_types::Text, DB>>::from_sql(bytes)?.as_str() {
            "Scheduled" => Ok(MaintenanceScheduleStatus::Scheduled),
            "Ongoing" => Ok(MaintenanceScheduleStatus::Ongoing),
            "Completed" => Ok(MaintenanceScheduleStatus::Completed),
            "Immediate" => Ok(MaintenanceScheduleStatus::Immediate),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}
