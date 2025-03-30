use super::prelude::*;
use db_ops_derive::DbOps;
use diesel::{QueryResult, RunQueryDsl};
use crate::internal::roles::UserRoles;

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
    pub community_id: Option<Uuid>
}

impl AdminModel {
    pub fn new_id_user(conn: &mut PgConnection) -> uuid::Uuid {
        let mut uuid_new = uuid::Uuid::new_v4();
        let mut exists = true;
        let mut tries = 0;

        while exists && tries < 10 {
            let adm_table_exists: bool = match admins::table.filter(admins::columns::id.eq(uuid_new)).count().get_result::<i64>(conn) {
                Ok(count) => {
                    count != 0
                },
                Err(e) => {
                    tries += 1;
                    true
                }
            };

            let res_table_exists: bool = match residents::table.filter(residents::columns::id.eq(uuid_new)).count().get_result::<i64>(conn) {
                Ok(count) => {
                    count != 0
                },
                Err(e) => {
                    tries += 1;
                    true
                }
            };
            
            if !adm_table_exists && !res_table_exists {
                exists = false;
            }
        };

        uuid_new
    }
}