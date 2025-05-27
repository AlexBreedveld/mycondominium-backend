use super::prelude::*;
use crate::models::{common_area_model, user_role_model};
use std::io::ErrorKind;

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
#[diesel(table_name = reservations)]
pub struct ReservationModel {
    pub id: Uuid,
    pub resident_id: Uuid,
    pub common_area_id: Uuid,
    pub reservation_date: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub status: ReservationStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct ReservationModelNew {
    pub resident_id: Uuid,
    pub common_area_id: Uuid,
    pub reservation_date: NaiveDateTime,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum ReservationStatus {
    Reserved,
    Ongoing,
    Finished,
}

impl<DB> ToSql<diesel::sql_types::Text, DB> for ReservationStatus
where
    DB: Backend,
    str: ToSql<diesel::sql_types::Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match self {
            ReservationStatus::Reserved => "Reserved",
            ReservationStatus::Ongoing => "Ongoing",
            ReservationStatus::Finished => "Finished",
        };
        s.to_sql(out)
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for ReservationStatus
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match <String as FromSql<diesel::sql_types::Text, DB>>::from_sql(bytes)?.as_str() {
            "Reserved" => Ok(ReservationStatus::Reserved),
            "Ongoing" => Ok(ReservationStatus::Ongoing),
            "Finished" => Ok(ReservationStatus::Finished),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

impl ReservationModel {
    pub fn check_for_overlap(&self, conn: &mut PgConnection) -> Result<(), std::io::Error> {
        use crate::schema::reservations::dsl::*;
        use diesel::prelude::*;

        let overlapping_reservations = reservations
            .filter(common_area_id.eq(self.common_area_id))
            .filter(id.ne(self.id))
            .filter(
                start_time
                    .le(self.end_time)
                    .and(end_time.ge(self.start_time)),
            )
            .first::<ReservationModel>(conn)
            .optional();

        match overlapping_reservations {
            Ok(Some(overlap)) => {
                let overlap_start = overlap.start_time.format("%H:%M:%S").to_string();
                let overlap_end = overlap.end_time.format("%H:%M:%S").to_string();
                let new_start = self.start_time.format("%H:%M:%S").to_string();
                let new_end = self.end_time.format("%H:%M:%S").to_string();

                Err(std::io::Error::new(
                    ErrorKind::AddrInUse,
                    format!(
                        "Reservation time slot ({} to {}) overlaps with an existing reservation ({} to {})",
                        new_start, new_end, overlap_start, overlap_end
                    ),
                ))
            }
            Ok(None) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                ErrorKind::ConnectionRefused,
                format!("Database error while checking for overlaps: {}", e),
            )),
        }
    }

    pub fn get_community_id(&self, conn: &mut PgConnection) -> diesel::QueryResult<Uuid> {
        match common_area_model::CommonAreaModel::db_read_by_id(conn, self.common_area_id) {
            Ok(area) => Ok(area.community_id),
            Err(e) => Err(e),
        }
    }

    pub fn db_count_all_matching(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
    ) -> diesel::QueryResult<i64> {
        let mut query = ReservationModel::table()
            .inner_join(common_areas::table.on(reservations::common_area_id.eq(common_areas::id)))
            .inner_join(communities::table.on(common_areas::community_id.eq(communities::id)))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin | UserRoles::Resident => {
                query = query.filter(communities::id.eq(user_role.community_id.unwrap()));
            }
        }

        query
            .select(reservations::all_columns)
            .count()
            .get_result::<i64>(conn)
    }

    pub fn db_read_all_matching_by_range(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
        per_page: i64,
        offset: i64,
    ) -> diesel::QueryResult<Vec<ReservationModel>> {
        let mut query = ReservationModel::table()
            .inner_join(common_areas::table.on(reservations::common_area_id.eq(common_areas::id)))
            .inner_join(communities::table.on(common_areas::community_id.eq(communities::id)))
            .into_boxed();

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin | UserRoles::Resident => {
                query = query.filter(communities::id.eq(user_role.community_id.unwrap()));
            }
        }

        query
            .select(reservations::all_columns)
            .limit(per_page)
            .offset(offset)
            .load::<ReservationModel>(conn)
    }

    pub fn db_read_by_id_matching(
        user_role: user_role_model::UserRoleModel,
        conn: &mut PgConnection,
        id: uuid::Uuid,
    ) -> diesel::QueryResult<ReservationModel> {
        let reservation = ReservationModel::db_read_by_id(conn, id)?;

        let community_id = reservation.get_community_id(conn)?;

        match user_role.role {
            UserRoles::Root => {}
            UserRoles::Admin => {
                if user_role.community_id != Some(community_id) {
                    return Err(diesel::result::Error::NotFound);
                }
            }
            UserRoles::Resident => {
                if user_role.community_id != Some(community_id) {
                    return Err(diesel::result::Error::NotFound);
                }
            }
        }

        Ok(reservation)
    }
}

impl ReservationModelNew {
    pub fn check_for_overlap(&self, conn: &mut PgConnection) -> Result<(), std::io::Error> {
        use crate::schema::reservations::dsl::*;
        use diesel::prelude::*;

        let overlapping_reservations = reservations
            .filter(common_area_id.eq(self.common_area_id))
            .filter(
                start_time
                    .le(self.end_time)
                    .and(end_time.ge(self.start_time)),
            )
            .first::<ReservationModel>(conn)
            .optional();

        match overlapping_reservations {
            Ok(Some(overlap)) => {
                let overlap_start = overlap.start_time.format("%H:%M:%S").to_string();
                let overlap_end = overlap.end_time.format("%H:%M:%S").to_string();
                let new_start = self.start_time.format("%H:%M:%S").to_string();
                let new_end = self.end_time.format("%H:%M:%S").to_string();

                Err(std::io::Error::new(
                    ErrorKind::AddrInUse,
                    format!(
                        "Reservation time slot ({} to {}) overlaps with an existing reservation ({} to {})",
                        new_start, new_end, overlap_start, overlap_end
                    ),
                ))
            }
            Ok(None) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                ErrorKind::ConnectionRefused,
                format!("Database error while checking for overlaps: {}", e),
            )),
        }
    }
    pub fn get_community_id(&self, conn: &mut PgConnection) -> diesel::QueryResult<Uuid> {
        match common_area_model::CommonAreaModel::db_read_by_id(conn, self.common_area_id) {
            Ok(area) => Ok(area.community_id),
            Err(e) => Err(e),
        }
    }
}
