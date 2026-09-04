use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Algorithm, Params, Version,
};
use anyhow::{anyhow, Result};

pub fn hash_password(pwd: &str) -> Result<String> {
    let argon2 = Argon2::new_with_secret(
        b"secret pepper",
        Algorithm::default(),
        Version::default(),
        Params::default(),
    )
    .map_err(|e| anyhow!("failed to init argon2: {e}"))?;

    let salt = SaltString::generate(&mut OsRng);
    let pwd_hash = argon2
        .hash_password(pwd.as_bytes(), &salt)
        .map_err(|e| anyhow!("failed to hash password: {e}"))?
        .to_string();

    Ok(pwd_hash)
}