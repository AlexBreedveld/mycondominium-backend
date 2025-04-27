use super::prelude::*;
use crate::models::maintenance_schedule_model;
use crate::models::user_model::{UserModel, UserModelResult};
use crate::models::user_role_model::UserRoleModel;
use crate::services::{HttpResponseObjectEmpty, UserRoles, UserTypes};
use actix_web::HttpResponse;
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
    #[validate(length(max = 20, message = "Status is too long"))]
    pub status: String,
    pub details: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct MaintenanceScheduleModelNew {
    pub community_id: Option<Uuid>,
    pub description: String,
    pub scheduled_date: NaiveDateTime,
    #[validate(length(max = 20, message = "Status is too long"))]
    pub status: String,
    pub details: Option<String>,
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
