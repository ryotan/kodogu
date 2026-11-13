use kodogu_runtime::{
    ToolId,
    messages::{
        Argument, Catalog, CatalogError, CatalogResource, Locale, Message, Namespace, Preferences,
        RenderedMessage,
    },
};
use proptest::prelude::*;

fn application() -> Namespace {
    Namespace::Application("runtime-test".to_owned())
}

fn locale(tag: &str) -> Locale {
    tag.parse().unwrap()
}

fn preferences(requested: &[&str], fallback: &[&str]) -> Preferences {
    Preferences::new(
        requested.iter().map(|tag| locale(tag)).collect(),
        fallback.iter().map(|tag| locale(tag)).collect(),
    )
}

fn resource(namespace: Namespace, tag: &str, source: &str) -> CatalogResource {
    CatalogResource { namespace, locale: locale(tag), source: source.to_owned() }
}

fn catalog() -> Catalog {
    Catalog::try_from_resources([
        resource(application(), "en", "greeting = Hello\nenglish-only = English resource\n"),
        resource(application(), "ja", "greeting = こんにちは\n"),
    ])
    .unwrap()
}

macro_rules! languages {
    ($($name:ident: $requested:expr, $fallback:expr, $expected:literal, $actual:literal),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let message = Message::localized(application(), "greeting", "Emergency English");
            let rendered = catalog().render(&message, &preferences($requested, $fallback));
            assert_eq!(rendered.text, $expected);
            assert_eq!(rendered.locale, Some(locale($actual)));
        }
    )+};
}

languages! {
    the_requested_language_is_rendered: &["ja"], &["en"], "こんにちは", "ja",
    english_is_available_as_a_normal_catalog: &["en"], &[], "Hello", "en",
    a_region_falls_back_to_its_language_catalog: &["ja-JP"], &["en"], "こんにちは", "ja",
    unicode_extensions_survive_request_matching: &["ja-JP-u-ca-japanese"], &["en"], "こんにちは", "ja",
    an_unsupported_request_uses_the_host_fallback: &["fr"], &["ja", "en"], "こんにちは", "ja",
    request_order_is_preserved: &["en", "ja"], &[], "Hello", "en",
    an_unsupported_request_without_host_fallback_uses_the_english_catalog: &["fr"], &[], "Hello", "en",
}

#[test]
fn regional_preference_uses_the_compatible_script_or_the_next_language() {
    let message = Message::localized(application(), "greeting", "Emergency English");
    let preferences = preferences(&["zh-TW", "ja"], &[]);
    let simplified = resource(application(), "zh-Hans", "greeting = 你好（简体）\n");
    let japanese = resource(application(), "ja", "greeting = こんにちは\n");
    let traditional = resource(application(), "zh-Hant", "greeting = 你好（繁體）\n");

    let without_traditional =
        Catalog::try_from_resources([simplified.clone(), japanese.clone()]).unwrap();
    let with_traditional =
        Catalog::try_from_resources([simplified, japanese, traditional]).unwrap();

    assert_eq!(
        without_traditional.render(&message, &preferences),
        RenderedMessage { text: "こんにちは".into(), locale: Some(locale("ja")) },
    );
    assert_eq!(
        with_traditional.render(&message, &preferences),
        RenderedMessage { text: "你好（繁體）".into(), locale: Some(locale("zh-Hant")) },
    );
}

#[test]
fn a_missing_message_uses_the_explicit_english_catalog() {
    let message = Message::localized(application(), "english-only", "Emergency English");
    let rendered = catalog().render(&message, &preferences(&["ja"], &["en"]));
    assert_eq!(rendered.text, "English resource");
    assert_eq!(rendered.locale, Some(Locale::english()));
}

#[test]
fn a_catalog_is_not_needed_for_the_safe_english_fallback() {
    let message = Message::localized(application(), "missing", "Safe bootstrap message");
    let rendered = Catalog::default().render(&message, &Preferences::default());
    assert_eq!(rendered.text, "Safe bootstrap message");
    assert_eq!(rendered.locale, Some(Locale::english()));
}

#[test]
fn an_excluded_base_language_cannot_return_through_region_fallback() {
    let preferences = preferences(&["ja-JP"], &["en"]).excluding(vec![locale("ja")]);
    let message = Message::localized(application(), "greeting", "Emergency English");
    let rendered = catalog().render(&message, &preferences);
    assert_eq!(rendered.text, "Hello");
    assert_eq!(rendered.locale, Some(Locale::english()));
}

