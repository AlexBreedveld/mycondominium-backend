pub fn hash_password(password: String) -> Result<String, std::io::Error> {
    use password_hash::PasswordHasher;
    
    let salt = password_hash::SaltString::generate(&mut password_hash::rand_core::OsRng);
    let argon2 = argon2::Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error hashing password:".to_string(),
        )),
    }
}


pub fn check_password(password: String, hashed_password: String) -> Result<bool, std::io::Error> {
    use password_hash::PasswordVerifier;
    
    let argon2 = argon2::Argon2::default();
    let parsed_hash = match password_hash::PasswordHash::new(&hashed_password) {
        Ok(hash) => hash,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error parsing hashed password".to_string(),
            ))
        }
    };
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

