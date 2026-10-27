//! Parsing a company name into core and legal form.

use crate::fold::{core_tokens, raw_tokens};
use crate::forms::{lookup, LegalForm, MAX_FORM_TOKENS};

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

impl Company {
    /// Parses `name`. A trailing legal form is split off (the longest spelling wins, so
    /// `GmbH & Co. KG` is not read as `KG`) unless that would leave nothing of the name.
    pub fn parse(name: &str) -> Self {
        let raw = raw_tokens(name);
        let mut form = None;
        let mut cut = raw.len();
        for k in (1..=MAX_FORM_TOKENS.min(raw.len().saturating_sub(1))).rev() {
            if let Some(f) = lookup(&raw[raw.len() - k..]) {
                form = Some(f);
                cut = raw.len() - k;
                break;
            }
        }
        Self {
            original: name.to_string(),
            form,
            tokens: core_tokens(&raw[..cut]),
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

    /// The name tidied for display: core words as written, canonical legal form appended.
    pub fn display(&self) -> String {
        match (&self.form, self.original.trim()) {
            (None, o) => o.to_string(),
            (Some(f), _) => format!("{} {}", self.key(), f.canonical),
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
}
