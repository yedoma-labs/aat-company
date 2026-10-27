//! Parsing a company name into core and legal form.

use std::ops::Range;

use crate::fold::{core_tokens, raw_tokens};
use crate::forms::{lookup, lookup_prefix, LegalForm, MAX_FORM_TOKENS};

/// Descriptive words that do not identify a company (`Group`, `Holding`, `Deutschland`, ...).
/// They are ignored by [`Company::loose_key`].
const GENERIC: &[&str] = &[
    "the",
    "die",
    "der",
    "das",
    "le",
    "la",
    "les",
    "el",
    "los",
    "group",
    "gruppe",
    "groupe",
    "holding",
    "holdings",
    "beteiligungs",
    "beteiligung",
    "verwaltungs",
    "verwaltung",
    "international",
    "intl",
    "global",
    "europe",
    "europa",
    "deutschland",
    "germany",
    "service",
    "services",
    "solutions",
    "systems",
    "technologies",
    "technology",
    "technologie",
    "consulting",
    "trading",
    "handels",
    "handel",
    "immobilien",
    "company",
];

/// A parsed company name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Company {
    /// The name as given.
    pub original: String,
    /// The legal form found at the end of the name, if any.
    pub form: Option<LegalForm>,
    /// The folded words of the name without legal form and connectors (`ap moller`).
    pub tokens: Vec<String>,
}

/// Finds the legal form of a name and the range of tokens that make up the name.
///
/// A trailing form is split off first (the longest spelling wins, so `GmbH & Co. KG` is
/// not read as `KG`); a leading prefix form (`OOO`, `PT`, ...) only if there is none.
/// A form is never split off if that would leave nothing of the name.
fn split_form(raw: &[String]) -> (Option<LegalForm>, Range<usize>) {
    for k in (1..=MAX_FORM_TOKENS.min(raw.len().saturating_sub(1))).rev() {
        if let Some(f) = lookup(&raw[raw.len() - k..]) {
            return (Some(f), 0..raw.len() - k);
        }
    }
    match lookup_prefix(raw) {
        Some(f) => (Some(f), 1..raw.len()),
        None => (None, 0..raw.len()),
    }
}

impl Company {
    /// Parses `name`. A legal form is split off at the end or, for forms such as `OOO`
    /// and `PT`, at the start.
    pub fn parse(name: &str) -> Self {
        let raw = raw_tokens(name);
        let (form, range) = split_form(&raw);
        Self {
            original: name.to_string(),
            form,
            tokens: core_tokens(&raw[range]),
        }
    }

    /// The comparison key: folded core words joined by spaces, without the legal form.
    /// `Müller & Söhne GmbH` and `MUELLER und SOEHNE G.m.b.H.` have the same key.
    pub fn key(&self) -> String {
        self.tokens.join(" ")
    }

    /// Like [`key`](Company::key) but without descriptive words (`Group`, `Holding`, ...),
    /// unless nothing else would be left.
    pub fn loose_key(&self) -> String {
        let kept: Vec<&str> = self
            .tokens
            .iter()
            .map(String::as_str)
            .filter(|t| !GENERIC.contains(t))
            .collect();
        if kept.is_empty() {
            self.key()
        } else {
            kept.join(" ")
        }
    }

    /// The name tidied for display: core words as written, canonical legal form appended
    /// (or put in front, for a prefix form such as `OOO`).
    pub fn display(&self) -> String {
        match (&self.form, self.original.trim()) {
            (None, o) => o.to_string(),
            (Some(f), o) => {
                let (_, range) = split_form(&raw_tokens(o));
                if range.start > 0 {
                    format!("{} {}", f.canonical, self.key())
                } else {
                    format!("{} {}", self.key(), f.canonical)
                }
            }
        }
    }

    /// Whether nothing is left of the name (it had no letters or digits).
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.form.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spelling_variants_share_a_key() {
        let a = Company::parse("Müller & Söhne GmbH");
        let b = Company::parse("MUELLER und SOEHNE G.m.b.H.");
        assert_eq!(a.key(), "mueller soehne");
        assert_eq!(a.key(), b.key());
        assert_eq!(a.form, b.form);
    }

    #[test]
    fn longest_form_wins() {
        let c = Company::parse("Beispiel Verwaltungs GmbH & Co. KG");
        assert_eq!(c.form.unwrap().canonical, "GmbH & Co. KG");
        assert_eq!(c.key(), "beispiel verwaltungs");
        assert_eq!(c.loose_key(), "beispiel");
    }

    #[test]
    fn a_name_that_is_only_a_form_keeps_it() {
        let c = Company::parse("AG");
        assert!(c.form.is_none());
        assert_eq!(c.key(), "ag");
    }

    #[test]
    fn no_form_is_fine() {
        let c = Company::parse("Yedoma Labs");
        assert!(c.form.is_none());
        assert_eq!(c.key(), "yedoma labs");
    }

    #[test]
    fn display_uses_canonical_form() {
        assert_eq!(Company::parse("acme g.m.b.h.").display(), "acme GmbH");
    }

    #[test]
    fn empty_input() {
        assert!(Company::parse(" - ").is_empty());
    }

    #[test]
    fn prefix_forms_are_split_off_the_start() {
        let c = Company::parse("ООО «Ромашка»");
        assert_eq!(c.form.map(|f| f.canonical), Some("OOO"));
        assert_eq!(c.key(), "ромашка");
        assert_eq!(c.display(), "OOO ромашка");

        let c = Company::parse("PT Astra International Tbk");
        assert_eq!(c.form.map(|f| f.canonical), Some("PT"));
        assert_eq!(c.key(), "astra international tbk");
        assert_eq!(c.display(), "PT astra international tbk");
    }

    #[test]
    fn leading_words_that_are_not_prefix_forms_stay_in_the_name() {
        // AB is a legal form in Sweden but "AB InBev" is a name
        let c = Company::parse("AB InBev");
        assert!(c.form.is_none());
        assert_eq!(c.key(), "ab inbev");
        // a prefix form alone is a name, not a form
        let c = Company::parse("OOO");
        assert!(c.form.is_none());
        assert_eq!(c.key(), "ooo");
    }

    #[test]
    fn a_trailing_form_wins_over_a_leading_one() {
        let c = Company::parse("OOO Acme GmbH");
        assert_eq!(c.form.map(|f| f.canonical), Some("GmbH"));
        assert_eq!(c.key(), "ooo acme");
        assert_eq!(c.display(), "ooo acme GmbH");
    }

    #[test]
    fn a_trailing_prefix_spelling_stays_at_the_end() {
        let c = Company::parse("Acme OOO");
        assert_eq!(c.form.map(|f| f.canonical), Some("OOO"));
        assert_eq!(c.display(), "acme OOO");
    }
}
