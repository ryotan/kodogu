use super::*;
use proptest::prelude::*;
use serde_json::{Value, json};

fn full_request() -> Value {
    serde_json::to_value(AppConfig::default()).unwrap()
}

macro_rules! missing_fields {
    ($($name:ident: $parent:literal, $field:literal),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let mut request = full_request();
            request.pointer_mut($parent).unwrap().as_object_mut().unwrap().remove($field);
            let error = translate_request(request).unwrap_err();
            assert!(matches!(error, RequestError::Shape { .. }), "{error:?}");
        }
    )+};
}

missing_fields! {
    missing_popover_does_not_apply_storage_defaults: "", "popover",
    missing_behavior_is_rejected: "/popover", "behavior",
    missing_layout_is_rejected: "/popover", "layout",
    missing_appearance_is_rejected: "/popover", "appearance",
    missing_debounce_is_rejected: "/popover/behavior", "toggleDebounceMs",
    missing_width_is_rejected: "/popover/layout", "initialWidth",
    missing_bottom_margin_is_rejected: "/popover/layout", "bottomMargin",
    missing_tray_gap_is_rejected: "/popover/layout", "trayGap",
    missing_fallback_height_is_rejected: "/popover/layout", "fallbackMaxHeight",
    missing_minimum_height_is_rejected: "/popover/layout", "minMaxHeight",
    missing_header_height_is_rejected: "/popover/layout", "headerHeight",
    missing_card_padding_is_rejected: "/popover/layout", "cardListPadding",
    missing_surface_color_is_rejected: "/popover/appearance", "surfaceColor",
    missing_border_color_is_rejected: "/popover/appearance", "borderColor",
    missing_backdrop_filter_is_rejected: "/popover/appearance", "backdropFilter",
    missing_font_family_is_rejected: "/popover/appearance", "fontFamily",
}

macro_rules! unexpected_fields {
    ($($name:ident: $parent:literal),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let mut request = full_request();
            request.pointer_mut($parent).unwrap().as_object_mut().unwrap().insert("privateUnexpectedValue".into(), json!("secret"));
            assert!(matches!(translate_request(request).unwrap_err(), RequestError::Shape { .. }));
        }
    )+};
}

unexpected_fields! {
    an_unknown_root_field_is_rejected: "",
    an_unknown_popover_field_is_rejected: "/popover",
    an_unknown_behavior_field_is_rejected: "/popover/behavior",
    an_unknown_layout_field_is_rejected: "/popover/layout",
    an_unknown_appearance_field_is_rejected: "/popover/appearance",
}

macro_rules! positional_arrays {
    ($($name:ident: $path:literal, [$($field:literal),+]),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let mut request = full_request();
            let object = request.pointer_mut($path).unwrap();
            *object = json!([$ (object[$field].clone()),+]);
            assert!(matches!(translate_request(request).unwrap_err(), RequestError::NotObject { path: $path, .. }));
        }
    )+};
}

positional_arrays! {
    a_positional_root_array_cannot_replace_the_named_object: "", ["popover"],
    a_positional_popover_array_cannot_replace_the_named_object: "/popover", ["behavior", "layout", "appearance"],
    a_positional_behavior_array_cannot_replace_the_named_object: "/popover/behavior", ["toggleDebounceMs"],
    a_positional_layout_array_cannot_replace_the_named_object: "/popover/layout", ["initialWidth", "bottomMargin", "trayGap", "fallbackMaxHeight", "minMaxHeight", "headerHeight", "cardListPadding"],
    a_positional_appearance_array_cannot_replace_the_named_object: "/popover/appearance", ["surfaceColor", "borderColor", "backdropFilter", "fontFamily"],
}

macro_rules! malformed_fields {
    ($($name:ident: $path:literal, $value:expr => $error:ident),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let mut request = full_request();
            *request.pointer_mut($path).unwrap() = $value;
            assert!(matches!(translate_request(request).unwrap_err(), RequestError::$error { .. }));
        }
    )+};
}

malformed_fields! {
    null_root_is_rejected: "", Value::Null => NotObject,
    array_root_is_rejected: "", json!([]) => NotObject,
    null_geometry_is_rejected: "/popover/layout/initialWidth", Value::Null => Shape,
    string_geometry_is_rejected: "/popover/layout/initialWidth", json!("480") => Shape,
    boolean_duration_is_rejected: "/popover/behavior/toggleDebounceMs", json!(true) => Shape,
    string_duration_is_rejected: "/popover/behavior/toggleDebounceMs", json!("300") => Shape,
    numeric_css_is_rejected: "/popover/appearance/surfaceColor", json!(300) => Shape,
}

