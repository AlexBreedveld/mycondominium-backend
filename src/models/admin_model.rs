use super::prelude::*;
use crate::internal::roles::UserRoles;
use crate::models::user_model::{UserModel, UserModelResult};
use crate::models::user_role_model::UserRoleModel;
use db_ops_derive::DbOps;
use diesel::{QueryResult, RunQueryDsl};

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
#[diesel(table_name = crate::schema::admins)]
pub struct AdminModel {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AdminModelNew {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: String,
    pub password: String,
    pub role: UserRoles,
    pub community_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AdminModelNewSelfService {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct AdminModelResult {
    pub admin: AdminModel,
    pub user: UserModelResult,
    pub role: UserRoleModel,
}

impl AdminModel {
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

            let res_table_exists: bool = match residents::table
                .filter(residents::columns::id.eq(uuid_new))
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
    ) -> diesel::QueryResult<Vec<AdminModelResult>> {
        use crate::schema::{admins, user_roles, users};

        let mut query = user_roles::table
            .inner_join(users::table.on(users::id.eq(user_roles::user_id)))
            .inner_join(admins::table.on(admins::id.eq(users::entity_id)))
            .filter(users::entity_type.eq("admin"))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {
                query = query.filter(
                    user_roles::role
                        .eq(UserRoles::Root)
                        .or(user_roles::role.eq(UserRoles::Admin)),
                );
            }
            UserRoles::Admin => {
                query = query.filter(
                    user_roles::role
                        .eq(UserRoles::Admin)
                        .and(user_roles::community_id.eq(user_role.community_id)),
                );
            }
            _ => return Ok(vec![]),
        };

        let results = query
            .select((
                AdminModel::as_select(),
                UserModel::as_select(),
                UserRoleModel::as_select(),
            ))
            .limit(per_page)
            .offset(offset)
            .load::<(AdminModel, UserModel, UserRoleModel)>(conn)?
            .into_iter()
            .map(|(admin, user, role)| AdminModelResult {
                admin,
                user: UserModelResult {
                    id: user.id,
                    entity_id: user.entity_id,
                    entity_type: user.entity_type,
                    admin_id: user.admin_id,
                    resident_id: user.resident_id,
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                },
                role,
            })
            .collect::<Vec<AdminModelResult>>();

        println!("reached code");
        println!("{:?}", results);

        Ok(results)
    }

    pub fn db_read_by_id_matching_community(
        &self,
        id: uuid::Uuid,
        conn: &mut PgConnection,
    ) -> diesel::QueryResult<AdminModelResult> {
        let self_user = crate::models::user_model::UserModel::table()
            .filter(users::admin_id.eq(self.id))
            .first::<crate::models::user_model::UserModel>(conn)?;

        let self_role = crate::models::user_role_model::UserRoleModel::table()
            .filter(user_roles::user_id.eq(self_user.id))
            .first::<crate::models::user_role_model::UserRoleModel>(conn)?;

        let admin = crate::models::admin_model::AdminModel::db_read_by_id(conn, id)?;

        let user = crate::models::user_model::UserModel::table()
            .filter(users::entity_id.eq(admin.id))
            .filter(users::entity_type.eq("admin"))
            .first::<crate::models::user_model::UserModel>(conn)?;

        let role = crate::models::user_role_model::UserRoleModel::table()
            .filter(user_roles::user_id.eq(user.id))
            .first::<crate::models::user_role_model::UserRoleModel>(conn)?;

        if self_role.role == UserRoles::Root
            || (self_role.role == UserRoles::Admin && self_role.community_id == role.community_id)
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

            Ok(AdminModelResult {
                admin,
                user: user_result,
                role,
            })
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }
}
