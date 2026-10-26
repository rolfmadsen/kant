//! Information Model Studio View.
//!
//! Orchestrates the left repository browser, center UML diagram canvas,
//! and right context inspector using modular feature slices.

use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::{NodeId, RelationKind};
use crate::features::concepts::Concept;
pub use crate::features::information_model::ui::DataTypeOption;
use crate::features::information_model::ui::{association_editor, class_detail, class_list};
use crate::features::information_model::{ClassGraph, InformationModel, Multiplicity};
use crate::ui::app::{ConceptOption, Message, NodeOption, RelationDialogState};
use crate::ui::diagram_canvas::{
    render_uml_class_node, render_uml_datatype_node, render_uml_enumeration_node, CanvasViewport,
    DiagramCanvas,
};
use crate::ui::theme::{
    card_container_style, modern_input_style, primary_button_style, secondary_button_style,
    ThemeColors,
};
use iced::widget::{button, column, container, pick_list, row, text, text_input, tooltip, Space};
use iced::{Alignment, Element, Length};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    info_model: &'a InformationModel,
    class_graph: &'a ClassGraph,
    concepts: &'a [Concept],
    selected_class_id: Option<Uuid>,
    selected_enum_id: Option<Uuid>,
    selected_node_id: Option<NodeId>,
    selected_node_ids: &'a std::collections::HashSet<NodeId>,
    selected_edge: Option<(NodeId, NodeId)>,
    search_query: &'a str,
    palette_active_index: usize,
    new_enum_value_input: &'a str,
    viewport: CanvasViewport,
    is_space_pressed: bool,
    relation_dialog: Option<&'a RelationDialogState>,
    show_left_sidebar: bool,
    is_palette_dragging: bool,
    inspector_focused_button: Option<usize>,
    is_concept_picker_open: bool,
    concept_picker_search: &'a str,
    concept_picker_index: usize,
) -> Element<'a, Message> {
    let existing_class_names: std::collections::HashSet<String> = info_model
        .classes()
        .iter()
        .map(|c| c.name().trim().to_lowercase())
        .collect();

    let concept_options: Vec<ConceptOption> = concepts
        .iter()
        .filter(|c| !existing_class_names.contains(&c.preferred_term().trim().to_lowercase()))
        .map(|c| ConceptOption {
            id: c.id(),
            term: c.preferred_term().to_string(),
        })
        .collect();

    // 1. VENSTRE PALET (Repository Browser)
    let left_palette = class_list::view(
        info_model,
        class_graph,
        concept_options,
        selected_class_id,
        selected_enum_id,
        search_query,
        palette_active_index,
    );

    // 2. CENTER CANVAS (UML Klassediagram)
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
        button(text("+ Opret klasse").size(11))
            .style(primary_button_style)
            .on_press(Message::CreateInformationClassAtCenter)
            .padding([4, 10])
            .into(),
    );
    toolbar_elements.push(Space::new().width(4).into());
    toolbar_elements.push(
        button(text("+ Opret enumeration").size(11))
            .style(primary_button_style)
            .on_press(Message::CreateInformationEnumerationAtCenter)
            .padding([4, 10])
            .into(),
    );
    toolbar_elements.push(Space::new().width(4).into());
    toolbar_elements.push(
        button(text("+ Opret Relation").size(11))
            .style(primary_button_style)
            .on_press(Message::OpenInfoRelationDialog)
            .padding([4, 10])
            .into(),
    );
    toolbar_elements.push(Space::new().width(Length::Fill).into());
    toolbar_elements.push(
        text(format!(
            "{} elementer på diagram • {} relationer",
            class_graph.node_count(),
            class_graph.edge_count()
        ))
        .size(11)
        .color(ThemeColors::TEXT_MUTED)
        .into(),
    );

    let toolbar = row(toolbar_elements).spacing(2).align_y(Alignment::Center);

    let canvas_widget = iced::widget::canvas(
        DiagramCanvas::new(
            class_graph.nodes(),
            class_graph.edges(),
            selected_node_id,
            viewport,
            is_space_pressed,
            |frame, node, is_selected, vp| {
                if let Some(c) = info_model.get_class(node.class_id()) {
                    let class_name = if c.name().trim().is_empty() {
                        "NyKlasse"
                    } else {
                        c.name()
                    };
                    let is_borrowed = !c.is_local();
                    let is_abstract = c.is_abstract();
                    let attributes: Vec<(String, String, String, bool)> = c
                        .attributes()
                        .iter()
                        .map(|a| {
                            let name = if a.name().trim().is_empty() {
                                "nyAttribut"
                            } else {
                                a.name()
                            };
                            (
                                name.to_string(),
                                a.data_type().display_name(info_model),
                                a.multiplicity().to_string(),
                                !a.concept_ids().is_empty(),
                            )
                        })
                        .collect();
                    render_uml_class_node(
                        frame,
                        node,
                        class_name,
                        &attributes,
                        is_borrowed,
                        is_abstract,
                        is_selected,
                        vp,
                    );
                } else if let Some(e) = info_model.get_enumeration(node.class_id()) {
                    let enum_name = if e.name().trim().is_empty() {
                        "NyEnumeration"
                    } else {
                        e.name()
                    };
                    render_uml_enumeration_node(
                        frame,
                        node,
                        enum_name,
                        e.values(),
                        is_selected,
                        vp,
                    );
                } else if let Some(st) = info_model.get_structured_type(node.class_id()) {
                    let st_name = if st.name().trim().is_empty() {
                        "NyDatatype"
                    } else {
                        st.name()
                    };
                    let attributes: Vec<(String, String, String, bool)> = st
                        .attributes()
                        .iter()
                        .map(|a| {
                            let name = if a.name().trim().is_empty() {
                                "nyAttribut"
                            } else {
                                a.name()
                            };
                            (
                                name.to_string(),
                                a.data_type().display_name(info_model),
                                a.multiplicity().to_string(),
                                !a.concept_ids().is_empty(),
                            )
                        })
                        .collect();
                    render_uml_datatype_node(frame, node, st_name, &attributes, is_selected, vp);
                }
            },
            Message::SelectInfoGraphNode,
            Message::UpdateClassNodePosition,
            Message::CreateInformationClassAt,
            Message::InfoNodeDoubleClicked,
            Message::InfoCanvasViewportChanged,
        )
        .selected_node_ids(selected_node_ids.iter().copied())
        .on_selection_changed(Message::SelectInfoGraphNodes)
        .on_nodes_moved(Message::UpdateClassNodesPositions)
        .selected_edge(selected_edge)
        .on_edge_selected(Message::InfoEdgeSelected)
        .on_edge_created(Message::InfoEdgeCreated)
        .is_palette_dragging(is_palette_dragging)
        .on_canvas_drop(|x, y| Message::CanvasDropAt { x, y })
        .on_drag_released(|| Message::CanvasNodesDragFinished),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let center_content: Element<'a, Message> = if let Some(dialog) = relation_dialog {
        let node_options: Vec<NodeOption> = class_graph
            .nodes()
            .iter()
            .map(|n| {
                let name = info_model
                    .get_class(n.class_id())
                    .map(|c| c.name().to_string())
                    .unwrap_or_else(|| "Klasse".to_string());
                NodeOption {
                    id: n.id(),
                    label: name,
                }
            })
            .collect();

        let mut dialog_content = column![
            row![
                text("Opret Relation mellem Klasser")
                    .size(14)
                    .color(ThemeColors::PRIMARY),
                Space::new().width(Length::Fill),
                button(text("✕").size(12))
                    .style(secondary_button_style)
                    .on_press(Message::CloseInfoRelationDialog)
                    .padding([2, 6]),
            ]
            .align_y(Alignment::Center),
            row![
                column![
                    text("Fra klasse:").size(11).color(ThemeColors::SLATE_600),
                    pick_list(
                        node_options.clone(),
                        dialog.from_node.clone(),
                        Message::InfoRelationFromChanged,
                    )
                    .padding(5)
                    .width(Length::Fixed(180.0)),
                ]
                .spacing(4),
                column![
                    text("Til klasse:").size(11).color(ThemeColors::SLATE_600),
                    pick_list(
                        node_options,
                        dialog.to_node.clone(),
                        Message::InfoRelationToChanged,
                    )
                    .padding(5)
                    .width(Length::Fixed(180.0)),
                ]
                .spacing(4),
            ]
            .spacing(10),
            row![
                column![
                    text("Type:").size(11).color(ThemeColors::SLATE_600),
                    pick_list(
                        RelationKind::ALL,
                        Some(dialog.kind),
                        Message::InfoRelationKindChanged,
                    )
                    .padding(5)
                    .width(Length::Fixed(180.0)),
                ]
                .spacing(4),
                column![
                    text("Label (f.eks. rolle):")
                        .size(11)
                        .color(ThemeColors::SLATE_600),
                    text_input("Valgfri rolle...", &dialog.label)
                        .style(modern_input_style)
                        .on_input(Message::InfoRelationLabelChanged)
                        .padding(5)
                        .width(Length::Fixed(180.0)),
                ]
                .spacing(4),
            ]
            .spacing(10),
        ]
        .spacing(8);

        if dialog.kind != RelationKind::Generalization {
            dialog_content = dialog_content.push(
                row![
                    column![
                        text("Kildemultiplicitet:")
                            .size(11)
                            .color(ThemeColors::SLATE_600),
                        pick_list(Multiplicity::PRESETS, dialog.source_multiplicity, |m| {
                            Message::InfoRelationSourceMultiplicityChanged(Some(m))
                        },)
                        .placeholder("Vælg...")
                        .padding(5)
                        .width(Length::Fixed(180.0)),
                    ]
                    .spacing(4),
                    column![
                        text("Målmultiplicitet:")
                            .size(11)
                            .color(ThemeColors::SLATE_600),
                        pick_list(Multiplicity::PRESETS, dialog.target_multiplicity, |m| {
                            Message::InfoRelationTargetMultiplicityChanged(Some(m))
                        },)
                        .placeholder("Vælg...")
                        .padding(5)
                        .width(Length::Fixed(180.0)),
                    ]
                    .spacing(4),
                ]
                .spacing(10),
            );
        }

        if let Some(err) = &dialog.error {
            dialog_content = dialog_content.push(text(err).size(11).color(ThemeColors::ACCENT_RED));
        }

        dialog_content = dialog_content.push(
            row![
                Space::new().width(Length::Fill),
                button(text("Annuller").size(12))
                    .style(secondary_button_style)
                    .on_press(Message::CloseInfoRelationDialog)
                    .padding([4, 10]),
                button(text("Opret Relation").size(12))
                    .style(primary_button_style)
                    .on_press(Message::InfoCreateRelation)
                    .padding([4, 12]),
            ]
            .spacing(8),
        );

        let dialog_card = container(dialog_content)
            .style(card_container_style)
            .padding(14)
            .width(Length::Fixed(400.0));

        column![
            toolbar,
            dialog_card,
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
        .height(Length::Fill)
        .into()
    };

    let center_area = container(center_content)
        .width(Length::Fill)
        .height(Length::Fill);

    // 3. HØJRE INSPECTOR (Context Panel)
    let right_inspector: Element<'a, Message> = if let Some((from_id, to_id)) = selected_edge {
        association_editor::view(
            from_id,
            to_id,
            info_model,
            class_graph,
            inspector_focused_button,
        )
    } else if selected_node_ids.len() > 1 {
        association_editor::multi_selection_view(selected_node_ids.len())
    } else if let Some(class) = selected_class_id.and_then(|id| info_model.get_class(id)) {
        class_detail::view(
            class,
            info_model,
            class_graph,
            concepts,
            inspector_focused_button,
            is_concept_picker_open,
            concept_picker_search,
            concept_picker_index,
        )
    } else if let Some(e) = selected_enum_id
        .or(selected_class_id)
        .and_then(|id| info_model.get_enumeration(id))
    {
        class_detail::enum_detail_view(
            e,
            class_graph,
            new_enum_value_input,
            inspector_focused_button,
        )
    } else {
        let header = crate::ui::inspector_panel::panel_header(
            crate::ui::inspector_panel::PROPERTIES_TITLE,
            None,
            None,
        );
        let empty_hint = container(
            column![
                text("Intet element valgt")
                    .size(13)
                    .color(ThemeColors::TEXT_DARK),
                text("Vælg en klasse, en enumeration eller en relation for at inspicere og redigere detaljer.")
                    .size(11)
                    .color(ThemeColors::TEXT_MUTED),
            ]
            .spacing(4),
        )
        .padding(12);

        crate::ui::inspector_panel::panel_container(column![header, empty_hint].spacing(10).into())
    };

    let mut main_row = row![].spacing(8);
    if show_left_sidebar {
        main_row = main_row.push(left_palette);
    }
    main_row = main_row.push(center_area);
    main_row = main_row.push(right_inspector);

    container(main_row)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
}
