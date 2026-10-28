//! Comparing two company names, with reasons.

use std::cmp::Ordering;

use crate::company::Company;

/// How sure the crate is that two names denote the same company.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verdict {
    /// Clearly different companies.
    Different,
    /// Some resemblance; a human should decide.
    Possible,
    /// Probably the same company.
    Likely,
    /// The same company, spelled differently.
    Same,
    /// Equal after normalisation.
    Identical,
}

/// How the legal forms of the two names relate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormRelation {
    /// Both names have the same legal form.
    Same,
    /// At least one name states no legal form, so nothing contradicts.
    Unknown,
    /// Different legal forms: probably two separate legal entities.
    Conflict,
}

/// Score thresholds.
#[derive(Debug, Clone)]
pub struct Config {
    /// Score from which names count as [`Verdict::Same`].
    pub same: f32,
    /// Score from which names count as [`Verdict::Likely`].
    pub likely: f32,
    /// Score from which names count as [`Verdict::Possible`].
    pub possible: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            same: 0.93,
            likely: 0.85,
            possible: 0.75,
        }
    }
}

/// The result of comparing two names.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    /// `0.0..=1.0`, how alike the names are (after the penalties for differing forms and numbers).
    pub score: f32,
    /// The decision.
    pub verdict: Verdict,
    /// How the legal forms relate.
    pub forms: FormRelation,
    /// Why, in plain sentences.
    pub reasons: Vec<String>,
}

#[allow(clippy::needless_range_loop)]
fn jaro(a: &[char], b: &[char]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let reach = (a.len().max(b.len()) / 2).saturating_sub(1);
    let (mut am, mut bm) = (vec![false; a.len()], vec![false; b.len()]);
    let mut m = 0usize;
    for i in 0..a.len() {
        let (lo, hi) = (i.saturating_sub(reach), (i + reach + 1).min(b.len()));
        for j in lo..hi {
            if !bm[j] && a[i] == b[j] {
                am[i] = true;
                bm[j] = true;
                m += 1;
                break;
            }
        }
    }
    if m == 0 {
        return 0.0;
    }
    let (mut t, mut k) = (0usize, 0usize);
    for i in 0..a.len() {
        if am[i] {
            while !bm[k] {
                k += 1;
            }
            if a[i] != b[k] {
                t += 1;
            }
            k += 1;
        }
    }
    let m = m as f32;
    (m / a.len() as f32 + m / b.len() as f32 + (m - t as f32 / 2.0) / m) / 3.0
}

/// Jaro-Winkler similarity of two strings, `0.0..=1.0`.
pub fn jaro_winkler(a: &str, b: &str) -> f32 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let j = jaro(&a, &b);
    if j <= 0.7 {
        return j;
    }
    let prefix = a.iter().zip(&b).take(4).take_while(|(x, y)| x == y).count();
    j + prefix as f32 * 0.1 * (1.0 - j)
}

fn initials(tokens: &[String]) -> String {
    tokens.iter().filter_map(|t| t.chars().next()).collect()
}

fn digits(tokens: &[String]) -> Vec<&str> {
    let mut d: Vec<&str> = tokens
        .iter()
        .map(String::as_str)
        .filter(|t| t.chars().any(|c| c.is_ascii_digit()))
        .collect();
    d.sort_unstable();
    d
}

/// Compares two names with the default [`Config`].
pub fn compare(a: &str, b: &str) -> Match {
    compare_parsed(&Company::parse(a), &Company::parse(b), &Config::default())
}

