use super::prelude::*;
use crate::models::maintenance_schedule_model::MaintenanceScheduleModel;
use crate::models::resident_model;
use crate::models::resident_model::ResidentModel;
use crate::models::user_model::UserModel;
use crate::models::user_role_model::UserRoleModel;
use crate::services::{UserRoles, UserTypes, user_model};

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
#[diesel(table_name = parcels)]
pub struct ParcelModel {
    pub id: Uuid,
    pub resident_id: Uuid,
    #[validate(length(max = 50, message = "Type is too long"))]
    pub parcel_type: String,
    pub description: Option<String>,
    pub arrival_date: NaiveDateTime,
    pub received: bool,
    pub received_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ParcelModelNew {
    pub resident_id: Uuid,
    #[validate(length(max = 50, message = "Type is too long"))]
    pub parcel_type: String,
    pub description: Option<String>,
    pub arrival_date: NaiveDateTime,
    pub received: bool,
    pub received_at: Option<NaiveDateTime>,
}

impl ParcelModel {
    pub fn db_read_by_id_matching_resident(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        id: uuid::Uuid,
    ) -> diesel::QueryResult<ParcelModel> {
        let parcel = ParcelModel::db_read_by_id(conn, id)?;
        let resident = resident_model::ResidentModel::db_get_user(conn, parcel.resident_id)?;

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                if user_role.community_id != resident.role.community_id {
                    return Err(diesel::result::Error::NotFound);
                }
            }
            UserRoles::Resident => {
                let user = user_model::UserModel::db_read_by_id(conn, user_role.user_id)?;

                if user_role.community_id != resident.role.community_id
                    || resident.resident.id != user.entity_id
                {
                    return Err(diesel::result::Error::NotFound);
                }
            }
        }

        Ok(parcel)
    }

    pub fn db_read_all_matching_by_range(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<ParcelModel>> {
        let mut query = UserRoleModel::table()
            .inner_join(users::table.on(user_roles::user_id.eq(users::id)))
            .inner_join(residents::table.on(users::entity_id.eq(residents::id)))
            .inner_join(parcels::table.on(users::entity_id.eq(residents::id)))
            .filter(users::entity_type.eq(UserTypes::Resident))
            .filter(user_roles::role.eq(UserRoles::Resident))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                query = query.filter(user_roles::community_id.eq(user_role.community_id));
            }
            UserRoles::Resident => query = query.filter(users::id.eq(user_role.id)),
        }

        query
            .select(parcels::all_columns)
            .limit(per_page)
            .offset(offset)
            .load::<ParcelModel>(conn)
    }

    pub fn db_count_all_matching(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
    ) -> diesel::QueryResult<i64> {
        let mut query = UserRoleModel::table()
            .inner_join(users::table.on(user_roles::user_id.eq(users::id)))
            .inner_join(residents::table.on(users::entity_id.eq(residents::id)))
            .inner_join(parcels::table.on(users::entity_id.eq(residents::id)))
            .filter(users::entity_type.eq(UserTypes::Resident))
            .filter(user_roles::role.eq(UserRoles::Resident))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                query = query.filter(user_roles::community_id.eq(user_role.community_id));
            }
            UserRoles::Resident => query = query.filter(users::id.eq(user_role.id)),
        }

        query
            .select(parcels::all_columns)
            .count()
            .get_result::<i64>(conn)
    }
}
