// SPDX-License-Identifier: GPL-3.0-or-later
//! `BookkeepyAssets` — the app's asset source (RULE-UI-005, DESIGN-ASSETS): serves the
//! design language's **Phosphor** glyphs (references/phosphor, OFL/MIT, catalog.yaml;
//! TOKENS §08: regular at rest, duotone on interaction) at gpui-kit's lucide icon paths,
//! so every `IconName::…` call site renders the designed glyph with no API churn. Paths
//! with no Phosphor mapping fall through to the default gpui-kit lucide set, so nothing
//! renders empty.
//!
//! Two weights are served: `icons/<name>.svg` → Phosphor **regular** (the rest state),
//! `icons/duotone/<name>.svg` → Phosphor **duotone** (the interaction state). Components
//! that swap weight on hover draw the duotone path explicitly via [`crate::language::atoms`]
//! (gpui-kit 0.11's `Icon` has no weight axis, but `Icon::path` is the seam).

use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString};

/// Asset path prefix under which the duotone weight is served (`icons/duotone/<name>.svg`).
pub const DUOTONE_PREFIX: &str = "icons/duotone/";

/// Phosphor duotone SVG bytes for a gpui-kit lucide icon path (`icons/<name>.svg`).
/// The map is the lucide→phosphor name translation for the icons the design uses.
fn phosphor_bytes(path: &str) -> Option<&'static [u8]> {
    if let Some(name) = path.strip_prefix(DUOTONE_PREFIX) {
        return phosphor_duotone_bytes(name);
    }
    lucide_to_phosphor(path).and_then(phosphor_regular_bytes)
}

/// The lucide→Phosphor glyph-name translation for the icons the design uses
/// (`icons/circle-check.svg` → `check-circle`, …). Shared by the regular and the
/// duotone weights so both resolve the same master.
fn lucide_to_phosphor(path: &str) -> Option<&'static str> {
    Some(match path {
        "icons/arrow-up-right.svg" => "arrow-up-right",
        "icons/bell.svg" => "bell",
        "icons/calendar.svg" => "calendar-blank",
        "icons/chart-pie.svg" => "chart-pie-slice",
        "icons/trending-up.svg" => "chart-line-up",
        "icons/check.svg" => "check",
        "icons/chevron-down.svg" => "caret-down",
        "icons/chevron-right.svg" => "caret-right",
        "icons/chevron-up.svg" => "caret-up",
        "icons/chevrons-up-down.svg" => "caret-up-down",
        "icons/circle-check.svg" => "check-circle",
        "icons/alert-circle.svg" => "warning-circle",
        "icons/circle-alert.svg" => "warning-circle",
        "icons/circle-dollar-sign.svg" => "currency-circle-dollar",
        "icons/clock.svg" => "clock-counter-clockwise",
        "icons/copy.svg" => "copy",
        "icons/delete.svg" => "trash",
        "icons/download.svg" => "download",
        "icons/ellipsis.svg" => "dots-three",
        "icons/external-link.svg" => "arrow-square-out",
        "icons/eye.svg" => "eye",
        "icons/eye-off.svg" => "eye-closed",
        "icons/file.svg" => "file",
        "icons/file-text.svg" => "file-text",
        "icons/filter.svg" => "funnel",
        "icons/hard-drive.svg" => "hard-drive",
        "icons/heart.svg" => "heart",
        "icons/help-circle.svg" => "question",
        "icons/history.svg" => "clock-counter-clockwise",
        "icons/home.svg" => "house",
        "icons/house.svg" => "house",
        "icons/inbox.svg" => "tray",
        "icons/landmark.svg" => "bank",
        "icons/layout-dashboard.svg" => "squares-four",
        "icons/loader-circle.svg" => "spinner-gap",
        "icons/cloud-off.svg" => "cloud-slash",
        "icons/maximize.svg" => "arrows-out",
        "icons/minus.svg" => "minus",
        "icons/moon.svg" => "moon",
        "icons/palette.svg" => "palette",
        "icons/panel-left.svg" => "sidebar-simple",
        "icons/pencil.svg" => "pencil",
        "icons/piggy-bank.svg" => "piggy-bank",
        "icons/plus.svg" => "plus",
        "icons/receipt.svg" => "receipt",
        "icons/redo-2.svg" => "arrow-clockwise",
        "icons/refresh-cw.svg" => "arrows-clockwise",
        "icons/repeat.svg" => "repeat",
        "icons/search.svg" => "magnifying-glass",
        "icons/settings.svg" => "gear-six",
        "icons/square.svg" => "square",
        "icons/star.svg" => "star",
        "icons/sticky-note.svg" => "note",
        "icons/sun.svg" => "sun",
        "icons/tag.svg" => "tag",
        "icons/undo-2.svg" => "arrow-counter-clockwise",
        "icons/upload.svg" => "export",
        "icons/user.svg" => "user",
        "icons/users.svg" => "users",
        "icons/wallet.svg" => "wallet",
        "icons/x.svg" => "x",
        _ => return None,
    })
}

