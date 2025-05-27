use crate::models::auth_model::TokenClaims;
use crate::models::auth_token_model;
use crate::models::auth_token_model::{AuthTokenModel, FromUaParser, UserAgent};
use crate::models::user_role_model::UserRoleModel;
use crate::services::DatabaseTrait;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use std::io::ErrorKind;
use user_agent_parser::UserAgentParser;

pub fn hash_password(password: String) -> Result<String, std::io::Error> {
    use password_hash::PasswordHasher;

    let salt = password_hash::SaltString::generate(&mut password_hash::rand_core::OsRng);
    let argon2 = argon2::Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err(std::io::Error::other("Error hashing password:".to_string())),
    }
}

/// Verifies if a given plain-text password matches the provided hashed password.
///
/// This function takes a plain-text password and a hashed password string generated using the
/// Argon2 hashing algorithm, then verifies if the plain-text password matches the hashed password.
/// It returns a boolean indicating the result of verification.
///
/// # Arguments
///
/// * `password` - A `String` representing the plain-text password to check.
/// * `hashed_password` - A `String` representing the previously hashed password to compare against.
///
/// # Returns
///
/// * `Ok(true)` if the password verification succeeds.
/// * `Ok(false)` if the provided plain-text password does not match the hashed password.
/// * `Err` - an instance of `std::io::Error` if an error occurs while parsing the hashed password.
///
/// # Errors
///
/// Returns an error (`std::io::Error`) if the provided `hashed_password` cannot be parsed successfully.
pub fn check_password(password: String, hashed_password: String) -> Result<bool, std::io::Error> {
    use password_hash::PasswordVerifier;

    let argon2 = argon2::Argon2::default();
    let parsed_hash = match password_hash::PasswordHash::new(&hashed_password) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("Error parsing hashed password: {}", e);
            return Err(std::io::Error::other(
                "Error parsing hashed password".to_string(),
            ));
        }
    };
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn generate_jwt_token(
    user_id: uuid::Uuid,
    token_id: uuid::Uuid,
) -> Result<String, std::io::Error> {
    let exp_days = match std::env::var("AUTH_TOKEN_EXPIRATION_DAYS") {
        Ok(val) => val,
        Err(_) => {
            log::error!("AUTH_TOKEN_EXPIRATION_DAYS environment variable not found.");
            return Err(std::io::Error::other(
                "AUTH_TOKEN_EXPIRATION_DAYS environment variable not found".to_string(),
            ));
        }
    };
    let exp_days = match exp_days.parse::<i64>() {
        Ok(val) => val,
        Err(_) => {
            log::error!("Could not parse AUTH_TOKEN_EXPIRATION_DAYS as an integer.");
            return Err(std::io::Error::other(
                "AUTH_TOKEN_EXPIRATION_DAYS could not be parsed as an integer".to_string(),
            ));
        }
    };
    let secret_key = match std::env::var("AUTH_TOKEN_SECRET_KEY") {
        Ok(val) => val,
        Err(_) => {
            log::error!("AUTH_TOKEN_SECRET_KEY environment variable not found.");
            return Err(std::io::Error::other(
                "AUTH_TOKEN_SECRET_KEY environment variable not found".to_string(),
            ));
        }
    };
    generate_jwt_token_internal(user_id, token_id, secret_key, exp_days)
}

pub fn generate_jwt_token_no_env(
    user_id: uuid::Uuid,
    token_id: uuid::Uuid,
    secret_key: String,
    exp_days: i64,
) -> Result<String, std::io::Error> {
    generate_jwt_token_internal(user_id, token_id, secret_key, exp_days)
}

fn generate_jwt_token_internal(
    user_id: uuid::Uuid,
    token_id: uuid::Uuid,
    secret_key: String,
    exp_days: i64,
) -> Result<String, std::io::Error> {
    // Convert the UUID to a string representation
    //let user_id_str = user_id.to_string(); // UUID as a string

    // Get the current time in seconds since UNIX EPOCH
    let now = chrono::Utc::now();

    let expiration_time = now + chrono::Duration::days(exp_days);

    // Convert the expiration time to a Unix timestamp (seconds)
    let exp_timestamp = expiration_time.timestamp() as usize;

    // Define the claims of the token
    let claims = TokenClaims {
        token_id,
        user_id,
        exp: exp_timestamp,
    };

    // Encode the JWT with a secret key
    let encoding_key = jsonwebtoken::EncodingKey::from_secret(secret_key.as_bytes());
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &encoding_key,
    )
    .unwrap();

    Ok(token)
}

