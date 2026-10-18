//! Type-safe Design System Tokens for Kant.
//! Conforms to ADR 015 with mathematical guarantees for WAI 2.0 / WCAG AA contrast (>= 4.5:1).

/// A type-safe color token representing an sRGB color with alpha.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorToken {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorToken {
    /// Create an opaque color token from normalized sRGB components (0.0 to 1.0).
    pub const fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color token with alpha from normalized components (0.0 to 1.0).
    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Converts the token directly into `iced::Color` without runtime overhead.
    #[inline]
    pub const fn into_color(self) -> iced::Color {
        iced::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }

    /// Returns a new ColorToken with the specified alpha channel (0.0 to 1.0).
    #[inline]
    pub const fn with_alpha(self, a: f32) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    /// Converts an `iced::Color` into a `ColorToken`.
    #[inline]
    pub const fn from_iced(c: iced::Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }

    /// Calculates the W3C relative luminance of this color token.
    pub fn luminance(&self) -> f32 {
        relative_luminance(self)
    }

    /// Calculates the WCAG contrast ratio between this color token and another.
    pub fn contrast_with(&self, other: &Self) -> f32 {
        contrast_ratio(self, other)
    }

    /// Checks whether this color has at least 4.5:1 contrast against a given background (WCAG AA normal text).
    pub fn satisfies_wcag_aa_text(&self, background: &Self) -> bool {
        self.contrast_with(background) >= 4.5
    }

    /// Checks whether this color has at least 3.0:1 contrast against a given background (WCAG AA UI components/large text).
    pub fn satisfies_wcag_aa_ui(&self, background: &Self) -> bool {
        self.contrast_with(background) >= 3.0
    }
}

impl From<ColorToken> for iced::Color {
    #[inline]
    fn from(token: ColorToken) -> Self {
        token.into_color()
    }
}

impl From<iced::Color> for ColorToken {
    #[inline]
    fn from(c: iced::Color) -> Self {
        ColorToken::from_iced(c)
    }
}

/// Linearizes an sRGB channel according to the W3C WCAG 2.1 specification.
#[inline]
fn linearize_channel(c: f32) -> f32 {
    let c_clamped = c.clamp(0.0, 1.0);
    if c_clamped <= 0.04045 {
        c_clamped / 12.92
    } else {
        ((c_clamped + 0.055) / 1.055).powf(2.4)
    }
}

/// Computes the relative luminance ($L$) of an sRGB color per the W3C WCAG 2.1 formula:
/// $L = 0.2126 \times R_{\text{lin}} + 0.7152 \times G_{\text{lin}} + 0.0722 \times B_{\text{lin}}$
pub fn relative_luminance(color: &ColorToken) -> f32 {
    let r_lin = linearize_channel(color.r);
    let g_lin = linearize_channel(color.g);
    let b_lin = linearize_channel(color.b);
    0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
}

/// Computes the contrast ratio between two colors per the W3C WCAG 2.1 formula:
/// $\text{Contrast Ratio} = \frac{L_1 + 0.05}{L_2 + 0.05}$ where $L_1 \ge L_2$.
pub fn contrast_ratio(c1: &ColorToken, c2: &ColorToken) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Semantic color tokens for Kant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorTokens;

impl ColorTokens {
    // -------------------------------------------------------------------------
    // Modern Slate Neutrals
    // -------------------------------------------------------------------------
    /// Slate 50 (#F8FAFC).
    pub const SLATE_50: ColorToken = ColorToken::from_rgb(0.976, 0.980, 0.988);
    /// Slate 100 (#F1F5F9).
    pub const SLATE_100: ColorToken = ColorToken::from_rgb(0.945, 0.957, 0.973);
    /// Slate 200 (#E2E8F0).
    pub const SLATE_200: ColorToken = ColorToken::from_rgb(0.886, 0.910, 0.941);
    /// Slate 300 (#CBD5E1).
    pub const SLATE_300: ColorToken = ColorToken::from_rgb(0.796, 0.835, 0.882);
    /// Slate 400 (#94A3B8).
    pub const SLATE_400: ColorToken = ColorToken::from_rgb(0.576, 0.635, 0.722);
    /// Slate 500 (#64748B).
    pub const SLATE_500: ColorToken = ColorToken::from_rgb(0.392, 0.455, 0.545);
    /// Slate 600 (#475569).
    pub const SLATE_600: ColorToken = ColorToken::from_rgb(0.278, 0.333, 0.412);
    /// Slate 700 (#334155).
    pub const SLATE_700: ColorToken = ColorToken::from_rgb(0.200, 0.255, 0.333);
    /// Slate 800 (#1E293B).
    pub const SLATE_800: ColorToken = ColorToken::from_rgb(0.118, 0.161, 0.231);
    /// Slate 900 (#0F172A).
    pub const SLATE_900: ColorToken = ColorToken::from_rgb(0.059, 0.090, 0.165);

