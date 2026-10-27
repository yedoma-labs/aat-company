//! End-to-end checks through the public API.

use aat::{compare, dedupe, Company, FormRelation, Verdict};

#[test]
fn german_spelling_variants() {
    for other in [
        "MUELLER und SOEHNE G.m.b.H.",
        "Müller u. Söhne GmbH",
        "müller & söhne gmbh",
    ] {
        let m = compare("Müller & Söhne GmbH", other);
        assert_eq!(m.verdict, Verdict::Identical, "{other}");
    }
}

#[test]
fn legal_forms_across_countries() {
    for (name, form) in [
        ("Acme Ltd.", "Ltd"),
        ("Acme Limited", "Ltd"),
        ("Acme B.V.", "B.V."),
        ("Acme S.à r.l.", "SARL"),
        ("Acme Pty. Ltd.", "Pty Ltd"),
        ("Acme Co., Ltd.", "Co., Ltd."),
        ("Acme Sp. z o.o.", "Sp. z o.o."),
        ("Acme A/S", "AS"),
        ("Acme UG (haftungsbeschränkt) & Co. KG", "UG & Co. KG"),
        ("Acme S.A. de C.V.", "S.A. de C.V."),
    ] {
        let c = Company::parse(name);
        assert_eq!(c.form.map(|f| f.canonical), Some(form), "{name}");
        assert_eq!(c.key(), "acme", "{name}");
    }
}

#[test]
fn prefix_forms_match_their_suffix_spelling() {
    let m = compare("ООО Ромашка", "Ромашка ООО");
    assert_eq!(m.verdict, Verdict::Identical);
    assert_eq!(m.forms, FormRelation::Same);
    assert_eq!(
        compare("PT Astra International", "Astra International PT").verdict,
        Verdict::Identical
    );
}

#[test]
fn prefix_forms_conflict_like_suffix_forms() {
    let m = compare("ООО Ромашка", "ЗАО Ромашка");
    assert_eq!(m.forms, FormRelation::Conflict);
    assert!(m.verdict < Verdict::Likely);
}

#[test]
fn conflict_never_merges() {
    let m = compare("Acme GmbH", "Acme AG");
    assert_eq!(m.forms, FormRelation::Conflict);
    assert!(m.verdict < Verdict::Likely);
    assert_eq!(
        dedupe(&["Acme GmbH", "Acme AG"], Verdict::Possible).len(),
        2
    );
}

#[test]
fn deterministic() {
    let names = ["a gmbh", "A GmbH", "b", "B Inc", "b inc."];
    assert_eq!(dedupe(&names, Verdict::Same), dedupe(&names, Verdict::Same));
}

#[test]
fn hostile_input_does_not_panic() {
    for s in [
        "",
        " ",
        "&",
        "GmbH",
        "AG & Co. KG",
        "🙂🙂",
        "日本語株式会社",
        &"a ".repeat(10_000),
        &"x".repeat(100_000),
    ] {
        let _ = compare(s, "Acme GmbH");
        let _ = Company::parse(s);
    }
}

#[test]
fn large_lists_stay_fast() {
    let names: Vec<String> = (0..1500)
        .map(|i| {
            format!(
                "{} Handels GmbH",
                ["Alpha", "Beta", "Gamma", "Delta", "Omega"][i % 5].to_string()
                    + &(i / 5).to_string()
            )
        })
        .collect();
    let start = std::time::Instant::now();
    let clusters = dedupe(&names, Verdict::Same);
    assert_eq!(clusters.len(), 1500);
    assert!(start.elapsed().as_secs() < 20, "took {:?}", start.elapsed());
}