/// Embedded Phosphor **regular** bytes for a Phosphor glyph name.
fn phosphor_regular_bytes(svg: &str) -> Option<&'static [u8]> {
    Some(
        match svg {
            "arrow-clockwise" => {
                include_str!("../../../references/phosphor/regular/arrow-clockwise.svg")
            }
            "arrow-up-right" => {
                include_str!("../../../references/phosphor/regular/arrow-up-right.svg")
            }
            "export" => include_str!("../../../references/phosphor/regular/export.svg"),
            "arrow-counter-clockwise" => {
                include_str!("../../../references/phosphor/regular/arrow-counter-clockwise.svg")
            }
            "arrow-square-out" => {
                include_str!("../../../references/phosphor/regular/arrow-square-out.svg")
            }
            "arrows-clockwise" => {
                include_str!("../../../references/phosphor/regular/arrows-clockwise.svg")
            }
            "arrows-out" => include_str!("../../../references/phosphor/regular/arrows-out.svg"),
            "bank" => include_str!("../../../references/phosphor/regular/bank.svg"),
            "bell" => include_str!("../../../references/phosphor/regular/bell.svg"),
            "calendar-blank" => {
                include_str!("../../../references/phosphor/regular/calendar-blank.svg")
            }
            "caret-down" => include_str!("../../../references/phosphor/regular/caret-down.svg"),
            "caret-right" => include_str!("../../../references/phosphor/regular/caret-right.svg"),
            "caret-up" => include_str!("../../../references/phosphor/regular/caret-up.svg"),
            "caret-up-down" => {
                include_str!("../../../references/phosphor/regular/caret-up-down.svg")
            }
            "chart-line-up" => {
                include_str!("../../../references/phosphor/regular/chart-line-up.svg")
            }
            "chart-pie-slice" => {
                include_str!("../../../references/phosphor/regular/chart-pie-slice.svg")
            }
            "check" => include_str!("../../../references/phosphor/regular/check.svg"),
            "check-circle" => include_str!("../../../references/phosphor/regular/check-circle.svg"),
            "clock-counter-clockwise" => {
                include_str!("../../../references/phosphor/regular/clock-counter-clockwise.svg")
            }
            "cloud-slash" => include_str!("../../../references/phosphor/regular/cloud-slash.svg"),
            "copy" => include_str!("../../../references/phosphor/regular/copy.svg"),
            "currency-circle-dollar" => {
                include_str!("../../../references/phosphor/regular/currency-circle-dollar.svg")
            }
            "download" => include_str!("../../../references/phosphor/regular/download.svg"),
            "dots-three" => include_str!("../../../references/phosphor/regular/dots-three.svg"),
            "eye" => include_str!("../../../references/phosphor/regular/eye.svg"),
            "eye-closed" => include_str!("../../../references/phosphor/regular/eye-closed.svg"),
            "file" => include_str!("../../../references/phosphor/regular/file.svg"),
            "file-text" => include_str!("../../../references/phosphor/regular/file-text.svg"),
            "funnel" => include_str!("../../../references/phosphor/regular/funnel.svg"),
            "gear-six" => include_str!("../../../references/phosphor/regular/gear-six.svg"),
            "house" => include_str!("../../../references/phosphor/regular/house.svg"),
            "hard-drive" => include_str!("../../../references/phosphor/regular/hard-drive.svg"),
            "heart" => include_str!("../../../references/phosphor/regular/heart.svg"),
            "magnifying-glass" => {
                include_str!("../../../references/phosphor/regular/magnifying-glass.svg")
            }
            "minus" => include_str!("../../../references/phosphor/regular/minus.svg"),
            "moon" => include_str!("../../../references/phosphor/regular/moon.svg"),
            "note" => include_str!("../../../references/phosphor/regular/note.svg"),
            "palette" => include_str!("../../../references/phosphor/regular/palette.svg"),
            "pencil" => include_str!("../../../references/phosphor/regular/pencil.svg"),
            "piggy-bank" => include_str!("../../../references/phosphor/regular/piggy-bank.svg"),
            "plus" => include_str!("../../../references/phosphor/regular/plus.svg"),
            "question" => include_str!("../../../references/phosphor/regular/question.svg"),
            "receipt" => include_str!("../../../references/phosphor/regular/receipt.svg"),
            "repeat" => include_str!("../../../references/phosphor/regular/repeat.svg"),
            "sidebar-simple" => {
                include_str!("../../../references/phosphor/regular/sidebar-simple.svg")
            }
            "spinner-gap" => include_str!("../../../references/phosphor/regular/spinner-gap.svg"),
            "square" => include_str!("../../../references/phosphor/regular/square.svg"),
            "squares-four" => include_str!("../../../references/phosphor/regular/squares-four.svg"),
            "star" => include_str!("../../../references/phosphor/regular/star.svg"),
            "sun" => include_str!("../../../references/phosphor/regular/sun.svg"),
            "tag" => include_str!("../../../references/phosphor/regular/tag.svg"),
            "trash" => include_str!("../../../references/phosphor/regular/trash.svg"),
            "tray" => include_str!("../../../references/phosphor/regular/tray.svg"),
            "user" => include_str!("../../../references/phosphor/regular/user.svg"),
            "users" => include_str!("../../../references/phosphor/regular/users.svg"),
            "wallet" => include_str!("../../../references/phosphor/regular/wallet.svg"),
            "warning-circle" => {
                include_str!("../../../references/phosphor/regular/warning-circle.svg")
            }
            "x" => include_str!("../../../references/phosphor/regular/x.svg"),
            _ => return None,
        }
        .as_bytes(),
    )
}