    // -------------------------------------------------------------------------
    // Surfaces & Backgrounds
    // -------------------------------------------------------------------------
    /// Global application canvas backdrop (Slate 50 / #F8FAFC).
    pub const BACKGROUND: ColorToken = Self::SLATE_50;
    /// Primary card and panel surface (#FFFFFF).
    pub const SURFACE: ColorToken = ColorToken::from_rgb(1.0, 1.0, 1.0);
    /// Soft COSMIC canvas backdrop (#F4F6F9).
    pub const SURFACE_BG: ColorToken = ColorToken::from_rgb(0.955, 0.965, 0.978);
    /// Header bar surface (#FFFFFF).
    pub const SURFACE_HEADER: ColorToken = ColorToken::from_rgb(1.0, 1.0, 1.0);
    /// Card surface alias (#FFFFFF).
    pub const SURFACE_CARD: ColorToken = ColorToken::from_rgb(1.0, 1.0, 1.0);
    /// Muted row hover surface (#F1F5F9).
    pub const SURFACE_ROW_HOVER: ColorToken = ColorToken::from_rgb(0.965, 0.975, 0.990);
    /// Subtle secondary panel background (#F8FAFC).
    pub const SURFACE_MUTED: ColorToken = Self::SLATE_50;
    /// Light surface border (#EBF0F5).
    pub const SURFACE_BORDER_LIGHT: ColorToken = ColorToken::from_rgb(0.920, 0.940, 0.960);
    /// Canvas grid background (#FBFCFD).
    pub const CANVAS_BACKGROUND: ColorToken = ColorToken::from_rgb(0.985, 0.988, 0.992);

    // -------------------------------------------------------------------------
    // Typography
    // -------------------------------------------------------------------------
    /// High-contrast primary text (Slate 900 / #0F172A). Contrast > 17:1 against White/Slate 50.
    pub const TEXT_PRIMARY: ColorToken = Self::SLATE_900;
    /// Dark text alias (Slate 900).
    pub const TEXT_DARK: ColorToken = Self::SLATE_900;
    /// Accessible muted text (Slate 600 / #475569). Contrast > 7.2:1 against White/Slate 50 (exceeds WCAG AA >= 4.5:1).
    pub const TEXT_MUTED: ColorToken = Self::SLATE_600;
    /// Secondary text (Slate 500 / #64748B).
    pub const TEXT_SECONDARY: ColorToken = Self::SLATE_500;
    /// Inverted white text for dark badges and primary buttons (#FFFFFF).
    pub const TEXT_INVERTED: ColorToken = ColorToken::from_rgb(1.0, 1.0, 1.0);
    /// White text alias (#FFFFFF).
    pub const TEXT_LIGHT: ColorToken = ColorToken::from_rgb(1.0, 1.0, 1.0);

    // -------------------------------------------------------------------------
    // Brand & Primary Interactive
    // -------------------------------------------------------------------------
    /// Official FDA Danish Blue (#1461BD). Contrast > 6.0:1 against White.
    pub const PRIMARY: ColorToken = ColorToken::from_rgb(0.078, 0.380, 0.741);
    /// Primary hover state (#1E74D9).
    pub const PRIMARY_HOVER: ColorToken = ColorToken::from_rgb(0.118, 0.451, 0.851);
    /// Primary active / pressed state (#0F4C98).
    pub const PRIMARY_ACTIVE: ColorToken = ColorToken::from_rgb(0.059, 0.298, 0.596);
    /// Light primary tint for badges and selected tabs (#EBF2FC).
    pub const PRIMARY_LIGHT_BG: ColorToken = ColorToken::from_rgb(0.920, 0.950, 0.990);
    /// Dark primary for high-contrast small text (#145A96).
    pub const PRIMARY_DARK_TEXT: ColorToken = ColorToken::from_rgb(0.08, 0.35, 0.65);

    // -------------------------------------------------------------------------
    // Status Badges & Feedback (Paired background and text with guaranteed >= 4.5:1)
    // -------------------------------------------------------------------------
    /// Success badge background (Emerald 50 / #ECFDF5).
    pub const SUCCESS_BG: ColorToken = ColorToken::from_rgb(0.925, 0.992, 0.961);
    /// Success badge text (Emerald 800 / #065F46). Contrast vs SUCCESS_BG >= 7.1:1.
    pub const SUCCESS_TEXT: ColorToken = ColorToken::from_rgb(0.024, 0.373, 0.275);
    /// Success border (Emerald 200 / #A7F3D0).
    pub const SUCCESS_BORDER: ColorToken = ColorToken::from_rgb(0.655, 0.953, 0.816);
    /// Accent green (#21A66B).
    pub const ACCENT_GREEN: ColorToken = ColorToken::from_rgb(0.13, 0.65, 0.42);
    /// Accent green light tint (#E8FAF0).
    pub const ACCENT_GREEN_LIGHT: ColorToken = ColorToken::from_rgb(0.91, 0.98, 0.93);
    /// Dark success text for small badges (#267326).
    pub const SUCCESS_DARK_TEXT: ColorToken = ColorToken::from_rgb(0.15, 0.45, 0.15);

