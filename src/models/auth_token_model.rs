use diesel::{QueryResult, RunQueryDsl};
use crate::schema::auth_tokens::dsl::auth_tokens;
use super::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug, AsChangeset, Validate, ToSchema)]
#[diesel(table_name = crate::schema::auth_tokens)]
pub struct AuthTokenModel {
    pub user_id: Uuid,
    pub id: Uuid,
    pub time_added: NaiveDateTime,
    pub active: bool,
    pub time_last_used: NaiveDateTime,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub version: Option<String>,
    pub cpu_arch: Option<String>,
}

impl DatabaseTrait for AuthTokenModel {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(auth_tokens).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::update(auth_tokens.filter(crate::schema::auth_tokens::columns::id.eq(&self.id)))
            .set(self)
            .execute(conn)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(auth_tokens)
            .values(self)
            .on_conflict(crate::schema::auth_tokens::columns::id)
            .do_update()
            .set(self)
            .execute(conn)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<Self> {
        auth_tokens
            .filter(crate::schema::auth_tokens::columns::id.eq(id))
            .first(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<usize> {
        diesel::delete(auth_tokens.filter(crate::schema::auth_tokens::columns::id.eq(id))).execute(conn)
    }

    fn db_delete_all(conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::delete(auth_tokens).execute(conn)
    }

    fn db_count_all(conn: &mut PgConnection) -> QueryResult<i64> {
        auth_tokens.count().get_result::<i64>(conn)
    }
}

impl DatabaseTraitVec for Vec<AuthTokenModel> {
    type Id = Uuid;

    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(auth_tokens).values(self).execute(conn)
    }

    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_updated = 0;
        for item in self {
            let updated =
                diesel::update(auth_tokens.filter(crate::schema::auth_tokens::columns::id.eq(&item.id)))
                    .set(item)
                    .execute(conn)?;
            total_updated += updated;
        }
        Ok(total_updated)
    }

    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        let mut total_upserted = 0;
        for item in self {
            total_upserted += diesel::insert_into(auth_tokens)
                .values(item)
                .on_conflict(crate::schema::auth_tokens::columns::id)
                .do_update()
                .set(item)
                .execute(conn)?;
        }
        Ok(total_upserted)
    }

    fn db_read_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<Self> {
        auth_tokens
            .filter(crate::schema::auth_tokens::columns::id.eq_any(id))
            .load::<AuthTokenModel>(conn)
    }

    fn db_read_all(conn: &mut PgConnection) -> QueryResult<Self> {
        auth_tokens.load::<AuthTokenModel>(conn)
    }

    fn db_read_by_range(conn: &mut PgConnection, per_page: i64, offset: i64) -> QueryResult<Self> {
        auth_tokens.limit(per_page).offset(offset).load::<AuthTokenModel>(conn)
    }

    fn db_delete_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<usize> {
        diesel::delete(auth_tokens.filter(crate::schema::auth_tokens::columns::id.eq_any(id))).execute(conn)
    }
}