#[test]
fn excluded_catalogs_do_not_remove_the_descriptor_emergency_fallback() {
    let preferences = preferences(&["ja-JP"], &["en"]).excluding(vec![locale("ja"), locale("en")]);
    let message = Message::localized(application(), "greeting", "Emergency English");
    let rendered = catalog().render(&message, &preferences);
    assert_eq!(rendered.text, "Emergency English");
    assert_eq!(rendered.locale, Some(Locale::english()));
}

#[test]
fn application_and_tool_namespaces_cannot_collide() {
    let tool = Namespace::Tool(ToolId::try_from("runtime-test").unwrap());
    let catalog = Catalog::try_from_resources([
        resource(application(), "en", "greeting = Application\n"),
        resource(tool.clone(), "en", "greeting = Tool\n"),
    ])
    .unwrap();
    assert_eq!(
        catalog
            .render(
                &Message::localized(application(), "greeting", "Fallback"),
                &Preferences::default()
            )
            .text,
        "Application"
    );
    assert_eq!(
        catalog
            .render(&Message::localized(tool, "greeting", "Fallback"), &Preferences::default())
            .text,
        "Tool"
    );
}

#[test]
fn resources_for_one_namespace_and_language_can_be_split_across_files() {
    let catalog = Catalog::try_from_resources([
        resource(application(), "en", "first = First\n"),
        resource(application(), "en", "second = Second\n"),
    ])
    .unwrap();
    assert_eq!(
        catalog
            .render(
                &Message::localized(application(), "second", "Fallback"),
                &Preferences::default()
            )
            .text,
        "Second"
    );
}

macro_rules! invalid_resources {
    ($($name:ident: $variant:ident, $sources:expr),+ $(,)?) => {$(
        #[test]
        fn $name() {
            match Catalog::try_from_resources($sources) {
                Err(CatalogError::$variant { source, namespace, locale: actual, .. }) => {
                    assert!(source.issue_count() > 0);
                    assert_eq!(namespace, application());
                    assert_eq!(actual, Locale::english());
                }
                _ => panic!("Expected the typed resource failure with its namespace and locale"),
            }
        }
    )+};
}

invalid_resources! {
    malformed_resource_syntax_is_rejected: Parse, [resource(application(), "en", "greeting = {\n")],
    duplicate_messages_in_one_resource_are_rejected: Duplicate, [resource(application(), "en", "greeting = First\ngreeting = Second\n")],
    duplicate_messages_across_resources_are_rejected: Duplicate, [resource(application(), "en", "greeting = First\n"), resource(application(), "en", "greeting = Second\n")],
}

#[test]
fn available_locales_are_unique_and_do_not_depend_on_one_namespace() {
    let mut resources = vec![resource(application(), "en", "greeting = Hello\n")];
    resources.push(resource(
        Namespace::Tool(ToolId::try_from("other").unwrap()),
        "en",
        "greeting = Tool\n",
    ));
    resources.push(resource(application(), "ja", "greeting = こんにちは\n"));
    let actual = Catalog::try_from_resources(resources).unwrap().locales();
    assert_eq!(actual, vec![locale("en"), locale("ja")]);
}

#[test]
fn a_complete_locale_preserves_unicode_and_private_extensions() {
    let tag = "en-US-u-ca-gregory-nu-latn-x-kodogu";
    let parsed = locale(tag);
    assert_eq!(parsed.to_string(), tag);
    assert_eq!(parsed.to_string().parse::<Locale>().unwrap(), parsed);
}

#[test]
fn malformed_locale_input_returns_a_typed_failure() {
    assert!("private invalid locale".parse::<Locale>().is_err());
}

#[test]
fn preferences_expose_ordered_borrowed_tags() {
    let preferences = preferences(&["ja-JP", "en-GB"], &["en"]).excluding(vec![locale("fr")]);
    assert_eq!(preferences.requested(), &[locale("ja-JP"), locale("en-GB")]);
    assert_eq!(preferences.fallback(), &[Locale::english()]);
    assert_eq!(preferences.excluded(), &[locale("fr")]);
}