/// Compares two parsed names.
pub fn compare_parsed(a: &Company, b: &Company, cfg: &Config) -> Match {
    let mut reasons = Vec::new();
    let (ka, kb) = (a.key(), b.key());
    let (la, lb) = (a.loose_key(), b.loose_key());
    let squeeze = |s: &str| s.replace(' ', "");

    let mut score;
    let mut identical = false;
    if ka.is_empty() || kb.is_empty() {
        score = 0.0;
        reasons.push("a name has no letters or digits".to_string());
    } else if ka == kb {
        score = 1.0;
        identical = true;
        reasons.push(
            "same name core after normalisation (case, accents, punctuation; legal form set aside)"
                .into(),
        );
    } else if squeeze(&ka) == squeeze(&kb) {
        score = 0.98;
        reasons.push("differ only in spacing".into());
    } else if la == lb {
        score = 0.97;
        reasons.push("equal apart from descriptive words (group, holding, ...)".into());
    } else {
        let sorted = |t: &[String]| {
            let mut v = t.to_vec();
            v.sort();
            v.join(" ")
        };
        let ordered = jaro_winkler(&la, &lb);
        let reordered = jaro_winkler(&sorted(&a.tokens), &sorted(&b.tokens)) * 0.98;
        score = ordered.max(reordered);
        if reordered > ordered {
            reasons.push("same words in a different order".into());
        }
        let acronym = |short: &Company, long: &Company| {
            short.tokens.len() == 1
                && (2..=6).contains(&short.tokens[0].chars().count())
                && long.tokens.len() >= 2
                && initials(&long.tokens) == short.tokens[0]
        };
        let acronym_hit = acronym(a, b) || acronym(b, a);
        if acronym_hit {
            score = score.max(0.9);
            reasons.push("one name is the acronym of the other".into());
        }
        // a name under four letters is not fuzzy-matched, acronyms excepted
        if (ka.chars().count() < 4 || kb.chars().count() < 4) && !acronym_hit {
            score = score.min(0.6);
            reasons.push("names too short to trust a fuzzy match".into());
        }
        if score >= cfg.possible
            && !reasons
                .iter()
                .any(|r| r.contains("order") || r.contains("acronym"))
        {
            reasons.push(format!("names are {:.0}% alike", score * 100.0));
        }
    }

    if digits(&a.tokens) != digits(&b.tokens) && !identical && score > 0.8 {
        score = 0.8;
        reasons.push("numbers in the names differ".into());
    }

    let forms = match (&a.form, &b.form) {
        (Some(x), Some(y)) if x.canonical == y.canonical => FormRelation::Same,
        (Some(x), Some(y)) => {
            reasons.push(format!(
                "legal forms differ ({} vs {}): probably separate legal entities",
                x.canonical, y.canonical
            ));
            FormRelation::Conflict
        }
        _ => FormRelation::Unknown,
    };

    if forms == FormRelation::Conflict {
        identical = false;
        score = score.min(cfg.likely - 0.01);
    }

    let verdict = if identical {
        Verdict::Identical
    } else if score >= cfg.same {
        Verdict::Same
    } else if score >= cfg.likely {
        Verdict::Likely
    } else if score >= cfg.possible {
        Verdict::Possible
    } else {
        Verdict::Different
    };

    Match {
        score,
        verdict,
        forms,
        reasons,
    }
}

impl PartialOrd for Match {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaro_winkler_reference_values() {
        assert!((jaro_winkler("martha", "marhta") - 0.961).abs() < 0.001);
        assert!((jaro_winkler("dixon", "dicksonx") - 0.813).abs() < 0.001);
        assert_eq!(jaro_winkler("same", "same"), 1.0);
        assert_eq!(jaro_winkler("abc", "xyz"), 0.0);
    }

    #[test]
    fn spelling_variants_are_identical() {
        let m = compare("Müller & Söhne GmbH", "MUELLER und SOEHNE G.m.b.H.");
        assert_eq!(m.verdict, Verdict::Identical);
        assert_eq!(m.forms, FormRelation::Same);
    }

    #[test]
    fn missing_form_does_not_contradict() {
        let m = compare("Siemens AG", "Siemens");
        assert_eq!(m.verdict, Verdict::Identical);
        assert_eq!(m.forms, FormRelation::Unknown);
    }

    #[test]
    fn different_legal_forms_are_not_the_same_entity() {
        let m = compare("Siemens AG", "Siemens GmbH");
        assert_eq!(m.forms, FormRelation::Conflict);
        assert_eq!(m.verdict, Verdict::Possible);
        assert!(m.reasons.iter().any(|r| r.contains("AG vs GmbH")));
    }

    #[test]
    fn typos_and_spacing() {
        assert_eq!(
            compare("Deutsche Bank AG", "Deutschebank AG").verdict,
            Verdict::Same
        );
        assert!(compare("Volkswagen AG", "Volkswagn AG").verdict >= Verdict::Same);
    }

    #[test]
    fn word_order_and_descriptive_words() {
        assert!(compare("Bank Deutsche AG", "Deutsche Bank AG").verdict >= Verdict::Likely);
        assert_eq!(
            compare("Acme Holding GmbH", "ACME Group GmbH").verdict,
            Verdict::Same
        );
    }

    #[test]
    fn acronyms() {
        let m = compare("IBM", "International Business Machines Corp");
        assert!(m.verdict >= Verdict::Likely, "{m:?}");
        assert!(m.reasons.iter().any(|r| r.contains("acronym")));
    }

    #[test]
    fn numbers_matter() {
        let m = compare("Hotel Adler 1 GmbH", "Hotel Adler 2 GmbH");
        assert!(m.verdict <= Verdict::Possible);
        assert!(m.reasons.iter().any(|r| r.contains("numbers")));
    }

    #[test]
    fn unrelated_names_differ() {
        assert_eq!(
            compare("Apple Inc", "Microsoft Corp").verdict,
            Verdict::Different
        );
    }

    #[test]
    fn short_names_are_not_fuzzy_matched() {
        assert_eq!(compare("ABC", "ABD").verdict, Verdict::Different);
    }

    #[test]
    fn a_short_name_is_not_fuzzy_matched_against_a_longer_one() {
        // README: "names under four letters are not fuzzy-matched"
        let m = compare("ABC", "ABCD");
        assert_eq!(m.verdict, Verdict::Different, "{m:?}");
        let m = compare("ABC GmbH", "ABCD GmbH");
        assert_eq!(m.verdict, Verdict::Different, "{m:?}");
    }
}
