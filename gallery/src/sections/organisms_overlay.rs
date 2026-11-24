// SPDX-License-Identifier: GPL-3.0-or-later
//! Organisms — overlay surfaces: the modal Dialog, the AskDialog (AlertDialog),
//! the Sheet (drawer), toast Notifications, and the Command palette
//! (design sheet: "ORGANISMS · overlay layers"). Every surface opens from a
//! button trigger; the palette's `CommandState` is retained in this view.

use gpui_kit::component::button::{Button, ButtonVariant, ButtonVariants as _};
use gpui_kit::component::command::{Command, CommandGroup, CommandItem, CommandState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{
    IconName, IndexPath, Placement, Sizable as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Styled as _,
    Window, px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The overlay-organism section.
pub struct Section {
    /// The palette's retained interaction state (query, focus, selection).
    command: Entity<CommandState>,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let command = cx.new(|cx| CommandState::new(window, cx));
        Section { command }
    })
    .into()
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        // The palette's state outlives each opening, so its captured handle keeps
        // the query/focus across the dialog layer's rebuilds.
        let palette = self.command.clone();

        // Modal surfaces: Dialog, AskDialog, Sheet.
        let modal_row = demo_row([
            demo_card(
                cx,
                "gallery.organisms.overlay.dialog.caption",
                Button::new("overlay-dialog-trigger")
                    .primary()
                    .label(tr(cx, "gallery.organisms.overlay.dialog.open"))
                    .on_click(|_, window, cx| {
                        window.open_dialog(cx, |dialog, _win, cx| {
                            let done = tr(cx, "gallery.organisms.overlay.dialog.done");
                            dialog
                                .title(tr(cx, "gallery.organisms.overlay.dialog.title"))
                                .child(
                                    v_flex()
                                        .gap_2()
                                        .child(tr(cx, "gallery.organisms.overlay.dialog.body")),
                                )
                                .footer(
                                    h_flex()
                                        .justify_end()
                                        .gap_2()
                                        .child(
                                            Button::new("overlay-dialog-cancel")
                                                .outline()
                                                .label(tr(cx, "common.cancel"))
                                                .on_click(|_, window, cx| {
                                                    window.close_dialog(cx);
                                                }),
                                        )
                                        .child(
                                            Button::new("overlay-dialog-ok")
                                                .primary()
                                                .label(tr(
                                                    cx,
                                                    "gallery.organisms.overlay.dialog.confirm",
                                                ))
                                                .on_click(move |_, window, cx| {
                                                    window.push_notification(
                                                        Notification::success(done.clone()),
                                                        cx,
                                                    );
                                                    window.close_dialog(cx);
                                                }),
                                        ),
                                )
                        });
                    })
                    .into_any_element(),
            )
            .into_any_element(),
            demo_card(
                cx,
                "gallery.organisms.overlay.ask.caption",
                Button::new("overlay-ask-trigger")
                    .outline()
                    .label(tr(cx, "gallery.organisms.overlay.ask.open"))
                    .on_click(|_, window, cx| {
                        window.open_alert_dialog(cx, |alert, _win, cx| {
                            let confirmed = tr(cx, "gallery.organisms.overlay.ask.confirmed");
                            alert
                                .title(tr(cx, "gallery.organisms.overlay.ask.title"))
                                .description(tr(cx, "gallery.organisms.overlay.ask.description"))
                                .confirm()
                                .ok_text(tr(cx, "gallery.organisms.overlay.ask.delete"))
                                .ok_variant(ButtonVariant::Danger)
                                .cancel_text(tr(cx, "common.cancel"))
                                .on_ok(move |_, window, cx| {
                                    window.push_notification(
                                        Notification::warning(confirmed.clone()),
                                        cx,
                                    );
                                    true
                                })
                        });
                    })
                    .into_any_element(),
            )
            .into_any_element(),
            demo_card(
                cx,
                "gallery.organisms.overlay.sheet.caption",
                Button::new("overlay-sheet-trigger")
                    .outline()
                    .label(tr(cx, "gallery.organisms.overlay.sheet.open"))
                    .on_click(|_, window, cx| {
                        window.open_sheet_at(Placement::Right, cx, |sheet, _win, cx| {
                            sheet
                                .title(tr(cx, "gallery.organisms.overlay.sheet.title"))
                                .size(px(360.))
                                .child(
                                    v_flex()
                                        .gap_2()
                                        .pt_1()
                                        .child(tr(cx, "gallery.organisms.overlay.sheet.body")),
                                )
                                .footer(
                                    h_flex().justify_end().gap_2().child(
                                        Button::new("overlay-sheet-close")
                                            .outline()
                                            .label(tr(cx, "gallery.organisms.overlay.sheet.close"))
                                            .on_click(|_, window, cx| {
                                                window.close_sheet(cx);
                                            }),
                                    ),
                                )
                        });
                    })
                    .into_any_element(),
            )
            .into_any_element(),
        ])
        .into_any_element();

        // Transient layers: Notifications, Command palette.
        let transient_row = demo_row([
            demo_card(
                cx,
                "gallery.organisms.overlay.notification.caption",
                h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        Button::new("overlay-notification-info")
                            .outline()
                            .small()
                            .label(tr(cx, "gallery.organisms.overlay.notification.info_label"))
                            .on_click(|_, window, cx| {
                                window.push_notification(
                                    Notification::info(tr(
                                        cx,
                                        "gallery.organisms.overlay.notification.info",
                                    )),
                                    cx,
                                );
                            }),
                    )
                    .child(
                        Button::new("overlay-notification-success")
                            .outline()
                            .small()
                            .label(tr(
                                cx,
                                "gallery.organisms.overlay.notification.success_label",
                            ))
                            .on_click(|_, window, cx| {
                                window.push_notification(
                                    Notification::success(tr(
                                        cx,
                                        "gallery.organisms.overlay.notification.success",
                                    )),
                                    cx,
                                );
                            }),
                    )
                    .child(
                        Button::new("overlay-notification-warning")
                            .outline()
                            .small()
                            .label(tr(
                                cx,
                                "gallery.organisms.overlay.notification.warning_label",
                            ))
                            .on_click(|_, window, cx| {
                                window.push_notification(
                                    Notification::warning(tr(
                                        cx,
                                        "gallery.organisms.overlay.notification.warning",
                                    )),
                                    cx,
                                );
                            }),
                    )
                    .child(
                        Button::new("overlay-notification-error")
                            .danger()
                            .small()
                            .label(tr(
                                cx,
                                "gallery.organisms.overlay.notification.error_label",
                            ))
                            .on_click(|_, window, cx| {
                                window.push_notification(
                                    Notification::error(tr(
                                        cx,
                                        "gallery.organisms.overlay.notification.error",
                                    )),
                                    cx,
                                );
                            }),
                    )
                    .into_any_element(),
            )
            .into_any_element(),
            demo_card(
                cx,
                "gallery.organisms.overlay.command.caption",
                Button::new("overlay-command-trigger")
                    .outline()
                    .label(tr(cx, "gallery.organisms.overlay.command.open"))
                    .on_click(move |_, window, cx| {
                        let palette = palette.clone();
                        // One guard per opening: focus the search field only on the
                        // dialog layer's first build, never on rebuilds.
                        let initial_focus = std::cell::Cell::new(true);
                        window.open_dialog(cx, move |dialog, window, cx| {
                            if initial_focus.replace(false) {
                                palette.update(cx, |state, cx| state.focus(window, cx));
                            }
                            let palette = palette.clone();
                            dialog
                                .close_button(false)
                                .p_0()
                                .content(move |content, _win, cx| {
                                    let palette = palette.clone();
                                    content.child(
                                        Command::new(&palette)
                                            .bordered(false)
                                            .placeholder(tr(
                                                cx,
                                                "gallery.organisms.overlay.command.placeholder",
                                            ))
                                            .group(
                                                CommandGroup::new()
                                                    .label(tr(
                                                        cx,
                                                        "gallery.organisms.overlay.command.group",
                                                    ))
                                                    .item(
                                                        CommandItem::new()
                                                            .label(tr(
                                                                cx,
                                                                "gallery.organisms.overlay.command.action_report",
                                                            ))
                                                            .icon(IconName::ChartPie),
                                                    )
                                                    .item(
                                                        CommandItem::new()
                                                            .label(tr(
                                                                cx,
                                                                "gallery.organisms.overlay.command.action_backup",
                                                            ))
                                                            .icon(IconName::HardDrive),
                                                    ),
                                            )
                                            .on_confirm(
                                                |index: IndexPath, window, cx| {
                                                    let done_key = match index.row {
                                                        0 => "gallery.organisms.overlay.command.report_done",
                                                        _ => "gallery.organisms.overlay.command.backup_done",
                                                    };
                                                    window.push_notification(
                                                        Notification::info(tr(cx, done_key)),
                                                        cx,
                                                    );
                                                    window.close_dialog(cx);
                                                },
                                            )
                                            .w(px(380.)),
                                    )
                                })
                        });
                    })
                    .into_any_element(),
            )
            .into_any_element(),
        ])
        .into_any_element();

        section_page(
            cx,
            "gallery.section.organisms-overlay",
            [modal_row, transient_row],
        )
    }
}
