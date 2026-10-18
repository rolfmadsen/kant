//! Relation and association inspector for FDA Information Model UML diagram.

use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::{NodeId, RelationKind};
use crate::features::information_model::diagram::ClassGraph;
use crate::features::information_model::linter::NamingLinter;
use crate::features::information_model::models::{InformationModel, Multiplicity};
use crate::ui::app::Message;
use crate::ui::theme::{
    card_container_style, danger_button_style, focused_button_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view<'a>(
    from_id: NodeId,
    to_id: NodeId,
    info_model: &'a InformationModel,
    class_graph: &'a ClassGraph,
    inspector_focused_button: Option<usize>,
) -> Element<'a, Message> {
    if let Some(edge) = class_graph.find_edge(from_id, to_id) {
        let from_class = class_graph
            .find_node(from_id)
            .and_then(|n| info_model.get_class(n.class_id()))
            .map(|c| c.name())
            .unwrap_or("Kilde");
        let to_class = class_graph
            .find_node(to_id)
            .and_then(|n| info_model.get_class(n.class_id()))
            .map(|c| c.name())
            .unwrap_or("Mål");

        let header = crate::ui::inspector_panel::panel_header(
            crate::ui::inspector_panel::PROPERTIES_TITLE,
            Some(("Relation", ThemeColors::SLATE_100, ThemeColors::SLATE_300)),
            Some(Message::InfoEdgeSelected(None)),
        );

        let is_reverse_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_REVERSE);
        let nodes_info = column![
            crate::ui::inspector_panel::section_header("Forbindelse"),
            row![
                text(format!("{} ➔ {}", from_class, to_class))
                    .size(13)
                    .color(ThemeColors::SLATE_900),
                Space::new().width(Length::Fill),
                button(text("⇄ Vend").size(11))
                    .style(focused_button_style(
                        secondary_button_style,
                        is_reverse_focused
                    ))
                    .on_press(Message::InfoReverseEdge(from_id, to_id))
                    .padding([2, 6]),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(4);

        let is_assoc_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_KIND_ASSOC);
        let is_gen_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_KIND_GEN);
        let is_comp_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_KIND_COMP);
        let is_dep_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_KIND_DEP);

        let kind_selector = column![
            crate::ui::inspector_panel::section_header("Relationstype"),
            row![
                button(text("Association").size(11))
                    .style(focused_button_style(
                        if edge.kind() == RelationKind::Association {
                            primary_button_style
                        } else {
                            secondary_button_style
                        },
                        is_assoc_focused,
                    ))
                    .on_press(Message::InfoUpdateEdgeKind(
                        from_id,
                        to_id,
                        RelationKind::Association
                    ))
                    .padding([4, 6]),
                button(text("Generalisering").size(11))
                    .style(focused_button_style(
                        if edge.kind() == RelationKind::Generalization {
                            primary_button_style
                        } else {
                            secondary_button_style
                        },
                        is_gen_focused,
                    ))
                    .on_press(Message::InfoUpdateEdgeKind(
                        from_id,
                        to_id,
                        RelationKind::Generalization
                    ))
                    .padding([4, 6]),
                button(text("Komposition").size(11))
                    .style(focused_button_style(
                        if edge.kind() == RelationKind::Composition {
                            primary_button_style
                        } else {
                            secondary_button_style
                        },
                        is_comp_focused,
                    ))
                    .on_press(Message::InfoUpdateEdgeKind(
                        from_id,
                        to_id,
                        RelationKind::Composition
                    ))
                    .padding([4, 6]),
                button(text("Dependency").size(11))
                    .style(focused_button_style(
                        if edge.kind() == RelationKind::Dependency {
                            primary_button_style
                        } else {
                            secondary_button_style
                        },
                        is_dep_focused,
                    ))
                    .on_press(Message::InfoUpdateEdgeKind(
                        from_id,
                        to_id,
                        RelationKind::Dependency
                    ))
                    .padding([4, 6]),
            ]
            .spacing(4),
        ]
        .spacing(4);

        let is_dir_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_DIRECTED_CHECKBOX);
        let directed_selector: Element<'a, Message> = if edge.kind() == RelationKind::Association {
            let cb = checkbox(edge.is_directed())
                .label("Halv pil (rettet)")
                .size(14)
                .on_toggle(move |val| Message::InfoToggleEdgeDirected(from_id, to_id, val));
            let cb_container = container(cb)
                .padding([2, 4])
                .style(move |_| container::Style {
                    border: iced::Border {
                        color: if is_dir_focused {
                            ThemeColors::FOCUS_RING
                        } else {
                            iced::Color::TRANSPARENT
                        },
                        width: if is_dir_focused { 1.5 } else { 0.0 },
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                });
            column![
                crate::ui::inspector_panel::section_header("Retning / Navigabilitet"),
                cb_container,
            ]
            .spacing(4)
            .into()
        } else {
            Space::new().height(0).into()
        };

        let is_src_mult_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_SRC_MULT);
        let is_tgt_mult_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_TGT_MULT);
        let multiplicity_selector: Element<'a, Message> =
            if edge.kind() != RelationKind::Generalization {
                let src_pick = pick_list(
                    Multiplicity::PRESETS,
                    edge.source_multiplicity(),
                    move |m| Message::InfoUpdateEdgeSourceMultiplicity(from_id, to_id, Some(m)),
                )
                .placeholder("Vælg...")
                .text_size(11.0)
                .padding([3, 6])
                .width(Length::Fill);

                let src_container = container(src_pick).style(move |_| container::Style {
                    border: iced::Border {
                        color: if is_src_mult_focused {
                            ThemeColors::FOCUS_RING
                        } else {
                            iced::Color::TRANSPARENT
                        },
                        width: if is_src_mult_focused { 1.5 } else { 0.0 },
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                });

                let tgt_pick = pick_list(
                    Multiplicity::PRESETS,
                    edge.target_multiplicity(),
                    move |m| Message::InfoUpdateEdgeTargetMultiplicity(from_id, to_id, Some(m)),
                )
                .placeholder("Vælg...")
                .text_size(11.0)
                .padding([3, 6])
                .width(Length::Fill);

                let tgt_container = container(tgt_pick).style(move |_| container::Style {
                    border: iced::Border {
                        color: if is_tgt_mult_focused {
                            ThemeColors::FOCUS_RING
                        } else {
                            iced::Color::TRANSPARENT
                        },
                        width: if is_tgt_mult_focused { 1.5 } else { 0.0 },
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                });

                column![
                    crate::ui::inspector_panel::section_header("Multipliciteter (FDA §6)"),
                    row![
                        column![
                            text(format!("Kilde ({})", from_class))
                                .size(11)
                                .color(ThemeColors::SLATE_600),
                            src_container,
                        ]
                        .spacing(2)
                        .width(Length::FillPortion(1)),
                        column![
                            text(format!("Mål ({})", to_class))
                                .size(11)
                                .color(ThemeColors::SLATE_600),
                            tgt_container,
                        ]
                        .spacing(2)
                        .width(Length::FillPortion(1)),
                    ]
                    .spacing(8),
                ]
                .spacing(4)
                .into()
            } else {
                Space::new().height(0).into()
            };

        let mut label_col = column![
            crate::ui::inspector_panel::section_header("Associationsnavn (valgfri)"),
            text_input("f.eks. omfatter, ejer...", edge.label().unwrap_or(""))
                .id("info_edge_label_input")
                .style(modern_input_style)
                .on_input(move |v| Message::InfoUpdateEdgeLabel(from_id, to_id, v))
                .padding(6)
                .width(Length::Fill),
        ]
        .spacing(4);

        if let Some(lbl) = edge.label() {
            if let Some(issue) = NamingLinter::check_association_label(lbl) {
                let mut msg = format!("⚠️ {}: {}", issue.rule, issue.message);
                if let Some(fix) = &issue.suggested_fix {
                    msg.push_str(&format!(" (Forslag: {})", fix));
                }
                label_col = label_col.push(
                    text(msg)
                        .size(10.0)
                        .color(ColorTokens::WARNING_AMBER.into_color()),
                );
            }
        }

        let is_del_focused = inspector_focused_button == Some(crate::ui::app::INFO_EDGE_BTN_DELETE);
        let actions = row![button(text("🗑️ Slet relation").size(11))
            .style(focused_button_style(danger_button_style, is_del_focused))
            .on_press(Message::DeleteClassRelation(from_id, to_id))
            .padding([4, 10]),]
        .align_y(Alignment::Center);

        let insp_col = column![
            header,
            nodes_info,
            kind_selector,
            directed_selector,
            multiplicity_selector,
            label_col,
            actions
        ]
        .spacing(12);

        crate::ui::inspector_panel::panel_container(insp_col.into())
    } else {
        crate::ui::inspector_panel::panel_container(
            text("Relation ikke fundet")
                .size(12)
                .color(ThemeColors::TEXT_MUTED)
                .into(),
        )
    }
}

pub fn multi_selection_view<'a>(count: usize) -> Element<'a, Message> {
    let header = crate::ui::inspector_panel::panel_header(
        crate::ui::inspector_panel::PROPERTIES_TITLE,
        Some((
            "Multimarkering",
            ThemeColors::FDA_BORROWED_BLUE_BG,
            ThemeColors::FDA_BORROWED_BLUE,
        )),
        None,
    );
    let multi_col = column![
        header,
        container(
            column![
                text(format!("{} informationsklasser valgt", count))
                    .size(13)
                    .color(ThemeColors::TEXT_DARK),
                text("Brug musen til at parallelforskyde alle valgte klasser på lærredet.")
                    .size(11)
                    .color(ThemeColors::TEXT_MUTED),
            ]
            .spacing(6)
        )
        .style(card_container_style)
        .padding(10)
        .width(Length::Fill),
    ]
    .spacing(12);

    crate::ui::inspector_panel::panel_container(multi_col.into())
}
