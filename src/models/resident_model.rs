use super::prelude::*;
use crate::models::lib::DatabaseTrait;
use crate::models::lib::DatabaseTraitVec;

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
    pub email: Option<String>,
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
    pub email: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub resident_since: NaiveDateTime,
    pub is_active: bool,
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
}
