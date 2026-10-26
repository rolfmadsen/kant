//! Repository browser and entity palette for FDA Information Model.

use crate::design_system::tokens::ColorTokens;
use crate::features::information_model::diagram::ClassGraph;
use crate::features::information_model::models::{InformationClass, InformationModel};
use crate::ui::app::{ConceptOption, Message, PaletteDragItem};
use crate::ui::theme::{
    card_container_style, danger_button_style, list_item_button, list_item_container_style,
    modern_input_style, modern_scrollable_style, pill_container_style, secondary_button_style,
    ThemeColors,
};
use iced::widget::{
    button, column, container, mouse_area, pick_list, row, scrollable, text, text_input, tooltip,
    Space,
};
use iced::{mouse, Alignment, Element, Length};
use uuid::Uuid;

pub const PALETTE_WIDTH: f32 = 268.0;
pub const SCROLLBAR_WIDTH: f32 = 4.0;

pub fn view<'a>(
    info_model: &'a InformationModel,
    class_graph: &'a ClassGraph,
    concept_options: Vec<ConceptOption>,
    selected_class_id: Option<Uuid>,
    selected_enum_id: Option<Uuid>,
    search_query: &'a str,
    active_index: usize,
) -> Element<'a, Message> {
    let class_count = info_model.classes().len();
    let search_filter = search_query.trim().to_lowercase();
    let filtered_classes: Vec<&InformationClass> = info_model
        .classes()
        .iter()
        .filter(|c| {
            if search_filter.is_empty() {
                true
            } else {
                c.name().to_lowercase().contains(&search_filter)
                    || c.description()
                        .map(|d| d.to_lowercase().contains(&search_filter))
                        .unwrap_or(false)
            }
        })
        .collect();

    let palette_header = column![
        row![
            text("Klasser").size(15).color(ThemeColors::SLATE_900),
            container(
                text(format!("{}", class_count))
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
        pick_list(
            concept_options,
            None::<ConceptOption>,
            Message::CreateInformationClassFromConcept,
        )
        .placeholder("+ Fra begreb...")
        .padding(4)
        .width(Length::Fill),
        text_input("🔍 Søg klasser...", search_query)
            .id("info_class_search_input")
            .style(modern_input_style)
            .on_input(Message::InformationClassSearchChanged)
            .on_submit(Message::InfoPaletteSubmit)
            .padding(5),
    ]
    .spacing(8);

    let mut class_items = column![].spacing(4);
    for (idx, class) in filtered_classes.into_iter().enumerate() {
        let is_selected = selected_class_id == Some(class.id());
        let class_id = class.id();
        let is_on_canvas = class_graph.is_class_on_diagram(class_id);
        let is_combobox_active = idx == active_index;
        let attr_count = class.attributes().len();

        let action_controls: Element<'a, Message> = if is_on_canvas {
            container(text("✓").size(11).color(ThemeColors::ACCENT_GREEN))
                .style(pill_container_style)
                .padding([1, 5])
                .into()
        } else {
            row![
                button(text("+").size(11).color(ThemeColors::PRIMARY))
                    .style(secondary_button_style)
                    .on_press(Message::AddClassToDiagram(class_id))
                    .padding([2, 5]),
                button(text("🗑️").size(10))
                    .style(danger_button_style)
                    .on_press(Message::DeleteInformationClass(class_id))
                    .padding([2, 4]),
            ]
            .spacing(3)
            .align_y(Alignment::Center)
            .into()
        };

        let display_name = if class.name().trim().is_empty() {
            "NyKlasse"
        } else {
            class.name()
        };

        let is_abstract = class.is_abstract();
        let is_borrowed = !class.is_local();

        let mut name_text =
            text(display_name)
                .size(13)
                .color(if is_selected || is_combobox_active {
                    ThemeColors::PRIMARY
                } else {
                    ThemeColors::SLATE_900
                });
        if is_abstract {
            name_text = name_text.font(iced::Font {
                style: iced::font::Style::Italic,
                ..Default::default()
            });
        }

        let mut title_row = row![name_text].spacing(4).align_y(Alignment::Center);
        if is_borrowed {
            title_row = title_row.push(
                container(
                    text("Indlånt")
                        .size(9)
                        .color(ColorTokens::PRIMARY_DARK_TEXT.into_color()),
                )
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(ThemeColors::FDA_BORROWED_BLUE)),
                    border: iced::Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .padding([1, 4]),
            );
        }

        let item_content = column![
            title_row,
            text(format!("{} attr", attr_count))
                .size(10)
                .color(ThemeColors::TEXT_MUTED),
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
            .on_press(Message::StartPaletteDrag(PaletteDragItem::Class(class_id)))
            .into()
        } else {
            button(item_content)
                .style(list_item_button(is_selected || is_combobox_active))
                .on_press(Message::SelectInformationClass(Some(class_id)))
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

        class_items = class_items.push(item_row);
    }

    // Enumerationer i paletten
    let enum_count = info_model.enumerations().len();
    let filtered_enums: Vec<_> = info_model
        .enumerations()
        .iter()
        .filter(|e| {
            if search_filter.is_empty() {
                true
            } else {
                e.name().to_lowercase().contains(&search_filter)
                    || e.values()
                        .iter()
                        .any(|v| v.to_lowercase().contains(&search_filter))
            }
        })
        .collect();

    let enum_header = row![
        text("Enumerationer").size(14).color(ThemeColors::SLATE_900),
        container(
            text(format!("{}", enum_count))
                .size(11)
                .color(ThemeColors::ACCENT_GREEN)
        )
        .style(pill_container_style)
        .padding([2, 7]),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let mut enum_items = column![].spacing(4);
    for e in filtered_enums {
        let is_selected = selected_enum_id == Some(e.id());
        let enum_id = e.id();
        let is_on_canvas = class_graph.is_class_on_diagram(enum_id);
        let val_count = e.values().len();

        let action_controls: Element<'a, Message> = if is_on_canvas {
            container(text("✓").size(11).color(ThemeColors::ACCENT_GREEN))
                .style(pill_container_style)
                .padding([1, 5])
                .into()
        } else {
            row![
                button(text("+").size(11).color(ThemeColors::ACCENT_GREEN))
                    .style(secondary_button_style)
                    .on_press(Message::AddEnumerationToDiagram(enum_id))
                    .padding([2, 5]),
                button(text("🗑️").size(10))
                    .style(danger_button_style)
                    .on_press(Message::DeleteInformationEnumeration(enum_id))
                    .padding([2, 4]),
            ]
            .spacing(3)
            .align_y(Alignment::Center)
            .into()
        };

        let display_name = if e.name().trim().is_empty() {
            "NyEnumeration"
        } else {
            e.name()
        };

        let title_row = row![
            text(display_name).size(13).color(if is_selected {
                ThemeColors::ACCENT_GREEN
            } else {
                ThemeColors::SLATE_900
            }),
            container(
                text("Enum")
                    .size(9)
                    .color(ColorTokens::SUCCESS_DARK_TEXT.into_color()),
            )
            .style(|_theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(ThemeColors::FDA_ENUM_GREEN)),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding([1, 4]),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let item_content = column![
            title_row,
            text(format!("{} værdier", val_count))
                .size(10)
                .color(ThemeColors::TEXT_MUTED),
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
            .on_press(Message::StartPaletteDrag(PaletteDragItem::Enumeration(
                enum_id,
            )))
            .into()
        } else {
            button(item_content)
                .style(list_item_button(is_selected))
                .on_press(Message::SelectInformationEnumeration(Some(enum_id)))
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

        enum_items = enum_items.push(item_row);
    }

    container(
        column![
            palette_header,
            scrollable(class_items)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new()
                        .width(SCROLLBAR_WIDTH)
                        .scroller_width(SCROLLBAR_WIDTH)
                        .margin(1.0),
                ))
                .style(modern_scrollable_style)
                .height(Length::FillPortion(1)),
            enum_header,
            scrollable(enum_items)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::new()
                        .width(SCROLLBAR_WIDTH)
                        .scroller_width(SCROLLBAR_WIDTH)
                        .margin(1.0),
                ))
                .style(modern_scrollable_style)
                .height(Length::FillPortion(1)),
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
