// SPDX-License-Identifier: GPL-3.0-or-later
//! Molecules — structure & containers: Tabs (TabBar/Tab), Accordion, Collapsible,
//! GroupBox, Form (+ FocusTrap wrapping its fields — see the form card), Empty, Alert,
//! Questionnaire (design sheet: "MOLECULES · structure").

use gpui_kit::component::FocusTrapElement as _;
use gpui_kit::component::accordion::Accordion;
use gpui_kit::component::alert::Alert;

use gpui_kit::component::button::{
    Button, ButtonVariants as _, DropdownButton, Toggle, ToggleGroup,
};
use gpui_kit::component::collapsible::Collapsible;
use gpui_kit::component::empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use gpui_kit::component::form::{field, v_form};
use gpui_kit::component::group_box::{GroupBox, GroupBoxVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::menu::PopupMenuItem;
use gpui_kit::component::questionnaire::{
    Questionnaire, QuestionnaireActions, QuestionnaireChoice, QuestionnaireChoiceDefinition,
    QuestionnaireChoices, QuestionnaireDescription, QuestionnaireError, QuestionnaireInput,
    QuestionnaireInputDefinition, QuestionnaireItem, QuestionnaireItemDefinition,
    QuestionnaireNext, QuestionnairePrevious, QuestionnaireProgress, QuestionnaireSkip,
    QuestionnaireState, QuestionnaireSubmit, QuestionnaireTitle,
};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tab::{Tab, TabBar};
use gpui_kit::component::{Icon, IconName, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    AnyView, AppContext as _, Context, FocusHandle, IntoElement, ParentElement as _, Styled as _,
    Window, div, px,
};

use crate::app::{Gallery, tr};
use crate::sections::layout::{demo_card, demo_row, section_page};

/// The structure-molecule section.
pub struct Section {
    /// Index of the selected tab in the tabs demo.
    tab: usize,
    /// Whether the collapsible demo is expanded.
    expanded: bool,
    /// The group-box demo's switch (alert on new logins).
    login_alert: bool,
    /// The form demo's name field (localized placeholder, RULE-UI-005).
    name: gpui_kit::Entity<InputState>,
    /// The form demo's email field (localized placeholder, RULE-UI-005).
    email: gpui_kit::Entity<InputState>,
    /// The focus trap wrapping the form demo's fields (FocusTrap demo).
    trap: FocusHandle,
    /// The questionnaire demo's answers/navigation state (one entity for all parts).
    survey: gpui_kit::Entity<QuestionnaireState>,
}

pub fn view(window: &mut Window, cx: &mut Context<'_, Gallery>) -> AnyView {
    cx.new(|cx| {
        let name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(tr(cx, "gallery.molecules.structure.form.placeholder_name"))
        });
        let email = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(tr(cx, "gallery.molecules.structure.form.placeholder_email"))
        });
        let trap = cx.focus_handle();
        // The questionnaire: one required single-choice question + one optional
        // freeform question; the freeform input gets a localized placeholder.
        let survey = cx.new(|cx| {
            let notes = cx.new(|cx| {
                InputState::new(window, cx).placeholder(tr(
                    cx,
                    "gallery.molecules.structure.questionnaire.input_placeholder",
                ))
            });
            let items = vec![
                QuestionnaireItemDefinition::new(
                    "use",
                    tr(cx, "gallery.molecules.structure.questionnaire.q1"),
                )
                .with_required(true)
                .with_description(tr(
                    cx,
                    "gallery.molecules.structure.questionnaire.q1_description",
                ))
                .with_choices([
                    QuestionnaireChoiceDefinition::new(
                        "invoice",
                        tr(
                            cx,
                            "gallery.molecules.structure.questionnaire.choice_invoicing",
                        ),
                    ),
                    QuestionnaireChoiceDefinition::new(
                        "ledger",
                        tr(
                            cx,
                            "gallery.molecules.structure.questionnaire.choice_ledger",
                        ),
                    ),
                ]),
                QuestionnaireItemDefinition::new(
                    "notes",
                    tr(cx, "gallery.molecules.structure.questionnaire.q2"),
                )
                .with_description(tr(
                    cx,
                    "gallery.molecules.structure.questionnaire.q2_description",
                ))
                .with_input(QuestionnaireInputDefinition::new(
                    notes,
                    tr(cx, "gallery.molecules.structure.questionnaire.input_label"),
                )),
            ];
            questionnaire_state(items, cx)
        });
        Section {
            tab: 0,
            expanded: false,
            login_alert: true,
            name,
            email,
            trap,
            survey,
        }
    })
    .into()
}

