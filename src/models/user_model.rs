use super::prelude::*;
use crate::models::{admin_model, resident_model};

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
#[diesel(table_name = crate::schema::users)]
pub struct UserModel {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: UserTypes,
    pub admin_id: Option<Uuid>,
    pub resident_id: Option<Uuid>,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct UserModelResult {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: UserTypes,
    pub admin_id: Option<Uuid>,
    pub resident_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct UserModelEntityResult {
    pub user: UserModel,
    pub resident: Option<resident_model::ResidentModel>,
    pub admin: Option<admin_model::AdminModel>,
}

impl UserModel {
    pub fn find_user_by_email(
        conn: &mut PgConnection,
        email: String,
    ) -> QueryResult<UserModelEntityResult> {
        let mut found = false;
        let mut user_type = UserTypes::Resident;

        let mut entity_id = match crate::schema::residents::table
            .filter(crate::schema::residents::email.eq(email.clone()))
            .first::<resident_model::ResidentModel>(conn)
            .optional()
        {
            Ok(Some(ent)) => {
                found = true;
                user_type = UserTypes::Resident;
                Some(ent.id)
            }
            Ok(None) => None,
            Err(e) => {
                log::error!("Error getting resident: {}", e);
                return Err(diesel::result::Error::NotFound);
            }
        };

        if !found {
            entity_id = match crate::schema::admins::table
                .filter(crate::schema::admins::email.eq(email.clone()))
                .first::<admin_model::AdminModel>(conn)
                .optional()
            {
                Ok(Some(ent)) => {
                    user_type = UserTypes::Admin;
                    Some(ent.id)
                }
                Ok(None) => None,
                Err(e) => {
                    log::error!("Error getting admin: {}", e);
                    return Err(diesel::result::Error::NotFound);
                }
            };
        }

        if found {
            let user_obj = match UserModel::table()
                .filter(users::entity_id.eq(entity_id.unwrap()))
                .first::<UserModel>(conn)
            {
                Ok(user) => user,
                Err(e) => {
                    log::error!("Error getting user: {}", e);
                    return Err(diesel::result::Error::NotFound);
                }
            };

            match user_type {
                UserTypes::Admin => {
                    match admin_model::AdminModel::db_read_by_id(conn, user_obj.admin_id.unwrap()) {
                        Ok(adm) => Ok(UserModelEntityResult {
                            user: user_obj,
                            resident: None,
                            admin: Some(adm),
                        }),
                        Err(e) => Err(e),
                    }
                }
                UserTypes::Resident => {
                    match resident_model::ResidentModel::db_read_by_id(
                        conn,
                        user_obj.admin_id.unwrap(),
                    ) {
                        Ok(res) => Ok(UserModelEntityResult {
                            user: user_obj,
                            resident: Some(res),
                            admin: None,
                        }),
                        Err(e) => Err(e),
                    }
                }
            }
        } else {
            Err(diesel::result::Error::NotFound)
        }
    }
}
