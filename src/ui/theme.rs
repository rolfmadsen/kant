use crate::design_system::tokens::ColorTokens;
use iced::widget::{button, container, scrollable, text_input};
use iced::{Background, Border, Color, Shadow, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThemeColors;

impl ThemeColors {
    // Brand & Primary
    pub const PRIMARY: Color = ColorTokens::PRIMARY.into_color();
    pub const PRIMARY_HOVER: Color = ColorTokens::PRIMARY_HOVER.into_color();
    pub const PRIMARY_ACTIVE: Color = ColorTokens::PRIMARY_ACTIVE.into_color();
    pub const PRIMARY_LIGHT: Color = ColorTokens::PRIMARY_LIGHT_BG.into_color();

    // Modern Slate Neutrals
    pub const SLATE_50: Color = ColorTokens::SLATE_50.into_color();
    pub const SLATE_100: Color = ColorTokens::SLATE_100.into_color();
    pub const SLATE_200: Color = ColorTokens::SLATE_200.into_color();
    pub const SLATE_300: Color = ColorTokens::SLATE_300.into_color();
    pub const SLATE_400: Color = ColorTokens::SLATE_400.into_color();
    pub const SLATE_500: Color = ColorTokens::SLATE_500.into_color();
    pub const SLATE_600: Color = ColorTokens::SLATE_600.into_color();
    pub const SLATE_700: Color = ColorTokens::SLATE_700.into_color();
    pub const SLATE_800: Color = ColorTokens::SLATE_800.into_color();
    pub const SLATE_900: Color = ColorTokens::SLATE_900.into_color();

    // Surfaces & Backgrounds
    pub const SURFACE_BG: Color = ColorTokens::SURFACE_BG.into_color();
    pub const SURFACE_CARD: Color = ColorTokens::SURFACE_CARD.into_color();
    pub const SURFACE_HEADER: Color = ColorTokens::SURFACE_HEADER.into_color();
    pub const SURFACE_ROW_HOVER: Color = ColorTokens::SURFACE_ROW_HOVER.into_color();
    pub const SURFACE_BORDER: Color = ColorTokens::SLATE_200.into_color();
    pub const SURFACE_BORDER_LIGHT: Color = ColorTokens::SURFACE_BORDER_LIGHT.into_color();

    // Legacy Aliases for backwards compatibility
    pub const BACKGROUND_LIGHT: Color = Self::SURFACE_BG;
    pub const SURFACE_WHITE: Color = Self::SURFACE_CARD;
    pub const ROW_HOVER: Color = Self::SURFACE_ROW_HOVER;
    pub const BORDER_COLOR: Color = Self::SURFACE_BORDER;
    pub const BORDER_LIGHT: Color = Self::SURFACE_BORDER_LIGHT;

    // Typography
    pub const TEXT_DARK: Color = Self::SLATE_900;
    pub const TEXT_MUTED: Color = Self::SLATE_500;
    pub const TEXT_LIGHT: Color = ColorTokens::TEXT_INVERTED.into_color();

    // FDA Modelregler Anbefalede Farver (Kapitel 7.3)
    pub const FDA_SAND: Color = ColorTokens::FDA_SAND_BG.into_color();
    pub const FDA_SAND_BORDER: Color = ColorTokens::FDA_SAND_BORDER.into_color();
    pub const FDA_BORROWED_BLUE: Color = ColorTokens::FDA_BORROWED_BLUE.into_color();
    pub const FDA_BORROWED_BLUE_BG: Color = ColorTokens::FDA_BORROWED_BLUE_BG.into_color();
    pub const FDA_DATA_TYPE_YELLOW: Color = ColorTokens::FDA_DATA_TYPE_YELLOW.into_color();
    pub const FDA_ENUM_GREEN: Color = ColorTokens::FDA_ENUM_GREEN.into_color();

    // Status & Feedback
    pub const ACCENT_GREEN: Color = ColorTokens::ACCENT_GREEN.into_color();
    pub const ACCENT_GREEN_LIGHT: Color = ColorTokens::ACCENT_GREEN_LIGHT.into_color();
    pub const ACCENT_RED: Color = ColorTokens::ACCENT_RED.into_color();
    pub const ACCENT_RED_LIGHT: Color = ColorTokens::ACCENT_RED_LIGHT.into_color();

    // Focus Ring (tynd orange ligesom status pill i øverste højre hjørne af headeren)
    pub const FOCUS_RING: Color = ColorTokens::FOCUS_RING.into_color();
}

// -----------------------------------------------------------------------------
// Container Styles
// -----------------------------------------------------------------------------

pub fn card_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ThemeColors::SURFACE_CARD)),
        border: Border {
            color: ColorTokens::SLATE_200.with_alpha(0.85).into_color(),
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow: Shadow {
            color: ColorTokens::SLATE_900.with_alpha(0.07).into_color(),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    }
}

