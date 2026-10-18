//! Acceptance tests for Kant Typed Design System Tokens & WCAG AA Contrast Verification.
//! Task 082: Typed Design System Tokens Implementation.

use kant::design_system::tokens::{
    contrast_ratio, relative_luminance, ColorToken, ColorTokens, SpacingTokens, TypographyTokens,
};

#[test]
fn test_wcag_relative_luminance_and_contrast_ratio_formula() {
    let black = ColorToken::from_rgb(0.0, 0.0, 0.0);
    let white = ColorToken::from_rgb(1.0, 1.0, 1.0);

    let lum_black = relative_luminance(&black);
    let lum_white = relative_luminance(&white);

    assert!(
        (lum_black - 0.0).abs() < 1e-4,
        "Relative luminance of black should be 0.0, got {}",
        lum_black
    );
    assert!(
        (lum_white - 1.0).abs() < 1e-4,
        "Relative luminance of white should be 1.0, got {}",
        lum_white
    );

    let black_white_ratio = contrast_ratio(&black, &white);
    assert!(
        (black_white_ratio - 21.0).abs() < 1e-3,
        "Contrast ratio between pure black and white must be 21:1, got {}",
        black_white_ratio
    );

    let same_color_ratio = contrast_ratio(&black, &black);
    assert!(
        (same_color_ratio - 1.0).abs() < 1e-4,
        "Contrast ratio between identical colors must be 1:1, got {}",
        same_color_ratio
    );
}

#[test]
fn test_text_primary_contrast_against_background_and_surface() {
    let bg_ratio = contrast_ratio(&ColorTokens::TEXT_PRIMARY, &ColorTokens::BACKGROUND);
    assert!(
        bg_ratio >= 4.5,
        "TextPrimary vs Background must be >= 4.5:1, got {:.2}:1",
        bg_ratio
    );

    let surface_ratio = contrast_ratio(&ColorTokens::TEXT_PRIMARY, &ColorTokens::SURFACE);
    assert!(
        surface_ratio >= 4.5,
        "TextPrimary vs Surface must be >= 4.5:1, got {:.2}:1",
        surface_ratio
    );

    let card_ratio = contrast_ratio(&ColorTokens::TEXT_PRIMARY, &ColorTokens::SURFACE_CARD);
    assert!(
        card_ratio >= 4.5,
        "TextPrimary vs SurfaceCard must be >= 4.5:1, got {:.2}:1",
        card_ratio
    );
}

#[test]
fn test_text_muted_contrast_against_background_and_surface() {
    let bg_ratio = contrast_ratio(&ColorTokens::TEXT_MUTED, &ColorTokens::BACKGROUND);
    assert!(
        bg_ratio >= 4.5,
        "TextMuted vs Background must be >= 4.5:1, got {:.2}:1",
        bg_ratio
    );

    let surface_ratio = contrast_ratio(&ColorTokens::TEXT_MUTED, &ColorTokens::SURFACE);
    assert!(
        surface_ratio >= 4.5,
        "TextMuted vs Surface must be >= 4.5:1, got {:.2}:1",
        surface_ratio
    );
}

#[test]
fn test_status_badges_contrast() {
    let success_ratio = contrast_ratio(&ColorTokens::SUCCESS_TEXT, &ColorTokens::SUCCESS_BG);
    assert!(
        success_ratio >= 4.5,
        "Success badge text vs background must be >= 4.5:1, got {:.2}:1",
        success_ratio
    );

    let danger_ratio = contrast_ratio(&ColorTokens::DANGER_TEXT, &ColorTokens::DANGER_BG);
    assert!(
        danger_ratio >= 4.5,
        "Danger badge text vs background must be >= 4.5:1, got {:.2}:1",
        danger_ratio
    );

    let warning_ratio = contrast_ratio(&ColorTokens::WARNING_TEXT, &ColorTokens::WARNING_BG);
    assert!(
        warning_ratio >= 4.5,
        "Warning badge text vs background must be >= 4.5:1, got {:.2}:1",
        warning_ratio
    );

    let info_ratio = contrast_ratio(&ColorTokens::INFO_TEXT, &ColorTokens::INFO_BG);
    assert!(
        info_ratio >= 4.5,
        "Info badge text vs background must be >= 4.5:1, got {:.2}:1",
        info_ratio
    );
}

#[test]
fn test_iced_color_conversion_zero_overhead() {
    let token = ColorTokens::TEXT_PRIMARY;
    let iced_color: iced::Color = token.into();

    assert_eq!(iced_color.r, token.r);
    assert_eq!(iced_color.g, token.g);
    assert_eq!(iced_color.b, token.b);
    assert_eq!(iced_color.a, token.a);

    let direct_color: iced::Color = token.into_color();
    assert_eq!(direct_color, iced_color);

    let from_iced: ColorToken = iced_color.into();
    assert_eq!(from_iced, token);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_spacing_and_typography_tokens_scale() {
    assert!(SpacingTokens::XXS < SpacingTokens::XS);
    assert!(SpacingTokens::XS < SpacingTokens::SM);
    assert!(SpacingTokens::SM < SpacingTokens::MD);
    assert!(SpacingTokens::MD < SpacingTokens::LG);
    assert!(SpacingTokens::LG < SpacingTokens::XL);
    assert!(SpacingTokens::XL < SpacingTokens::XXL);

    assert!(TypographyTokens::CAPTION < TypographyTokens::BODY_SM);
    assert!(TypographyTokens::BODY_SM < TypographyTokens::BODY);
    assert!(TypographyTokens::BODY < TypographyTokens::TITLE_SM);
    assert!(TypographyTokens::TITLE_SM < TypographyTokens::TITLE);
    assert!(TypographyTokens::TITLE < TypographyTokens::TITLE_LG);
    assert!(TypographyTokens::TITLE_LG < TypographyTokens::DISPLAY);
}
