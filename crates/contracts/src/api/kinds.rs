// SPDX-License-Identifier: GPL-3.0-or-later
//! Event-kind constants (RULE-EVT-001, ADR-0009). This module is the ONLY place dotted
//! event-kind strings are defined in code — engines emit via these consts so a rename is
//! a compile error, not a silent drift. `data/events.toml` is the registry;
//! `cargo xtask govern events` asserts the const set and the registry are identical and
//! that no other file contains a kind literal.

/// `ledger::entries` posted a journal entry.
pub const LEDGER_ENTRY_POSTED: &str = "ledger.entry.posted";
/// `ledger::entries` posted a linked reversal.
pub const LEDGER_ENTRY_REVERSED: &str = "ledger.entry.reversed";
/// `ledger::transactions` created + posted a user transaction.
pub const TRANSACTION_POSTED: &str = "transaction.posted";
/// `ledger::transactions` changed status (incl. voided).
pub const TRANSACTION_STATUS_CHANGED: &str = "transaction.status_changed";
/// `ledger::transactions` edited a posted transaction (reversal + replacement).
pub const TRANSACTION_REPLACED: &str = "transaction.replaced";
/// `ledger::schedules` created a schedule.
pub const SCHEDULE_CREATED: &str = "schedule.created";
/// `ledger::accounts` created an account.
pub const ACCOUNT_CREATED: &str = "account.created";
/// `ledger::catalog` created a category.
pub const CATEGORY_CREATED: &str = "category.created";
/// `ledger::catalog` created a tag.
pub const TAG_CREATED: &str = "tag.created";
/// `ledger::catalog` created a party (payee).
pub const PARTY_CREATED: &str = "party.created";
/// `shared::actions` renamed an entity.
pub const ENTITY_RENAMED: &str = "entity.renamed";
/// `shared::actions` archived (soft-deleted) an entity.
pub const ENTITY_ARCHIVED: &str = "entity.archived";
/// `shared::actions` restored an archived entity.
pub const ENTITY_RESTORED: &str = "entity.restored";
/// `shared::actions` changed an entity's status.
pub const ENTITY_STATUS_CHANGED: &str = "entity.status_changed";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_dotted() {
        for k in [
            LEDGER_ENTRY_POSTED,
            LEDGER_ENTRY_REVERSED,
            TRANSACTION_POSTED,
            TRANSACTION_STATUS_CHANGED,
            TRANSACTION_REPLACED,
            SCHEDULE_CREATED,
            ACCOUNT_CREATED,
            CATEGORY_CREATED,
            TAG_CREATED,
            PARTY_CREATED,
            ENTITY_RENAMED,
            ENTITY_ARCHIVED,
            ENTITY_RESTORED,
            ENTITY_STATUS_CHANGED,
        ] {
            assert!(k.split('.').count() >= 2, "bad kind {k}");
        }
    }
}
