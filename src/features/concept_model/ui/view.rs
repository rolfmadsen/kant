use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::ui::inspector::view_inspector;
use crate::features::concept_model::ui::palette::view_palette;
use crate::features::concept_model::{ConceptGraph, NodeId, RelationKind};
use crate::features::concepts::ui::state::ConceptEditorState;
use crate::features::concepts::Concept;
use crate::ui::app::{Message, NodeOption, RelationDialogState};
use crate::ui::diagram_canvas::{render_concept_node, CanvasViewport, DiagramCanvas};
use crate::ui::theme::{
    card_container_style, modern_input_style, primary_button_style, secondary_button_style,
    ThemeColors,
};
use iced::widget::{button, column, container, pick_list, row, text, text_input, tooltip, Space};
use iced::{Alignment, Element, Length};
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    concepts: &'a [Concept],
    concept_graph: &'a ConceptGraph,
    selected_node_id: Option<NodeId>,
    selected_node_ids: &'a HashSet<NodeId>,
    selected_edge: Option<(NodeId, NodeId)>,
    search_query: &'a str,
    palette_active_index: usize,
    viewport: CanvasViewport,
    is_space_pressed: bool,
    is_inline_editing: bool,
    editor_state: Option<&'a ConceptEditorState>,
    relation_dialog: Option<&'a RelationDialogState>,
    show_left_sidebar: bool,
    is_palette_dragging: bool,
    inspector_focused_button: Option<usize>,
) -> Element<'a, Message> {
    // 1. Venstre palet
    let left_palette = view_palette(
        concepts,
        concept_graph,
        selected_node_id,
        search_query,
        palette_active_index,
    );

    // 2. Center canvas & toolbar
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

    let toolbar = row(toolbar_elements).spacing(2).align_y(Alignment::Center);

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
        .on_edge_bends_changed(|(from, to), bends| Message::GraphUpdateEdgeBends(from, to, bends))
        .is_palette_dragging(is_palette_dragging)
        .on_canvas_drop(|x, y| Message::CanvasDropAt { x, y })
        .on_drag_released(|| Message::CanvasNodesDragFinished),
    )
    .width(Length::Fill)
    .height(Length::Fill);

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
                    background: Some(iced::Background::Color(
                        ColorTokens::CANVAS_BACKGROUND.into_color()
                    )),
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
                    background: Some(iced::Background::Color(
                        ColorTokens::CANVAS_BACKGROUND.into_color()
                    )),
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

    // 3. Højre inspektør
    let right_inspector = view_inspector(
        concepts,
        concept_graph,
        selected_node_id,
        selected_node_ids,
        selected_edge,
        is_inline_editing,
        editor_state,
        inspector_focused_button,
    );

    let mut main_row = row![].spacing(12).width(Length::Fill).height(Length::Fill);
    if show_left_sidebar {
        main_row = main_row.push(left_palette);
    }
    main_row.push(center_content).push(right_inspector).into()
}
