use serde::Deserializer;
use super::prelude::*;

pub trait DatabaseTrait {

    type Id;
    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize>;
    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize>;
    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize>;
    fn db_read_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<Self>
    where
        Self: Sized;
    fn db_delete_by_id(conn: &mut PgConnection, id: Self::Id) -> QueryResult<usize>;
    fn db_delete_all(conn: &mut PgConnection) -> QueryResult<usize>;
}

pub trait DatabaseTraitVec {
    type Id;
    fn db_insert(&self, conn: &mut PgConnection) -> QueryResult<usize>;
    fn db_update(&self, conn: &mut PgConnection) -> QueryResult<usize>;
    fn db_upsert(&self, conn: &mut PgConnection) -> QueryResult<usize>;
    fn db_read_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<Self>
    where
        Self: Sized;
    fn db_read_all(conn: &mut PgConnection) -> QueryResult<Self>
    where
        Self: Sized;
    fn db_delete_by_id(conn: &mut PgConnection, id: Vec<Self::Id>) -> QueryResult<usize>;
}

pub fn parse_naive_datetime<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    let datetime_str = &s[..19]; // Remove the timezone offset
    let format = "%Y-%m-%dT%H:%M:%S";
    NaiveDateTime::parse_from_str(datetime_str, format).map_err(serde::de::Error::custom)
}

pub fn parse_naive_datetime_opt<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    let datetime_str = &s[..19]; // Remove the timezone offset
    let format = "%Y-%m-%dT%H:%M:%S";
    NaiveDateTime::parse_from_str(datetime_str, format)
        .map(Some)
        .map_err(serde::de::Error::custom)
}
