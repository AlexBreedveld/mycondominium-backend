use diesel::{deserialize, serialize, AsExpression, FromSqlRow};
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use crate::services::{Deserialize, Serialize, ToSchema};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum UserTypes {
    Admin,
    Resident,
}

impl<DB> ToSql<diesel::sql_types::Text, DB> for UserTypes
where
    DB: Backend,
    str: ToSql<diesel::sql_types::Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match self {
            UserTypes::Admin => "admin",
            UserTypes::Resident => "resident",
        };
        s.to_sql(out)
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for UserTypes
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match <String as FromSql<diesel::sql_types::Text, DB>>::from_sql(bytes)?.as_str() {
            "admin" => Ok(UserTypes::Admin),
            "resident" => Ok(UserTypes::Resident),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}
