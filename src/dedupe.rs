//! Grouping a list of names that denote the same company.

use std::collections::{BTreeMap, BTreeSet};

use crate::company::Company;
use crate::similarity::{compare_parsed, Config, Verdict};

/// Why two names ended up in one cluster.
#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    /// Index of the first name in the input.
    pub a: usize,
    /// Index of the second name in the input.
    pub b: usize,
    /// Their score.
    pub score: f32,
    /// The reasons of the comparison.
    pub reasons: Vec<String>,
}

/// Names that belong together.
#[derive(Debug, Clone, PartialEq)]
pub struct Cluster {
    /// Indices into the input, ascending.
    pub members: Vec<usize>,
    /// The member to keep: one with a legal form if there is one, then the longest name.
    pub canonical: usize,
    /// The comparisons that merged the cluster (empty for single names).
    pub links: Vec<Link>,
}

fn parent(p: &mut [usize], mut x: usize) -> usize {
    while p[x] != x {
        p[x] = p[p[x]];
        x = p[x];
    }
    x
}

/// Candidate blocks: names are only compared when they share one of these keys, which
/// keeps the work far below all-pairs on large lists. A pair whose names differ in the
/// first three letters of both its first and its last word, and whose spacing-free names
/// differ at the start, is not found.
fn blocks(c: &Company) -> Vec<String> {
    let loose = c.loose_key();
    let squeezed: String = loose.replace(' ', "");
    let mut out = vec![format!(
        "s:{}",
        squeezed.chars().take(4).collect::<String>()
    )];
    let words: Vec<&str> = loose.split(' ').collect();
    if let Some(w) = words.first() {
        out.push(format!("f:{}", w.chars().take(3).collect::<String>()));
    }
    if let Some(w) = words.last() {
        out.push(format!("l:{}", w.chars().take(3).collect::<String>()));
    }
    let initials: String = c.tokens.iter().filter_map(|t| t.chars().next()).collect();
    if initials.chars().count() >= 2 {
        out.push(format!("i:{initials}"));
    }
    if c.tokens.len() == 1 {
        out.push(format!("i:{}", c.tokens[0]));
    }
    out
}

/// Groups `names` whose pairwise comparison reaches `min`. Pairs with conflicting legal
/// forms are never merged, whatever `min` is.
///
/// Clusters are formed transitively (A~B and B~C put A and C together), so use
/// [`Verdict::Same`] or higher when chains of near-matches would be a problem.
pub fn dedupe<S: AsRef<str>>(names: &[S], min: Verdict) -> Vec<Cluster> {
    let cfg = Config::default();
    let parsed: Vec<Company> = names.iter().map(|n| Company::parse(n.as_ref())).collect();
    let mut by_block: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, c) in parsed.iter().enumerate() {
        if c.tokens.is_empty() {
            continue;
        }
        for b in blocks(c) {
            by_block.entry(b).or_default().push(i);
        }
    }

    let mut done: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut root: Vec<usize> = (0..parsed.len()).collect();
    let mut links = Vec::new();
    for members in by_block.values() {
        for (n, &i) in members.iter().enumerate() {
            for &j in &members[n + 1..] {
                if !done.insert((i, j)) {
                    continue;
                }
                let m = compare_parsed(&parsed[i], &parsed[j], &cfg);
                if m.verdict >= min && m.forms != crate::similarity::FormRelation::Conflict {
                    let (ri, rj) = (parent(&mut root, i), parent(&mut root, j));
                    root[ri.max(rj)] = ri.min(rj);
                    links.push(Link {
                        a: i,
                        b: j,
                        score: m.score,
                        reasons: m.reasons,
                    });
                }
            }
        }
    }

    let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..parsed.len() {
        let r = parent(&mut root, i);
        groups.entry(r).or_default().push(i);
    }
    let mut clusters: Vec<Cluster> = groups
        .into_values()
        .map(|members| {
            let canonical = *members
                .iter()
                .max_by_key(|&&i| {
                    (
                        parsed[i].form.is_some(),
                        parsed[i].original.chars().count(),
                        std::cmp::Reverse(i),
                    )
                })
                .unwrap_or(&members[0]);
            let links = links
                .iter()
                .filter(|l| members.contains(&l.a))
                .cloned()
                .collect();
            Cluster {
                members,
                canonical,
                links,
            }
        })
        .collect();
    clusters.sort_by_key(|c| c.members[0]);
    clusters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spelling_variants_cluster_and_the_fullest_name_is_canonical() {
        let names = [
            "Siemens",
            "Müller & Söhne GmbH",
            "MUELLER und SOEHNE",
            "Siemens AG",
            "Mueller & Soehne G.m.b.H.",
            "Apple Inc",
        ];
        let c = dedupe(&names, Verdict::Same);
        let sets: Vec<&[usize]> = c.iter().map(|c| c.members.as_slice()).collect();
        assert_eq!(sets, [&[0, 3][..], &[1, 2, 4], &[5]]);
        assert_eq!(c[0].canonical, 3);
        assert_eq!(c[1].canonical, 4);
        assert!(!c[1].links.is_empty());
        assert!(c[2].links.is_empty());
    }

    #[test]
    fn conflicting_forms_stay_apart() {
        let c = dedupe(&["Siemens AG", "Siemens GmbH"], Verdict::Possible);
        assert_eq!(c.len(), 2);
    }

    #[test]
    fn empty_and_unparsable_names_are_singletons() {
        let c = dedupe(&["", "-", "Acme"], Verdict::Same);
        assert_eq!(c.len(), 3);
    }
}