/// Build the demo [`QuestionnaireState`] without panicking. The schema above is
/// static and hand-checked (unique item names, unique choice values, no default
/// selection), so the first call always succeeds; the empty fallback schema is
/// valid by definition and ends the recursion on the impossible error path.
fn questionnaire_state(
    items: Vec<QuestionnaireItemDefinition>,
    cx: &mut Context<QuestionnaireState>,
) -> QuestionnaireState {
    if let Ok(state) = QuestionnaireState::new(items, cx) {
        return state;
    }
    questionnaire_state(Vec::new(), cx)
}

impl gpui_kit::Render for Section {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui_kit::Context<'_, Self>,
    ) -> impl IntoElement {
        let mode = cx.global::<crate::app::Prefs>().mode;
        section_page(
            cx,
            "gallery.section.molecules-structure",
            [
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.structure.tabs",
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(
                                TabBar::new("structure-tabs")
                                    .segmented()
                                    // MOL·01 tabs: the segmented bar is an rx6 #1c1c1c
                                    // plate (not gpui's rx20), active chip white@4%.
                                    .rounded(px(bookkeepy_primary_data::theme::radii().item as f32))
                                    .selected_index(self.tab)
                                    .on_click(cx.listener(
                                        |this, index: &usize, _window, cx| {
                                            this.tab = *index;
                                            cx.notify();
                                        },
                                    ))
                                    .child(Tab::new().label(tr(cx, "gallery.molecules.structure.tabs.tab_overview")))
                                    .child(Tab::new().label(tr(cx, "gallery.molecules.structure.tabs.tab_clients")))
                                    .child(Tab::new().label(tr(cx, "gallery.molecules.structure.tabs.tab_reports"))),
                            )
                            .child(div().p_2().child(match self.tab {
                                0 => tr(cx, "gallery.molecules.structure.tabs.panel_overview"),
                                1 => tr(cx, "gallery.molecules.structure.tabs.panel_clients"),
                                _ => tr(cx, "gallery.molecules.structure.tabs.panel_reports"),
                            }))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.structure.accordion",
                        Accordion::new("structure-accordion")
                            .bordered(false)
                            .item(|item| {
                                item.title(tr(cx, "gallery.molecules.structure.accordion.title_clients"))
                                    .child(tr(cx, "gallery.molecules.structure.accordion.content_clients"))
                            })
                            .item(|item| {
                                item.title(tr(cx, "gallery.molecules.structure.accordion.title_vat"))
                                    .child(tr(cx, "gallery.molecules.structure.accordion.content_vat"))
                            })
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.structure.collapsible",
                        Collapsible::new()
                            .open(self.expanded)
                            .child(tr(cx, "gallery.molecules.structure.collapsible.summary"))
                            .content(tr(cx, "gallery.molecules.structure.collapsible.content"))
                            .child(
                                h_flex().justify_center().child(
                                    Button::new("collapsible-toggle")
                                        .ghost()
                                        .xsmall()
                                        .icon(IconName::ChevronDown)
                                        .label(tr(cx, "gallery.molecules.structure.collapsible.show_more"))
                                        .when(self.expanded, |this| {
                                            this.icon(IconName::ChevronUp)
                                                .label(tr(cx, "gallery.molecules.structure.collapsible.show_less"))
                                        })
                                        .on_click(cx.listener(
                                            |this, _ev: &gpui_kit::ClickEvent, _window, cx| {
                                                this.expanded = !this.expanded;
                                                cx.notify();
                                            },
                                        )),
                                ),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.structure.group_box",
                        bookkeepy_ui::api::language::surface::group_box(
                            GroupBox::new()
                                .id("structure-group-box")
                                .fill()
                                .title(tr(cx, "gallery.molecules.structure.group_box.title")),
                            mode,
                        )
                            .child(
                                h_flex()
                                    .justify_between()
                                    .items_center()
                                    .child(tr(cx, "gallery.molecules.structure.group_box.row_login"))
                                    .child(
                                        Switch::new("login-alert")
                                            .checked(self.login_alert)
                                            .on_change(cx.listener(
                                                |this, checked: &bool, _window, cx| {
                                                    this.login_alert = *checked;
                                                    cx.notify();
                                                },
                                            )),
                                    ),
                            )
                            .footer(tr(cx, "gallery.molecules.structure.group_box.footer"))
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.structure.form",
                        // FocusTrap demo: the form's fields sit inside a focus trap, so
                        // Tab cycles name -> email -> buttons and never escapes the form
                        // (what Dialog/Sheet do natively for their overlays).
                        div()
                            .focus_trap("structure-form-trap", &self.trap)
                            .child(
                                v_form()
                                    .child(
                                        field()
                                            .label(tr(cx, "gallery.molecules.structure.form.label_name"))
                                            .required(true)
                                            .child(Input::new(&self.name)),
                                    )
                                    .child(
                                        field()
                                            .label(tr(cx, "gallery.molecules.structure.form.label_email"))
                                            .required(true)
                                            .description(tr(
                                                cx,
                                                "gallery.molecules.structure.form.description_email",
                                            ))
                                            .child(Input::new(&self.email)),
                                    )
                                    .footer(
                                        h_flex().justify_end().gap_2()
                                            .child(Button::new("form-cancel").label(tr(
                                                cx,
                                                "gallery.molecules.structure.form.action_cancel",
                                            )))
                                            .child(Button::new("form-save").primary().label(tr(
                                                cx,
                                                "gallery.molecules.structure.form.action_save",
                                            ))),
                                    ),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.structure.empty",
                        Empty::new()
                            // MOL·01 empty: borderless rx16 panel (no dashed outline).
                            .border_0()
                            .rounded(px(bookkeepy_primary_data::theme::radii().overlay as f32))
                            .header(
                                EmptyHeader::new()
                                    .media(
                                        EmptyMedia::new()
                                            .with_variant(EmptyMediaVariant::Icon)
                                            .child(Icon::new(IconName::Inbox)),
                                    )
                                    .title(EmptyTitle::new().child(tr(
                                        cx,
                                        "gallery.molecules.structure.empty.title",
                                    )))
                                    .description(EmptyDescription::new().child(tr(
                                        cx,
                                        "gallery.molecules.structure.empty.description",
                                    ))),
                            )
                            .content(
                                EmptyContent::new()
                                    .flex_row()
                                    .flex_wrap()
                                    .justify_center()
                                    .gap_2()
                                    .child(Button::new("empty-new-invoice").primary().label(tr(
                                        cx,
                                        "gallery.molecules.structure.empty.action_new",
                                    )))
                                    .child(Button::new("empty-import").outline().label(tr(
                                        cx,
                                        "gallery.molecules.structure.empty.action_import",
                                    ))),
                            )
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                demo_row([
                demo_card(
                    cx,
                    "gallery.molecules.structure.alert",
                    v_flex()
                        .gap_2()
                        .child(
                            bookkeepy_ui::api::language::feedback::alert(
                                Alert::info(
                                    "structure-alert-info",
                                    tr(cx, "gallery.molecules.structure.alert.info_message"),
                                )
                                .title(tr(cx, "gallery.molecules.structure.alert.info_title")),
                                bookkeepy_ui::api::language::feedback::AlertTone::Info,
                                mode,
                            ),
                        )
                        .child(
                            bookkeepy_ui::api::language::feedback::alert(
                                Alert::warning(
                                    "structure-alert-warning",
                                    tr(cx, "gallery.molecules.structure.alert.warning_message"),
                                )
                                .banner(),
                                bookkeepy_ui::api::language::feedback::AlertTone::Warning,
                                mode,
                            ),
                        )
                        .child(
                            bookkeepy_ui::api::language::feedback::alert(
                                Alert::success(
                                    "structure-alert-success",
                                    tr(cx, "gallery.molecules.structure.alert.success_message"),
                                )
                                .title(tr(cx, "gallery.molecules.structure.alert.success_title")),
                                bookkeepy_ui::api::language::feedback::AlertTone::Success,
                                mode,
                            ),
                        )
                        .child(
                            bookkeepy_ui::api::language::feedback::alert(
                                Alert::error(
                                    "structure-alert-danger",
                                    tr(cx, "gallery.molecules.structure.alert.danger_message"),
                                )
                                .title(tr(cx, "gallery.molecules.structure.alert.danger_title")),
                                bookkeepy_ui::api::language::feedback::AlertTone::Danger,
                                mode,
                            ),
                        )
                        .child(
                            bookkeepy_ui::api::language::feedback::alert(
                                Alert::new(
                                    "structure-alert-accent",
                                    tr(cx, "gallery.molecules.structure.alert.accent_message"),
                                )
                                .title(tr(cx, "gallery.molecules.structure.alert.accent_title")),
                                bookkeepy_ui::api::language::feedback::AlertTone::Accent,
                                mode,
                            ),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                // A two-step questionnaire: only the active item renders, the
                // actions row (Previous/Skip/Next/Submit) navigates it.
                demo_card(
                    cx,
                    "gallery.molecules.structure.questionnaire",
                    Questionnaire::new(&self.survey)
                        .child(QuestionnaireProgress::new(&self.survey))
                        .child(
                            QuestionnaireItem::new(&self.survey, "use")
                                .child(QuestionnaireTitle::new(&self.survey, "use"))
                                .child(QuestionnaireDescription::new(&self.survey, "use"))
                                .child(
                                    QuestionnaireChoices::new(&self.survey, "use")
                                        .child(QuestionnaireChoice::new(
                                            &self.survey,
                                            "use",
                                            "invoice",
                                        ))
                                        .child(QuestionnaireChoice::new(
                                            &self.survey,
                                            "use",
                                            "ledger",
                                        )),
                                )
                                .child(QuestionnaireError::new(&self.survey, "use")),
                        )
                        .child(
                            QuestionnaireItem::new(&self.survey, "notes")
                                .child(QuestionnaireTitle::new(&self.survey, "notes"))
                                .child(QuestionnaireDescription::new(&self.survey, "notes"))
                                .child(
                                    QuestionnaireChoices::new(&self.survey, "notes")
                                        .child(QuestionnaireInput::new(&self.survey, "notes")),
                                )
                                .child(QuestionnaireError::new(&self.survey, "notes")),
                        )
                        .child(
                            QuestionnaireActions::new(&self.survey)
                                .child(QuestionnairePrevious::new(&self.survey))
                                .child(QuestionnaireSkip::new(&self.survey))
                                .child(QuestionnaireNext::new(&self.survey))
                                .child(QuestionnaireSubmit::new(&self.survey)),
                        )
                        .into_any_element(),
                )
                .into_any_element(),
                demo_row([
                    demo_card(
                        cx,
                        "gallery.molecules.structure.toggle_group",
                        // MOL·02 toggle group: segmented #1c1c1c rx6 container, segments
                        // white@12% rx4 when selected.
                        bookkeepy_ui::api::language::controls::toggle_group(
                            ToggleGroup::new("structure-toggle-group")
                                .segmented()
                                .child(
                                    bookkeepy_ui::api::language::controls::toggle(
                                        Toggle::new("tg-day"),
                                        mode,
                                    )
                                    .label(tr(cx, "gallery.atoms.button.day"))
                                    .checked(true),
                                )
                                .child(
                                    bookkeepy_ui::api::language::controls::toggle(
                                        Toggle::new("tg-week"),
                                        mode,
                                    )
                                    .label(tr(cx, "gallery.atoms.button.week")),
                                )
                                .child(
                                    bookkeepy_ui::api::language::controls::toggle(
                                        Toggle::new("tg-month"),
                                        mode,
                                    )
                                    .label(tr(cx, "gallery.atoms.button.month")),
                                ),
                            mode,
                        )
                        .into_any_element(),
                    )
                    .into_any_element(),
                    demo_card(
                        cx,
                        "gallery.molecules.structure.dropdown_button",
                        DropdownButton::new("structure-dropdown-button")
                            .button(
                                Button::new("structure-dd-btn")
                                    .secondary()
                                    .label(tr(cx, "gallery.molecules.structure.dropdown_button.action")),
                            )
                            .dropdown_menu(|menu, _window, _cx| {
                                menu.item(PopupMenuItem::new(tr(
                                    _cx,
                                    "gallery.molecules.popups.menu.item_new",
                                )))
                            })
                            .into_any_element(),
                    )
                    .into_any_element(),
                ])
                .into_any_element(),
                ])
                .into_any_element(),
            ],
        )
    }
}