#[test]
fn text_arguments_keep_bidirectional_isolation() {
    let catalog =
        Catalog::try_from_resources([resource(application(), "ja", "greeting = { $name }さん\n")])
            .unwrap();
    let message = Message::localized_with_arguments(
        application(),
        "greeting",
        "Safe fallback",
        [("name".into(), Argument::Text("أحمد".to_owned()))],
    );
    let rendered = catalog.render(&message, &preferences(&["ja"], &[]));
    assert_eq!(rendered.text, "\u{2068}أحمد\u{2069}さん");
    assert_eq!(rendered.locale, Some(locale("ja")));
}

#[test]
fn missing_arguments_do_not_publish_partial_formatter_output() {
    let catalog = Catalog::try_from_resources([resource(
        application(),
        "ja",
        "greeting = { $missing }さん\n",
    )])
    .unwrap();
    let message = Message::localized(application(), "greeting", "Safe fallback");
    let rendered = catalog.render(&message, &preferences(&["ja"], &[]));
    assert_eq!(rendered.text, "Safe fallback");
    assert_eq!(rendered.locale, Some(Locale::english()));
}

#[test]
fn a_formatter_failure_tries_the_next_catalog_before_the_literal_fallback() {
    let catalog = Catalog::try_from_resources([
        resource(application(), "ja", "greeting = { $missing }さん\n"),
        resource(application(), "en", "greeting = Safe English catalog\n"),
    ])
    .unwrap();
    let rendered = catalog.render(
        &Message::localized(application(), "greeting", "Emergency fallback"),
        &preferences(&["ja"], &["en"]),
    );
    assert_eq!(rendered.text, "Safe English catalog");
    assert_eq!(rendered.locale, Some(Locale::english()));
}

macro_rules! numeric_selection {
    ($($name:ident: $value:expr, $expected:literal),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let catalog = Catalog::try_from_resources([resource(
                application(),
                "en",
                "count = { $count ->\n    [one] One item\n   *[other] Many items\n}\n",
            )]).unwrap();
            let message = Message::localized_with_arguments(
                application(), "count", "Fallback", [("count".into(), Argument::Unsigned($value))],
            );
            assert_eq!(catalog.render(&message, &Preferences::default()), RenderedMessage {
                text: $expected.to_owned(), locale: Some(Locale::english()),
            });
        }
    )+};
}

numeric_selection! {
    numeric_one_retains_its_plural_category: 1, "One item",
    numeric_other_retains_its_plural_category: 2, "Many items",
}

#[test]
fn an_integer_outside_the_exact_numeric_range_does_not_silently_change_plural_selection() {
    let catalog = Catalog::try_from_resources([resource(
        application(),
        "ru",
        "count = { $count ->\n    [few] Few items\n   *[other] Other items\n}\n",
    )])
    .unwrap();
    let value = 9_007_199_254_740_993_u64;
    let fallback = format!("Count: {value}");
    let message = Message::localized_with_arguments(
        application(),
        "count",
        fallback.clone(),
        [("count".into(), Argument::Unsigned(value))],
    );
    let rendered = catalog.render(&message, &preferences(&["ru"], &[]));
    assert_eq!(rendered.text, fallback);
    assert_eq!(rendered.locale, Some(Locale::english()));
}

macro_rules! numeric_boundaries {
    ($($name:ident: $argument:expr, $expected:literal),+ $(,)?) => {$(
        #[test]
        fn $name() {
            let catalog = Catalog::try_from_resources([resource(
                application(), "en", "number = Count: { $number }\n",
            )]).unwrap();
            let message = Message::localized_with_arguments(
                application(), "number", "Literal fallback", [("number".into(), $argument)],
            );
            assert_eq!(catalog.render(&message, &Preferences::default()), RenderedMessage {
                text: $expected.to_owned(), locale: Some(Locale::english()),
            });
        }
    )+};
}

numeric_boundaries! {
    numeric_unsigned_safe_maximum_is_formatted: Argument::Unsigned(9_007_199_254_740_991), "Count: \u{2068}9007199254740991\u{2069}",
    numeric_unsigned_above_safe_maximum_uses_fallback: Argument::Unsigned(9_007_199_254_740_992), "Literal fallback",
    numeric_signed_safe_minimum_is_formatted: Argument::Signed(-9_007_199_254_740_991), "Count: \u{2068}-9007199254740991\u{2069}",
    numeric_signed_below_safe_minimum_uses_fallback: Argument::Signed(-9_007_199_254_740_992), "Literal fallback",
    numeric_signed_minimum_uses_fallback: Argument::Signed(i64::MIN), "Literal fallback",
    numeric_unsigned_maximum_uses_fallback: Argument::Unsigned(u64::MAX), "Literal fallback",
}

