//! Attribute list and editor for FDA Information Model classes.

use crate::design_system::tokens::ColorTokens;
use crate::features::concepts::Concept;
use crate::features::information_model::linter::NamingLinter;
use crate::features::information_model::models::{
    InformationClass, InformationDataType, InformationModel, Multiplicity, PrimitiveType,
};
use crate::features::information_model::ui::DataTypeOption;
use crate::ui::app::{AttributeConceptOption, Message};
use crate::ui::theme::{
    card_container_style, danger_button_style, focused_button_style, modern_input_style,
    pill_container_style, secondary_button_style, ThemeColors,
};
use iced::widget::{button, column, container, pick_list, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view<'a>(
    class: &'a InformationClass,
    info_model: &'a InformationModel,
    concepts: &'a [Concept],
    inspector_focused_button: Option<usize>,
) -> Element<'a, Message> {
    let class_id = class.id();
    let is_add_attr_focused =
        inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_BTN_ADD_ATTRIBUTE);

    let attr_header = row![
        crate::ui::inspector_panel::section_header("Attributter"),
        Space::new().width(Length::Fill),
        button(text("+ Tilføj").size(11))
            .style(focused_button_style(
                secondary_button_style,
                is_add_attr_focused,
            ))
            .on_press(Message::AddAttributeToClass(class_id))
            .padding([2, 6]),
    ]
    .align_y(Alignment::Center);

    let mut attr_list = column![].spacing(6);
    for (j, attr) in class.attributes().iter().enumerate() {
        let attr_id = attr.id();
        let is_delete_attr_focused =
            inspector_focused_button == Some(crate::ui::app::INFO_INSPECTOR_ATTR_DELETE_START + j);
        let name_val = attr.name().to_string();

        let mut data_type_options = Vec::new();
        for pt in PrimitiveType::ALL {
            data_type_options.push(DataTypeOption {
                label: pt.as_str().to_string(),
                data_type: InformationDataType::Primitive(*pt),
            });
        }
        for e in info_model.enumerations() {
            data_type_options.push(DataTypeOption {
                label: format!("{} (enum)", e.name()),
                data_type: InformationDataType::Enumeration {
                    enumeration_id: e.id(),
                },
            });
        }
        for st in info_model.structured_types() {
            data_type_options.push(DataTypeOption {
                label: format!("{} (type)", st.name()),
                data_type: InformationDataType::Structured {
                    structured_id: st.id(),
                },
            });
        }

        let current_dt = attr.data_type();
        let selected_dt_opt = data_type_options
            .iter()
            .find(|opt| opt.data_type == *current_dt)
            .cloned();

        let mult_val = attr.multiplicity();

        let linked_concept_id = attr.concept_ids().first().copied();
        let linked_concept =
            linked_concept_id.and_then(|cid| concepts.iter().find(|c| c.id() == cid));

        let mut attr_concept_options = vec![AttributeConceptOption {
            id: None,
            label: "— Intet begreb (teknisk felt) —".to_string(),
        }];
        for c in concepts {
            attr_concept_options.push(AttributeConceptOption {
                id: Some(c.id()),
                label: c.preferred_term().to_string(),
            });
        }

        let selected_concept_opt = match linked_concept {
            Some(c) => AttributeConceptOption {
                id: Some(c.id()),
                label: c.preferred_term().to_string(),
            },
            None => AttributeConceptOption {
                id: None,
                label: "— Intet begreb (teknisk felt) —".to_string(),
            },
        };

        let mut top_row = row![text_input("attributNavn", &name_val)
            .style(modern_input_style)
            .size(12.0)
            .on_input(move |s| { Message::UpdateAttributeName(class_id, attr_id, s) })
            .padding([3, 6])
            .width(Length::Fill),];

        if linked_concept.is_some() {
            top_row = top_row.push(text("🔗").size(11).color(ThemeColors::PRIMARY));
        }

        top_row = top_row.push(
            button(text("✕").size(10))
                .style(focused_button_style(
                    danger_button_style,
                    is_delete_attr_focused,
                ))
                .on_press(Message::DeleteAttribute(class_id, attr_id))
                .padding([2, 5]),
        );

        let mut attr_col = column![
            top_row.spacing(4).align_y(Alignment::Center),
            row![
                pick_list(data_type_options, selected_dt_opt, move |opt| {
                    Message::UpdateAttributeDataType(class_id, attr_id, opt.data_type)
                })
                .text_size(11.0)
                .padding([3, 6])
                .width(Length::FillPortion(3)),
                pick_list(Multiplicity::PRESETS, Some(mult_val), move |m| {
                    Message::UpdateAttributeMultiplicity(class_id, attr_id, m)
                },)
                .text_size(11.0)
                .padding([3, 6])
                .width(Length::FillPortion(2)),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
            row![
                text("Begreb:").size(11.0).color(ThemeColors::SLATE_500),
                pick_list(
                    attr_concept_options,
                    Some(selected_concept_opt),
                    move |opt| Message::SetAttributeConcept(class_id, attr_id, opt.id)
                )
                .text_size(11.0)
                .padding([3, 6])
                .width(Length::Fill),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ]
        .spacing(4);

        if let Some(c) = linked_concept {
            let badge = container(
                row![
                    text("🔗").size(10).color(ThemeColors::PRIMARY),
                    text(format!("Lineage: {}", c.preferred_term()))
                        .size(10.5)
                        .color(ThemeColors::PRIMARY),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .style(pill_container_style)
            .padding([2, 6]);

            attr_col = attr_col.push(badge);
        }

        if let Some(issue) = NamingLinter::check_attribute_name(&name_val) {
            if !name_val.trim().is_empty() {
                let mut msg = format!("⚠️ {}: {}", issue.rule, issue.message);
                if let Some(fix) = &issue.suggested_fix {
                    msg.push_str(&format!(" (Forslag: {})", fix));
                }
                attr_col = attr_col.push(
                    text(msg)
                        .size(10.0)
                        .color(ColorTokens::WARNING_AMBER.into_color()),
                );
            }
        }

        let attr_card = container(attr_col)
            .style(card_container_style)
            .padding(6)
            .width(Length::Fill);

        attr_list = attr_list.push(attr_card);
    }

    column![attr_header, attr_list].spacing(6).into()
}
