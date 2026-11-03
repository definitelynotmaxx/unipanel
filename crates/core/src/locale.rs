//! Instance language.
//!
//! UniPanel is single-language per installation: the locale is chosen once
//! during first-run setup and stored as the immutable `instance.locale`
//! setting. There is intentionally no per-user language and no switcher in the
//! UI. Because of that, this type is tiny and explicit instead of relying on a
//! full i18n framework on the server side.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A supported instance language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    /// English.
    En,
    /// German.
    De,
}

impl Locale {
    /// Every supported locale, used by the setup wizard and the CLI.
    pub const ALL: [Self; 2] = [Self::En, Self::De];

    /// The stable string persisted in the database and exposed to the frontend.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::De => "de",
        }
    }

    /// The locale's own name, shown in the language picker.
    #[must_use]
    pub const fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::De => "Deutsch",
        }
    }

    /// Parses a locale, accepting the usual `de-DE`, `en_US` and `de` spellings.
    ///
    /// Returns `None` for anything that is not an English or German variant.
    #[must_use]
    pub fn parse_tag(tag: &str) -> Option<Self> {
        let primary = tag
            .split(['-', '_'])
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Self::En),
            "de" => Some(Self::De),
            _ => None,
        }
    }

    /// Picks the best supported locale from an `Accept-Language` header value.
    ///
    /// The header is a comma separated list with optional `;q=` weights. We do
    /// not need full RFC 4647 matching: honouring the order of appearance is
    /// good enough for a two-language product, but we do respect explicit
    /// weights so `de;q=0.2, en;q=0.9` resolves to English.
    #[must_use]
    pub fn from_accept_language(header: &str) -> Option<Self> {
        let mut best: Option<(f32, Self)> = None;
        for part in header.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let mut pieces = part.split(';');
            let tag = pieces.next().unwrap_or_default().trim();
            if tag == "*" {
                continue;
            }
            let weight = pieces
                .find_map(|p| p.trim().strip_prefix("q="))
                .and_then(|q| q.trim().parse::<f32>().ok())
                .unwrap_or(1.0);
            if weight <= 0.0 {
                continue;
            }
            if let Some(locale) = Self::parse_tag(tag) {
                if best.is_none_or(|(best_weight, _)| weight > best_weight) {
                    best = Some((weight, locale));
                }
            }
        }
        best.map(|(_, locale)| locale)
    }

    /// Renders an `Accept-Language`-compatible tag for the given locale.
    #[must_use]
    pub const fn html_lang(self) -> &'static str {
        self.as_str()
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::En
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Locale {
    type Err = UnsupportedLocale;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_tag(s).ok_or(UnsupportedLocale)
    }
}

impl TryFrom<String> for Locale {
    type Error = UnsupportedLocale;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// Returned when a locale string is not `en` or `de`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported locale; expected one of: en, de")]
pub struct UnsupportedLocale;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_and_regional_tags() {
        assert_eq!("de".parse::<Locale>(), Ok(Locale::De));
        assert_eq!("de-DE".parse::<Locale>(), Ok(Locale::De));
        assert_eq!("en_US".parse::<Locale>(), Ok(Locale::En));
        assert!("fr".parse::<Locale>().is_err());
    }

    #[test]
    fn accept_language_prefers_weight() {
        assert_eq!(
            Locale::from_accept_language("de;q=0.2, en;q=0.9"),
            Some(Locale::En)
        );
        assert_eq!(
            Locale::from_accept_language("de-CH,de;q=0.9,en;q=0.8"),
            Some(Locale::De)
        );
        assert_eq!(Locale::from_accept_language("fr-FR,fr;q=0.9"), None);
        assert_eq!(Locale::from_accept_language("*"), None);
    }

    #[test]
    fn native_names_are_stable() {
        assert_eq!(Locale::De.native_name(), "Deutsch");
        assert_eq!(Locale::En.native_name(), "English");
    }
}