#[test]
fn a_full_extension_catalog_requires_the_matching_full_preference() {
    let catalog = Catalog::try_from_resources([
        resource(application(), "ja", "greeting = 通常\n"),
        resource(application(), "ja-u-ca-japanese", "greeting = 拡張\n"),
    ])
    .unwrap();
    let message = Message::localized(application(), "greeting", "Fallback");
    let exact = catalog.render(&message, &preferences(&["ja-u-ca-japanese"], &[]));
    assert_eq!(exact.text, "拡張");
    assert_eq!(exact.locale, Some(locale("ja-u-ca-japanese")));
    let ordinary = catalog.render(&message, &preferences(&["ja"], &[]));
    assert_eq!(ordinary.text, "通常");
    assert_eq!(ordinary.locale, Some(locale("ja")));
}

#[test]
fn an_excluded_exact_language_cannot_return_as_a_host_fallback() {
    let preferences = preferences(&["ja"], &["ja", "en"]).excluding(vec![locale("ja")]);
    let rendered = catalog().render(
        &Message::localized(application(), "greeting", "Emergency fallback"),
        &preferences,
    );
    assert_eq!(rendered.text, "Hello");
    assert_eq!(rendered.locale, Some(Locale::english()));
}

#[test]
fn nonfinite_numeric_arguments_use_the_accurate_safe_fallback() {
    let catalog =
        Catalog::try_from_resources([resource(application(), "en", "number = { $number }\n")])
            .unwrap();
    let message = Message::localized_with_arguments(
        application(),
        "number",
        "Safe fallback",
        [("number".into(), Argument::Decimal(f64::INFINITY))],
    );
    assert_eq!(catalog.render(&message, &Preferences::default()).text, "Safe fallback");
}

#[test]
fn shared_catalog_snapshots_support_independent_concurrent_preferences() {
    let catalog = catalog();
    let japanese = catalog.clone();
    let english = catalog.clone();
    let render = |catalog: Catalog, preferences| {
        std::thread::spawn(move || {
            catalog.render(&Message::localized(application(), "greeting", "Fallback"), &preferences)
        })
    };
    assert_eq!(render(japanese, preferences(&["ja"], &[])).join().unwrap().text, "こんにちは");
    assert_eq!(render(english, preferences(&["en"], &[])).join().unwrap().text, "Hello");
}

proptest! {
    #[test]
    fn opaque_remote_messages_are_preserved_without_claiming_their_language(text in any::<String>()) {
        let rendered = catalog().render(&Message::opaque(text.clone()), &preferences(&["ja"], &["en"]));
        prop_assert_eq!(rendered.text, text);
        prop_assert_eq!(rendered.locale, None);
    }

    #[test]
    fn unsigned_integer_arguments_never_round_through_f64(value in any::<u64>()) {
        let catalog = Catalog::try_from_resources([resource(application(), "en", "number = Count: { $number }\n")]).unwrap();
        let fallback = format!("Count: {value}");
        let message = Message::localized_with_arguments(
            application(), "number", fallback.clone(), [("number".into(), Argument::Unsigned(value))],
        );
        let expected = if value < (1_u64 << 53) { format!("Count: \u{2068}{value}\u{2069}") } else { fallback };
        prop_assert_eq!(catalog.render(&message, &Preferences::default()).text,
            expected);
    }

    #[test]
    fn signed_integer_arguments_never_round_through_f64(value in any::<i64>()) {
        let catalog = Catalog::try_from_resources([resource(application(), "en", "number = Count: { $number }\n")]).unwrap();
        let fallback = format!("Count: {value}");
        let message = Message::localized_with_arguments(
            application(), "number", fallback.clone(), [("number".into(), Argument::Signed(value))],
        );
        let expected = if value.unsigned_abs() < (1_u64 << 53) { format!("Count: \u{2068}{value}\u{2069}") } else { fallback };
        prop_assert_eq!(catalog.render(&message, &Preferences::default()).text,
            expected);
    }
}
