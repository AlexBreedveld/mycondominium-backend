use diesel::{QueryResult, RunQueryDsl};
use crate::schema::residents::dsl::residents;
use super::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema)]
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

impl DatabaseTrait for ResidentModel {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(residents).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::update(residents.filter(crate::schema::residents::columns::id.eq(&self.id)))
            .set(self)
            .execute(conn)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(residents)
            .values(self)
            .on_conflict(crate::schema::residents::columns::id)
            .do_update()
            .set(self)
            .execute(conn)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<Self> {
        residents
            .filter(crate::schema::residents::columns::id.eq(id))
            .first(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<usize> {
        diesel::delete(residents.filter(crate::schema::residents::columns::id.eq(id))).execute(conn)
    }

    fn db_delete_all(conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::delete(residents).execute(conn)
    }

    fn db_count_all(conn: &mut PgConnection) -> QueryResult<i64> {
        residents.count().get_result::<i64>(conn)
    }
}

impl DatabaseTraitVec for Vec<ResidentModel> {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(residents).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_updated = 0;
        for item in self {
            let updated =
                diesel::update(residents.filter(crate::schema::residents::columns::id.eq(&item.id)))
                    .set(item)
                    .execute(conn)?;
            total_updated += updated;
        }
        Ok(total_updated)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_upserted = 0;
        for item in self {
            total_upserted += diesel::insert_into(residents)
                .values(item)
                .on_conflict(crate::schema::residents::columns::id)
                .do_update()
                .set(item)
                .execute(conn)?;
        }
        Ok(total_upserted)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<Self> {
        residents
            .filter(crate::schema::residents::columns::id.eq_any(id))
            .load::<ResidentModel>(conn)
    }

    fn db_read_all(conn: &mut PgConnection) -> QueryResult<Self> {
        residents.load::<ResidentModel>(conn)
    }

    fn db_read_by_range(conn: &mut PgConnection, per_page: i64, offset: i64) -> QueryResult<Self> {
        residents.limit(per_page).offset(offset).load::<ResidentModel>(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<usize> {
        diesel::delete(residents.filter(crate::schema::residents::columns::id.eq_any(id))).execute(conn)
    }
}

