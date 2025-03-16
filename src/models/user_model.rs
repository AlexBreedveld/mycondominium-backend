use diesel::{QueryResult, RunQueryDsl};
use crate::schema::users::dsl::users;
use super::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema)]
#[diesel(table_name = crate::schema::users)]
pub struct UserModel {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub roles: Option<String>,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl DatabaseTrait for UserModel {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(users).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::update(users.filter(crate::schema::users::columns::id.eq(&self.id)))
            .set(self)
            .execute(conn)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(users)
            .values(self)
            .on_conflict(crate::schema::users::columns::id)
            .do_update()
            .set(self)
            .execute(conn)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<Self> {
        users
            .filter(crate::schema::users::columns::id.eq(id))
            .first(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<usize> {
        diesel::delete(users.filter(crate::schema::users::columns::id.eq(id))).execute(conn)
    }

    fn db_delete_all(conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::delete(users).execute(conn)
    }

    fn db_count_all(conn: &mut PgConnection) -> QueryResult<i64> {
        users.count().get_result::<i64>(conn)
    }
}

impl DatabaseTraitVec for Vec<UserModel> {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(users).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_updated = 0;
        for item in self {
            let updated =
                diesel::update(users.filter(crate::schema::users::columns::id.eq(&item.id)))
                    .set(item)
                    .execute(conn)?;
            total_updated += updated;
        }
        Ok(total_updated)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_upserted = 0;
        for item in self {
            total_upserted += diesel::insert_into(users)
                .values(item)
                .on_conflict(crate::schema::users::columns::id)
                .do_update()
                .set(item)
                .execute(conn)?;
        }
        Ok(total_upserted)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<Self> {
        users
            .filter(crate::schema::users::columns::id.eq_any(id))
            .load::<UserModel>(conn)
    }

    fn db_read_all(conn: &mut PgConnection) -> QueryResult<Self> {
        users.load::<UserModel>(conn)
    }

    fn db_read_by_range(conn: &mut PgConnection, per_page: i64, offset: i64) -> QueryResult<Self> {
        users.limit(per_page).offset(offset).load::<UserModel>(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<usize> {
        diesel::delete(users.filter(crate::schema::users::columns::id.eq_any(id))).execute(conn)
    }
}