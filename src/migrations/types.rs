pub use barrel::types::*;

pub fn bigint() -> Type {
    custom("BIGINT")
}

pub fn utc_timestamp() -> Type {
    custom("TIMESTAMP WITHOUT TIME ZONE")
}
