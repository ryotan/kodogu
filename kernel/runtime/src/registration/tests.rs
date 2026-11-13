use super::{DuplicateToolId, InvalidToolId, ToolId, ToolRegistry};
use proptest::prelude::*;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

fn id(value: &str) -> ToolId {
    ToolId::try_from(value).unwrap()
}

macro_rules! accepted_ids {
    ($($name:ident: $value:expr;)*) => {
        $(#[test]
        fn $name() {
            let value = $value;
            let parsed = ToolId::try_from(value).unwrap();
            assert_eq!((parsed.as_ref(), parsed.to_string()), (value, value.to_owned()));
        })*
    };
}

accepted_ids! {
    accepts_one_lowercase_letter: "a";
    accepts_one_digit: "0";
    accepts_a_single_segment: "verification";
    accepts_digit_segments: "tool-2";
    accepts_a_leading_digit_segment: "2-tools";
}

macro_rules! rejected_ids {
    ($($name:ident: $value:expr;)*) => {
        $(#[test]
        fn $name() {
            let value = $value;
            let error: InvalidToolId = ToolId::try_from(value).unwrap_err();
            assert_eq!(error.value, value);
            assert_eq!(error.to_string(), format!(
                "Invalid tool identifier {value:?}; expected lower-kebab-case"
            ));
            assert!(std::error::Error::source(&error).is_none());
        })*
    };
}

rejected_ids! {
    rejects_an_empty_identifier: "";
    rejects_an_uppercase_identifier: "Tool";
    rejects_leading_space: " tool";
    rejects_trailing_space: "tool ";
    rejects_embedded_space: "two tools";
    rejects_only_whitespace: " ";
    rejects_underscores: "two_tools";
    rejects_dot_separators: "two.tools";
    rejects_forward_slashes: "two/tools";
    rejects_backslashes: "two\\tools";
    rejects_namespace_delimiters: "tool:command";
    rejects_consecutive_hyphens: "two--tools";
    rejects_a_leading_hyphen: "-tool";
    rejects_a_trailing_hyphen: "tool-";
    rejects_control_characters: "tool\n";
    rejects_a_nul_character: "tool\0";
    rejects_non_ascii_characters: "道具";
}

#[test]
fn owned_and_borrowed_inputs_have_the_same_identity() {
    let value = "owned-tool".to_owned();
    assert_eq!(
        ToolId::try_from(value.clone()).unwrap(),
        ToolId::try_from(value.as_str()).unwrap()
    );
}

#[test]
fn an_owned_invalid_input_is_returned_without_normalizing_it() {
    let value = " Invalid-Tool ".to_owned();
    let error = ToolId::try_from(value.clone()).unwrap_err();
    assert_eq!(error.value, value);
}

#[test]
fn identifier_length_is_not_arbitrarily_limited() {
    let value = "tool".repeat(4096);
    assert_eq!(ToolId::try_from(value.clone()).unwrap().as_ref(), value);
}

#[test]
fn no_registrations_form_an_empty_registry() {
    let registry = ToolRegistry::<()>::try_from_iter([]).unwrap();
    assert_eq!((registry.len(), registry.is_empty(), registry.iter().count()), (0, true, 0));
}

#[test]
fn a_payload_is_retained_under_its_own_identity() {
    let registry = ToolRegistry::try_from_iter([(id("first"), 10), (id("second"), 20)]).unwrap();
    assert_eq!(
        (
            registry.len(),
            registry.is_empty(),
            registry.get(&id("first")),
            registry.get(&id("second"))
        ),
        (2, false, Some(&10), Some(&20))
    );
    assert_eq!(registry.get(&id("absent")), None);
}

#[test]
fn ownership_can_be_transferred_without_cloning_payloads() {
    struct Payload(u8);
    let registry = ToolRegistry::try_from_iter([(id("first"), Payload(7))]).unwrap();
    let entries: Vec<_> = registry.into_iter().map(|(id, payload)| (id, payload.0)).collect();
    assert_eq!(entries, vec![(id("first"), 7)]);
}

#[test]
fn a_registry_may_borrow_application_local_payloads() {
    let value = String::from("application-local");
    let registry = ToolRegistry::try_from_iter([(id("first"), value.as_str())]).unwrap();
    assert_eq!(registry.get(&id("first")), Some(&value.as_str()));
}

macro_rules! duplicate_ids {
    ($($name:ident: $entries:expr;)*) => {
        $(#[test]
        fn $name() {
            let error: DuplicateToolId = ToolRegistry::try_from_iter($entries).unwrap_err();
            assert_eq!(error.id, id("first"));
            assert_eq!(error.to_string(), "Duplicate tool identifier first");
            assert!(std::error::Error::source(&error).is_none());
        })*
    };
}

duplicate_ids! {
    rejects_adjacent_duplicate_identifiers: [(id("first"), 1), (id("first"), 2)];
    rejects_separated_duplicate_identifiers: [(id("first"), 1), (id("second"), 2), (id("first"), 3)];
    rejects_duplicate_identifiers_with_equal_payloads: [(id("first"), 1), (id("first"), 1)];
}

#[test]
fn registration_never_invokes_factory_payloads() {
    let calls = Rc::new(Cell::new(0));
    let factory = || calls.set(calls.get() + 1);
    let result = ToolRegistry::try_from_iter([(id("first"), &factory), (id("first"), &factory)]);
    assert!(result.is_err());
    assert_eq!(calls.get(), 0);
}

proptest! {
    #[test]
    fn canonical_identifiers_round_trip(value in "[a-z0-9]{1,24}(-[a-z0-9]{1,24}){0,4}") {
        let borrowed = ToolId::try_from(value.as_str()).unwrap();
        let owned = ToolId::try_from(value.clone()).unwrap();
        prop_assert_eq!(&borrowed, &owned);
        prop_assert_eq!(owned.as_ref(), value);
    }

    #[test]
    fn invalid_suffixes_are_not_silently_removed(
        prefix in "[a-z0-9]{1,24}", suffix in prop_oneof![Just(":"), Just("/"), Just(" "), Just("A")]
    ) {
        let value = format!("{prefix}{suffix}");
        let error = ToolId::try_from(value.as_str()).unwrap_err();
        prop_assert_eq!(error.value, value);
    }

    #[test]
    fn registration_input_order_does_not_change_id_payload_associations(
        values in proptest::collection::btree_map("[a-z0-9]{1,12}(-[a-z0-9]{1,12}){0,3}", any::<i32>(), 0..24)
    ) {
        let forward = values.iter().map(|(key, value)| (id(key), *value));
        let reverse = values.iter().rev().map(|(key, value)| (id(key), *value));
        let first = ToolRegistry::try_from_iter(forward).unwrap();
        let second = ToolRegistry::try_from_iter(reverse).unwrap();
        let associations = |registry: ToolRegistry<i32>| registry.into_iter().collect::<BTreeMap<_, _>>();
        prop_assert_eq!(associations(first), associations(second));
    }

    #[test]
    fn an_existing_identifier_is_always_rejected_on_reinsertion(
        value in "[a-z0-9]{1,12}(-[a-z0-9]{1,12}){0,3}", first in any::<i32>(), second in any::<i32>()
    ) {
        let key = id(&value);
        let error = ToolRegistry::try_from_iter([(key.clone(), first), (key.clone(), second)]).unwrap_err();
        prop_assert_eq!(error.id, key);
    }
}
