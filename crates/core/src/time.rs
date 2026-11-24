//! Time types (jiff, with the IANA tz database — primary data as a dependency, doc 04 §4.1).
//!
//! fintecheng (doc 06 §6.4) distinguishes **value time** (when it happened) vs **booking
//! time** (when recorded); both are stored per entry, reports use value time. All timestamps
//! are UTC instants; civil dates are timezone-aware via jiff.

pub use jiff::Timestamp;
pub use jiff::civil::{Date, DateTime};

/// Current UTC instant.
pub fn now() -> Timestamp {
    Timestamp::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_sane() {
        let t = now();
        assert!(t.as_second() > 1_700_000_000);
    }
}
