//! Class detail and enumeration inspector for FDA Information Model.

use crate::design_system::tokens::ColorTokens;
use crate::features::concepts::Concept;
use crate::features::information_model::diagram::ClassGraph;
use crate::features::information_model::linter::NamingLinter;
use crate::features::information_model::models::{InformationClass, InformationModel};
use crate::features::information_model::ui::attribute_editor;
pub use crate::features::information_model::ui::enum_detail::view as enum_detail_view;
use crate::ui::app::{ConceptOption, Message};
use crate::ui::theme::{
    danger_button_style, focused_button_style, modern_input_style, pill_container_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Color, Element, Length};

#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    class: &'a InformationClass,
    info_model: &'a InformationModel,
    class_graph: &'a ClassGraph,
    concepts: &'a [Concept],
    inspector_focused_button: Option<usize>,
    is_concept_picker_open: bool,
    concept_picker_search: &'a str,
    concept_picker_index: usize,
) -> Element<'a, Message> {
    let class_id = class.id();
    let is_on_canvas = class_graph.is_class_on_diagram(class_id);

    let mut name_col = column![text_input("Klassenavn...", class.name())
        .id("info_class_name_input")
        .style(modern_input_style)
        .size(13.0)
        .on_input(move |s| Message::UpdateInformationClassName(class_id, s))
        .padding([4, 6])]
    .spacing(4);

    if let Some(issue) = NamingLinter::check_class_name(class.name()) {
        if !class.name().trim().is_empty() {
            let mut msg = format!("⚠️ {}: {}", issue.rule, issue.message);
            if let Some(fix) = &issue.suggested_fix {
                msg.push_str(&format!(" (Forslag: {})", fix));
            }
            name_col = name_col.push(
                text(msg)
                    .size(10.5)
                    .color(ColorTokens::WARNING_AMBER.into_color()),
            );
        }
    }

    let desc_input = text_input("Beskrivelse...", class.description().unwrap_or(""))
        .id("info_class_desc_input")
        .style(modern_input_style)
        .size(12.0)
        .on_input(move |s| Message::UpdateInformationClassDescription(class_id, s))
        .padding([4, 6]);

    let is_abstract = class.is_abstract();
    let is_local = class.is_local();

    let is_abstract_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_ABSTRACT_CHECKBOX);
    let abstract_checkbox = container(
        checkbox(is_abstract)
            .label("Abstrakt klasse ({abstract})")
            .size(13)
            .on_toggle(move |val| Message::SetInformationClassAbstract(class_id, val)),
    )
    .style(move |_| container::Style {
        border: iced::Border {
            color: if is_abstract_focused {
                ThemeColors::FOCUS_RING
            } else {
                Color::TRANSPARENT
            },
            width: 1.5,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .padding(2);

    let is_type_local_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_TYPE_LOCAL);
    let is_type_borrowed_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_TYPE_BORROWED);
    let local_base = if is_local {
        primary_button_style
    } else {
        secondary_button_style
    };
    let borrowed_base = if !is_local {
        primary_button_style
    } else {
        secondary_button_style
    };

    let type_selector = column![
        crate::ui::inspector_panel::section_header("Klassetype (FDA)"),
        row![
            button(text("Lokal (Sand)").size(11))
                .style(focused_button_style(local_base, is_type_local_focused))
                .on_press(Message::SetInformationClassLocal(class_id, true))
                .padding([3, 8]),
            button(text("Indlånt (Blå)").size(11))
                .style(focused_button_style(
                    borrowed_base,
                    is_type_borrowed_focused
                ))
                .on_press(Message::SetInformationClassLocal(class_id, false))
                .padding([3, 8]),
        ]
        .spacing(6),
    ]
    .spacing(4);

    let origin_section: Element<'a, Message> = if !is_local {
        column![
            text("Oprindelsesmodel / Kilde URI:")
                .size(11.0)
                .color(ThemeColors::SLATE_600),
            text_input(
                "https://data.gov.dk/model/...",
                class.origin_model().unwrap_or("")
            )
            .id("info_class_origin_model_input")
            .style(modern_input_style)
            .size(11.5)
            .on_input(move |s| Message::SetInformationClassOriginModel(class_id, s))
            .padding([3, 6]),
        ]
        .spacing(4)
        .into()
    } else {
        Space::new().height(0).into()
    };

    let mut concept_badges_row = row![].spacing(4);
    for (k, cid) in class.concept_ids().iter().enumerate() {
        let term = concepts
            .iter()
            .find(|c| c.id() == *cid)
            .map(|c| c.preferred_term())
            .unwrap_or("Ukendt");
        let cid_val = *cid;
        let is_remove_focused = inspector_focused_button
            == Some(crate::ui::app::INFO_INSPECTOR_CONCEPT_REMOVE_START + k);
        let badge = container(
            row![
                text(term).size(11).color(ThemeColors::PRIMARY),
                button(text("✕").size(9))
                    .style(focused_button_style(
                        secondary_button_style,
                        is_remove_focused
                    ))
                    .on_press(Message::RemoveConceptFromInformationClass(
                        class_id, cid_val
                    ))
                    .padding([1, 4]),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        )
        .style(pill_container_style)
        .padding([2, 6]);
        concept_badges_row = concept_badges_row.push(badge);
    }
    let concept_badges: Element<'a, Message> = concept_badges_row.wrap().into();

    let available_concepts: Vec<ConceptOption> = concepts
        .iter()
        .filter(|c| !class.concept_ids().contains(&c.id()))
        .map(|c| ConceptOption {
            id: c.id(),
            term: c.preferred_term().to_string(),
        })
        .collect();

    let is_picker_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_ADD_CONCEPT_PICKER);
    let add_concept_widget: Element<'a, Message> = if !is_concept_picker_open {
        button(
            row![
                text("+ Knyt begreb...")
                    .size(11.5)
                    .color(ThemeColors::SLATE_700),
                Space::new().width(Length::Fill),
                text("▾").size(11.0).color(ThemeColors::SLATE_400),
            ]
            .align_y(Alignment::Center),
        )
        .style(move |t, s| {
            let mut st = secondary_button_style(t, s);
            st.background = Some(iced::Background::Color(ThemeColors::SURFACE_CARD));
            st.border = iced::Border {
                color: if is_picker_focused {
                    ThemeColors::FOCUS_RING
                } else {
                    ThemeColors::SURFACE_BORDER
                },
                width: if is_picker_focused { 2.0 } else { 1.0 },
                radius: 6.0.into(),
            };
            st
        })
        .on_press(Message::OpenConceptPicker)
        .padding([4, 8])
        .width(Length::Fill)
        .into()
    } else {
        let search_input = text_input("Søg begreb...", concept_picker_search)
            .id("info_concept_picker_input")
            .style(modern_input_style)
            .size(11.5)
            .on_input(Message::ConceptPickerSearchChanged)
            .on_submit(Message::ConceptPickerSubmit)
            .padding([4, 6])
            .width(Length::Fill);
        let close_btn = button(text("✕").size(10.0))
            .style(secondary_button_style)
            .on_press(Message::CloseConceptPicker)
            .padding([3, 6]);
        let top_row = row![search_input, close_btn]
            .spacing(4)
            .align_y(Alignment::Center);

        let q = concept_picker_search.trim().to_lowercase();
        let filtered: Vec<&ConceptOption> = available_concepts
            .iter()
            .filter(|o| q.is_empty() || o.term.to_lowercase().contains(&q))
            .collect();
        let mut list_col = column![].spacing(2);
        if filtered.is_empty() {
            list_col = list_col.push(
                container(
                    text("Ingen matchende begreber")
                        .size(11.0)
                        .color(ThemeColors::TEXT_MUTED),
                )
                .padding([6, 8]),
            );
        } else {
            for (idx, opt) in filtered.iter().enumerate() {
                let is_active = idx == concept_picker_index;
                let opt_clone = (*opt).clone();
                let prefix = if is_active { "▸ " } else { "   " };
                let item_btn = button(text(format!("{}{}", prefix, opt.term)).size(11.5).color(
                    if is_active {
                        ThemeColors::PRIMARY
                    } else {
                        ThemeColors::SLATE_800
                    },
                ))
                .style(move |_, st| iced::widget::button::Style {
                    background: Some(iced::Background::Color(if is_active {
                        ThemeColors::FDA_SAND
                    } else if st == iced::widget::button::Status::Hovered {
                        ThemeColors::SLATE_100
                    } else {
                        Color::TRANSPARENT
                    })),
                    border: iced::Border {
                        color: if is_active {
                            ThemeColors::FDA_SAND_BORDER
                        } else {
                            Color::TRANSPARENT
                        },
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .on_press(Message::AddConceptToInformationClass(class_id, opt_clone))
                .padding([3, 6])
                .width(Length::Fill);
                list_col = list_col.push(item_btn);
            }
        }

        container(
            column![
                top_row,
                container(scrollable(list_col).height(Length::Shrink))
                    .style(|_| container::Style {
                        background: Some(iced::Background::Color(ThemeColors::SURFACE_CARD)),
                        border: iced::Border {
                            color: ThemeColors::SURFACE_BORDER,
                            width: 1.0,
                            radius: 4.0.into()
                        },
                        ..Default::default()
                    })
                    .max_height(160.0)
                    .padding(2)
                    .width(Length::Fill),
            ]
            .spacing(4),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(ThemeColors::SLATE_50)),
            border: iced::Border {
                color: ThemeColors::FOCUS_RING,
                width: 1.5,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .padding(4)
        .width(Length::Fill)
        .into()
    };

    let attr_section =
        attribute_editor::view(class, info_model, concepts, inspector_focused_button);

    let is_canvas_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_CANVAS_ACTION);
    let is_delete_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_DELETE_CLASS);

    let canvas_btn = if is_on_canvas {
        if let Some(n) = class_graph.find_node_by_class(class_id) {
            let nid = n.id();
            button(text("Fjern fra diagram").size(11))
                .style(focused_button_style(
                    secondary_button_style,
                    is_canvas_focused,
                ))
                .on_press(Message::RemoveClassFromDiagram(nid))
                .padding([4, 8])
        } else {
            button(text("Tilføj til diagram").size(11))
                .style(focused_button_style(
                    primary_button_style,
                    is_canvas_focused,
                ))
                .on_press(Message::AddClassToDiagram(class_id))
                .padding([4, 8])
        }
    } else {
        button(text("Tilføj til diagram").size(11))
            .style(focused_button_style(
                primary_button_style,
                is_canvas_focused,
            ))
            .on_press(Message::AddClassToDiagram(class_id))
            .padding([4, 8])
    };

    let actions = row![
        canvas_btn,
        Space::new().width(Length::Fill),
        button(text("🗑️ Slet klasse").size(11))
            .style(focused_button_style(danger_button_style, is_delete_focused))
            .on_press(Message::DeleteInformationClass(class_id))
            .padding([4, 8]),
    ]
    .align_y(Alignment::Center);

    let header = crate::ui::inspector_panel::panel_header(
        crate::ui::inspector_panel::PROPERTIES_TITLE,
        Some((
            "Klasse",
            ThemeColors::FDA_SAND,
            ColorTokens::FDA_SAND_ACCENT.into_color(),
        )),
        Some(Message::SelectInformationClass(None)),
    );

    let inspector_content = column![
        header,
        column![
            crate::ui::inspector_panel::section_header("Identifikation"),
            name_col,
            desc_input,
            abstract_checkbox,
            type_selector,
            origin_section
        ]
        .spacing(6),
        column![
            crate::ui::inspector_panel::section_header("Begrebssporing (FDA)"),
            concept_badges,
            add_concept_widget
        ]
        .spacing(6),
        attr_section,
        actions,
    ]
    .spacing(10);

    crate::ui::inspector_panel::panel_container(inspector_content.into())
}
