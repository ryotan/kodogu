use serde::{Deserialize, Serialize};
use serde_json::Value;
use suzunari_error::*;

use super::domain::*;

// This is a JavaScript interface constraint, not a native duration or storage limit.
const MAX_SAFE_INTEGER: u64 = (1_u64 << 53) - 1;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ConfigRequest {
    popover: PopoverRequest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct PopoverRequest {
    behavior: BehaviorRequest,
    layout: LayoutRequest,
    appearance: AppearanceRequest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct BehaviorRequest {
    toggle_debounce_ms: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct LayoutRequest {
    initial_width: LogicalPixel,
    bottom_margin: LogicalPixel,
    tray_gap: LogicalPixel,
    fallback_max_height: LogicalPixel,
    min_max_height: LogicalPixel,
    header_height: CssLength,
    card_list_padding: CssPadding,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct AppearanceRequest {
    surface_color: CssColor,
    border_color: CssColor,
    backdrop_filter: CssFilter,
    font_family: CssFontFamily,
}

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum RequestError {
    #[suzu(display("Configuration request requires a named object at {path}"))]
    NotObject { path: &'static str },
    #[suzu(display("Invalid configuration request shape"))]
    Shape { source: serde_json::Error },
    #[suzu(display("Configuration duration is not a safe nonnegative JavaScript integer"))]
    UnsafeDuration,
    #[suzu(display("Configuration update violates a domain rule"))]
    Validation { source: ConfigValidationError },
}

pub(crate) fn translate_request(value: Value) -> Result<ConfigUpdate, RequestError> {
    require_named_objects(&value)?;
    let request: ConfigRequest = serde_json::from_value(value).context(ShapeError)?;
    let debounce = request.popover.behavior.toggle_debounce_ms;
    ensure!(
        debounce >= 0.0 && debounce <= MAX_SAFE_INTEGER as f64 && debounce.fract() == 0.0,
        UnsafeDurationError
    );
    ConfigUpdate::try_from(AppConfig {
        popover: PopoverConfig {
            behavior: PopoverBehaviorConfig { toggle_debounce_ms: Milliseconds(debounce as u64) },
            layout: request.popover.layout.into(),
            appearance: request.popover.appearance.into(),
        },
    })
    .context(ValidationError)
}

fn require_named_objects(value: &Value) -> Result<(), RequestError> {
    // Serde's struct visitor also accepts positional arrays; this interface requires named objects.
    for path in ["", "/popover", "/popover/behavior", "/popover/layout", "/popover/appearance"] {
        if let Some(field) = value.pointer(path) {
            ensure!(field.is_object(), NotObjectError { path });
        }
    }
    Ok(())
}

impl From<LayoutRequest> for PopoverLayoutConfig {
    fn from(request: LayoutRequest) -> Self {
        Self {
            initial_width: request.initial_width,
            bottom_margin: request.bottom_margin,
            tray_gap: request.tray_gap,
            fallback_max_height: request.fallback_max_height,
            min_max_height: request.min_max_height,
            header_height: request.header_height,
            card_list_padding: request.card_list_padding,
        }
    }
}

impl From<AppearanceRequest> for PopoverAppearanceConfig {
    fn from(request: AppearanceRequest) -> Self {
        Self {
            surface_color: request.surface_color,
            border_color: request.border_color,
            backdrop_filter: request.backdrop_filter,
            font_family: request.font_family,
        }
    }
}

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub(crate) enum ResponseError {
    #[suzu(display("Saved {field} is not finite"))]
    NonFiniteGeometry { field: GeometryField },
    #[suzu(display("Saved duration is not a safe JavaScript integer"))]
    UnsafeInteger,
}

/// Only values safely representable by this JavaScript interface can be serialized.
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub(crate) struct ConfigResponse(AppConfig);

impl TryFrom<AppConfig> for ConfigResponse {
    type Error = ResponseError;

    fn try_from(config: AppConfig) -> Result<Self, Self::Error> {
        for (field, value) in config.popover.layout.geometry() {
            ensure!(value.0.is_finite(), NonFiniteGeometryError { field });
        }
        ensure!(
            config.popover.behavior.toggle_debounce_ms.0 <= MAX_SAFE_INTEGER,
            UnsafeIntegerError
        );
        Ok(Self(config))
    }
}

#[cfg(test)]
#[path = "translators/tests.rs"]
mod tests;
