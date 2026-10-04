use crate::features::concept_model::{ConceptGraph, NodeId, RelationKind};
use crate::features::concepts::Concept;
use crate::ui::app::{Message, NodeOption, PaletteDragItem, RelationDialogState};
use crate::ui::concept_editor::{ConceptEditorState, ConceptFormField, BTN_CANCEL, BTN_SAVE};
use crate::ui::diagram_canvas::{render_concept_node, CanvasViewport, DiagramCanvas};
use crate::ui::theme::{
    card_container_style, danger_button_style, focused_button_style, list_item_button,
    list_item_container_style, modern_input_style, pill_container_style, primary_button_style,
    secondary_button_style, ThemeColors,
};
use iced::widget::{
    button, checkbox, column, container, mouse_area, pick_list, row, scrollable, text, text_input,
    tooltip, Space,
};
use iced::{mouse, Alignment, Color, Element, Length};

#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    concepts: &'a [Concept],
    concept_graph: &'a ConceptGraph,
    selected_node_id: Option<NodeId>,
    selected_node_ids: &'a std::collections::HashSet<NodeId>,
    selected_edge: Option<(NodeId, NodeId)>,
    search_query: &'a str,
    viewport: CanvasViewport,
    is_space_pressed: bool,
    is_inline_editing: bool,
    editor_state: Option<&'a ConceptEditorState>,
    relation_dialog: Option<&'a RelationDialogState>,
    show_left_sidebar: bool,
    is_palette_dragging: bool,
) -> Element<'a, Message> {
    // ==========================================
    // 1. VENSTRE PALET (Repository Browser ~240px)
    // ==========================================
    let concept_count = concepts.len();
    let search_filter = search_query.trim().to_lowercase();
    let mut filtered_concepts: Vec<&Concept> = concepts
        .iter()
        .filter(|c| {
            if search_filter.is_empty() {
                true
            } else {
                c.preferred_term().to_lowercase().contains(&search_filter)
                    || c.definition().to_lowercase().contains(&search_filter)
            }
        })
        .collect();
    filtered_concepts.sort_by(|a, b| {
        a.preferred_term()
            .to_lowercase()
            .cmp(&b.preferred_term().to_lowercase())
    });

    let palette_header = column![
        row![
            text("Begreber").size(15).color(ThemeColors::SLATE_900),
            container(
                text(format!("{}", concept_count))
                    .size(11)
                    .color(ThemeColors::PRIMARY)
            )
            .style(pill_container_style)
            .padding([2, 7]),
            Space::new().width(Length::Fill),
            tooltip(
                button(text("◨").size(13).color(ThemeColors::SLATE_500))
                    .style(secondary_button_style)
                    .on_press(Message::ToggleLeftSidebar)
                    .padding([2, 6]),
                text("Skjul palet (Ctrl+B)").size(11),
                tooltip::Position::Bottom,
            ),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
        text_input("🔍 Søg begreber...", search_query)
            .style(modern_input_style)
            .on_input(Message::ConceptModelSearchChanged)
            .padding(5),
    ]
    .spacing(8);

    let mut concept_items = column![].spacing(4);
    for concept in filtered_concepts {
        let concept_id = concept.id();
        let is_on_canvas = concept_graph.is_concept_on_diagram(concept_id);
        let node_id = concept_graph
            .find_node_by_concept(concept_id)
            .map(|n| n.id());
        let is_selected = selected_node_id.is_some() && selected_node_id == node_id;

        let action_controls: Element<'a, Message> = if is_on_canvas {
            container(text("✓").size(11).color(ThemeColors::ACCENT_GREEN))
                .style(pill_container_style)
                .padding([1, 5])
                .into()
        } else {
            row![
                button(text("+").size(11).color(ThemeColors::PRIMARY))
                    .style(secondary_button_style)
                    .on_press(Message::AddConceptToDiagram(concept_id))
                    .padding([2, 5]),
                button(text("🗑️").size(10))
                    .style(danger_button_style)
                    .on_press(Message::DeleteConcept(concept_id))
                    .padding([2, 4]),
            ]
            .spacing(3)
            .align_y(Alignment::Center)
            .into()
        };

        let is_local = concept.belongs_to_domain().is_local();
        let subtitle = if is_local { "Lokalt" } else { "Indlånt" };

        let item_content = column![
            text(concept.preferred_term())
                .size(13)
                .color(if is_selected {
                    ThemeColors::PRIMARY
                } else {
                    ThemeColors::SLATE_900
                }),
            text(subtitle).size(10).color(ThemeColors::TEXT_MUTED),
        ]
        .width(Length::Fill);

        let item_widget: Element<'a, Message> = if !is_on_canvas {
            mouse_area(
                container(item_content)
                    .style(list_item_container_style(is_selected))
                    .width(Length::Fill)
                    .padding([4, 6]),
            )
            .interaction(mouse::Interaction::Grab)
            .on_press(Message::StartPaletteDrag(PaletteDragItem::Concept(
                concept_id,
            )))
            .into()
        } else {
            button(item_content)
                .style(list_item_button(is_selected))
                .on_press(if let Some(nid) = node_id {
                    Message::GraphNodeSelected(Some(nid))
                } else {
                    Message::AddConceptToDiagram(concept_id)
                })
                .width(Length::Fill)
                .padding([4, 6])
                .into()
        };

        let item_row = container(
            row![item_widget, action_controls]
                .align_y(Alignment::Center)
                .spacing(4),
        )
        .width(Length::Fill);

        concept_items = concept_items.push(item_row);
    }

    let left_palette = container(
        column![
            palette_header,
            scrollable(concept_items).height(Length::Fill),
        ]
        .spacing(10)
        .height(Length::Fill),
    )
    .style(card_container_style)
    .padding(12)
    .width(Length::Fixed(240.0))
    .height(Length::Fill);

    // ==========================================
    // 2. CENTER CANVAS (Begrebsmodel Graf)
    // ==========================================
    let mut toolbar_elements: Vec<Element<'a, Message>> = Vec::new();
    if !show_left_sidebar {
        toolbar_elements.push(
            tooltip(
                button(
                    row![
                        text("◨").size(13).color(ThemeColors::PRIMARY),
                        Space::new().width(4),
                        text("Palet").size(11).color(ThemeColors::SLATE_800),
                    ]
                    .align_y(Alignment::Center),
                )
                .style(secondary_button_style)
                .on_press(Message::ToggleLeftSidebar)
                .padding([4, 8]),
                text("Vis palet (Ctrl+B)").size(11),
                tooltip::Position::Bottom,
            )
            .into(),
        );
        toolbar_elements.push(Space::new().width(4).into());
    }

    toolbar_elements.push(
        button(text("+ Opret begreb").size(11))
            .style(primary_button_style)
            .on_press(Message::CreateConceptAtCenter)
            .padding([4, 10])
            .into(),
    );
    toolbar_elements.push(Space::new().width(4).into());
    toolbar_elements.push(
        button(text("+ Opret Relation").size(11))
            .style(primary_button_style)
            .on_press(Message::GraphOpenRelationDialog)
            .padding([4, 10])
            .into(),
    );
    toolbar_elements.push(Space::new().width(Length::Fill).into());
    toolbar_elements.push(
        text(format!(
            "{} begreber på diagram • {} relationer",
            concept_graph.node_count(),
            concept_graph.edge_count()
        ))
        .size(11)
        .color(ThemeColors::TEXT_MUTED)
        .into(),
    );

    let toolbar = row(toolbar_elements)
        .spacing(2)
        .align_y(Alignment::Center);

    let canvas_widget = iced::widget::canvas(
        DiagramCanvas::new(
            concept_graph.nodes(),
            concept_graph.edges(),
            selected_node_id,
            viewport,
            is_space_pressed,
            render_concept_node,
            Message::GraphNodeSelected,
            Message::GraphNodeMoved,
            Message::CanvasDoubleClicked,
            Message::GraphNodeDoubleClicked,
            Message::CanvasViewportChanged,
        )
        .selected_node_ids(selected_node_ids.iter().copied())
        .on_selection_changed(Message::GraphNodesSelected)
        .on_nodes_moved(Message::GraphNodesMoved)
        .selected_edge(selected_edge)
        .on_edge_selected(Message::GraphEdgeSelected)
        .on_edge_created(Message::GraphEdgeCreated)
        .is_palette_dragging(is_palette_dragging)
        .on_canvas_drop(|x, y| Message::CanvasDropAt { x, y })
        .on_drag_released(|| Message::CanvasNodesDragFinished),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    // Relations-dialog modal hvis aktiv
    let center_content: Element<'a, Message> = if let Some(dialog) = relation_dialog {
        let node_options: Vec<NodeOption> = concept_graph
            .nodes()
            .iter()
            .map(|n| NodeOption {
                id: n.id(),
                label: n.label().to_string(),
            })
            .collect();

        let mut dialog_content = column![
            row![
                text("Opret Relation mellem Begreber")
                    .size(14)
                    .color(ThemeColors::PRIMARY),
                Space::new().width(Length::Fill),
                button(text("✕").size(12))
                    .style(secondary_button_style)
                    .on_press(Message::GraphCloseRelationDialog)
                    .padding([2, 6]),
            ]
            .align_y(Alignment::Center),
            row![
                column![
                    text("Fra begreb:").size(11).color(ThemeColors::SLATE_600),
                    pick_list(
                        node_options.clone(),
                        dialog.from_node.clone(),
                        Message::GraphRelationFromChanged,
                    )
                    .placeholder("Vælg kilde...")
                    .padding(5)
                    .width(Length::Fixed(180.0)),
                ]
                .spacing(3),
                column![
                    text("Relationstype:")
                        .size(11)
                        .color(ThemeColors::SLATE_600),
                    pick_list(
                        RelationKind::CONCEPT_RELATIONS,
                        Some(dialog.kind),
                        Message::GraphRelationKindChanged,
                    )
                    .padding(5)
                    .width(Length::Fixed(180.0)),
                ]
                .spacing(3),
                column![
                    text("Til begreb:").size(11).color(ThemeColors::SLATE_600),
                    pick_list(
                        node_options,
                        dialog.to_node.clone(),
                        Message::GraphRelationToChanged,
                    )
                    .placeholder("Vælg mål...")
                    .padding(5)
                    .width(Length::Fixed(180.0)),
                ]
                .spacing(3),
            ]
            .spacing(10),
            column![
                text("Rolle / Navn (valgfri):")
                    .size(11)
                    .color(ThemeColors::SLATE_600),
                text_input("f.eks. ejer, omfatter, udsteder...", &dialog.label)
                    .style(modern_input_style)
                    .on_input(Message::GraphRelationLabelChanged)
                    .padding(5),
            ]
            .spacing(3),
            row![
                Space::new().width(Length::Fill),
                button(text("Annuller").size(12))
                    .style(secondary_button_style)
                    .on_press(Message::GraphCloseRelationDialog)
                    .padding([4, 10]),
                button(text("Opret Relation").size(12))
                    .style(primary_button_style)
                    .on_press(Message::GraphCreateRelation)
                    .padding([4, 12]),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10);

        if let Some(err) = &dialog.error {
            dialog_content = dialog_content.push(
                container(
                    text(format!("⚠️ {}", err))
                        .size(11)
                        .color(ThemeColors::ACCENT_RED),
                )
                .style(card_container_style)
                .padding(6),
            );
        }

        let dialog_card = container(dialog_content)
            .style(card_container_style)
            .padding(14)
            .width(Length::Fixed(590.0));

        column![
            toolbar,
            container(canvas_widget)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(
                        0.985, 0.988, 0.992
                    ))),
                    border: iced::Border {
                        color: ThemeColors::SLATE_200,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(Length::Fill),
            container(dialog_card).center_x(Length::Fill),
        ]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        column![
            toolbar,
            container(canvas_widget)
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(
                        0.985, 0.988, 0.992
                    ))),
                    border: iced::Border {
                        color: ThemeColors::SLATE_200,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(Length::Fill),
        ]
        .spacing(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    };

    // ==========================================
    // 3. HØJRE INSPECTOR (Context Panel ~290px)
    // ==========================================
    let right_inspector: Element<'a, Message> = if let Some((from_id, to_id)) = selected_edge {
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

            let nodes_info = column![
                crate::ui::inspector_panel::section_header("Forbindelse"),
                row![
                    text(format!("{} ➔ {}", from_name, to_name))
                        .size(13)
                        .color(ThemeColors::SLATE_900),
                    Space::new().width(Length::Fill),
                    button(text("⇄ Vend").size(11))
                        .style(secondary_button_style)
                        .on_press(Message::GraphReverseEdge(from_id, to_id))
                        .padding([2, 6]),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(4);

            let mut buttons = vec![
                button(text("Association").size(11))
                    .style(if edge.kind() == RelationKind::Association {
                        primary_button_style
                    } else {
                        secondary_button_style
                    })
                    .on_press(Message::GraphUpdateEdgeKind(
                        from_id,
                        to_id,
                        RelationKind::Association,
                    ))
                    .padding([4, 6])
                    .into(),
                button(text("Generalisering").size(11))
                    .style(if edge.kind() == RelationKind::Generalization {
                        primary_button_style
                    } else {
                        secondary_button_style
                    })
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

            let directed_selector: Element<'a, Message> =
                if edge.kind() == RelationKind::Association {
                    column![
                        crate::ui::inspector_panel::section_header("Retning / Navigabilitet"),
                        checkbox(edge.is_directed())
                            .label("Halv pil (rettet)")
                            .size(14)
                            .on_toggle(move |val| Message::GraphToggleEdgeDirected(
                                from_id, to_id, val
                            )),
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

            let actions = row![button(text("🗑️ Slet relation").size(11))
                .style(danger_button_style)
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
                .spacing(6)
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
                            .style(focused_button_style(secondary_button_style, is_cancel_focused))
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
    };

    let mut main_row = row![].spacing(12).width(Length::Fill).height(Length::Fill);
    if show_left_sidebar {
        main_row = main_row.push(left_palette);
    }
    main_row.push(center_content).push(right_inspector).into()
}
