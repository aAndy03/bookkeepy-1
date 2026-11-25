// SPDX-License-Identifier: GPL-3.0-or-later
//! Molecules — the data-list family: List (selectable), searchable List,
//! DescriptionList, Pagination, Breadcrumb, Clipboard (design sheet:
//! "MOLECULES · data list"; Attachment lives in the MOL·03 section
//! `molecules-calendar`). List content is i18n'd; list state is retained in this view.

use gpui_kit::component::breadcrumb::Breadcrumb;
use gpui_kit::component::clipboard::Clipboard;
use gpui_kit::component::description_list::{DescriptionItem, DescriptionList};
use gpui_kit::component::label::Label;
use gpui_kit::component::list::{List, ListDelegate, ListItem, ListState};
use gpui_kit::component::pagination::Pagination;
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme as _, Icon, IconName, IndexPath, Sizable as _, h_flex};
use gpui_kit::{
    AnyView, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, SharedString,
    Styled as _, Task, Window, div, px,
};

use crate::app::{Gallery, Prefs, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// Delegate for the plain [`List`] demo: three ledger accounts, click to select.
struct AccountsDelegate {
    items: Vec<SharedString>,
    selected_index: Option<IndexPath>,
}

impl ListDelegate for AccountsDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.items.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        self.items.get(ix.row).map(|item| {
            ListItem::new(ix)
                .child(item.clone())
                .selected(self.selected_index == Some(ix))
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
    }
}

/// Delegate for the searchable [`List`]: filters the billed services by the query.
struct ServicesDelegate {
    items: Vec<SharedString>,
    matched: Vec<SharedString>,
    selected_index: Option<IndexPath>,
}

impl ListDelegate for ServicesDelegate {
    type Item = ListItem;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.matched.len()
    }

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        let needle = query.to_lowercase();
        self.matched = self
            .items
            .iter()
            .filter(|item| item.to_lowercase().contains(&needle))
            .cloned()
            .collect();
        cx.notify();
        Task::ready(())
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        self.matched.get(ix.row).map(|item| {
            // MOL·02 list row: leading icon chip + label + trailing tag.
            ListItem::new(ix)
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .size(px(24.))
                                .rounded(px(4.))
                                .bg(cx.theme().accent)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(Icon::new(IconName::FileText).small()),
                        )
                        .child(item.clone()),
                )
                .suffix(|_window, cx| {
                    bookkeepy_ui::api::language::display::tag_soft(
                        Tag::secondary(),
                        bookkeepy_ui::api::language::display::TagTone::Neutral,
                        cx.global::<Prefs>().mode,
                    )
                    .child(tr(cx, "gallery.molecules.data_list.list.row_tag"))
                    .into_any_element()
                })
                .selected(self.selected_index == Some(ix))
        })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
    }
}

/// The data-list molecules section.
pub struct Section {
    accounts: Entity<ListState<AccountsDelegate>>,
    services: Entity<ListState<ServicesDelegate>>,
    page: usize,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let accounts = cx.new(|cx| {
            ListState::new(
                AccountsDelegate {
                    items: vec![
                        tr(cx, "gallery.molecules.data_list.list.item_cash"),
                        tr(cx, "gallery.molecules.data_list.list.item_bank"),
                        tr(cx, "gallery.molecules.data_list.list.item_vat"),
                    ],
                    selected_index: None,
                },
                window,
                cx,
            )
        });
        let services_items = vec![
            tr(cx, "gallery.molecules.data_list.searchable.item_design"),
            tr(cx, "gallery.molecules.data_list.searchable.item_hosting"),
            tr(cx, "gallery.molecules.data_list.searchable.item_audit"),
        ];
        let services = cx.new(|cx| {
            ListState::new(
                ServicesDelegate {
                    matched: services_items.clone(),
                    items: services_items,
                    selected_index: None,
                },
                window,
                cx,
            )
            .searchable(true)
        });
        Section {
            accounts,
            services,
            page: 1,
        }
    })
    .into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        section_page(
            cx,
            "gallery.section.molecules-data-list",
            [
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.data_list.list",
                        List::new(&self.accounts).into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.data_list.searchable",
                        List::new(&self.services)
                            .search_placeholder(tr(
                                cx,
                                "gallery.molecules.data_list.searchable.placeholder",
                            ))
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.data_list.description",
                        DescriptionList::new()
                            .columns(2)
                            .children([
                                DescriptionItem::new(tr(
                                    cx,
                                    "gallery.molecules.data_list.description.company",
                                ))
                                .value(tr(
                                    cx,
                                    "gallery.molecules.data_list.description.company_value",
                                )),
                                DescriptionItem::new(tr(
                                    cx,
                                    "gallery.molecules.data_list.description.vat",
                                ))
                                .value(tr(cx, "gallery.molecules.data_list.description.vat_value")),
                                DescriptionItem::new(tr(
                                    cx,
                                    "gallery.molecules.data_list.description.currency",
                                ))
                                .value(tr(
                                    cx,
                                    "gallery.molecules.data_list.description.currency_value",
                                )),
                            ])
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.data_list.pagination",
                        Pagination::new("data-list-pagination")
                            .current_page(self.page)
                            .total_pages(8)
                            .on_click(cx.listener(|this: &mut Self, page: &usize, _window, cx| {
                                this.page = *page;
                                cx.notify();
                            }))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.data_list.breadcrumb",
                        Breadcrumb::new()
                            .child(tr(cx, "gallery.molecules.data_list.breadcrumb.item_home"))
                            .child(tr(
                                cx,
                                "gallery.molecules.data_list.breadcrumb.item_invoices",
                            ))
                            .child(tr(
                                cx,
                                "gallery.molecules.data_list.breadcrumb.item_invoice",
                            ))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.data_list.clipboard",
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Label::new(tr(
                                cx,
                                "gallery.molecules.data_list.clipboard.value",
                            )))
                            .child(
                                Clipboard::new("data-list-clipboard")
                                    .value(tr(cx, "gallery.molecules.data_list.clipboard.value")),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
            ],
        )
    }
}
