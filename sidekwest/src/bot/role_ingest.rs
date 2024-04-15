use std::num::NonZeroU64;

use nutype::nutype;

#[nutype(
    validate(greater = 0),
    derive(
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
struct Snowflake(u64);

impl From<Snowflake> for NonZeroU64 {
    fn from(value: Snowflake) -> Self {
        // SAFETY: We sanitize snowflake inputs, so that this cannot fail.
        // In the (hopefully) impossible case this does fail, we will not
        // see memory unsafety, only API errors.
        unsafe { NonZeroU64::new_unchecked(value.into_inner()) }
    }
}