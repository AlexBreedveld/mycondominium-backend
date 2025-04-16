use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::{AsExpression, FromSqlRow, deserialize, serialize};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum UserRoles {
    Root,
    Admin,
    Resident
}

impl<DB> ToSql<diesel::sql_types::Text, DB> for UserRoles
where
    DB: Backend,
    str: ToSql<diesel::sql_types::Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        let s = match self {
            UserRoles::Root => "Root",
            UserRoles::Admin => "Admin",
            UserRoles::Resident => "Resident",
        };
        s.to_sql(out)
    }
}

impl<DB> FromSql<diesel::sql_types::Text, DB> for UserRoles
where
    DB: Backend,
    String: FromSql<diesel::sql_types::Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match <String as FromSql<diesel::sql_types::Text, DB>>::from_sql(bytes)?.as_str() {
            "Root" => Ok(UserRoles::Root),
            "Admin" => Ok(UserRoles::Admin),
            "Resident" => Ok(UserRoles::Resident),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}
