use super::{
    Argument, CatalogResource, Locale, Message, MessageData, Namespace, Preferences,
    RenderedMessage,
};
use fluent_bundle::{
    FluentArgs, FluentError, FluentResource, FluentValue, concurrent::FluentBundle,
};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    sync::Arc,
};
use suzunari_error::snafu::IntoError;
use suzunari_error::*;

struct Bundle {
    locale: Locale,
    messages: FluentBundle<FluentResource>,
}

impl Bundle {
    fn new(locale: Locale) -> Result<Self, FluentError> {
        let mut messages = FluentBundle::new_concurrent(vec![locale.identifier().clone()]);
        messages.add_builtins()?;
        Ok(Self { locale, messages })
    }
}

type Bundles = BTreeMap<Namespace, BTreeMap<String, Bundle>>;

/// An immutable, concurrently shareable catalog snapshot without a dependency on application startup.
#[derive(Clone, Default)]
pub struct Catalog {
    bundles: Arc<Bundles>,
}

/// Original backend issues stay private instead of becoming public Fluent API types.
#[suzunari_error]
#[suzu(display("Message resource contains invalid entries"))]
pub struct ResourceIssues {
    issues: Vec<FluentError>,
}

impl ResourceIssues {
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }
}

#[suzunari_error]
#[suzu(context(suffix(Error)))]
pub enum CatalogError {
    #[suzu(display("Failed to parse message resource for {namespace:?} in {locale}"))]
    Parse { source: Box<ResourceIssues>, namespace: Namespace, locale: Locale },
    #[suzu(display("Duplicate message resource entry for {namespace:?} in {locale}"))]
    Duplicate { source: Box<ResourceIssues>, namespace: Namespace, locale: Locale },
    #[suzu(display("Failed to register message functions for {namespace:?} in {locale}"))]
    Builtins { source: Box<ResourceIssues>, namespace: Namespace, locale: Locale },
}

fn resource_issues(issues: Vec<FluentError>) -> Box<ResourceIssues> {
    Box::new(ResourceIssuesSnafu { issues }.into_error(suzunari_error::snafu::NoneError))
}

impl Catalog {
    /// Build a complete snapshot, rejecting malformed syntax and duplicate entries atomically.
    pub fn try_from_resources(
        resources: impl IntoIterator<Item = CatalogResource>,
    ) -> Result<Self, CatalogError> {
        let mut bundles = Bundles::new();
        for CatalogResource { namespace, locale, source } in resources {
            let resource = FluentResource::try_new(source)
                .map_err(|(_, issues)| {
                    resource_issues(issues.into_iter().map(FluentError::from).collect())
                })
                .context(ParseError { namespace: namespace.clone(), locale: locale.clone() })?;
            let entry =
                bundles.entry(namespace.clone()).or_default().entry(locale.as_str().to_owned());
            let bundle = match entry {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(
                    Bundle::new(locale.clone())
                        .map_err(|issue| resource_issues(vec![issue]))
                        .context(BuiltinsError {
                            namespace: namespace.clone(),
                            locale: locale.clone(),
                        })?,
                ),
            };
            bundle
                .messages
                .add_resource(resource)
                .map_err(resource_issues)
                .context(DuplicateError { namespace, locale })?;
        }
        Ok(Self { bundles: Arc::new(bundles) })
    }

    /// Return the unique resource locales across all namespaces in canonical tag order.
    pub fn locales(&self) -> Vec<Locale> {
        let mut locales = BTreeMap::new();
        for bundle in self.bundles.values().flat_map(|bundles| bundles.values()) {
            locales.entry(bundle.locale.as_str()).or_insert_with(|| bundle.locale.clone());
        }
        locales.into_values().collect()
    }

    /// Try requested locales, explicit fallbacks and English, respecting excluded catalog locales.
    ///
    /// Missing messages and formatting failures try the next catalog. The final literal English
    /// fallback needs no catalog and remains available even when every catalog locale is excluded.
    /// Opaque text is preserved with no asserted language.
    pub fn render(&self, message: &Message, preferences: &Preferences) -> RenderedMessage {
        let (namespace, id, arguments, fallback_english) = match message.data.as_ref() {
            MessageData::Localized { namespace, id, arguments, fallback_english } => {
                (namespace, id, arguments, fallback_english)
            }
            MessageData::Opaque { text } => {
                return RenderedMessage { text: text.clone(), locale: None };
            }
        };
        if let Some(bundles) = self.bundles.get(namespace)
            && let Some(arguments) = fluent_arguments(arguments)
        {
            for bundle in candidates(bundles, preferences) {
                let Some(pattern) =
                    bundle.messages.get_message(id).and_then(|message| message.value())
                else {
                    continue;
                };
                let mut issues = Vec::new();
                let text = bundle.messages.format_pattern(pattern, Some(&arguments), &mut issues);
                if issues.is_empty() {
                    return RenderedMessage {
                        text: text.into_owned(),
                        locale: Some(bundle.locale.clone()),
                    };
                }
            }
        }
        RenderedMessage { text: fallback_english.clone(), locale: Some(Locale::english()) }
    }
}

fn candidates<'a>(
    bundles: &'a BTreeMap<String, Bundle>,
    preferences: &Preferences,
) -> Vec<&'a Bundle> {
    let available = bundles
        .values()
        .filter(|bundle| preferences.accepts(&bundle.locale) && !bundle.locale.has_extensions())
        .collect::<Vec<_>>();
    let identifiers = available
        .iter()
        .map(|bundle| bundle.locale.identifier().clone())
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    let mut selected = BTreeSet::new();
    let english = Locale::english();
    for requested in preferences
        .requested()
        .iter()
        .chain(preferences.fallback())
        .chain(std::iter::once(&english))
    {
        if let Some(exact) = bundles.get(requested.as_str())
            && preferences.accepts(&exact.locale)
            && selected.insert(exact.locale.as_str())
        {
            candidates.push(exact);
        }
        for identifier in negotiate_languages(
            &[requested.identifier()],
            &identifiers,
            None,
            NegotiationStrategy::Filtering,
        ) {
            if let Some(bundle) =
                available.iter().find(|bundle| bundle.locale.identifier() == identifier)
                && selected.insert(bundle.locale.as_str())
            {
                candidates.push(*bundle);
            }
        }
    }
    candidates
}

fn fluent_arguments(arguments: &BTreeMap<String, Argument>) -> Option<FluentArgs<'_>> {
    const MAX_SAFE_INTEGER: u64 = (1_u64 << 53) - 1;
    let mut values = FluentArgs::new();
    for (name, argument) in arguments {
        let value = match argument {
            Argument::Text(text) => FluentValue::String(Cow::Borrowed(text)),
            Argument::Signed(value) if value.unsigned_abs() <= MAX_SAFE_INTEGER => {
                FluentValue::from(*value as f64)
            }
            Argument::Unsigned(value) if *value <= MAX_SAFE_INTEGER => {
                FluentValue::from(*value as f64)
            }
            Argument::Signed(_) | Argument::Unsigned(_) => return None,
            Argument::Decimal(value) if value.is_finite() => FluentValue::from(*value),
            Argument::Decimal(_) => return None,
        };
        values.set(name.as_str(), value);
    }
    Some(values)
}
