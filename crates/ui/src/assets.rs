// SPDX-License-Identifier: GPL-3.0-or-later
//! `BookkeepyAssets` — the app's asset source (RULE-UI-005, DESIGN-ASSETS): serves the
//! design language's **Phosphor duotone** glyphs (references/phosphor, MIT, catalog.yaml)
//! at gpui-kit's lucide icon paths, so every `IconName::…` call site renders the designed
//! duotone glyph with no API churn. Paths with no Phosphor mapping fall through to the
//! default gpui-kit lucide set, so nothing renders empty.

use std::borrow::Cow;

use gpui_kit::{AssetSource, Result, SharedString};

/// Phosphor duotone SVG bytes for a gpui-kit lucide icon path (`icons/<name>.svg`).
/// The map is the lucide→phosphor name translation for the icons the design uses.
fn phosphor_bytes(path: &str) -> Option<&'static [u8]> {
    let svg = match path {
        "icons/bell.svg" => "bell",
        "icons/calendar.svg" => "calendar-blank",
        "icons/chart-pie.svg" => "chart-pie",
        "icons/check.svg" => "check",
        "icons/chevron-down.svg" => "caret-down",
        "icons/chevron-up.svg" => "caret-up",
        "icons/circle-check.svg" => "check-circle",
        "icons/clock.svg" => "clock",
        "icons/copy.svg" => "copy",
        "icons/delete.svg" => "trash",
        "icons/download.svg" => "download",
        "icons/ellipsis.svg" => "dots-three",
        "icons/external-link.svg" => "arrow-square-out",
        "icons/file.svg" => "file",
        "icons/file-text.svg" => "file-text",
        "icons/filter.svg" => "funnel",
        "icons/hard-drive.svg" => "hard-drive",
        "icons/heart.svg" => "heart",
        "icons/inbox.svg" => "tray",
        "icons/layout-dashboard.svg" => "squares-four",
        "icons/maximize.svg" => "arrows-out",
        "icons/moon.svg" => "moon",
        "icons/palette.svg" => "palette",
        "icons/pencil.svg" => "pencil",
        "icons/plus.svg" => "plus",
        "icons/redo-2.svg" => "arrow-clockwise",
        "icons/search.svg" => "magnifying-glass",
        "icons/settings.svg" => "gear",
        "icons/star.svg" => "star",
        "icons/sun.svg" => "sun",
        "icons/tag.svg" => "tag",
        "icons/undo-2.svg" => "arrow-counter-clockwise",
        "icons/user.svg" => "user",
        "icons/x.svg" => "x",
        _ => return None,
    };
    Some(
        match svg {
            "arrow-clockwise" => {
                include_str!("../../../references/phosphor/duotone/arrow-clockwise.svg")
            }
            "arrow-counter-clockwise" => {
                include_str!("../../../references/phosphor/duotone/arrow-counter-clockwise.svg")
            }
            "arrow-square-out" => {
                include_str!("../../../references/phosphor/duotone/arrow-square-out.svg")
            }
            "arrows-out" => include_str!("../../../references/phosphor/duotone/arrows-out.svg"),
            "bell" => include_str!("../../../references/phosphor/duotone/bell.svg"),
            "calendar-blank" => {
                include_str!("../../../references/phosphor/duotone/calendar-blank.svg")
            }
            "caret-down" => include_str!("../../../references/phosphor/duotone/caret-down.svg"),
            "caret-up" => include_str!("../../../references/phosphor/duotone/caret-up.svg"),
            "chart-pie" => include_str!("../../../references/phosphor/duotone/chart-pie.svg"),
            "check" => include_str!("../../../references/phosphor/duotone/check.svg"),
            "check-circle" => include_str!("../../../references/phosphor/duotone/check-circle.svg"),
            "clock" => include_str!("../../../references/phosphor/duotone/clock.svg"),
            "copy" => include_str!("../../../references/phosphor/duotone/copy.svg"),
            "download" => include_str!("../../../references/phosphor/duotone/download.svg"),
            "dots-three" => include_str!("../../../references/phosphor/duotone/dots-three.svg"),
            "file" => include_str!("../../../references/phosphor/duotone/file.svg"),
            "file-text" => include_str!("../../../references/phosphor/duotone/file-text.svg"),
            "funnel" => include_str!("../../../references/phosphor/duotone/funnel.svg"),
            "gear" => include_str!("../../../references/phosphor/duotone/gear.svg"),
            "hard-drive" => include_str!("../../../references/phosphor/duotone/hard-drive.svg"),
            "heart" => include_str!("../../../references/phosphor/duotone/heart.svg"),
            "magnifying-glass" => {
                include_str!("../../../references/phosphor/duotone/magnifying-glass.svg")
            }
            "moon" => include_str!("../../../references/phosphor/duotone/moon.svg"),
            "palette" => include_str!("../../../references/phosphor/duotone/palette.svg"),
            "pencil" => include_str!("../../../references/phosphor/duotone/pencil.svg"),
            "plus" => include_str!("../../../references/phosphor/duotone/plus.svg"),
            "squares-four" => include_str!("../../../references/phosphor/duotone/squares-four.svg"),
            "star" => include_str!("../../../references/phosphor/duotone/star.svg"),
            "sun" => include_str!("../../../references/phosphor/duotone/sun.svg"),
            "tag" => include_str!("../../../references/phosphor/duotone/tag.svg"),
            "trash" => include_str!("../../../references/phosphor/duotone/trash.svg"),
            "tray" => include_str!("../../../references/phosphor/duotone/tray.svg"),
            "user" => include_str!("../../../references/phosphor/duotone/user.svg"),
            "x" => include_str!("../../../references/phosphor/duotone/x.svg"),
            _ => unreachable!("phosphor map entry without embedded svg"),
        }
        .as_bytes(),
    )
}

/// The lucide paths this source overrides with Phosphor duotone glyphs.
const MAPPED: &[&str] = &[
    "icons/bell.svg",
    "icons/calendar.svg",
    "icons/chart-pie.svg",
    "icons/check.svg",
    "icons/chevron-down.svg",
    "icons/chevron-up.svg",
    "icons/circle-check.svg",
    "icons/clock.svg",
    "icons/copy.svg",
    "icons/delete.svg",
    "icons/download.svg",
    "icons/ellipsis.svg",
    "icons/external-link.svg",
    "icons/file.svg",
    "icons/file-text.svg",
    "icons/filter.svg",
    "icons/hard-drive.svg",
    "icons/heart.svg",
    "icons/inbox.svg",
    "icons/layout-dashboard.svg",
    "icons/maximize.svg",
    "icons/moon.svg",
    "icons/palette.svg",
    "icons/pencil.svg",
    "icons/plus.svg",
    "icons/redo-2.svg",
    "icons/search.svg",
    "icons/settings.svg",
    "icons/star.svg",
    "icons/sun.svg",
    "icons/tag.svg",
    "icons/undo-2.svg",
    "icons/user.svg",
    "icons/x.svg",
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
    fn mapped_paths_serve_phosphor_duotone() {
        for path in MAPPED {
            let bytes = BookkeepyAssets.load(path).ok().flatten();
            let Some(bytes) = bytes else {
                panic!("{path} must serve embedded phosphor bytes");
            };
            let text = std::str::from_utf8(&bytes).unwrap_or("");
            assert!(
                text.contains("currentColor"),
                "{path} must be a themable duotone svg"
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
