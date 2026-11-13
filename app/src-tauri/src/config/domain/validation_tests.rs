use super::*;
use proptest::prelude::*;

fn config_with(field: GeometryField, value: f64) -> AppConfig {
    let mut config = AppConfig::default();
    let layout = &mut config.popover.layout;
    match field {
        GeometryField::InitialWidth => layout.initial_width = LogicalPixel(value),
        GeometryField::BottomMargin => layout.bottom_margin = LogicalPixel(value),
        GeometryField::TrayGap => layout.tray_gap = LogicalPixel(value),
        GeometryField::FallbackMaxHeight => layout.fallback_max_height = LogicalPixel(value),
        GeometryField::MinMaxHeight => layout.min_max_height = LogicalPixel(value),
    }
    config
}

macro_rules! rejected_geometry {
    ($($name:ident: $field:ident, $value:expr, $rule:ident),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let error = ConfigUpdate::try_from(config_with(GeometryField::$field, $value)).unwrap_err();
            assert!(matches!(error, ConfigValidationError {
                field: GeometryField::$field, rule: GeometryRule::$rule, ..
            }), "{error:?}");
            assert_eq!(error.to_string(), format!("Invalid {}: {}", GeometryField::$field, GeometryRule::$rule));
        }
    )+};
}

rejected_geometry! {
    zero_initial_width_is_rejected: InitialWidth, 0.0, Positive,
    negative_initial_width_is_rejected: InitialWidth, -1.0, Positive,
    zero_fallback_height_is_rejected: FallbackMaxHeight, 0.0, Positive,
    negative_fallback_height_is_rejected: FallbackMaxHeight, -1.0, Positive,
    zero_minimum_height_is_rejected: MinMaxHeight, 0.0, Positive,
    negative_minimum_height_is_rejected: MinMaxHeight, -1.0, Positive,
    nan_initial_width_is_rejected: InitialWidth, f64::NAN, Finite,
    infinite_initial_width_is_rejected: InitialWidth, f64::INFINITY, Finite,
    negative_infinite_initial_width_is_rejected: InitialWidth, f64::NEG_INFINITY, Finite,
    nan_bottom_margin_is_rejected: BottomMargin, f64::NAN, Finite,
    infinite_bottom_margin_is_rejected: BottomMargin, f64::INFINITY, Finite,
    negative_infinite_bottom_margin_is_rejected: BottomMargin, f64::NEG_INFINITY, Finite,
    nan_tray_gap_is_rejected: TrayGap, f64::NAN, Finite,
    infinite_tray_gap_is_rejected: TrayGap, f64::INFINITY, Finite,
    negative_infinite_tray_gap_is_rejected: TrayGap, f64::NEG_INFINITY, Finite,
    nan_fallback_height_is_rejected: FallbackMaxHeight, f64::NAN, Finite,
    infinite_fallback_height_is_rejected: FallbackMaxHeight, f64::INFINITY, Finite,
    negative_infinite_fallback_height_is_rejected: FallbackMaxHeight, f64::NEG_INFINITY, Finite,
    nan_minimum_height_is_rejected: MinMaxHeight, f64::NAN, Finite,
    infinite_minimum_height_is_rejected: MinMaxHeight, f64::INFINITY, Finite,
    negative_infinite_minimum_height_is_rejected: MinMaxHeight, f64::NEG_INFINITY, Finite,
}

macro_rules! accepted_geometry {
    ($($name:ident: $field:ident, $value:expr),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let config = config_with(GeometryField::$field, $value);
            let expected = serde_json::to_value(&config).unwrap();
            let update = ConfigUpdate::try_from(config).unwrap();
            assert_eq!(serde_json::to_value(update.into_inner()).unwrap(), expected);
        }
    )+};
}

accepted_geometry! {
    a_tiny_positive_width_is_allowed: InitialWidth, f64::from_bits(1),
    the_largest_finite_width_is_allowed: InitialWidth, f64::MAX,
    a_tiny_positive_fallback_height_is_allowed: FallbackMaxHeight, f64::from_bits(1),
    the_largest_finite_fallback_height_is_allowed: FallbackMaxHeight, f64::MAX,
    a_tiny_positive_minimum_height_is_allowed: MinMaxHeight, f64::from_bits(1),
    the_largest_finite_minimum_height_is_allowed: MinMaxHeight, f64::MAX,
    zero_bottom_margin_is_allowed: BottomMargin, 0.0,
    a_negative_bottom_margin_is_allowed: BottomMargin, -f64::MAX,
    zero_tray_gap_is_allowed: TrayGap, 0.0,
    a_negative_tray_gap_is_allowed: TrayGap, -f64::MAX,
}

#[test]
fn native_updates_keep_the_full_duration_range_and_opaque_css() {
    let mut config = AppConfig::default();
    config.popover.behavior.toggle_debounce_ms = Milliseconds(u64::MAX);
    config.popover.layout.header_height = CssLength::from("");
    config.popover.layout.card_list_padding = CssPadding::from("arbitrary CSS token");
    config.popover.appearance.surface_color = CssColor::from("inherit");
    let expected = serde_json::to_value(&config).unwrap();
    assert_eq!(
        serde_json::to_value(ConfigUpdate::try_from(config).unwrap().into_inner()).unwrap(),
        expected
    );
}

#[test]
fn zero_debounce_is_allowed() {
    let mut config = AppConfig::default();
    config.popover.behavior.toggle_debounce_ms = Milliseconds(0);
    assert_eq!(
        ConfigUpdate::try_from(config)
            .unwrap()
            .into_inner()
            .popover
            .behavior
            .toggle_debounce_ms,
        Milliseconds(0)
    );
}

#[test]
fn the_height_fields_have_no_new_ordering_constraint() {
    let mut config = AppConfig::default();
    config.popover.layout.fallback_max_height = LogicalPixel(1.0);
    config.popover.layout.min_max_height = LogicalPixel(1000.0);
    assert!(ConfigUpdate::try_from(config).is_ok());
}

proptest! {
    #[test]
    fn every_finite_gap_is_accepted(value in any::<f64>().prop_filter("finite", |value| value.is_finite())) {
        prop_assert!(ConfigUpdate::try_from(config_with(GeometryField::TrayGap, value)).is_ok());
        prop_assert!(ConfigUpdate::try_from(config_with(GeometryField::BottomMargin, value)).is_ok());
    }

    #[test]
    fn the_field_validator_and_update_constructor_agree(value in any::<f64>(), index in 0_usize..5) {
        let fields = [GeometryField::InitialWidth, GeometryField::BottomMargin, GeometryField::TrayGap,
            GeometryField::FallbackMaxHeight, GeometryField::MinMaxHeight];
        let field = fields[index];
        prop_assert_eq!(field.validate(LogicalPixel(value)).is_ok(), ConfigUpdate::try_from(config_with(field, value)).is_ok());
    }
}
