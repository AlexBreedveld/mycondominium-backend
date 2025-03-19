use diesel::{deserialize, serialize};
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserRoles {
    Root,
    Admin,
    Resident,
    Guest
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
            UserRoles::Guest => "Guest",
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
            "Guest" => Ok(UserRoles::Guest),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}
