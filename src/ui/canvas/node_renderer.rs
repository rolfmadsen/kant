use super::viewport::CanvasViewport;
use crate::design_system::tokens::ColorTokens;
use crate::features::concept_model::DiagramNode;
use crate::features::information_model::ClassDiagramNode;
use crate::ui::theme::ThemeColors;
use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{alignment, Point, Size};

/// Standard rendering af FDA begrebsnode
pub fn render_concept_node(
    frame: &mut Frame,
    node: &DiagramNode,
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 8.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());

    let fill_color = if node.is_local() {
        ThemeColors::FDA_SAND
    } else {
        ThemeColors::FDA_BORROWED_BLUE
    };

    frame.fill(&node_path, fill_color);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else if node.is_local() {
        (ThemeColors::FDA_SAND_BORDER, 1.2)
    } else {
        (ColorTokens::FDA_BORROWED_ACCENT.into_color(), 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    let badge_text = concept_stereotype_text();

    frame.fill_text(Text {
        content: badge_text.to_string(),
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 24.0),
        color: ThemeColors::SLATE_600,
        size: 10.5.into(),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });

    frame.fill_text(Text {
        content: node.label().to_string(),
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 48.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });
}

/// Officiel FDA stereotype for begreber jf. Modelregel 03
pub fn concept_stereotype_text() -> &'static str {
    "«Concept»"
}

/// Officiel FDA stereotype for UML klasser jf. Modelregel 03
pub fn uml_class_stereotype_text(is_abstract: bool) -> String {
    if is_abstract {
        "«Concept» {abstract}".to_string()
    } else {
        "«Concept»".to_string()
    }
}

/// Officielt FDA UML keyword for enumerationer jf. Modelreglerne kapitel 5.2
pub fn enumeration_keyword_text() -> &'static str {
    "«enumeration»"
}

/// Officielt FDA UML keyword for strukturerede datatyper jf. Modelreglerne kapitel 5.5
pub fn datatype_keyword_text() -> &'static str {
    "«dataType»"
}

