use std::env;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use fernet::Fernet;
use lazy_static::lazy_static;
use regex::Regex;

use crate::schedule::Webhook;

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

const URL_PATTERN: &str = r"/webhooks/(?<id>.*?)/(?<token>.*?)/?$";

pub fn url_to_webhook(s: &str) -> Result<Webhook> {
    let re = Regex::new(URL_PATTERN)?;
    let caps = re
        .captures(s)
        .ok_or_else(|| eyre!("Did not match webhook URL"))?;
    let id = caps
        .name("id")
        .ok_or_else(|| eyre!("Could not find webhook id"))?
        .as_str();
    let token = caps
        .name("token")
        .ok_or_else(|| eyre!("Could not find webhook token"))?
        .as_str();
    let hook = Webhook::new(id.parse()?, encrypt(token)?);
    Ok(hook)
}
