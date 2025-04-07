use super::prelude::*;
use crate::models::admin_model::{AdminModel, AdminModelResult};
use crate::models::lib::DatabaseTrait;
use crate::models::lib::DatabaseTraitVec;
use crate::models::user_model::{UserModel, UserModelResult};
use crate::models::user_role_model::UserRoleModel;
use crate::services::UserRoles;

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
#[diesel(table_name = crate::schema::residents)]
pub struct ResidentModel {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    #[validate(length(max = 20, message = "Unit number is too long"))]
    pub unit_number: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: String,
    pub date_of_birth: Option<NaiveDate>,
    pub resident_since: NaiveDateTime,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ResidentModelNew {
    pub first_name: String,
    pub last_name: String,
    #[validate(length(max = 20, message = "Unit number is too long"))]
    pub unit_number: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: String,
    pub date_of_birth: Option<NaiveDate>,
    pub resident_since: NaiveDateTime,
    pub password: Option<String>,
    pub role: UserRoles,
    pub community_id: Option<Uuid>,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ResidentModelResult {
    pub resident: ResidentModel,
    pub user: UserModelResult,
    pub role: UserRoleModel,
}

impl ResidentModel {
    pub fn new_id_user(conn: &mut PgConnection) -> uuid::Uuid {
        let mut uuid_new = uuid::Uuid::new_v4();
        let mut exists = true;
        let mut tries = 0;

        while exists && tries < 10 {
            let adm_table_exists: bool = match admins::table
                .filter(admins::columns::id.eq(uuid_new))
                .count()
                .get_result::<i64>(conn)
            {
                Ok(count) => count != 0,
                Err(e) => {
                    tries += 1;
                    true
                }
            };

            let res_table_exists: bool = match crate::schema::residents::table
                .filter(crate::schema::residents::columns::id.eq(uuid_new))
                .count()
                .get_result::<i64>(conn)
            {
                Ok(count) => count != 0,
                Err(e) => {
                    tries += 1;
                    true
                }
            };

            if !adm_table_exists && !res_table_exists {
                exists = false;
            }
        }

        uuid_new
    }

    pub fn db_read_all_matching_community_by_range(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<ResidentModelResult>> {
        let mut all_roles_matching_comm =
            Vec::<crate::models::user_role_model::UserRoleModel>::new();

        if user_role.role == UserRoles::Root {
            all_roles_matching_comm = crate::models::user_role_model::UserRoleModel::table()
                .filter(
                    user_roles::role
                        .eq(UserRoles::Resident)
                        .or(user_roles::role.eq(UserRoles::Guest)),
                )
                .limit(per_page)
                .offset(offset)
                .load::<crate::models::user_role_model::UserRoleModel>(conn)?;
        } else if user_role.role == UserRoles::Admin {
            all_roles_matching_comm = crate::models::user_role_model::UserRoleModel::table()
                .filter(user_roles::community_id.eq(user_role.community_id))
                .filter(
                    user_roles::role
                        .eq(UserRoles::Resident)
                        .or(user_roles::role.eq(UserRoles::Guest)),
                )
                .limit(per_page)
                .offset(offset)
                .load::<crate::models::user_role_model::UserRoleModel>(conn)?;
        } else {
            return Err(diesel::result::Error::NotFound);
        }

        let all_users_matching_comm: Vec<ResidentModelResult> = all_roles_matching_comm
            .into_iter()
            .filter_map(|role| {
                let user = match crate::models::user_model::UserModel::table()
                    .filter(users::id.eq(role.user_id))
                    .filter(users::entity_type.eq("resident"))
                    .first::<crate::models::user_model::UserModel>(conn)
                {
                    Ok(user) => user,
                    Err(e) => return None,
                };

                let resident = match crate::models::resident_model::ResidentModel::table()
                    .filter(residents::id.eq(user.entity_id))
                    .first::<crate::models::resident_model::ResidentModel>(conn)
                {
                    Ok(admin) => admin,
                    Err(e) => return None,
                };

                let user_result = UserModelResult {
                    id: user.id,
                    entity_id: user.entity_id,
                    entity_type: user.entity_type,
                    admin_id: user.admin_id,
                    resident_id: user.resident_id,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                };

                Some(ResidentModelResult {
                    resident,
                    user: user_result,
                    role,
                })
            })
            .collect();

        Ok(all_users_matching_comm)
    }

    pub fn db_read_by_id_matching_community(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        id: uuid::Uuid,
    ) -> diesel::QueryResult<ResidentModelResult> {
        let resident = crate::models::resident_model::ResidentModel::db_read_by_id(conn, id)?;

        let user = crate::models::user_model::UserModel::table()
            .filter(users::entity_id.eq(resident.id))
            .filter(users::entity_type.eq("resident"))
            .first::<crate::models::user_model::UserModel>(conn)?;

        let role = crate::models::user_role_model::UserRoleModel::table()
            .filter(user_roles::user_id.eq(user.id))
            .first::<crate::models::user_role_model::UserRoleModel>(conn)?;

        if user_role.role == UserRoles::Root
            || (user_role.role == UserRoles::Admin && user_role.community_id == role.community_id)
            || (user_role.role == UserRoles::Resident
                && user_role.community_id == role.community_id)
            || (user_role.role == UserRoles::Guest && user_role.community_id == role.community_id)
        {
            let user_result = UserModelResult {
                id: user.id,
                entity_id: user.entity_id,
                entity_type: user.entity_type,
                admin_id: user.admin_id,
                resident_id: user.resident_id,
                created_at: user.created_at,
                updated_at: user.updated_at,
            };

            Ok(ResidentModelResult {
                resident,
                user: user_result,
                role,
            })
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }
}
