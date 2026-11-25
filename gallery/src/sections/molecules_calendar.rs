// SPDX-License-Identifier: GPL-3.0-or-later
//! Molecules — MOL·03 sheet: Calendar, TimeField, ColorPicker, Attachment (moved here
//! from molecules-pickers / molecules-data-list so the section mirrors the design sheet
//! "MOLECULES · 03 calendar, time field, color picker, attachment").

use gpui_kit::component::attachment::{
    Attachment, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia,
    AttachmentStatus, AttachmentTitle,
};
use gpui_kit::component::calendar::{Calendar, CalendarState};
use gpui_kit::component::color_picker::{ColorPicker, ColorPickerState};
use gpui_kit::component::time_field::{TimeField, TimeFieldState};
use gpui_kit::component::{ActiveTheme as _, Icon, IconName};
use gpui_kit::{
    AnyView, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Styled as _,
    Window, px,
};

use bookkeepy_ui::api::language::{ControlSize, color_picker_size};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The MOL·03 section.
pub struct Section {
    month: Entity<CalendarState>,
    time: Entity<TimeFieldState>,
    category: Entity<ColorPickerState>,
    /// Second swatch state for the dense (Small) scale demo of the one size-aware picker.
    category_sm: Entity<ColorPickerState>,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let month = cx.new(|cx| CalendarState::new(window, cx));
        let time = cx.new(|cx| TimeFieldState::new(window, cx));
        let category =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cx.theme().primary));
        // The dense-scale demo rides the same control ladder (ATOMS · 01: 24px).
        let category_sm =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cx.theme().accent));
        Section {
            month,
            time,
            category,
            category_sm,
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
            "gallery.section.molecules-calendar",
            [
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.calendar.caption",
                        Calendar::new(&self.month)
                            .border_0()
                            .rounded(px(bookkeepy_primary_data::theme::radii().card as f32))
                            .bg(cx.theme().popover)
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.time_field.caption",
                        TimeField::new(&self.time).into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.pickers.color_picker.caption",
                        // Left: the standard 32px swatch. Right: the design's dense
                        // 24px scale step, applied through the picker's own `Sizable`
                        // API via the ui crate's language scale.
                        demo_row([
                            ColorPicker::new(&self.category)
                                .label(tr(cx, "gallery.molecules.pickers.color_picker.label"))
                                .into_any_element(),
                            color_picker_size(
                                ColorPicker::new(&self.category_sm),
                                ControlSize::Small,
                            )
                            .into_any_element(),
                        ])
                        .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_card(
                    cx,
                    "gallery.molecules.data_list.attachment",
                    AttachmentGroup::new("calendar-attachment-group")
                        .child(
                            Attachment::new()
                                .media(AttachmentMedia::new().child(Icon::new(IconName::FileText)))
                                .content(
                                    AttachmentContent::new()
                                        .title(AttachmentTitle::new(tr(
                                            cx,
                                            "gallery.molecules.data_list.attachment.file_invoice",
                                        )))
                                        .description(AttachmentDescription::new(tr(
                                            cx,
                                            "gallery.molecules.data_list.attachment.meta_invoice",
                                        ))),
                                ),
                        )
                        .child(
                            Attachment::new()
                                .status(AttachmentStatus::Uploading)
                                .progress(45.)
                                .media(AttachmentMedia::new().child(Icon::new(IconName::File)))
                                .content(
                                    AttachmentContent::new()
                                        .title(AttachmentTitle::new(tr(
                                            cx,
                                            "gallery.molecules.data_list.attachment.file_receipt",
                                        )))
                                        .description(AttachmentDescription::new(tr(
                                            cx,
                                            "gallery.molecules.data_list.attachment.meta_receipt",
                                        ))),
                                ),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
            ],
        )
    }
}
