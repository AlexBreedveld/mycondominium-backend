use super::prelude::*;
use crate::models::resident_model::{ResidentModel, ResidentModelResult};
use crate::models::user_role_model::UserRoleModel;
use crate::services::UserRoles;
use diesel::prelude::*;

#[derive(
    Queryable,
    Selectable,
    Insertable,
    Serialize,
    Deserialize,
    DbOps,
    Clone,
    Debug,
    AsChangeset,
    Validate,
    ToSchema,
)]
#[diesel(table_name = vehicles)]
pub struct VehicleModel {
    pub id: Uuid,
    pub resident_id: Uuid,
    #[validate(length(max = 20, message = "License plate is too long"))]
    pub license_plate: String,
    #[validate(length(max = 100, message = "Model is too long"))]
    pub model: Option<String>,
    #[validate(length(max = 50, message = "Color is too long"))]
    pub color: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct VehicleModelNew {
    pub resident_id: Uuid,
    #[validate(length(max = 20, message = "License plate is too long"))]
    pub license_plate: String,
    #[validate(length(max = 100, message = "Model is too long"))]
    pub model: Option<String>,
    #[validate(length(max = 50, message = "Color is too long"))]
    pub color: Option<String>,
}

impl VehicleModel {
    pub fn db_read_all_matching_community_by_range(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<VehicleModel>> {
        use crate::schema::{residents, user_roles, users, vehicles};

        let mut query = vehicles::table
            .inner_join(residents::table.on(vehicles::resident_id.eq(residents::id)))
            .inner_join(users::table.on(users::entity_id.eq(residents::id)))
            .inner_join(user_roles::table.on(user_roles::user_id.eq(users::id)))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {
                // No additional filters for Root; all vehicles available
            }
            UserRoles::Admin => {
                query = query.filter(user_roles::community_id.eq(user_role.community_id));
            }
            UserRoles::Resident => {
                //query = query.filter(residents::id.eq(user_role.user_id));
                query = query.filter(user_roles::user_id.eq(user_role.user_id));
            }
        }

        query
            .select(VehicleModel::as_select())
            .limit(per_page)
            .offset(offset)
            .load(conn)
    }

    pub fn db_read_by_id_matching_community(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        vehicle_id: Uuid,
    ) -> diesel::QueryResult<VehicleModel> {
        use crate::schema::{residents, user_roles, users, vehicles};

        let mut query = vehicles::table
            .inner_join(residents::table.on(vehicles::resident_id.eq(residents::id)))
            .inner_join(users::table.on(users::entity_id.eq(residents::id)))
            .inner_join(user_roles::table.on(user_roles::user_id.eq(users::id)))
            .filter(vehicles::id.eq(vehicle_id))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => { /* No additional filters */ }
            UserRoles::Admin => {
                query = query.filter(user_roles::community_id.eq(user_role.community_id));
            }
            UserRoles::Resident => {
                //query = query.filter(residents::id.eq(user_role.user_id));
                query = query.filter(user_roles::user_id.eq(user_role.user_id));
            }
        }

        query.select(VehicleModel::as_select()).first(conn)
    }
}
