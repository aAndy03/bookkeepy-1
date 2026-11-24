// SPDX-License-Identifier: GPL-3.0-or-later
//! Entity ids — UUIDv7 (time-ordered), monotonic-ish per process (doc 04 §4.2 naming).

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A time-ordered (v7) UUID entity id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(uuid::Uuid);

impl EntityId {
    /// A fresh id (UUIDv7: roughly time-ordered).
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }

    /// The underlying UUID.
    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for EntityId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ids_are_unique_and_parseable() {
        let a = EntityId::new();
        let b = EntityId::new();
        assert_ne!(a, b);
        assert_eq!(a.to_string().parse::<EntityId>().unwrap(), a);
    }
}
