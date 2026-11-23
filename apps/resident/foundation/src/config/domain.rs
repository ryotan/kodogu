use serde::{Deserialize, Serialize};

// =============================================================================
// Value types
// =============================================================================

macro_rules! newtype_string {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) String);

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
    };
}

macro_rules! newtype_u64 {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) u64);
    };
}

macro_rules! newtype_f64 {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
        #[serde(transparent)]
        pub(crate) struct $name(pub(crate) f64);
    };
}

newtype_u64! {
    /// Duration in milliseconds.
    Milliseconds
}

newtype_f64! {
    /// Device-independent logical pixels.
    LogicalPixel
}

newtype_string! {
    /// A CSS `<color>` value (e.g. `"rgba(28, 28, 30, 0.92)"`).
    CssColor
}

newtype_string! {
    /// A CSS `<filter-function-list>` value (e.g. `"blur(24px) saturate(180%)"`).
    CssFilter
}

newtype_string! {
    /// A CSS `font-family` value (e.g. `"-apple-system, BlinkMacSystemFont, sans-serif"`).
    CssFontFamily
}

newtype_string! {
    /// A single CSS `<length>` value (e.g. `"40px"`).
    CssLength
}

newtype_string! {
    /// A CSS `padding` shorthand value (e.g. `"6px 0"`).
    CssPadding
}

// =============================================================================
// Config types
// =============================================================================

/// Timing and interaction settings for the popover.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverBehaviorConfig {
    /// Debounce interval for tray icon clicks.
    pub(crate) toggle_debounce_ms: Milliseconds,
}

impl Default for PopoverBehaviorConfig {
    fn default() -> Self {
        Self { toggle_debounce_ms: Milliseconds(300) }
    }
}

/// Geometry and sizing settings for the popover.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverLayoutConfig {
    /// Initial popover window width.
    pub(crate) initial_width: LogicalPixel,
    /// Minimum gap between the popover bottom and the screen bottom edge (macOS only).
    pub(crate) bottom_margin: LogicalPixel,
    /// Gap between the tray icon bottom edge and the popover top edge (macOS only).
    pub(crate) tray_gap: LogicalPixel,
    /// Fallback max height used when monitor info is unavailable.
    pub(crate) fallback_max_height: LogicalPixel,
    /// Lower bound for the computed max height, ensuring a minimum usable height.
    pub(crate) min_max_height: LogicalPixel,
    /// Height of the header bar.
    pub(crate) header_height: CssLength,
    /// Padding for the card list container.
    pub(crate) card_list_padding: CssPadding,
}

impl Default for PopoverLayoutConfig {
    fn default() -> Self {
        Self {
            initial_width: LogicalPixel(480.0),
            bottom_margin: LogicalPixel(8.0),
            tray_gap: LogicalPixel(4.0),
            fallback_max_height: LogicalPixel(600.0),
            min_max_height: LogicalPixel(200.0),
            header_height: CssLength::from("40px"),
            card_list_padding: CssPadding::from("6px 0"),
        }
    }
}

/// Visual design tokens for the popover shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverAppearanceConfig {
    /// Background color of the popover window.
    pub(crate) surface_color: CssColor,
    /// Border color used for the header bottom separator.
    pub(crate) border_color: CssColor,
    /// Backdrop filter applied to the popover for a frosted glass effect.
    pub(crate) backdrop_filter: CssFilter,
    /// Font family for the entire popover.
    pub(crate) font_family: CssFontFamily,
}

impl Default for PopoverAppearanceConfig {
    fn default() -> Self {
        Self {
            surface_color: CssColor::from("rgba(28, 28, 30, 0.92)"),
            border_color: CssColor::from("rgba(255, 255, 255, 0.08)"),
            backdrop_filter: CssFilter::from("blur(24px) saturate(180%)"),
            font_family: CssFontFamily::from(
                "-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif",
            ),
        }
    }
}

/// All settings for the popover window.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PopoverConfig {
    pub(crate) behavior: PopoverBehaviorConfig,
    pub(crate) layout: PopoverLayoutConfig,
    pub(crate) appearance: PopoverAppearanceConfig,
}

