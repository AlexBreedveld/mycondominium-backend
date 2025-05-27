use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use mycondominium_backend::models::resident_model::ResidentModel;
use mycondominium_backend::services::{DatabaseTrait, MyCondominiumConfig, check_email_exist};
use mycondominium_backend::utilities::auth_utils::*;
use mycondominium_backend::{MIGRATIONS, establish_connection_pg, schema};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_password_hashing() {
    let password: String = "ABCDEF123456".to_string();
    let hashed_password = hash_password(password.clone()).unwrap();

    assert!(check_password(password, hashed_password).unwrap());
}

#[test]
fn test_hash_check() {
    let hash: String = "$argon2id$v=19$m=19456,t=2,p=1$LTw9lap27D4Y0b+G2EGWfQ$JAvHYTII9zhbLkZg/f6w6oAYQASjLoXAMA1krNGgJY8".to_string();
    let password: String = "ABCDEF123456".to_string();

    assert!(check_password(password, hash).unwrap());
}

#[test]
fn test_jwt_token() {
    let user_id = uuid::Uuid::parse_str("cf6762a6-c695-44b5-9a84-cba61b031eea").unwrap();
    let token_id = uuid::Uuid::parse_str("19d92659-3874-401e-a902-2e9c03e07007").unwrap();
    let secret_key = "1a855a3f6a0de874ae624013646e1c8f13e2bbe69f31dd286c99c85e95a43285".to_string();
    let exp_days: i64 = 1;

    let token = generate_jwt_token_no_env(user_id, token_id, secret_key.clone(), exp_days).unwrap();

    assert!(
        validate_token_no_env(&token, secret_key)
            .unwrap()
            .user_id
            .eq(&user_id)
    );
}

#[test]
fn test_email_exist() {
    let config_path = &PathBuf::from("config.yaml");

    if !config_path.exists() || !config_path.is_file() {
        log::error!("Configuration file not found at: {}", config_path.display());
        panic!("Configuration file not found at: {}", config_path.display());
    }

    let conf_str = match std::fs::read_to_string(config_path) {
        Ok(dat) => dat,
        Err(e) => {
            log::error!("Failed to read configuration file: {}", e);
            panic!("Failed to read configuration file: {}", e);
        }
    };

    let conf: MyCondominiumConfig = match serde_yaml::from_str(&conf_str) {
        Ok(dat) => dat,
        Err(e) => {
            log::error!("Failed to parse configuration file: {}", e);
            panic!("Failed to parse configuration file: {}", e);
        }
    };

    let conf: Arc<MyCondominiumConfig> = Arc::new(conf);

    let conn = &mut establish_connection_pg(&conf);
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    match ResidentModel::table()
        .filter(schema::residents::email.eq("example@example.com".to_string()))
        .first::<ResidentModel>(conn)
    {
        Ok(usr) => {
            ResidentModel::db_delete_by_id(conn, usr.id);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    };

    let resident = ResidentModel {
        id: ResidentModel::new_id_user(conn),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        unit_number: None,
        address: None,
        phone: None,
        email: "example@example.com".to_string(),
        date_of_birth: None,
        is_active: false,
        created_at: Default::default(),
        updated_at: Default::default(),
    };

    resident.db_insert(conn);

    let result = check_email_exist(conn, "example@example.com".to_string());
    let result_ne = check_email_exist(conn, "example1@example.com".to_string());

    assert!(result.is_err());
    assert!(result_ne.is_ok());
}