macro_rules! durations {
    ($($name:ident: $value:expr => $expected:expr),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let mut request = full_request();
            request["popover"]["behavior"]["toggleDebounceMs"] = json!($value);
            let translated = translate_request(request);
            match $expected {
                Some(expected) => assert_eq!(translated.unwrap().as_ref().popover.behavior.toggle_debounce_ms, Milliseconds(expected)),
                None => assert!(matches!(translated.unwrap_err(), RequestError::UnsafeDuration { .. })),
            }
        }
    )+};
}

durations! {
    zero_duration_is_translated: 0 => Some(0_u64),
    a_decimal_integer_is_translated_without_rounding: 300.0 => Some(300_u64),
    below_the_javascript_integer_limit_is_translated: 9_007_199_254_740_990_u64 => Some(9_007_199_254_740_990_u64),
    the_javascript_integer_limit_is_translated: 9_007_199_254_740_991_u64 => Some(9_007_199_254_740_991_u64),
    above_the_javascript_integer_limit_is_rejected: 9_007_199_254_740_992_u64 => None::<u64>,
    the_native_duration_maximum_is_rejected_at_the_javascript_boundary: u64::MAX => None::<u64>,
    a_negative_duration_is_rejected: -1 => None::<u64>,
    a_fractional_duration_is_rejected: 0.5 => None::<u64>,
}

#[test]
fn a_complete_request_preserves_the_entire_wire_shape() {
    let mut request = full_request();
    request["popover"]["layout"]["bottomMargin"] = json!(-8.0);
    request["popover"]["layout"]["trayGap"] = json!(-4.0);
    let update = translate_request(request.clone()).unwrap();
    let response = ConfigResponse::try_from(update.into_inner()).unwrap();
    assert_eq!(serde_json::to_value(response).unwrap(), request);
}

#[test]
fn new_update_geometry_failures_keep_the_typed_domain_cause() {
    let mut request = full_request();
    request["popover"]["layout"]["initialWidth"] = json!(0);
    let error = translate_request(request).unwrap_err();
    assert!(matches!(
        error,
        RequestError::Validation {
            source: ConfigValidationError {
                field: GeometryField::InitialWidth,
                rule: GeometryRule::Positive,
                ..
            },
            ..
        }
    ));
}

#[test]
fn finite_legacy_geometry_remains_readable_without_new_update_validation() {
    let mut config = AppConfig::default();
    config.popover.layout.initial_width = LogicalPixel(0.0);
    config.popover.layout.fallback_max_height = LogicalPixel(-1.0);
    config.popover.layout.min_max_height = LogicalPixel(-2.0);
    let expected = serde_json::to_value(&config).unwrap();
    assert_eq!(
        serde_json::to_value(ConfigResponse::try_from(config).unwrap()).unwrap(),
        expected
    );
}

macro_rules! unsafe_responses {
    ($($name:ident: $field:ident, $value:expr),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let mut config = AppConfig::default();
            config.popover.layout.$field = LogicalPixel($value);
            assert!(matches!(ConfigResponse::try_from(config).unwrap_err(), ResponseError::NonFiniteGeometry { .. }));
        }
    )+};
}

unsafe_responses! {
    nan_width_does_not_become_null: initial_width, f64::NAN,
    infinite_width_does_not_become_null: initial_width, f64::INFINITY,
    negative_infinite_width_does_not_become_null: initial_width, f64::NEG_INFINITY,
    nan_margin_does_not_become_null: bottom_margin, f64::NAN,
    nan_gap_does_not_become_null: tray_gap, f64::NAN,
    infinite_fallback_does_not_become_null: fallback_max_height, f64::INFINITY,
    negative_infinite_minimum_does_not_become_null: min_max_height, f64::NEG_INFINITY,
}

macro_rules! response_durations {
    ($($name:ident: $value:expr => $allowed:literal),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let mut config = AppConfig::default();
            config.popover.behavior.toggle_debounce_ms = Milliseconds($value);
            let result = ConfigResponse::try_from(config);
            assert_eq!(result.is_ok(), $allowed);
            if !$allowed { assert!(matches!(result.unwrap_err(), ResponseError::UnsafeInteger { .. })); }
        }
    )+};
}

response_durations! {
    a_response_below_the_integer_limit_is_safe: 9_007_199_254_740_990 => true,
    a_response_at_the_integer_limit_is_safe: 9_007_199_254_740_991 => true,
    a_response_above_the_integer_limit_is_rejected: 9_007_199_254_740_992 => false,
    a_native_maximum_duration_is_not_rounded_for_javascript: u64::MAX => false,
}

proptest! {
    #[test]
    fn safe_duration_translation_is_exact(value in 0_u64..=9_007_199_254_740_991) {
        let mut request = full_request();
        request["popover"]["behavior"]["toggleDebounceMs"] = json!(value);
        let actual = translate_request(request).unwrap().into_inner().popover.behavior.toggle_debounce_ms;
        prop_assert_eq!(actual, Milliseconds(value));
    }
}
