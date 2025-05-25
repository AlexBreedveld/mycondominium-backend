use super::prelude::*;
use super::*;

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
#[diesel(table_name = incidents)]
pub struct IncidentModel {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub community_id: Uuid,
    #[validate(length(min = 5, max = 30, message = "Name is too short or too long"))]
    pub name: String,
    #[validate(length(min = 10, message = "Description is too short"))]
    pub description: String,
    pub status: IncidentStatus,
    pub report_date: NaiveDateTime,
    pub resolution_date: Option<NaiveDateTime>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct IncidentModelNew {
    pub resident_id: Uuid,
    pub community_id: Uuid,
    #[validate(length(min = 5, max = 30, message = "Name is too short or too long"))]
    pub name: String,
    #[validate(length(min = 10, message = "Description is too short"))]
    pub description: String,
    pub status: IncidentStatus,
    pub resolution_date: Option<NaiveDateTime>,
    pub notes: Option<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum IncidentStatus {
    Reported,
    Ongoing,
    Resolved,
    Closed,
}

impl IncidentModel {
    pub fn db_read_by_id_matching_resident(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
        id: uuid::Uuid,
    ) -> diesel::QueryResult<IncidentModel> {
        let incident = IncidentModel::db_read_by_id(conn, id)?;
        let resident = resident_model::ResidentModel::db_get_user(conn, incident.resident_id)?;

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

        Ok(incident)
    }

    pub fn db_read_all_matching_by_range(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<IncidentModel>> {
        let mut query = UserRoleModel::table()
            .inner_join(users::table.on(user_roles::user_id.eq(users::id)))
            .inner_join(residents::table.on(users::entity_id.eq(residents::id)))
            .inner_join(incidents::table.on(users::entity_id.eq(residents::id)))
            .filter(users::entity_type.eq(UserTypes::Resident))
            .filter(user_roles::role.eq(UserRoles::Resident))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                query = query.filter(user_roles::community_id.eq(user_role.community_id));
            }
            UserRoles::Resident => query = query.filter(users::id.eq(user_role.user_id)),
        }

        query
            .select(incidents::all_columns)
            .limit(per_page)
            .offset(offset)
            .load::<IncidentModel>(conn)
    }

    pub fn db_count_all_matching(
        user_role: UserRoleModel,
        conn: &mut PgConnection,
    ) -> diesel::QueryResult<i64> {
        let mut query = UserRoleModel::table()
            .inner_join(users::table.on(user_roles::user_id.eq(users::id)))
            .inner_join(residents::table.on(users::entity_id.eq(residents::id)))
            .inner_join(incidents::table.on(users::entity_id.eq(residents::id)))
            .filter(users::entity_type.eq(UserTypes::Resident))
            .filter(user_roles::role.eq(UserRoles::Resident))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                query = query.filter(user_roles::community_id.eq(user_role.community_id));
            }
            UserRoles::Resident => query = query.filter(users::id.eq(user_role.user_id)),
        }

        query
            .select(incidents::all_columns)
            .count()
            .get_result::<i64>(conn)
    }
}

impl<DB> ToSql<diesel::sql_types::Text, DB> for IncidentStatus
where
    DB: Backend,
    str: ToSql<diesel::sql_types::Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match self {
            IncidentStatus::Reported => "Reported",
            IncidentStatus::Ongoing => "Ongoing",
            IncidentStatus::Resolved => "Resolved",
            IncidentStatus::Closed => "Closed",
        };
        s.to_sql(out)
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for IncidentStatus
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match <String as FromSql<diesel::sql_types::Text, DB>>::from_sql(bytes)?.as_str() {
            "Reported" => Ok(IncidentStatus::Reported),
            "Ongoing" => Ok(IncidentStatus::Ongoing),
            "Resolved" => Ok(IncidentStatus::Resolved),
            "Closed" => Ok(IncidentStatus::Closed),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}