/// The duotone asset path for a lucide icon path (`icons/circle-check.svg` →
/// `icons/duotone/check-circle.svg`), when a Phosphor duotone master is embedded.
pub fn duotone_path(lucide_path: &str) -> Option<SharedString> {
    let name = lucide_to_phosphor(lucide_path)?;
    let path = format!("{DUOTONE_PREFIX}{name}.svg");
    phosphor_duotone_bytes(name).map(|_| SharedString::from(path))
}

/// Phosphor **duotone** SVG bytes for `icons/duotone/<name>.svg` — the interaction
/// weight (TOKENS §08). Only the glyphs the design uses are embedded.
fn phosphor_duotone_bytes(name: &str) -> Option<&'static [u8]> {
    let svg = match name.strip_suffix(".svg").unwrap_or(name) {
        "arrow-up-right" => "arrow-up-right",
        "bell" => "bell",
        "caret-down" => "caret-down",
        "chart-line-up" => "chart-line-up",
        "check-circle" => "check-circle",
        "eye-closed" => "eye-closed",
        "export" => "export",
        "funnel" => "funnel",
        "gear-six" => "gear-six",
        "magnifying-glass" => "magnifying-glass",
        "plus" => "plus",
        "spinner-gap" => "spinner-gap",
        "wallet" => "wallet",
        "warning-circle" => "warning-circle",
        "x" => "x",
        _ => return None,
    };
    Some(
        match svg {
            "arrow-up-right" => {
                include_str!("../../../references/phosphor/duotone/arrow-up-right.svg")
            }
            "bell" => include_str!("../../../references/phosphor/duotone/bell.svg"),
            "caret-down" => include_str!("../../../references/phosphor/duotone/caret-down.svg"),
            "chart-line-up" => {
                include_str!("../../../references/phosphor/duotone/chart-line-up.svg")
            }
            "check-circle" => {
                include_str!("../../../references/phosphor/duotone/check-circle.svg")
            }
            "eye-closed" => include_str!("../../../references/phosphor/duotone/eye-closed.svg"),
            "export" => include_str!("../../../references/phosphor/duotone/export.svg"),
            "funnel" => include_str!("../../../references/phosphor/duotone/funnel.svg"),
            "gear-six" => include_str!("../../../references/phosphor/duotone/gear-six.svg"),
            "magnifying-glass" => {
                include_str!("../../../references/phosphor/duotone/magnifying-glass.svg")
            }
            "plus" => include_str!("../../../references/phosphor/duotone/plus.svg"),
            "spinner-gap" => include_str!("../../../references/phosphor/duotone/spinner-gap.svg"),
            "wallet" => include_str!("../../../references/phosphor/duotone/wallet.svg"),
            "warning-circle" => {
                include_str!("../../../references/phosphor/duotone/warning-circle.svg")
            }
            "x" => include_str!("../../../references/phosphor/duotone/x.svg"),
            _ => unreachable!("duotone map entry without embedded svg"),
        }
        .as_bytes(),
    )
}

