use crate::features::concept_model::{ConceptGraph, NodeId};
use crate::features::concepts::Concept;
use crate::ui::app::{Message, PaletteDragItem};
use crate::ui::theme::{
    card_container_style, danger_button_style, list_item_button, list_item_container_style,
    modern_input_style, modern_scrollable_style, pill_container_style, secondary_button_style,
    ThemeColors,
};
use iced::widget::{
    button, column, container, mouse_area, row, scrollable, text, text_input, tooltip, Space,
};
use iced::{mouse, Alignment, Element, Length};

pub const PALETTE_WIDTH: f32 = 268.0;
pub const SCROLLBAR_WIDTH: f32 = 4.0;
pub const SEARCH_INPUT_PADDING: [f32; 2] = [7.0, 9.0];

pub fn view_palette<'a>(
    concepts: &'a [Concept],
    concept_graph: &'a ConceptGraph,
    selected_node_id: Option<NodeId>,
    search_query: &'a str,
    active_index: usize,
) -> Element<'a, Message> {
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
            .id("concept_model_search_input")
            .style(modern_input_style)
            .on_input(Message::ConceptModelSearchChanged)
            .on_submit(Message::ConceptPaletteSubmit)
            .padding(SEARCH_INPUT_PADDING),
    ]
    .spacing(8);

    let mut concept_items = column![].spacing(4);
    for (idx, concept) in filtered_concepts.into_iter().enumerate() {
        let concept_id = concept.id();
        let is_on_canvas = concept_graph.is_concept_on_diagram(concept_id);
        let node_id = concept_graph
            .find_node_by_concept(concept_id)
            .map(|n| n.id());
        let is_selected = selected_node_id.is_some() && selected_node_id == node_id;
        let is_combobox_active = idx == active_index;

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
                .color(if is_selected || is_combobox_active {
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
                    .style(list_item_container_style(is_selected || is_combobox_active))
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
                .style(list_item_button(is_selected || is_combobox_active))
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

    container(
        column![
            palette_header,
            scrollable(concept_items)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new()
                        .width(SCROLLBAR_WIDTH)
                        .scroller_width(SCROLLBAR_WIDTH)
                        .margin(1.0),
                ))
                .style(modern_scrollable_style)
                .height(Length::Fill),
        ]
        .spacing(10)
        .height(Length::Fill),
    )
    .style(card_container_style)
    .padding(12)
    .width(Length::Fixed(PALETTE_WIDTH))
    .height(Length::Fill)
    .into()
}
