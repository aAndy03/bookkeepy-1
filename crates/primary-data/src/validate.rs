// SPDX-License-Identifier: GPL-3.0-or-later
//! Whole-registry validation, run at app startup and by `cargo xtask govern data`.

use crate::{
    PrimaryDataError, currencies, defaults, errors, load_policies, locales, perf_budgets, settings,
};

/// Validate every primary-data scope (structure + referential integrity, RULE-DATA-002).
pub fn validate_all() -> Result<(), PrimaryDataError> {
    currencies::validate_structure()?;
    errors::validate_registry()?;
    errors::validate_core_codes_registered()?;
    locales::validate_structure()?;
    locales::validate_covers_errors(errors::all())?;
    defaults::validate_structure()?;
    load_policies::validate_structure()?;
    perf_budgets::validate_structure()?;
    settings::validate_structure()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn everything_validates() {
        super::validate_all().unwrap();
    }
}
