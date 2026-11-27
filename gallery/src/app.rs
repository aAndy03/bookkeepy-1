// SPDX-License-Identifier: GPL-3.0-or-later
//! Gallery shell (the design-language TEMPLATES sheet: title bar + sidebar + content).
//! The sidebar navigates by atomic layer; the content hosts one retained section view at
//! a time. Locale + theme switchers live on the title bar; switching refreshes windows.

use std::collections::HashMap;

use bookkeepy_i18n::Locale;
use bookkeepy_primary_data::theme::Mode;
use gpui_kit::component::button::Button;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::label::Label;
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::sidebar::{Sidebar, SidebarGroup, SidebarMenuItem};
use gpui_kit::component::{ActiveTheme as _, IconName, Sizable as _};
use gpui_kit::component::{Side, TitleBar};
use gpui_kit::{
    AnyView, App, InteractiveElement as _, IntoElement as _, ParentElement as _, SharedString,
    Styled as _, Window, div,
};

use crate::Quit;
use crate::sections::{SECTIONS, sections_by_layer};

/// Global runtime prefs. Switching writes the global, applies the theme (mode), and
/// refreshes every window — no per-view locale plumbing.
pub struct Prefs {
    /// Active UI locale (EN/RO).
    pub locale: Locale,
    /// Active theme mode (dark is the design language's native theme).
    pub mode: Mode,
}

impl gpui_kit::Global for Prefs {}

impl Default for Prefs {
    fn default() -> Self {
        // The gallery starts on the design language's native theme + reference locale;
        // the app itself follows data/settings-defaults.toml.
        Self {
            locale: Locale::En,
            mode: Mode::Dark,
        }
    }
}

impl Prefs {
    /// The current locale.
    pub fn locale(cx: &App) -> Locale {
        cx.global::<Prefs>().locale
    }
}

/// Translate `key` with the current locale (the helper every gallery demo uses).
pub fn tr(cx: &App, key: &str) -> SharedString {
    bookkeepy_i18n::t(Prefs::locale(cx), key).into()
}

/// The gallery window: shell chrome + atomic-layer sidebar + one retained section host.
pub struct Gallery {
    active: &'static str,
    hosts: HashMap<&'static str, AnyView>,
}

impl Gallery {
    /// Create the gallery view; the default section is the first in the registry,
    /// or `section` when deep-linked via `--section <id>`.
    pub fn new(
        _window: &mut Window,
        _cx: &mut gpui_kit::Context<'_, Self>,
        section: Option<&str>,
    ) -> Self {
        let active = section
            .and_then(|id| SECTIONS.iter().find(|s| s.id == id).map(|s| s.id))
            .unwrap_or_else(|| SECTIONS.first().map(|s| s.id).unwrap_or_default());
        Self {
            active,
            hosts: HashMap::new(),
        }
    }

    /// The retained host of `id` (lazily created; state survives navigation).
    fn host(
        &mut self,
        id: &'static str,
        window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> AnyView {
        if let Some(host) = self.hosts.get(id) {
            return host.clone();
        }
        // `id` always comes from SECTIONS (the registry govern checks), so this cannot
        // miss for gallery-internal ids.
        let spec = SECTIONS.iter().find(|s| s.id == id);
        let Some(spec) = spec else {
            unreachable!("gallery: section id must come from SECTIONS");
        };
        let host = (spec.new)(window, cx);
        self.hosts.insert(spec.id, host.clone());
        host
    }

    fn select_section(
        &mut self,
        id: &'static str,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) {
        if self.active != id {
            self.active = id;
            cx.notify();
        }
    }

    fn toggle_locale(
        &mut self,
        _: &gpui_kit::ClickEvent,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) {
        let prefs = cx.global_mut::<Prefs>();
        prefs.locale = match prefs.locale {
            Locale::En => Locale::Ro,
            Locale::Ro => Locale::En,
        };
        cx.refresh_windows();
    }

    fn toggle_theme(
        &mut self,
        _: &gpui_kit::ClickEvent,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) {
        let prefs = cx.global_mut::<Prefs>();
        prefs.mode = match prefs.mode {
            Mode::Dark => Mode::Light,
            Mode::Light => Mode::Dark,
        };
        let mode = prefs.mode;
        bookkeepy_ui::theme::apply(mode, cx);
        cx.refresh_windows();
    }

    fn quit_action(
        &mut self,
        _: &Quit,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) {
        cx.quit();
    }

    fn render_sidebar(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement + use<> {
        let active = self.active;
        // Shell preset from the ui crate (TEMPLATES · shell): sidebar at the design's
        // 284px expanded width — geometry from data/theme/layout.toml, never raw px.
        let mut sidebar = bookkeepy_ui::api::language::shell::sidebar(
            Sidebar::new("gallery-sidebar").side(Side::Left),
        );
        for (layer, specs) in sections_by_layer() {
            let mut group = SidebarGroup::new(tr(cx, layer.title_key()));
            for spec in specs {
                let item = SidebarMenuItem::new(tr(cx, spec.title_key))
                    .h(gpui_kit::px(24.))
                    .active(spec.id == active)
                    .on_click(cx.listener({
                        let id = spec.id;
                        move |this: &mut Self, _ev: &gpui_kit::ClickEvent, window, cx| {
                            this.select_section(id, window, cx);
                        }
                    }));
                group = group.child(item);
            }
            sidebar = sidebar.child(group);
        }
        sidebar
    }
}

impl gpui_kit::Render for Gallery {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl gpui_kit::IntoElement {
        let locale_label: SharedString = match Prefs::locale(cx) {
            Locale::En => "EN".into(),
            Locale::Ro => "RO".into(),
        };
        let mode_icon = match cx.global::<Prefs>().mode {
            Mode::Dark => IconName::Sun,
            Mode::Light => IconName::Moon,
        };
        let title = tr(cx, "gallery.app.title");
        let (bg, fg) = (cx.theme().background, cx.theme().foreground);
        // While the section registry is empty (M3 rebuild in progress), the content
        // area shows a placeholder instead of a section host.
        let content: gpui_kit::AnyElement = if SECTIONS.is_empty() {
            div()
                .p_6()
                .text_color(cx.theme().muted_foreground)
                .child(tr(cx, "gallery.app.rebuilding"))
                .into_any_element()
        } else {
            self.host(self.active, window, cx).into_any_element()
        };

        div()
            .key_context("Gallery")
            .on_action(cx.listener(Self::quit_action))
            .flex()
            .flex_col()
            .size_full()
            .child(
                TitleBar::new()
                    .bg(bookkeepy_ui::api::language::role_hsla(
                        cx.global::<Prefs>().mode,
                        "background",
                    )
                    .unwrap_or_default())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Label::new(title))
                            .child(
                                Button::new("locale-switch")
                                    .label(locale_label)
                                    .ghost()
                                    .small()
                                    .on_click(cx.listener(Self::toggle_locale)),
                            )
                            .child(
                                Button::new("theme-switch")
                                    .icon(mode_icon)
                                    .ghost()
                                    .small()
                                    .on_click(cx.listener(Self::toggle_theme)),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_stretch()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_sidebar(window, cx))
                    .child(
                        div()
                            .id("gallery-content")
                            .flex_1()
                            .min_w_0()
                            .min_h_0()
                            .bg(bg)
                            .text_color(fg)
                            .overflow_y_scrollbar()
                            .child(content),
                    ),
            )
    }
}