pub fn validate_token(token: &str) -> jsonwebtoken::errors::Result<TokenClaims> {
    let secret_key =
        std::env::var("AUTH_TOKEN_SECRET_KEY").expect("AUTH_TOKEN_SECRET_KEY must be set");
    validate_token_internal(token, secret_key)
}

pub fn validate_token_from_header(
    req: actix_web::HttpRequest,
    secret_key: String,
) -> Result<TokenClaims, std::io::Error> {
    match req.headers().get("X-Auth-Token") {
        Some(header) => {
            let token = header.to_str().unwrap_or("");
            match validate_token_no_env(token, secret_key) {
                Ok(claims) => Ok(claims),
                Err(_) => Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Error validating token".to_string(),
                )),
            }
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "X-Auth-Token header is missing".to_string(),
        )),
    }
}

pub fn validate_token_no_env(
    token: &str,
    secret_key: String,
) -> jsonwebtoken::errors::Result<TokenClaims> {
    validate_token_internal(token, secret_key)
}

fn validate_token_internal(
    token: &str,
    secret_key: String,
) -> jsonwebtoken::errors::Result<TokenClaims> {
    // Define validation parameters
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;

    // Decode the token
    let decoded_token = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret_key.as_bytes()),
        &validation,
    )?;

    // Extract claims
    let claims = decoded_token.claims;

    Ok(claims)
}

pub fn parse_user_agent(ua_str: String) -> Result<UserAgent, std::io::Error> {
    let parser = match UserAgentParser::from_str(include_str!("../../res/ua-regexes.yaml")) {
        Ok(parser) => parser,
        Err(e) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!("Error parsing User-Agent regexes: {}", e),
            ));
        }
    };

    let ua_str = &mut ua_str.clone();

    let ua_cpu: user_agent_parser::CPU = parser.parse_cpu(ua_str.as_str());
    let ua_os: user_agent_parser::OS = parser.parse_os(ua_str.as_str());
    let ua_device: user_agent_parser::Device = parser.parse_device(ua_str.as_str());
    let ua_engine: user_agent_parser::Engine = parser.parse_engine(ua_str.as_str());
    let ua_product: user_agent_parser::Product = parser.parse_product(ua_str.as_str());

    let ua: UserAgent = UserAgent {
        cpu: auth_token_model::UaCPU::from_ua_parser(ua_cpu),
        os: auth_token_model::UaOS::from_ua_parser(ua_os),
        device: auth_token_model::UaDevice::from_ua_parser(ua_device),
        engine: auth_token_model::UaEngine::from_ua_parser(ua_engine),
        product: auth_token_model::UaProduct::from_ua_parser(ua_product),
    };

    Ok(ua)
}

pub fn authenticate_user(
    req: actix_web::HttpRequest,
    conn: &mut PgConnection,
    conf: actix_web::web::Data<std::sync::Arc<crate::MyCondominiumConfig>>,
) -> Result<(UserRoleModel, TokenClaims, AuthTokenModel), std::io::Error> {
    match validate_token_from_header(req, conf.auth.token_secret_key.clone()) {
        Ok(claims) => {
            match auth_token_model::AuthTokenModel::db_read_by_id(conn, claims.token_id) {
                Ok(token) => {
                    if !token.active {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            "Token is not active".to_string(),
                        ));
                    }

                    match UserRoleModel::table()
                        .filter(crate::schema::user_roles::user_id.eq(claims.user_id))
                        .first::<UserRoleModel>(conn)
                    {
                        Ok(role) => Ok((role, claims, token)),
                        Err(e) => {
                            log::error!("Error getting user role: {}", e);
                            Err(std::io::Error::other("Error getting user role".to_string()))
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error reading token from database: {}", e);
                    Err(std::io::Error::other(
                        "Error reading token from database".to_string(),
                    ))
                }
            }
        }
        Err(e) => {
            log::error!("Invalid token: {}", e);
            Err(std::io::Error::other("Invalid token".to_string()))
        }
    }
}