pub fn floating_panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ThemeColors::SURFACE_CARD)),
        border: Border {
            color: ColorTokens::SLATE_200.with_alpha(0.85).into_color(),
            width: 1.0,
            radius: 14.0.into(),
        },
        shadow: Shadow {
            color: ColorTokens::SLATE_900.with_alpha(0.10).into_color(),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 20.0,
        },
        ..Default::default()
    }
}

pub fn pill_container_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            ColorTokens::SLATE_100.with_alpha(0.85).into_color(),
        )),
        border: Border {
            color: ColorTokens::SLATE_300.with_alpha(0.70).into_color(),
            width: 1.0,
            radius: 18.0.into(),
        },
        shadow: Shadow {
            color: ColorTokens::SHADOW_SUBTLE.into_color(),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        ..Default::default()
    }
}

pub fn modern_scrollable_style(
    theme: &iced::Theme,
    status: scrollable::Status,
) -> scrollable::Style {
    let invisible_rail = scrollable::Rail {
        background: None,
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(Color::TRANSPARENT),
            border: Border::default(),
        },
    };

    match status {
        scrollable::Status::Active { .. } => scrollable::Style {
            container: container::Style::default(),
            vertical_rail: invisible_rail,
            horizontal_rail: invisible_rail,
            gap: None,
            auto_scroll: scrollable::default(theme, status).auto_scroll,
        },
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered,
            ..
        } => {
            let scroller_color = if is_vertical_scrollbar_hovered {
                ColorTokens::SLATE_500.with_alpha(0.80).into_color()
            } else {
                ColorTokens::SLATE_400.with_alpha(0.40).into_color()
            };

            let visible_rail = scrollable::Rail {
                background: None,
                border: Border::default(),
                scroller: scrollable::Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        radius: 3.0.into(),
                        ..Border::default()
                    },
                },
            };

            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: visible_rail,
                horizontal_rail: invisible_rail,
                gap: None,
                auto_scroll: scrollable::default(theme, status).auto_scroll,
            }
        }
        scrollable::Status::Dragged { .. } => {
            let visible_rail = scrollable::Rail {
                background: None,
                border: Border::default(),
                scroller: scrollable::Scroller {
                    background: Background::Color(
                        ColorTokens::SLATE_600.with_alpha(0.85).into_color(),
                    ),
                    border: Border {
                        radius: 3.0.into(),
                        ..Border::default()
                    },
                },
            };

            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: visible_rail,
                horizontal_rail: invisible_rail,
                gap: None,
                auto_scroll: scrollable::default(theme, status).auto_scroll,
            }
        }
    }
}

pub fn modal_backdrop_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ColorTokens::MODAL_BACKDROP.into_color())),
        ..Default::default()
    }
}

pub fn modal_card_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ThemeColors::SURFACE_CARD)),
        border: Border {
            color: ColorTokens::SLATE_300.with_alpha(0.80).into_color(),
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: Shadow {
            color: ColorTokens::SHADOW_MODAL.into_color(),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 32.0,
        },
        ..Default::default()
    }
}

// -----------------------------------------------------------------------------
// Button Styles
// -----------------------------------------------------------------------------

