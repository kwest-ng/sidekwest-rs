use std::env;

use anyhow::Result;
use fernet::Fernet;
use lazy_static::lazy_static;

lazy_static! {
    static ref FERNET: Fernet = {
        let key = env::var("SECRET_KEY").expect("Env var invalid: SECRET_KEY");
        Fernet::new(&key).expect("Failed to build fernet struct")
    };
}

pub fn encrypt(s: &str) -> Result<String> {
    Ok(FERNET.encrypt(s.as_bytes()))
}

pub fn decrypt(token: &str) -> Result<String> {
    let bytes = FERNET.decrypt(token)?;
    let str = String::from_utf8(bytes)?;
    Ok(str)
}