/// Rendering af UML Enumeration kasse (grøn jf. FDA Modelreglerne Kapitel 5.2 & 7.3)
pub fn render_uml_enumeration_node(
    frame: &mut Frame,
    node: &ClassDiagramNode,
    name: &str,
    values: &[String],
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 6.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());
    frame.fill(&node_path, ThemeColors::FDA_ENUM_GREEN);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else {
        (ColorTokens::FDA_ENUM_ACCENT.into_color(), 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    // Header: «enumeration» + Name
    let header_y = node.y() + 10.0;
    frame.fill_text(Text {
        content: enumeration_keyword_text().to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y),
        color: ThemeColors::SLATE_600,
        size: 11.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    frame.fill_text(Text {
        content: name.to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y + 16.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    let divider_y = node.y() + 48.0;
    let divider = Path::line(
        Point::new(node.x(), divider_y),
        Point::new(node.x() + node.width(), divider_y),
    );
    frame.stroke(
        &divider,
        Stroke::default()
            .with_color(ColorTokens::SUCCESS_BORDER.into_color())
            .with_width(1.0),
    );

    // Values in lowerCamelCase
    let mut val_y = divider_y + 8.0;
    for val in values {
        frame.fill_text(Text {
            content: val.clone(),
            position: Point::new(node.x() + 12.0, val_y),
            color: ThemeColors::SLATE_800,
            size: 12.0.into(),
            ..Default::default()
        });
        val_y += 18.0;
    }
}

/// Rendering af UML Datatype kasse (gul jf. FDA Modelreglerne Kapitel 5.5 & 7.3)
pub fn render_uml_datatype_node(
    frame: &mut Frame,
    node: &ClassDiagramNode,
    name: &str,
    attributes: &[(String, String, String, bool)],
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 6.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());
    frame.fill(&node_path, ThemeColors::FDA_DATA_TYPE_YELLOW);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else {
        (ColorTokens::FDA_SAND_ACCENT.into_color(), 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    // Header: «dataType» + Name
    let header_y = node.y() + 10.0;
    frame.fill_text(Text {
        content: datatype_keyword_text().to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y),
        color: ThemeColors::SLATE_600,
        size: 11.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    frame.fill_text(Text {
        content: name.to_string(),
        position: Point::new(node.x() + node.width() / 2.0, header_y + 16.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        align_x: iced::alignment::Horizontal::Center.into(),
        ..Default::default()
    });

    let divider_y = node.y() + 48.0;
    let divider = Path::line(
        Point::new(node.x(), divider_y),
        Point::new(node.x() + node.width(), divider_y),
    );
    frame.stroke(
        &divider,
        Stroke::default()
            .with_color(ColorTokens::WARNING_BORDER.into_color())
            .with_width(1.0),
    );

    // Attributes
    let mut attr_y = divider_y + 8.0;
    for (attr_name, attr_type, mult, _has_concept) in attributes {
        let text_content = format!("{}: {} [{}]", attr_name, attr_type, mult);
        frame.fill_text(Text {
            content: text_content,
            position: Point::new(node.x() + 12.0, attr_y),
            color: ThemeColors::SLATE_800,
            size: 12.0.into(),
            ..Default::default()
        });
        attr_y += 18.0;
    }
}

/// Afkorter en tekststreng pænt med ellipsis `...`, hvis den overstiger max_chars
pub fn truncate_with_ellipsis(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{}...", prefix)
    }
}

/// Standard rendering af UML 3-sektions kasse
#[allow(clippy::too_many_arguments)]
pub fn render_uml_class_node(
    frame: &mut Frame,
    node: &ClassDiagramNode,
    class_name: &str,
    attributes: &[(String, String, String, bool)],
    is_borrowed: bool,
    is_abstract: bool,
    is_selected: bool,
    _viewport: CanvasViewport,
) {
    let top_left = Point::new(node.x(), node.y());
    let size = Size::new(node.width(), node.height());
    let radius = 6.0;

    let shadow_path =
        Path::rounded_rectangle(Point::new(node.x(), node.y() + 2.0), size, radius.into());
    frame.fill(&shadow_path, ColorTokens::SHADOW_LIGHT.into_color());

    let node_path = Path::rounded_rectangle(top_left, size, radius.into());

    let fill_color = if is_borrowed {
        ThemeColors::FDA_BORROWED_BLUE
    } else {
        ThemeColors::FDA_SAND
    };

    frame.fill(&node_path, fill_color);

    let (border_color, border_width) = if is_selected {
        (ThemeColors::PRIMARY, 2.5)
    } else if is_borrowed {
        (ColorTokens::FDA_BORROWED_ACCENT.into_color(), 1.2)
    } else {
        (ThemeColors::FDA_SAND_BORDER, 1.2)
    };

    frame.stroke(
        &node_path,
        Stroke::default()
            .with_color(border_color)
            .with_width(border_width),
    );

    let stereotype_text = uml_class_stereotype_text(is_abstract);

    frame.fill_text(Text {
        content: stereotype_text,
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 14.0),
        color: ThemeColors::SLATE_600,
        size: 10.5.into(),
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });

    let name_font = if is_abstract {
        iced::Font {
            style: iced::font::Style::Italic,
            ..Default::default()
        }
    } else {
        iced::Font::DEFAULT
    };

    let display_class_name = truncate_with_ellipsis(class_name, 22);
    frame.fill_text(Text {
        content: display_class_name,
        position: Point::new(node.x() + node.width() / 2.0, node.y() + 32.0),
        color: ThemeColors::SLATE_900,
        size: 14.0.into(),
        font: name_font,
        align_x: alignment::Horizontal::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Default::default()
    });

    let divider_y = node.y() + 48.0;
    let divider_path = Path::new(|b| {
        b.move_to(Point::new(node.x(), divider_y));
        b.line_to(Point::new(node.x() + node.width(), divider_y));
    });

    let divider_color = if is_borrowed {
        ColorTokens::INFO_BORDER.into_color()
    } else {
        ThemeColors::FDA_SAND_BORDER
    };

    frame.stroke(
        &divider_path,
        Stroke::default().with_color(divider_color).with_width(1.0),
    );

    let mut attr_y = divider_y + 14.0;
    if attributes.is_empty() {
        frame.fill_text(Text {
            content: "(ingen attributter)".to_string(),
            position: Point::new(node.x() + 14.0, attr_y),
            color: ThemeColors::SLATE_400,
            size: 11.0.into(),
            align_x: alignment::Horizontal::Left.into(),
            align_y: alignment::Vertical::Center,
            ..Default::default()
        });
    } else {
        for (attr_name, attr_type, attr_mult, has_concept) in attributes {
            let line_str = if *has_concept {
                format!("+ {} : {} [{}] 🔗", attr_name, attr_type, attr_mult)
            } else {
                format!("+ {} : {} [{}]", attr_name, attr_type, attr_mult)
            };
            let display_str = truncate_with_ellipsis(&line_str, 26);
            frame.fill_text(Text {
                content: display_str,
                position: Point::new(node.x() + 14.0, attr_y),
                color: ThemeColors::SLATE_800,
                size: 12.0.into(),
                align_x: alignment::Horizontal::Left.into(),
                align_y: alignment::Vertical::Center,
                ..Default::default()
            });
            attr_y += 20.0;
        }
    }
}