pub fn primary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(ThemeColors::PRIMARY)),
            text_color: ThemeColors::TEXT_LIGHT,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            shadow: Shadow {
                color: ColorTokens::PRIMARY.with_alpha(0.22).into_color(),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(ThemeColors::PRIMARY_HOVER)),
            text_color: ThemeColors::TEXT_LIGHT,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            shadow: Shadow {
                color: ColorTokens::PRIMARY.with_alpha(0.35).into_color(),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(ThemeColors::PRIMARY_ACTIVE)),
            text_color: ThemeColors::TEXT_LIGHT,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(ThemeColors::SLATE_200)),
            text_color: ThemeColors::SLATE_400,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

pub fn secondary_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(
                ColorTokens::SURFACE.with_alpha(0.90).into_color(),
            )),
            text_color: ThemeColors::SLATE_700,
            border: Border {
                color: ColorTokens::SLATE_200.with_alpha(0.85).into_color(),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: ColorTokens::SHADOW_SUBTLE.into_color(),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0,
            },
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color::WHITE)),
            text_color: ThemeColors::SLATE_900,
            border: Border {
                color: ThemeColors::SLATE_300,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: ColorTokens::SHADOW_LIGHT.into_color(),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 5.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(ThemeColors::SLATE_100)),
            text_color: ThemeColors::SLATE_900,
            border: Border {
                color: ThemeColors::SLATE_300,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(ThemeColors::SLATE_100)),
            text_color: ThemeColors::SLATE_400,
            border: Border {
                color: ThemeColors::SLATE_200,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
    }
}

pub fn danger_button_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active => button::Style {
            background: Some(Background::Color(
                ColorTokens::DANGER_BG.with_alpha(0.90).into_color(),
            )),
            text_color: ThemeColors::ACCENT_RED,
            border: Border {
                color: ColorTokens::ACCENT_RED.with_alpha(0.40).into_color(),
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(ThemeColors::ACCENT_RED)),
            text_color: ThemeColors::TEXT_LIGHT,
            border: Border {
                color: ThemeColors::ACCENT_RED,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: ColorTokens::ACCENT_RED.with_alpha(0.30).into_color(),
                offset: Vector::new(0.0, 3.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(
                ColorTokens::ACCENT_RED_PRESSED.into_color(),
            )),
            text_color: ThemeColors::TEXT_LIGHT,
            border: Border {
                color: ThemeColors::ACCENT_RED,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        },
        button::Status::Disabled => secondary_button_style(_theme, status),
    }
}

pub fn segmented_tab_button(
    is_active: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        if is_active {
            button::Style {
                background: Some(Background::Color(Color::WHITE)),
                text_color: ThemeColors::PRIMARY,
                border: Border {
                    color: ColorTokens::SLATE_200.with_alpha(0.80).into_color(),
                    width: 1.0,
                    radius: 14.0.into(),
                },
                shadow: Shadow {
                    color: ColorTokens::SLATE_800.with_alpha(0.10).into_color(),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 6.0,
                },
                ..Default::default()
            }
        } else {
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(
                        ColorTokens::SURFACE.with_alpha(0.65).into_color(),
                    )),
                    text_color: ThemeColors::SLATE_900,
                    border: Border {
                        radius: 14.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow {
                        color: ColorTokens::SHADOW_SUBTLE.with_alpha(0.02).into_color(),
                        offset: Vector::new(0.0, 1.0),
                        blur_radius: 2.0,
                    },
                    ..Default::default()
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(
                        ColorTokens::SLATE_200.with_alpha(0.80).into_color(),
                    )),
                    text_color: ThemeColors::SLATE_900,
                    border: Border {
                        radius: 14.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                },
                _ => button::Style {
                    background: None,
                    text_color: ThemeColors::SLATE_600,
                    border: Border {
                        radius: 14.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow::default(),
                    ..Default::default()
                },
            }
        }
    }
}

