use diesel::{QueryResult, RunQueryDsl};
use crate::schema::admins::dsl::admins;
use super::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema)]
#[diesel(table_name = crate::schema::admins)]
pub struct AdminModel {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema)]
#[diesel(table_name = crate::schema::admins)]
pub struct AdminModelNew {
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

impl DatabaseTrait for AdminModel {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(admins).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::update(admins.filter(crate::schema::admins::columns::id.eq(&self.id)))
            .set(self)
            .execute(conn)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(admins)
            .values(self)
            .on_conflict(crate::schema::admins::columns::id)
            .do_update()
            .set(self)
            .execute(conn)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<Self> {
        admins
            .filter(crate::schema::admins::columns::id.eq(id))
            .first(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<usize> {
        diesel::delete(admins.filter(crate::schema::admins::columns::id.eq(id))).execute(conn)
    }

    fn db_delete_all(conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::delete(admins).execute(conn)
    }

    fn db_count_all(conn: &mut PgConnection) -> QueryResult<i64> {
        admins.count().get_result::<i64>(conn)
    }
}

impl DatabaseTraitVec for Vec<AdminModel> {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(admins).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_updated = 0;
        for item in self {
            let updated =
                diesel::update(admins.filter(crate::schema::admins::columns::id.eq(&item.id)))
                    .set(item)
                    .execute(conn)?;
            total_updated += updated;
        }
        Ok(total_updated)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_upserted = 0;
        for item in self {
            total_upserted += diesel::insert_into(admins)
                .values(item)
                .on_conflict(crate::schema::admins::columns::id)
                .do_update()
                .set(item)
                .execute(conn)?;
        }
        Ok(total_upserted)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<Self> {
        admins
            .filter(crate::schema::admins::columns::id.eq_any(id))
            .load::<AdminModel>(conn)
    }

    fn db_read_all(conn: &mut PgConnection) -> QueryResult<Self> {
        admins.load::<AdminModel>(conn)
    }

    fn db_read_by_range(conn: &mut PgConnection, per_page: i64, offset: i64) -> QueryResult<Self> {
        admins.limit(per_page).offset(offset).load::<AdminModel>(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<usize> {
        diesel::delete(admins.filter(crate::schema::admins::columns::id.eq_any(id))).execute(conn)
    }
}