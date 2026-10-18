use crate::features::concept_model::{ConceptGraph, NodeId, RelationKind};
use crate::features::concepts::ui::state::{
    ConceptEditorState, ConceptFormField, BTN_CANCEL, BTN_SAVE,
};
use crate::features::concepts::Concept;
use crate::ui::app::Message;
use crate::ui::theme::{
    card_container_style, danger_button_style, focused_button_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, checkbox, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
pub fn view_inspector<'a>(
    concepts: &'a [Concept],
    concept_graph: &'a ConceptGraph,
    selected_node_id: Option<NodeId>,
    selected_node_ids: &'a HashSet<NodeId>,
    selected_edge: Option<(NodeId, NodeId)>,
    is_inline_editing: bool,
    editor_state: Option<&'a ConceptEditorState>,
    inspector_focused_button: Option<usize>,
) -> Element<'a, Message> {
    if let Some((from_id, to_id)) = selected_edge {
        if let Some(edge) = concept_graph.find_edge(from_id, to_id) {
            let from_node = concept_graph.find_node(from_id);
            let to_node = concept_graph.find_node(to_id);
            let from_name = from_node.map(|n| n.label()).unwrap_or("Kilde");
            let to_name = to_node.map(|n| n.label()).unwrap_or("Mål");

            let header = crate::ui::inspector_panel::panel_header(
                crate::ui::inspector_panel::PROPERTIES_TITLE,
                Some(("Relation", ThemeColors::SLATE_100, ThemeColors::SLATE_300)),
                Some(Message::GraphEdgeSelected(None)),
            );

            let is_reverse_focused =
                inspector_focused_button == Some(crate::ui::app::CONCEPT_EDGE_BTN_REVERSE);
            let nodes_info = column![
                crate::ui::inspector_panel::section_header("Forbindelse"),
                row![
                    text(format!("{} ➔ {}", from_name, to_name))
                        .size(13)
                        .color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("⇄ Vend").size(11))
                        .style(focused_button_style(
                            secondary_button_style,
                            is_reverse_focused,
                        ))
                        .on_press(Message::GraphReverseEdge(from_id, to_id))
                        .padding([2, 6]),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(4);

            let is_assoc_focused =
                inspector_focused_button == Some(crate::ui::app::CONCEPT_EDGE_BTN_KIND_ASSOC);
            let is_gen_focused =
                inspector_focused_button == Some(crate::ui::app::CONCEPT_EDGE_BTN_KIND_GEN);
            let mut buttons = vec![
                button(text("Association").size(11))
                    .style(focused_button_style(
                        if edge.kind() == RelationKind::Association {
                            primary_button_style
                        } else {
                            secondary_button_style
                        },
                        is_assoc_focused,
                    ))
                    .on_press(Message::GraphUpdateEdgeKind(
                        from_id,
                        to_id,
                        RelationKind::Association,
                    ))
                    .padding([4, 6])
                    .into(),
                button(text("Generalisering").size(11))
                    .style(focused_button_style(
                        if edge.kind() == RelationKind::Generalization {
                            primary_button_style
                        } else {
                            secondary_button_style
                        },
                        is_gen_focused,
                    ))
                    .on_press(Message::GraphUpdateEdgeKind(
                        from_id,
                        to_id,
                        RelationKind::Generalization,
                    ))
                    .padding([4, 6])
                    .into(),
            ];
            if edge.kind() == RelationKind::Composition {
                buttons.push(
                    button(text("Komposition (Udfases)").size(11))
                        .style(primary_button_style)
                        .padding([4, 6])
                        .into(),
                );
            }

            let kind_selector = column![
                crate::ui::inspector_panel::section_header(
                    "Relationstype (Begrebsmodel jf. Tabel A)"
                ),
                row(buttons).spacing(4),
            ]
            .spacing(4);

            let is_dir_focused = inspector_focused_button
                == Some(crate::ui::app::CONCEPT_EDGE_BTN_DIRECTED_CHECKBOX);
            let directed_selector: Element<'a, Message> = if edge.kind()
                == RelationKind::Association
            {
                let cb = checkbox(edge.is_directed())
                    .label("Halv pil (rettet)")
                    .size(14)
                    .on_toggle(move |val| Message::GraphToggleEdgeDirected(from_id, to_id, val));
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

            let label_input = column![
                crate::ui::inspector_panel::section_header("Associationsnavn (valgfri)"),
                text_input("f.eks. omfatter, ejer...", edge.label().unwrap_or(""))
                    .id("edge_label_input")
                    .style(modern_input_style)
                    .on_input(move |v| Message::GraphUpdateEdgeLabel(from_id, to_id, v))
                    .padding(6)
                    .width(Length::Fill),
            ]
            .spacing(4);

            let is_del_focused =
                inspector_focused_button == Some(crate::ui::app::CONCEPT_EDGE_BTN_DELETE);
            let actions = row![button(text("🗑️ Slet relation").size(11))
                .style(focused_button_style(danger_button_style, is_del_focused))
                .on_press(Message::GraphDeleteRelation(from_id, to_id))
                .padding([4, 10]),]
            .align_y(Alignment::Center);

            let insp_col = column![
                header,
                nodes_info,
                kind_selector,
                directed_selector,
                label_input,
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
    } else if selected_node_ids.len() > 1 {
        let count = selected_node_ids.len();
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
                    text(format!("{} begreber valgt", count))
                        .size(13)
                        .color(ThemeColors::TEXT_DARK),
                    text("Brug musen til at parallelforskyde alle valgte noder på lærredet.")
                        .size(11)
                        .color(ThemeColors::TEXT_MUTED),
                ]
                .spacing(6),
            )
            .style(card_container_style)
            .padding(10)
            .width(Length::Fill),
        ]
        .spacing(12);

        crate::ui::inspector_panel::panel_container(multi_col.into())
    } else if let Some(selected_id) = selected_node_id {
        if let Some(node) = concept_graph.find_node(selected_id) {
            match (is_inline_editing, editor_state) {
                (true, Some(editor)) => {
                    let header = crate::ui::inspector_panel::panel_header(
                        crate::ui::inspector_panel::PROPERTIES_TITLE,
                        Some((
                            "Hurtigredigering",
                            ThemeColors::FDA_BORROWED_BLUE_BG,
                            ThemeColors::FDA_BORROWED_BLUE,
                        )),
                        Some(Message::CancelConceptEdit),
                    );

                    let mut edit_col = column![
                        header,
                        text("Rediger nodens begreb direkte:")
                            .size(11)
                            .color(ThemeColors::TEXT_MUTED),
                    ]
                    .spacing(8);

                    if let Some(err) = &editor.validation_error {
                        edit_col = edit_col.push(
                            container(
                                text(format!("⚠️ {}", err))
                                    .size(11)
                                    .color(ThemeColors::ACCENT_RED),
                            )
                            .style(card_container_style)
                            .padding(6)
                            .width(Length::Fill),
                        );
                    }

                    edit_col = edit_col.push(
                        column![
                            crate::ui::inspector_panel::section_header("Foretrukken term *"),
                            text_input("Foretrukken term...", &editor.preferred_term)
                                .id("preferred_term_input")
                                .style(modern_input_style)
                                .on_input(|v| {
                                    Message::UpdateConceptField(ConceptFormField::PreferredTerm, v)
                                })
                                .on_submit(Message::SaveConcept)
                                .padding(6)
                                .width(Length::Fill),
                        ]
                        .spacing(3),
                    );

                    edit_col = edit_col.push(
                        column![
                            crate::ui::inspector_panel::section_header(
                                "Definition (Aristoteles' formel) *"
                            ),
                            text_input("Definition...", &editor.definition)
                                .id("inline_concept_definition")
                                .style(modern_input_style)
                                .on_input(|v| {
                                    Message::UpdateConceptField(ConceptFormField::Definition, v)
                                })
                                .on_submit(Message::SaveConcept)
                                .padding(6)
                                .width(Length::Fill),
                        ]
                        .spacing(3),
                    );

                    let is_cancel_focused = editor.focused_button() == Some(BTN_CANCEL);
                    let is_save_focused = editor.focused_button() == Some(BTN_SAVE);

                    let edit_actions = row![
                        button(text("Annuller").size(11))
                            .style(focused_button_style(
                                secondary_button_style,
                                is_cancel_focused,
                            ))
                            .on_press(Message::CancelConceptEdit)
                            .padding([5, 10]),
                        Space::new().width(Length::Fill),
                        button(text("Gem Begreb").size(11))
                            .style(focused_button_style(primary_button_style, is_save_focused))
                            .on_press(Message::SaveConcept)
                            .padding([5, 12]),
                    ]
                    .align_y(Alignment::Center);

                    edit_col = edit_col.push(edit_actions);

                    crate::ui::inspector_panel::panel_container(edit_col.spacing(10).into())
                }
                _ => {
                    let concept = concepts.iter().find(|c| c.id() == node.concept_id());
                    let title = node.label();
                    let is_local = node.is_local();

                    let (badge_text, badge_bg, badge_border) = if is_local {
                        (
                            "Lokalt begreb",
                            ThemeColors::FDA_SAND,
                            ThemeColors::FDA_SAND_BORDER,
                        )
                    } else {
                        (
                            "Indlånt begreb",
                            ThemeColors::FDA_BORROWED_BLUE_BG,
                            ThemeColors::FDA_BORROWED_BLUE,
                        )
                    };

                    let header = crate::ui::inspector_panel::panel_header(
                        crate::ui::inspector_panel::PROPERTIES_TITLE,
                        Some((badge_text, badge_bg, badge_border)),
                        Some(Message::GraphNodeSelected(None)),
                    );

                    let mut insp =
                        column![header, text(title).size(16).color(ThemeColors::SLATE_900),]
                            .spacing(8);

                    if let Some(c) = concept {
                        insp = insp.push(
                            container(
                                column![
                                    crate::ui::inspector_panel::section_header("Definition"),
                                    text(if c.definition().is_empty() {
                                        "Ingen definition angivet."
                                    } else {
                                        c.definition()
                                    })
                                    .size(12)
                                    .color(ThemeColors::SLATE_800),
                                ]
                                .spacing(4),
                            )
                            .style(card_container_style)
                            .padding(8)
                            .width(Length::Fill),
                        );

                        if let Some(en_term) = c.english_preferred_term() {
                            let mut en_col = column![
                                crate::ui::inspector_panel::section_header("Engelsk term (DA+EN)"),
                                row![
                                    text("🇬🇧").size(11),
                                    Space::new().width(4),
                                    text(en_term).size(12).color(ThemeColors::SLATE_900),
                                ]
                                .align_y(Alignment::Center),
                            ]
                            .spacing(4);

                            if let Some(en_def) = c.english_definition() {
                                en_col = en_col
                                    .push(text(en_def).size(11).color(ThemeColors::SLATE_600));
                            }

                            insp = insp.push(
                                container(en_col)
                                    .style(card_container_style)
                                    .padding(8)
                                    .width(Length::Fill),
                            );
                        }

                        insp = insp.push(
                            row![
                                button(text("✏️ Hurtigrediger").size(11))
                                    .style(secondary_button_style)
                                    .on_press(Message::EditConcept(c.id()))
                                    .padding([4, 8]),
                                Space::new().width(6),
                                button(text("Fjern fra diagram").size(11))
                                    .style(danger_button_style)
                                    .on_press(Message::RemoveConceptFromDiagram(selected_id))
                                    .padding([4, 8]),
                            ]
                            .align_y(Alignment::Center),
                        );
                    }

                    // Tilknyttede relationer
                    let connected_edges: Vec<_> = concept_graph
                        .edges()
                        .iter()
                        .filter(|e| e.from() == selected_id || e.to() == selected_id)
                        .collect();

                    if !connected_edges.is_empty() {
                        insp = insp.push(Space::new().height(4));
                        insp = insp.push(crate::ui::inspector_panel::section_header(
                            "Tilknyttede relationer:",
                        ));

                        for edge in connected_edges {
                            let from_node = concept_graph.find_node(edge.from());
                            let to_node = concept_graph.find_node(edge.to());
                            let from_name = from_node.map(|n| n.label()).unwrap_or("?");
                            let to_name = to_node.map(|n| n.label()).unwrap_or("?");
                            let desc = match edge.kind() {
                                RelationKind::Generalization => {
                                    format!("{} ⮞ {}", from_name, to_name)
                                }
                                RelationKind::Association => {
                                    if let Some(lbl) = edge.label() {
                                        format!("{} ──({})── {}", from_name, lbl, to_name)
                                    } else {
                                        format!("{} ── {}", from_name, to_name)
                                    }
                                }
                                RelationKind::Composition => {
                                    format!("{} ◆── {}", from_name, to_name)
                                }
                                RelationKind::Dependency => {
                                    format!("{} ⤏ {}", from_name, to_name)
                                }
                            };

                            let edge_from = edge.from();
                            let edge_to = edge.to();
                            let edge_row = row![
                                text(desc)
                                    .size(11)
                                    .color(ThemeColors::SLATE_800)
                                    .width(Length::Fill),
                                button(text("🗑️").size(11))
                                    .style(danger_button_style)
                                    .on_press(Message::GraphDeleteRelation(edge_from, edge_to))
                                    .padding([2, 5]),
                            ]
                            .spacing(4)
                            .align_y(Alignment::Center);

                            insp = insp.push(edge_row);
                        }
                    }

                    crate::ui::inspector_panel::panel_container(insp.spacing(8).into())
                }
            }
        } else {
            crate::ui::inspector_panel::panel_container(
                text("Ingen node valgt")
                    .size(12)
                    .color(ThemeColors::TEXT_MUTED)
                    .into(),
            )
        }
    } else {
        crate::ui::inspector_panel::guidance_panel(
            "Begrebsmodel",
            &[
                (
                    "Tilføj begreber til diagram",
                    "Klik [+] ud for et begreb i venstre palet for at placere det på canvas.",
                ),
                (
                    "Flyt og placer",
                    "Træk noder på canvas med musen for at organisere diagrammet overskueligt.",
                ),
                (
                    "Opret relation",
                    "Klik '+ Opret Relation' for at forbinde begreber med association, generalisering eller komposition.",
                ),
                (
                    "FDA Farvekoder",
                    "Sand (#FEFAF7) = Eget lokalt begreb. Blå (#87CDEB) = Indlånt/genbrugt begreb.",
                ),
                (
                    "Fjern fra diagram",
                    "'Fjern fra diagram' fjerner kun kassen på canvas – begrebet bevares i begrebslisten.",
                ),
            ],
        )
    }
}