pub fn list_item_button(
    is_selected: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        if is_selected {
            button::Style {
                background: Some(Background::Color(
                    ColorTokens::PRIMARY_LIGHT_BG.with_alpha(0.92).into_color(),
                )),
                text_color: ThemeColors::PRIMARY,
                border: Border {
                    color: ThemeColors::PRIMARY,
                    width: 1.5,
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: ColorTokens::PRIMARY.with_alpha(0.12).into_color(),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            }
        } else {
            match status {
                button::Status::Hovered => button::Style {
                    background: Some(Background::Color(Color::WHITE)),
                    text_color: ThemeColors::SLATE_900,
                    border: Border {
                        color: ThemeColors::SLATE_300,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow {
                        color: ColorTokens::SHADOW_SUBTLE.into_color(),
                        offset: Vector::new(0.0, 1.0),
                        blur_radius: 3.0,
                    },
                    ..Default::default()
                },
                button::Status::Pressed => button::Style {
                    background: Some(Background::Color(ThemeColors::SLATE_200)),
                    text_color: ThemeColors::SLATE_900,
                    border: Border {
                        color: ThemeColors::SLATE_400,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                },
                _ => button::Style {
                    background: Some(Background::Color(
                        ColorTokens::SURFACE.with_alpha(0.75).into_color(),
                    )),
                    text_color: ThemeColors::SLATE_800,
                    border: Border {
                        color: ColorTokens::SLATE_200.with_alpha(0.75).into_color(),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                },
            }
        }
    }
}

pub fn list_item_container_style(is_selected: bool) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| {
        if is_selected {
            container::Style {
                background: Some(Background::Color(
                    ColorTokens::PRIMARY_LIGHT_BG.with_alpha(0.92).into_color(),
                )),
                border: Border {
                    color: ThemeColors::PRIMARY,
                    width: 1.5,
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: ColorTokens::PRIMARY.with_alpha(0.12).into_color(),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            }
        } else {
            container::Style {
                background: Some(Background::Color(
                    ColorTokens::SURFACE.with_alpha(0.85).into_color(),
                )),
                border: Border {
                    color: ColorTokens::SLATE_200.with_alpha(0.85).into_color(),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                shadow: Shadow {
                    color: ColorTokens::SHADOW_SUBTLE.with_alpha(0.02).into_color(),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
                ..Default::default()
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Input Styles
// -----------------------------------------------------------------------------

pub fn modern_input_style(_theme: &iced::Theme, status: text_input::Status) -> text_input::Style {
    match status {
        text_input::Status::Active => text_input::Style {
            background: Background::Color(ColorTokens::SURFACE.with_alpha(0.92).into_color()),
            border: Border {
                color: ColorTokens::SLATE_200.with_alpha(0.85).into_color(),
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: ThemeColors::SLATE_400,
            placeholder: ThemeColors::SLATE_400,
            value: ThemeColors::SLATE_900,
            selection: ThemeColors::PRIMARY_LIGHT,
        },
        text_input::Status::Hovered => text_input::Style {
            background: Background::Color(Color::WHITE),
            border: Border {
                color: ThemeColors::SLATE_300,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: ThemeColors::SLATE_500,
            placeholder: ThemeColors::SLATE_400,
            value: ThemeColors::SLATE_900,
            selection: ThemeColors::PRIMARY_LIGHT,
        },
        text_input::Status::Focused { .. } => text_input::Style {
            background: Background::Color(Color::WHITE),
            border: Border {
                color: ThemeColors::PRIMARY,
                width: 1.5,
                radius: 8.0.into(),
            },
            icon: ThemeColors::PRIMARY,
            placeholder: ColorTokens::SLATE_400.with_alpha(0.60).into_color(),
            value: ThemeColors::SLATE_900,
            selection: ThemeColors::PRIMARY_LIGHT,
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(ThemeColors::SLATE_100),
            border: Border {
                color: ThemeColors::SLATE_200,
                width: 1.0,
                radius: 8.0.into(),
            },
            icon: ThemeColors::SLATE_400,
            placeholder: ThemeColors::SLATE_400,
            value: ThemeColors::SLATE_500,
            selection: ThemeColors::PRIMARY_LIGHT,
        },
    }
}

pub fn focused_button_style(
    base_style: fn(&iced::Theme, button::Status) -> button::Style,
    is_focused: bool,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |theme, status| {
        let mut s = base_style(theme, status);
        if is_focused {
            s.border = Border {
                color: ThemeColors::FOCUS_RING,
                width: 1.5,
                radius: s.border.radius,
            };
            s.shadow = Shadow {
                color: ColorTokens::FOCUS_RING.with_alpha(0.40).into_color(),
                offset: Vector::new(0.0, 0.0),
                blur_radius: 4.0,
            };
        }
        s
    }
}
