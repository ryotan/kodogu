use std::{fmt, str::FromStr};
use suzunari_error::*;

/// A normalized locale tag with extensions retained for exact matching.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Locale {
    tag: String,
    identifier: unic_langid::LanguageIdentifier,
    extensions: bool,
}

#[suzunari_error]
enum LocaleParseError {
    #[suzu(display("Invalid Unicode locale identifier"))]
    Unicode { source: icu_locale_core::ParseError },
    #[suzu(display("Locale language cannot be used for message matching"))]
    MessageLanguage { source: unic_langid::LanguageIdentifierError },
}

#[suzunari_error]
#[suzu(display("Invalid locale {value:?}"))]
pub struct InvalidLocale {
    value: String,
    source: LocaleParseError,
}

impl FromStr for Locale {
    type Err = InvalidLocale;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parsed = icu_locale_core::Locale::try_from_str(value)
            .context(UnicodeSnafu)
            .context(InvalidLocaleSnafu { value })?;
        let identifier = parsed
            .id
            .to_string()
            .parse()
            .context(MessageLanguageSnafu)
            .context(InvalidLocaleSnafu { value })?;
        Ok(Self { tag: parsed.to_string(), identifier, extensions: !parsed.extensions.is_empty() })
    }
}

impl Locale {
    pub fn english() -> Self {
        Self { tag: "en".into(), identifier: unic_langid::langid!("en"), extensions: false }
    }

    pub fn japanese() -> Self {
        Self { tag: "ja".into(), identifier: unic_langid::langid!("ja"), extensions: false }
    }

    pub fn as_str(&self) -> &str {
        &self.tag
    }

    pub(super) fn identifier(&self) -> &unic_langid::LanguageIdentifier {
        &self.identifier
    }

    pub(super) fn has_extensions(&self) -> bool {
        self.extensions
    }
}

impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.tag)
    }
}

/// Request-local preferences, independent of HTTP headers, environment variables and OS APIs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preferences {
    requested: Vec<Locale>,
    fallback: Vec<Locale>,
    excluded: Vec<Locale>,
}

impl Preferences {
    pub fn new(requested: Vec<Locale>, fallback: Vec<Locale>) -> Self {
        Self { requested, fallback, excluded: Vec::new() }
    }

    pub fn excluding(mut self, locales: Vec<Locale>) -> Self {
        self.excluded = locales;
        self
    }

    pub fn requested(&self) -> &[Locale] {
        &self.requested
    }
    pub fn fallback(&self) -> &[Locale] {
        &self.fallback
    }
    pub fn excluded(&self) -> &[Locale] {
        &self.excluded
    }

    pub(super) fn accepts(&self, locale: &Locale) -> bool {
        !self.excluded.contains(locale)
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self::new(Vec::new(), vec![Locale::english()])
    }
}
