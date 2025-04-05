use mycondominium_backend::utilities::auth_utils::*;

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

    assert!(validate_token_no_env(&token, secret_key).unwrap().user_id.eq(&user_id));
}