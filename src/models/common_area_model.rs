use super::prelude::*;
use crate::models::parcel_model::ParcelModel;
use crate::models::user_role_model::UserRoleModel;
use crate::models::{resident_model, user_model};
use crate::services::{UserRoles, UserTypes};

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
#[diesel(table_name = common_areas)]
pub struct CommonAreaModel {
    pub id: Uuid,
    #[validate(length(max = 100, message = "Name is too long"))]
    pub name: String,
    pub description: Option<String>,
    pub community_id: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct CommonAreaModelNew {
    #[validate(length(max = 100, message = "Name is too long"))]
    pub name: String,
    pub description: Option<String>,
    pub community_id: Uuid,
}

impl CommonAreaModel {
    pub fn db_read_by_id_matching(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        id: uuid::Uuid,
    ) -> diesel::QueryResult<CommonAreaModel> {
        let common_area = CommonAreaModel::db_read_by_id(conn, id)?;

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                if user_role.community_id != Some(common_area.community_id) {
                    return Err(diesel::result::Error::NotFound);
                }
            }
            UserRoles::Resident => {
                if user_role.community_id != Some(common_area.community_id) {
                    return Err(diesel::result::Error::NotFound);
                }
            }
        }

        Ok(common_area)
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
                query = query.filter(user_roles::community_id.eq(user_role.community_id))
            }
            UserRoles::Resident => {
                query = query.filter(user_roles::community_id.eq(user_role.community_id))
            }
        }

        query
            .select(parcels::all_columns)
            .count()
            .get_result::<i64>(conn)
    }

    pub fn db_read_all_matching_by_range(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<CommonAreaModel>> {
        let mut query = CommonAreaModel::table().into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                query = query.filter(common_areas::community_id.eq(user_role.community_id.unwrap()))
            }
            UserRoles::Resident => {
                query = query.filter(common_areas::community_id.eq(user_role.community_id.unwrap()))
            }
        }

        query
            .limit(per_page)
            .offset(offset)
            .load::<CommonAreaModel>(conn)
    }
}
