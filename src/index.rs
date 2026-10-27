//! Looking up one name in a list of known companies.

use std::collections::{BTreeMap, BTreeSet};

use crate::company::Company;
use crate::dedupe::blocks;
use crate::similarity::{compare_parsed, Config, Match, Verdict};

/// A known name that resembles the query.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// Index of the name in the list the [`Index`] was built from.
    pub index: usize,
    /// The known name as given.
    pub name: String,
    /// How it compares to the query, with reasons.
    pub matched: Match,
}

/// A list of known company names that can be searched without comparing against every one.
///
/// Typical use: before creating a customer record, look whether the company is already
/// there. For deduplicating a whole list, use [`dedupe`](crate::dedupe()) instead.
///
/// ```
/// use aat::{Index, Verdict};
///
/// let crm = Index::new(&["Müller & Söhne GmbH", "Siemens AG", "Siemens Energy AG"]);
/// let hits = crm.search("MUELLER und SOEHNE G.m.b.H.", Verdict::Likely, 3);
/// assert_eq!(hits[0].name, "Müller & Söhne GmbH");
/// assert_eq!(hits[0].matched.verdict, Verdict::Identical);
/// ```
#[derive(Debug, Clone)]
pub struct Index {
    names: Vec<String>,
    parsed: Vec<Company>,
    blocks: BTreeMap<String, Vec<usize>>,
    cfg: Config,
}

impl Index {
    /// Indexes `names` with the default thresholds.
    pub fn new<S: AsRef<str>>(names: &[S]) -> Self {
        Self::with_config(names, Config::default())
    }

    /// Indexes `names` and judges matches by `cfg`.
    pub fn with_config<S: AsRef<str>>(names: &[S], cfg: Config) -> Self {
        let parsed: Vec<Company> = names.iter().map(|n| Company::parse(n.as_ref())).collect();
        let mut blocks_by_key: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, c) in parsed.iter().enumerate() {
            if c.tokens.is_empty() {
                continue;
            }
            for b in blocks(c) {
                blocks_by_key.entry(b).or_default().push(i);
            }
        }
        Self {
            names: names.iter().map(|n| n.as_ref().to_string()).collect(),
            parsed,
            blocks: blocks_by_key,
            cfg,
        }
    }

    /// Number of indexed names.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether the index holds no names.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// The known names that reach at least `min` against `query`, best first, at most
    /// `limit` of them. A name with a conflicting legal form (`Siemens GmbH` for
    /// `Siemens AG`) can show up as [`Verdict::Possible`] at best; check
    /// [`Match::forms`] if you must not link such records.
    ///
    /// Like [`dedupe`](crate::dedupe()), the search only compares names that share a
    /// block, so a name that differs in the start of both its first and its last word and
    /// in its spacing-free beginning is not found.
    pub fn search(&self, query: &str, min: Verdict, limit: usize) -> Vec<Hit> {
        let q = Company::parse(query);
        if q.tokens.is_empty() {
            return Vec::new();
        }
        let mut candidates = BTreeSet::new();
        for b in blocks(&q) {
            if let Some(ids) = self.blocks.get(&b) {
                candidates.extend(ids.iter().copied());
            }
        }
        let mut hits: Vec<Hit> = candidates
            .into_iter()
            .filter_map(|i| {
                let matched = compare_parsed(&q, &self.parsed[i], &self.cfg);
                (matched.verdict >= min).then(|| Hit {
                    index: i,
                    name: self.names[i].clone(),
                    matched,
                })
            })
            .collect();
        hits.sort_by(|a, b| {
            b.matched
                .score
                .total_cmp(&a.matched.score)
                .then(a.index.cmp(&b.index))
        });
        hits.truncate(limit);
        hits
    }

    /// The best match of at least `min`, if any.
    pub fn best(&self, query: &str, min: Verdict) -> Option<Hit> {
        self.search(query, min, 1).into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::similarity::FormRelation;

    fn crm() -> Index {
        Index::new(&[
            "Müller & Söhne GmbH",
            "Siemens AG",
            "Siemens Energy AG",
            "Hotel Adler 1",
            "Hotel Adler 2",
            "",
        ])
    }

    #[test]
    fn finds_spelling_variants() {
        let hit = crm()
            .best("MUELLER und SOEHNE G.m.b.H.", Verdict::Likely)
            .unwrap();
        assert_eq!(hit.index, 0);
        assert_eq!(hit.matched.verdict, Verdict::Identical);
    }

    #[test]
    fn results_are_sorted_and_limited() {
        let hits = crm().search("Siemens", Verdict::Possible, 5);
        assert!(!hits.is_empty());
        assert!(hits
            .windows(2)
            .all(|w| w[0].matched.score >= w[1].matched.score));
        assert_eq!(crm().search("Siemens", Verdict::Possible, 1).len(), 1);
    }

    #[test]
    fn a_conflicting_legal_form_never_reads_as_the_same_company() {
        let hits = crm().search("Siemens GmbH", Verdict::Possible, 5);
        let siemens_ag = hits.iter().find(|h| h.name == "Siemens AG").unwrap();
        assert!(siemens_ag.matched.verdict < Verdict::Same);
        assert_eq!(siemens_ag.matched.forms, FormRelation::Conflict);
    }

    #[test]
    fn differing_numbers_do_not_match_strongly() {
        let hit = crm().best("Hotel Adler 1", Verdict::Likely).unwrap();
        assert_eq!(hit.name, "Hotel Adler 1");
        assert!(crm()
            .search("Hotel Adler 1", Verdict::Likely, 5)
            .iter()
            .all(|h| h.name != "Hotel Adler 2"));
    }

    #[test]
    fn empty_queries_and_empty_indexes_find_nothing() {
        assert!(crm().search("", Verdict::Possible, 5).is_empty());
        let none = Index::new::<&str>(&[]);
        assert!(none.is_empty());
        assert!(none.search("Siemens AG", Verdict::Possible, 5).is_empty());
    }
}
