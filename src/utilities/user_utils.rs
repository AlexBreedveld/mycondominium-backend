use diesel::prelude::*;
use diesel::{PgConnection, RunQueryDsl};
use std::io::ErrorKind;

pub fn check_email_exist(conn: &mut PgConnection, email: String) -> Result<(), std::io::Error> {
    let email = email.trim().to_string();
    match crate::schema::admins::table
        .filter(crate::schema::admins::email.eq(&email))
        .count()
        .get_result::<i64>(conn)
    {
        Ok(num) => {
            if num != 0 {
                return Err(std::io::Error::new(
                    ErrorKind::AddrInUse,
                    "Email already exists",
                ));
            }
        }
        Err(e) => {
            log::error!("Error checking if email exists: {}", e);
            return Err(std::io::Error::other("Error checking if email exists"));
        }
    };

    match crate::schema::residents::table
        .filter(crate::schema::residents::email.eq(&email))
        .count()
        .get_result::<i64>(conn)
    {
        Ok(num) => {
            if num != 0 {
                return Err(std::io::Error::new(
                    ErrorKind::AddrInUse,
                    "Email already exists",
                ));
            }
        }
        Err(e) => {
            log::error!("Error checking if email exists: {}", e);
            return Err(std::io::Error::other("Error checking if email exists"));
        }
    };

    match crate::schema::resident_invites::table
        .filter(crate::schema::resident_invites::email.eq(&email))
        .count()
        .get_result::<i64>(conn)
    {
        Ok(num) => {
            if num != 0 {
                return Err(std::io::Error::new(
                    ErrorKind::AddrInUse,
                    "Email already exists",
                ));
            }
        }
        Err(e) => {
            log::error!("Error checking if email exists: {}", e);
            return Err(std::io::Error::other("Error checking if email exists"));
        }
    };

    Ok(())
}

pub fn user_check_email_valid(
    conn: &mut PgConnection,
    req_email: String,
    curr_email: String,
) -> Result<(), std::io::Error> {
    if req_email.trim() == curr_email.trim() {
        return Ok(());
    }

    match crate::schema::admins::table
        .filter(crate::schema::admins::email.eq(req_email.clone()))
        .count()
        .get_result::<i64>(conn)
    {
        Ok(num) => {
            if num != 0 {
                return Err(std::io::Error::new(
                    ErrorKind::AddrInUse,
                    "Email already exists",
                ));
            }
        }
        Err(e) => {
            log::error!("Error checking if email exists: {}", e);
            return Err(std::io::Error::other("Error checking if email exists"));
        }
    };

    match crate::schema::residents::table
        .filter(crate::schema::residents::email.eq(req_email.clone()))
        .count()
        .get_result::<i64>(conn)
    {
        Ok(num) => {
            if num != 0 {
                return Err(std::io::Error::new(
                    ErrorKind::AddrInUse,
                    "Email already exists",
                ));
            }
        }
        Err(e) => {
            log::error!("Error checking if email exists: {}", e);
            return Err(std::io::Error::other("Error checking if email exists"));
        }
    };

    Ok(())
}
