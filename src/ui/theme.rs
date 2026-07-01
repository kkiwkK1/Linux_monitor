//! Design tokens — single source of truth for the widget's appearance.
//!
//! Direction: **flat minimal**. Muted, cohesive accents on a clean dark card,
//! no glow, thin lines. All skins pull colors/spacing/fonts from here so a
//! palette change is a one-file edit instead of chasing hardcoded hex around
//! the skins.
//!
//! Two accessors per color: [`Color::hex`] for Pango `<span foreground=…>`
//! markup, [`Color::rgb_f`] for Cairo `set_source_rgb*`.

/// An 8-bit RGB color. Kept dependency-free so it works for both Pango markup
/// (hex strings) and Cairo drawing (0.0–1.0 floats).
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// `#rrggbb` for Pango markup.
    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// `(r, g, b)` in 0.0–1.0 for Cairo.
    pub fn rgb_f(&self) -> (f64, f64, f64) {
        (self.r as f64 / 255.0, self.g as f64 / 255.0, self.b as f64 / 255.0)
    }
}

// ── Surface & text ──────────────────────────────────────────────────────────

/// Card background — near-black for strong contrast against the accents.
/// Window-level opacity is applied separately via `set_opacity`.
pub const BG: Color = Color::new(13, 14, 17); // #0d0e11
/// Hairline border — kept as an rgba string since it needs alpha.
pub const BORDER_RGBA: &str = "rgba(255, 255, 255, 0.10)";
/// Card corner radius (px) and inner padding (px).
pub const RADIUS_PX: u32 = 10;
pub const PADDING_PX: u32 = 6;

/// Primary label text.
pub const TEXT: Color = Color::new(238, 240, 243); // #eef0f3
/// Secondary / unit / dim text.
pub const TEXT_DIM: Color = Color::new(150, 156, 164); // #969ca4
/// Chart gridlines — subtle, alpha applied at draw time.
pub const GRID_ALPHA: f64 = 0.06;

// ── Metric accents (flat, but lifted for contrast on the near-black card) ─────

pub const CPU: Color = Color::new(108, 178, 255); // blue    #6cb2ff
pub const MEM: Color = Color::new(74, 214, 164); // green    #4ad6a4
pub const NET_RX: Color = Color::new(240, 192, 96); // amber  #f0c060
pub const NET_TX: Color = Color::new(240, 146, 112); // orange #f09270
pub const DISK: Color = Color::new(200, 150, 224); // purple  #c896e0
pub const GPU: Color = Color::new(170, 166, 248); // indigo   #aaa6f8

// ── Temperature (graduated: normal → warn → critical) ────────────────────────

pub const TEMP: Color = Color::new(232, 160, 110); // warm normal #e8a06e
pub const TEMP_WARN: Color = Color::new(242, 178, 62); // amber   #f2b23e
pub const TEMP_CRIT: Color = Color::new(242, 96, 96); // red      #f26060

/// Pick a temperature color by thresholds (°C): ≥80 critical, ≥60 warning.
pub fn temp_color(celsius: f64) -> Color {
    if celsius >= 80.0 {
        TEMP_CRIT
    } else if celsius >= 60.0 {
        TEMP_WARN
    } else {
        TEMP
    }
}

/// Default card CSS for `.monitor-window`, derived from the tokens above.
/// Used as the `BackgroundType::None` baseline so the flat look is centralized.
pub fn card_css() -> String {
    let (r, g, b) = (BG.r, BG.g, BG.b);
    format!(
        ".monitor-window {{ background-color: rgba({r}, {g}, {b}, 0.94); \
         border: 1px solid {BORDER_RGBA}; border-radius: {RADIUS_PX}px; \
         padding: {PADDING_PX}px; }}"
    )
}