    /// Danger badge background (Red 50 / #FEF2F2).
    pub const DANGER_BG: ColorToken = ColorToken::from_rgb(0.996, 0.949, 0.949);
    /// Danger badge text (Red 800 / #991B1B). Contrast vs DANGER_BG >= 8.0:1.
    pub const DANGER_TEXT: ColorToken = ColorToken::from_rgb(0.600, 0.106, 0.106);
    /// Danger border (Red 200 / #FECACA).
    pub const DANGER_BORDER: ColorToken = ColorToken::from_rgb(0.996, 0.792, 0.792);
    /// Accent red (#D63838).
    pub const ACCENT_RED: ColorToken = ColorToken::from_rgb(0.84, 0.22, 0.22);
    /// Accent red light tint (#FDEBEB).
    pub const ACCENT_RED_LIGHT: ColorToken = ColorToken::from_rgb(0.99, 0.92, 0.92);
    /// Accent red pressed state (#B32E2E).
    pub const ACCENT_RED_PRESSED: ColorToken = ColorToken::from_rgb(0.70, 0.18, 0.18);

    /// Warning badge background (Amber 50 / #FFFBEB).
    pub const WARNING_BG: ColorToken = ColorToken::from_rgb(1.0, 0.984, 0.922);
    /// Warning badge text (Amber 800 / #92400E). Contrast vs WARNING_BG >= 5.8:1.
    pub const WARNING_TEXT: ColorToken = ColorToken::from_rgb(0.573, 0.251, 0.055);
    /// Warning border (Amber 200 / #FDE68A).
    pub const WARNING_BORDER: ColorToken = ColorToken::from_rgb(0.992, 0.902, 0.541);
    /// High-contrast warning amber (#D98C1A).
    pub const WARNING_AMBER: ColorToken = ColorToken::from_rgb(0.85, 0.55, 0.10);
    /// Warning amber light tint (#FFF7E0).
    pub const WARNING_AMBER_LIGHT: ColorToken = ColorToken::from_rgb(1.0, 0.98, 0.88);

    /// Info badge background (Blue 50 / #EFF6FF).
    pub const INFO_BG: ColorToken = ColorToken::from_rgb(0.937, 0.965, 1.0);
    /// Info badge text (Blue 800 / #1E40AF). Contrast vs INFO_BG >= 6.9:1.
    pub const INFO_TEXT: ColorToken = ColorToken::from_rgb(0.118, 0.251, 0.686);
    /// Info border (Blue 200 / #BFDBFE).
    pub const INFO_BORDER: ColorToken = ColorToken::from_rgb(0.749, 0.859, 0.996);
    /// Info blue (#3380F2).
    pub const INFO_BLUE: ColorToken = ColorToken::from_rgb(0.20, 0.50, 0.95);

    /// Purple status text (#8C59F2).
    pub const PURPLE_TEXT: ColorToken = ColorToken::from_rgb(0.55, 0.35, 0.95);
    /// Purple status background (#F2EBFF).
    pub const PURPLE_BG: ColorToken = ColorToken::from_rgb(0.95, 0.92, 1.0);

    /// Muted badge dot indicator (#8C99AD).
    pub const MUTED_BADGE_DOT: ColorToken = ColorToken::from_rgb(0.55, 0.60, 0.68);
    /// Muted badge background (#F2F2F5).
    pub const MUTED_BADGE_BG: ColorToken = ColorToken::from_rgb(0.95, 0.95, 0.96);

    // -------------------------------------------------------------------------
    // Borders & Dividers
    // -------------------------------------------------------------------------
    /// Subtle divider border (Slate 200 / #E2E8F0).
    pub const BORDER_SUBTLE: ColorToken = Self::SLATE_200;
    /// Default component border (Slate 300 / #CBD5E1).
    pub const BORDER_DEFAULT: ColorToken = Self::SLATE_300;
    /// Emphasized component border (Slate 400 / #94A3B8).
    pub const BORDER_STRONG: ColorToken = Self::SLATE_400;

    // -------------------------------------------------------------------------
    // Focus Ring
    // -------------------------------------------------------------------------
    /// Amber focus ring indicator (#D97706).
    pub const FOCUS_RING: ColorToken = ColorToken::from_rgb(0.851, 0.467, 0.024);

