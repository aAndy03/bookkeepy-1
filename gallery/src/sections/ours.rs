// SPDX-License-Identifier: GPL-3.0-or-later
//! Ours — Bookkeepy's own versioned widgets (RULE-UI-006/008, ADR-0017): the `amount`
//! seed atom with its capability-trait variant (`signed`) and the zero edge case, plus
//! the version history panel rendered from the widget's `meta.toml` — the same file
//! `cargo xtask govern ui` cross-checks (one source of truth for the widget registry).

use std::sync::OnceLock;

use bookkeepy_core::CurrencyCode;
use bookkeepy_core::Money;
use bookkeepy_ui::api::widgets::atoms::amount::{Amount, META_TOML};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::{
    AnyView, App, AppContext as _, Context, IntoElement, ParentElement as _, SharedString,
    Styled as _, Window, div,
};

use crate::app::Gallery;
use crate::app::Prefs;
use crate::app::tr;
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The demo currency (an ISO-shape literal, so construction cannot fail at runtime).
fn currency() -> &'static CurrencyCode {
    static CODE: OnceLock<CurrencyCode> = OnceLock::new();
    CODE.get_or_init(|| match CurrencyCode::new("RON") {
        Ok(code) => code,
        Err(_) => unreachable!("the literal \"RON\" is a valid ISO shape (CurrencyCode::new)"),
    })
}

/// Parsed `meta.toml` (RULE-UI-008) — only what the history panel renders.
#[derive(serde::Deserialize)]
struct Meta {
    widget: MetaWidget,
    #[serde(default)]
    variant: Vec<MetaLink>,
    #[serde(rename = "edge_case", default)]
    edge_case: Vec<MetaLink>,
    changelog: Vec<MetaChange>,
}

#[derive(serde::Deserialize)]
struct MetaWidget {
    id: String,
    version: u32,
    #[serde(default)]
    traits: Vec<String>,
}

#[derive(serde::Deserialize)]
struct MetaLink {
    id: String,
    file: String,
    #[serde(rename = "trait")]
    trait_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct MetaChange {
    version: u32,
    summary_en: String,
    summary_ro: Option<String>,
}

/// Parse the embedded `meta.toml` (govern validates its schema; the panel degrades
/// gracefully — no panics in the render path).
fn meta() -> Option<Meta> {
    toml::from_str(META_TOML).ok()
}

impl MetaChange {
    /// The changelog summary in the active locale (EN fallback).
    fn summary(&self, cx: &App) -> SharedString {
        match (Prefs::locale(cx), &self.summary_ro) {
            (bookkeepy_i18n::Locale::Ro, Some(ro)) => ro.clone(),
            _ => self.summary_en.clone(),
        }
        .into()
    }
}

/// The `ours` section view.
pub struct Section;

/// The section spec (id `ours`).
pub fn view(_window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|_| Section).into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let money = |minor: i64| Money::new(minor, currency().clone());
        // The seed atom, its variant, and its edge case — the versioned-variant
        // pattern every future Bookkeepy widget follows (ADR-0017).
        let atom = demo_row([
            Amount::new(money(1234)).into_any_element(),
            Amount::new(money(-500)).into_any_element(),
        ])
        .into_any_element();
        let signed = demo_row([
            Amount::new(money(5900)).signed().into_any_element(),
            Amount::new(money(-5900)).signed().into_any_element(),
            Amount::zero(currency().clone()).signed().into_any_element(),
        ])
        .into_any_element();
        let cards = vec![
            demo_card(cx, "gallery.ours.amount.caption", atom).into_any_element(),
            demo_card(cx, "gallery.ours.amount.signed", signed).into_any_element(),
            demo_card(cx, "gallery.ours.history.caption", history(cx)).into_any_element(),
        ];
        section_page(cx, "gallery.section.ours", cards)
    }
}

/// The version-history panel: widget id + current version + changelog + the variant/
/// edge-case links, all straight from `meta.toml`.
fn history(cx: &App) -> gpui_kit::AnyElement {
    let theme = cx.theme();
    let Some(meta) = meta() else {
        return div()
            .text_sm()
            .child(tr(cx, "gallery.ours.history.unparsed"))
            .into_any_element();
    };
    let row = |label: SharedString, body: SharedString| {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(label),
            )
            .child(div().text_sm().child(body))
    };
    let mut list = div().flex().flex_col().gap_4();
    list = list.child(row(
        tr(cx, "gallery.ours.history.widget"),
        format!(
            "{} · v{} · {}",
            meta.widget.id,
            meta.widget.version,
            meta.widget.traits.join(", ")
        )
        .into(),
    ));
    let links = |meta_links: &[MetaLink]| {
        meta_links
            .iter()
            .map(|l| {
                format!(
                    "{} → {} ({})",
                    l.id,
                    l.file,
                    l.trait_id.as_deref().unwrap_or("-")
                )
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    if !meta.variant.is_empty() {
        list = list.child(row(
            tr(cx, "gallery.ours.history.variants"),
            links(&meta.variant).into(),
        ));
    }
    if !meta.edge_case.is_empty() {
        list = list.child(row(
            tr(cx, "gallery.ours.history.edge_cases"),
            links(&meta.edge_case).into(),
        ));
    }
    for change in &meta.changelog {
        list = list.child(row(
            format!("v{}", change.version).into(),
            change.summary(cx),
        ));
    }
    list.into_any_element()
}