/// The lucide paths this source overrides with Phosphor regular glyphs.
const MAPPED: &[&str] = &[
    "icons/alert-circle.svg",
    "icons/arrow-up-right.svg",
    "icons/circle-alert.svg",
    "icons/bell.svg",
    "icons/calendar.svg",
    "icons/chart-pie.svg",
    "icons/trending-up.svg",
    "icons/check.svg",
    "icons/chevron-down.svg",
    "icons/chevron-right.svg",
    "icons/chevron-up.svg",
    "icons/chevrons-up-down.svg",
    "icons/circle-check.svg",
    "icons/circle-dollar-sign.svg",
    "icons/clock.svg",
    "icons/cloud-off.svg",
    "icons/copy.svg",
    "icons/delete.svg",
    "icons/download.svg",
    "icons/ellipsis.svg",
    "icons/external-link.svg",
    "icons/eye.svg",
    "icons/eye-off.svg",
    "icons/file.svg",
    "icons/file-text.svg",
    "icons/filter.svg",
    "icons/hard-drive.svg",
    "icons/heart.svg",
    "icons/help-circle.svg",
    "icons/history.svg",
    "icons/home.svg",
    "icons/house.svg",
    "icons/inbox.svg",
    "icons/landmark.svg",
    "icons/layout-dashboard.svg",
    "icons/loader-circle.svg",
    "icons/maximize.svg",
    "icons/minus.svg",
    "icons/moon.svg",
    "icons/palette.svg",
    "icons/panel-left.svg",
    "icons/pencil.svg",
    "icons/piggy-bank.svg",
    "icons/plus.svg",
    "icons/receipt.svg",
    "icons/redo-2.svg",
    "icons/refresh-cw.svg",
    "icons/repeat.svg",
    "icons/search.svg",
    "icons/settings.svg",
    "icons/square.svg",
    "icons/star.svg",
    "icons/sticky-note.svg",
    "icons/sun.svg",
    "icons/tag.svg",
    "icons/undo-2.svg",
    "icons/upload.svg",
    "icons/user.svg",
    "icons/users.svg",
    "icons/wallet.svg",
    "icons/x.svg",
];

/// The duotone paths this source serves from `references/phosphor/duotone/`.
const DUOTONE_MAPPED: &[&str] = &[
    "icons/duotone/arrow-up-right.svg",
    "icons/duotone/bell.svg",
    "icons/duotone/caret-down.svg",
    "icons/duotone/chart-line-up.svg",
    "icons/duotone/check-circle.svg",
    "icons/duotone/eye-closed.svg",
    "icons/duotone/export.svg",
    "icons/duotone/funnel.svg",
    "icons/duotone/gear-six.svg",
    "icons/duotone/magnifying-glass.svg",
    "icons/duotone/plus.svg",
    "icons/duotone/spinner-gap.svg",
    "icons/duotone/wallet.svg",
    "icons/duotone/warning-circle.svg",
    "icons/duotone/x.svg",
];

/// The app asset source: Phosphor duotone at the mapped lucide paths, gpui-kit's lucide
/// set for everything else.
#[derive(Clone, Copy, Debug, Default)]
pub struct BookkeepyAssets;

impl AssetSource for BookkeepyAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = phosphor_bytes(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut all = gpui_kit::assets::Assets.list(path)?;
        all.extend(
            MAPPED
                .iter()
                .chain(DUOTONE_MAPPED.iter())
                .filter(|p| p.starts_with(path))
                .map(|p| SharedString::from(*p)),
        );
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_paths_serve_phosphor_regular() {
        for path in MAPPED {
            let bytes = BookkeepyAssets.load(path).ok().flatten();
            let Some(bytes) = bytes else {
                panic!("{path} must serve embedded phosphor bytes");
            };
            let text = std::str::from_utf8(&bytes).unwrap_or("");
            assert!(
                text.contains("currentColor"),
                "{path} must be a themable regular svg"
            );
        }
    }

    #[test]
    fn duotone_path_translates_lucide_to_phosphor_names() {
        // lucide names differ from phosphor names — the duotone path must translate.
        assert_eq!(
            duotone_path("icons/circle-check.svg").as_deref(),
            Some("icons/duotone/check-circle.svg")
        );
        assert_eq!(
            duotone_path("icons/circle-alert.svg").as_deref(),
            Some("icons/duotone/warning-circle.svg")
        );
        assert_eq!(
            duotone_path("icons/wallet.svg").as_deref(),
            Some("icons/duotone/wallet.svg")
        );
        assert!(duotone_path("icons/arrow-down.svg").is_none());
    }

    #[test]
    fn duotone_paths_serve_phosphor_duotone() {
        for path in DUOTONE_MAPPED {
            let bytes = BookkeepyAssets.load(path).ok().flatten();
            let Some(bytes) = bytes else {
                panic!("{path} must serve embedded phosphor duotone bytes");
            };
            let text = std::str::from_utf8(&bytes).unwrap_or("");
            assert!(
                text.contains("currentColor") && text.contains("opacity"),
                "{path} must be a themable duotone svg (fill + 20% layer)"
            );
        }
    }

    #[test]
    fn unmapped_paths_fall_through_to_lucide() {
        // A default lucide path with no phosphor mapping must still resolve via gpui-kit
        // (icons/arrow-down.svg is in gpui-kit-assets' default-icons.txt).
        let bytes = BookkeepyAssets.load("icons/arrow-down.svg").ok().flatten();
        assert!(
            bytes.is_some(),
            "lucide fallback must serve icons/arrow-down.svg"
        );
    }
}
