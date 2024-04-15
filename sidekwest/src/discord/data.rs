use std::num::NonZeroU64;

use nutype::nutype;
use serde::{Deserialize, Serialize};

#[nutype(
    new_unchecked,
    validate(greater = 0),
    derive(
        Copy,
        Debug,
        Clone,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        FromStr,
        AsRef,
        Deref,
        TryFrom,
        Into,
        Hash,
        Borrow,
        Display,
        Serialize,
        Deserialize
    )
)]
pub struct Snowflake(u64);

impl From<Snowflake> for NonZeroU64 {
    fn from(value: Snowflake) -> Self {
        // SAFETY: We sanitize snowflake inputs, so that this cannot fail.
        // In the (hopefully) impossible case this does fail, we will not
        // see memory unsafety, only API errors.
        unsafe { NonZeroU64::new_unchecked(value.into_inner()) }
    }
}

impl From<NonZeroU64> for Snowflake {
    fn from(value: NonZeroU64) -> Self {
        unsafe {
            Self::new_unchecked(value.into())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Webhook {
    id: Snowflake,
    encrypted_token: String,
}

impl Webhook {
    pub fn new(id: Snowflake, encrypted_token: String) -> Self {
        Self {
            id,
            encrypted_token,
        }
    }
    
    pub fn id(&self) -> Snowflake {
        self.id
    }
    
    pub fn encrypted_token(&self) -> &str {
        &self.encrypted_token
    }
}