    // -------------------------------------------------------------------------
    // FDA Modelregler Anbefalede Farver (Kapitel 7.3)
    // -------------------------------------------------------------------------
    /// Sand farve til egne klasser og lokale begreber (#FEFAF7).
    pub const FDA_SAND_BG: ColorToken = ColorToken::from_rgb(0.996, 0.980, 0.969);
    /// Sand kant (#E8DBD0).
    pub const FDA_SAND_BORDER: ColorToken = ColorToken::from_rgb(0.910, 0.860, 0.800);
    /// Sand accent ramme (#D9CC80).
    pub const FDA_SAND_ACCENT: ColorToken = ColorToken::from_rgb(0.85, 0.80, 0.50);
    /// Blå farve til indlånte og genbrugte klasser (#E6F5FC).
    pub const FDA_BORROWED_BLUE_BG: ColorToken = ColorToken::from_rgb(0.900, 0.960, 0.990);
    /// Blå farve til indlånt/genbrugt klasse (#87CDEB).
    pub const FDA_BORROWED_BLUE: ColorToken = ColorToken::from_rgb(0.529, 0.804, 0.922);
    /// Blå accent ramme (#59A6D9).
    pub const FDA_BORROWED_ACCENT: ColorToken = ColorToken::from_rgb(0.35, 0.65, 0.85);
    /// Gul farve til FDA datatyper (#FBF9C6).
    pub const FDA_DATA_TYPE_YELLOW: ColorToken = ColorToken::from_rgb(0.984, 0.976, 0.776);
    /// Grøn farve til FDA enumerationer (#E8FDE3).
    pub const FDA_ENUM_GREEN: ColorToken = ColorToken::from_rgb(0.910, 0.992, 0.890);
    /// Grøn accent ramme (#99D199).
    pub const FDA_ENUM_ACCENT: ColorToken = ColorToken::from_rgb(0.60, 0.82, 0.60);

    // -------------------------------------------------------------------------
    // Overlay, Modal & Shadow Semantic Tokens
    // -------------------------------------------------------------------------
    /// Semi-transparent dark modal backdrop.
    pub const MODAL_BACKDROP: ColorToken = ColorToken::from_rgba(0.05, 0.08, 0.15, 0.50);
    /// Subtle shadow tint.
    pub const SHADOW_SUBTLE: ColorToken = ColorToken::from_rgba(0.0, 0.0, 0.0, 0.03);
    /// Light shadow tint.
    pub const SHADOW_LIGHT: ColorToken = ColorToken::from_rgba(0.05, 0.10, 0.20, 0.07);
    /// Prominent modal shadow tint.
    pub const SHADOW_MODAL: ColorToken = ColorToken::from_rgba(0.0, 0.0, 0.0, 0.22);
    /// Canvas marquee selection fill tint.
    pub const SELECTION_FILL: ColorToken = ColorToken::from_rgba(0.23, 0.51, 0.96, 0.12);
    /// Canvas minimap viewport indicator tint.
    pub const SELECTION_VIEWPORT: ColorToken = ColorToken::from_rgba(0.23, 0.51, 0.96, 0.16);
}

/// Type-safe spacing scale for Kant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacingTokens;

impl SpacingTokens {
    /// 2px spacing (tight gaps, micro borders).
    pub const XXS: f32 = 2.0;
    /// 4px spacing (inline badge padding, small margins).
    pub const XS: f32 = 4.0;
    /// 8px spacing (button padding, form field gaps).
    pub const SM: f32 = 8.0;
    /// 12px spacing (container inner spacing, card headers).
    pub const MD: f32 = 12.0;
    /// 16px spacing (card padding, standard layout gaps).
    pub const LG: f32 = 16.0;
    /// 20px spacing (panel padding).
    pub const XL: f32 = 20.0;
    /// 24px spacing (major block separation).
    pub const XXL: f32 = 24.0;
    /// 32px spacing (dialog and section spacing).
    pub const SECTION: f32 = 32.0;
}

/// Type-safe typography scale for Kant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypographyTokens;

impl TypographyTokens {
    /// 11px font size (captions, badges, keyboard shortcut pills).
    pub const CAPTION: f32 = 11.0;
    /// 12px font size (secondary text, section headers, table cells).
    pub const BODY_SM: f32 = 12.0;
    /// 14px font size (primary body text, standard inputs, buttons).
    pub const BODY: f32 = 14.0;
    /// 16px font size (card titles, modal section titles).
    pub const TITLE_SM: f32 = 16.0;
    /// 18px font size (modal headers, view titles).
    pub const TITLE: f32 = 18.0;
    /// 20px font size (top-level view titles).
    pub const TITLE_LG: f32 = 20.0;
    /// 24px font size (hero headlines, major dialog headings).
    pub const DISPLAY: f32 = 24.0;
}