/// Application configuration persisted in redb and exposed to JavaScript via Tauri commands.
///
/// All values are serialized with camelCase field names (for both storage and JS interop).
/// Add new fields at any time — missing fields in stored data are filled from `Default::default()`.
///
/// ## Schema evolution
/// `#[serde(default)]` on each struct enables forward-compatible field additions.
/// Renaming or changing field types requires bumping `StoredAppConfig` to a new variant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct AppConfig {
    pub(crate) popover: PopoverConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Default values — full coverage of all config fields
    // -------------------------------------------------------------------------

    #[test]
    fn default_popover_behavior_config() {
        let config = PopoverBehaviorConfig::default();
        assert_eq!(config.toggle_debounce_ms, Milliseconds(300));
    }

    #[test]
    fn default_popover_layout_config() {
        let config = PopoverLayoutConfig::default();
        assert_eq!(config.initial_width, LogicalPixel(480.0));
        assert_eq!(config.bottom_margin, LogicalPixel(8.0));
        assert_eq!(config.tray_gap, LogicalPixel(4.0));
        assert_eq!(config.fallback_max_height, LogicalPixel(600.0));
        assert_eq!(config.min_max_height, LogicalPixel(200.0));
        assert_eq!(config.header_height, CssLength::from("40px"));
        assert_eq!(config.card_list_padding, CssPadding::from("6px 0"));
    }

    #[test]
    fn default_popover_appearance_config() {
        let config = PopoverAppearanceConfig::default();
        assert_eq!(config.surface_color, CssColor::from("rgba(28, 28, 30, 0.92)"));
        assert_eq!(config.border_color, CssColor::from("rgba(255, 255, 255, 0.08)"));
        assert_eq!(config.backdrop_filter, CssFilter::from("blur(24px) saturate(180%)"));
        assert_eq!(
            config.font_family,
            CssFontFamily::from("-apple-system, BlinkMacSystemFont, 'Helvetica Neue', sans-serif")
        );
    }

    // -------------------------------------------------------------------------
    // Newtype serde roundtrip — #[serde(transparent)]
    // -------------------------------------------------------------------------

    #[test]
    fn newtype_u64_json_roundtrip() {
        let ms = Milliseconds(42);
        let json = serde_json::to_string(&ms).expect("serialize");
        assert_eq!(json, "42");
        let decoded: Milliseconds = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, ms);
    }

    #[test]
    fn newtype_f64_json_roundtrip() {
        let px = LogicalPixel(12.5);
        let json = serde_json::to_string(&px).expect("serialize");
        assert_eq!(json, "12.5");
        let decoded: LogicalPixel = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, px);
    }

    #[test]
    fn newtype_string_json_roundtrip() {
        let color = CssColor::from("rgba(0, 0, 0, 1)");
        let json = serde_json::to_string(&color).expect("serialize");
        assert_eq!(json, r#""rgba(0, 0, 0, 1)""#);
        let decoded: CssColor = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, color);
    }

    // -------------------------------------------------------------------------
    // serde(default) — missing fields filled from Default
    // -------------------------------------------------------------------------

    #[test]
    fn missing_fields_filled_from_default() {
        let config: AppConfig = serde_json::from_str("{}").expect("deserialize");
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(300));
        assert_eq!(config.popover.layout.initial_width, LogicalPixel(480.0));
        assert_eq!(
            config.popover.appearance.surface_color,
            CssColor::from("rgba(28, 28, 30, 0.92)")
        );
    }

    #[test]
    fn partial_fields_merged_with_defaults() {
        let json = r#"{"popover": {"behavior": {"toggleDebounceMs": 500}}}"#;
        let config: AppConfig = serde_json::from_str(json).expect("deserialize");
        assert_eq!(config.popover.behavior.toggle_debounce_ms, Milliseconds(500));
        assert_eq!(config.popover.layout.initial_width, LogicalPixel(480.0));
        assert_eq!(
            config.popover.appearance.surface_color,
            CssColor::from("rgba(28, 28, 30, 0.92)")
        );
    }
}
