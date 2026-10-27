//! Legal forms (GmbH, AG, Ltd, B.V., S.A., ...): 91 forms in more than 50 countries across Europe, the Americas and Asia.
//!
//! A legal form is not part of a company's name but says *which legal entity* it is:
//! `Siemens AG` and `Siemens GmbH` are different companies.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::fold::{form_token, raw_tokens};

/// What kind of entity a legal form creates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class {
    /// Limited liability company (GmbH, Ltd, B.V., S.L., ...).
    Limited,
    /// Joint-stock corporation (AG, Inc, PLC, S.p.A., ...).
    Corporation,
    /// Partnership (KG, OHG, LLP, ...).
    Partnership,
    /// Sole trader (e.K., e.U.).
    SoleTrader,
    /// Cooperative (eG).
    Cooperative,
    /// Association (e.V.).
    Association,
    /// Foundation (Stiftung).
    Foundation,
    /// Generic suffix such as `Co`.
    Other,
}

/// A legal form with its canonical spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegalForm {
    /// The canonical spelling, for example `GmbH` for `G.m.b.H.` and `GesmbH`.
    pub canonical: &'static str,
    /// Comma-separated ISO 3166 codes of the countries that use it (`EU` for the European SE).
    pub countries: &'static str,
    /// The kind of entity.
    pub class: Class,
}

type Row = (&'static str, &'static str, Class, &'static [&'static str]);

const TABLE: &[Row] = &[
    (
        "GmbH",
        "DE,AT,CH",
        Class::Limited,
        &[
            "GmbH",
            "G.m.b.H.",
            "GesmbH",
            "Ges.m.b.H.",
            "Gesellschaft mit beschränkter Haftung",
        ],
    ),
    (
        "gGmbH",
        "DE",
        Class::Limited,
        &["gGmbH", "gemeinnützige GmbH"],
    ),
    (
        "UG",
        "DE",
        Class::Limited,
        &["UG", "UG (haftungsbeschränkt)", "Unternehmergesellschaft"],
    ),
    (
        "AG",
        "DE,AT,CH",
        Class::Corporation,
        &["AG", "A.G.", "Aktiengesellschaft"],
    ),
    ("SE", "EU", Class::Corporation, &["SE", "Societas Europaea"]),
    (
        "KGaA",
        "DE",
        Class::Corporation,
        &["KGaA", "Kommanditgesellschaft auf Aktien"],
    ),
    (
        "KG",
        "DE,AT,CH",
        Class::Partnership,
        &["KG", "Kommanditgesellschaft"],
    ),
    (
        "OHG",
        "DE,AT",
        Class::Partnership,
        &["OHG", "Offene Handelsgesellschaft"],
    ),
    (
        "GbR",
        "DE",
        Class::Partnership,
        &["GbR", "Gesellschaft bürgerlichen Rechts"],
    ),
    (
        "GmbH & Co. KG",
        "DE,AT",
        Class::Partnership,
        &["GmbH & Co. KG", "GmbH und Co. KG", "GmbH u. Co. KG"],
    ),
    ("AG & Co. KG", "DE", Class::Partnership, &["AG & Co. KG"]),
    (
        "UG & Co. KG",
        "DE",
        Class::Partnership,
        &["UG & Co. KG", "UG (haftungsbeschränkt) & Co. KG"],
    ),
    (
        "PartG mbB",
        "DE",
        Class::Partnership,
        &["PartG mbB", "PartG"],
    ),
    (
        "e.K.",
        "DE",
        Class::SoleTrader,
        &[
            "e.K.",
            "e.Kfm.",
            "e.Kfr.",
            "eingetragener Kaufmann",
            "eingetragene Kauffrau",
        ],
    ),
    (
        "e.U.",
        "AT",
        Class::SoleTrader,
        &["e.U.", "eingetragenes Unternehmen"],
    ),
    (
        "eG",
        "DE,AT,CH",
        Class::Cooperative,
        &["eG", "eingetragene Genossenschaft"],
    ),
    (
        "e.V.",
        "DE",
        Class::Association,
        &["e.V.", "eingetragener Verein"],
    ),
    ("Stiftung", "DE,AT,CH", Class::Foundation, &["Stiftung"]),
    (
        "Ltd",
        "GB,IE,CA,AU,IN",
        Class::Limited,
        &["Ltd", "Ltd.", "Limited"],
    ),
    (
        "PLC",
        "GB,IE",
        Class::Corporation,
        &["PLC", "Public Limited Company"],
    ),
    (
        "LLP",
        "GB,US",
        Class::Partnership,
        &["LLP", "Limited Liability Partnership"],
    ),
    (
        "LP",
        "US,GB",
        Class::Partnership,
        &["LP", "L.P.", "Limited Partnership"],
    ),
    (
        "LLC",
        "US",
        Class::Limited,
        &["LLC", "L.L.C.", "Limited Liability Company"],
    ),
    (
        "Inc",
        "US,CA",
        Class::Corporation,
        &["Inc", "Inc.", "Incorporated"],
    ),
    (
        "Corp",
        "US,CA",
        Class::Corporation,
        &["Corp", "Corp.", "Corporation"],
    ),
    ("Co", "US,GB", Class::Other, &["Co", "Co.", "Company"]),
    (
        "Co., Ltd.",
        "JP,CN,KR,TH,VN,TW",
        Class::Limited,
        &["Co., Ltd.", "Co Ltd", "Company Limited"],
    ),
    (
        "Pty Ltd",
        "AU,ZA",
        Class::Limited,
        &["Pty Ltd", "Pty. Ltd.", "Proprietary Limited"],
    ),
    ("Pte Ltd", "SG", Class::Limited, &["Pte Ltd", "Pte. Ltd."]),
    (
        "Pvt Ltd",
        "IN,PK",
        Class::Limited,
        &["Pvt Ltd", "Pvt. Ltd.", "Private Limited"],
    ),
    ("Sdn Bhd", "MY", Class::Limited, &["Sdn Bhd", "Sdn. Bhd."]),
    ("Bhd", "MY", Class::Corporation, &["Bhd", "Berhad"]),
    (
        "JSC",
        "RU,KZ,VN",
        Class::Corporation,
        &["JSC", "Joint Stock Company"],
    ),
    (
        "K.K.",
        "JP",
        Class::Corporation,
        &["K.K.", "KK", "Kabushiki Kaisha"],
    ),
    (
        "SA",
        "FR,CH,BE,LU,ES,PL",
        Class::Corporation,
        &[
            "SA",
            "S.A.",
            "Société Anonyme",
            "Sociedad Anónima",
            "Spółka Akcyjna",
        ],
    ),
    ("SAS", "FR", Class::Corporation, &["SAS", "S.A.S."]),
    ("SASU", "FR", Class::Corporation, &["SASU"]),
    (
        "SARL",
        "FR,CH,LU",
        Class::Limited,
        &[
            "SARL",
            "S.A.R.L.",
            "Sàrl",
            "S.à r.l.",
            "Société à responsabilité limitée",
        ],
    ),
    ("EURL", "FR", Class::Limited, &["EURL"]),
    ("SNC", "FR,IT", Class::Partnership, &["SNC", "S.N.C."]),
    (
        "B.V.",
        "NL",
        Class::Limited,
        &["B.V.", "BV", "Besloten Vennootschap"],
    ),
    (
        "N.V.",
        "NL,BE",
        Class::Corporation,
        &["N.V.", "NV", "Naamloze Vennootschap"],
    ),
    ("BVBA", "BE", Class::Limited, &["BVBA"]),
    ("VOF", "NL", Class::Partnership, &["V.O.F.", "VOF"]),
    (
        "S.L.",
        "ES",
        Class::Limited,
        &["S.L.", "SL", "Sociedad Limitada"],
    ),
    ("S.L.U.", "ES", Class::Limited, &["S.L.U.", "SLU"]),
    ("Lda", "PT", Class::Limited, &["Lda", "Lda.", "Limitada"]),
    ("Ltda", "BR,CL,CO", Class::Limited, &["Ltda", "Ltda."]),
    (
        "S.A. de C.V.",
        "MX",
        Class::Corporation,
        &["S.A. de C.V.", "SA de CV"],
    ),
    ("S.r.l.", "IT", Class::Limited, &["S.r.l.", "Srl", "S.R.L."]),
    ("S.p.A.", "IT", Class::Corporation, &["S.p.A.", "SpA"]),
    ("AB", "SE", Class::Corporation, &["AB", "Aktiebolag"]),
    (
        "AS",
        "NO,DK,EE,CZ,SK,TR",
        Class::Corporation,
        &["AS", "A/S", "A.S.", "A.Ş.", "Aksjeselskap", "Aktieselskab"],
    ),
    ("ASA", "NO", Class::Corporation, &["ASA"]),
    ("ApS", "DK", Class::Limited, &["ApS"]),
    ("Oy", "FI", Class::Limited, &["Oy"]),
    ("Oyj", "FI", Class::Corporation, &["Oyj"]),
    (
        "Sp. z o.o.",
        "PL",
        Class::Limited,
        &[
            "Sp. z o.o.",
            "Sp z oo",
            "Spółka z ograniczoną odpowiedzialnością",
        ],
    ),
    ("s.r.o.", "CZ,SK", Class::Limited, &["s.r.o.", "sro"]),
    ("Ltd. Şti.", "TR", Class::Limited, &["Ltd. Şti.", "Ltd Sti"]),
    ("OOO", "RU", Class::Limited, &["OOO", "ООО"]),
    ("AO", "RU", Class::Corporation, &["AO", "АО"]),
    ("PAO", "RU", Class::Corporation, &["PAO", "ПАО"]),
    ("ZAO", "RU", Class::Corporation, &["ZAO", "ЗАО"]),
    // ---- added in 0.2: Central and Eastern Europe, Baltics, Iceland, more German hybrids ----
    (
        "Kft.",
        "HU",
        Class::Limited,
        &["Kft.", "Korlátolt Felelősségű Társaság"],
    ),
    (
        "Zrt.",
        "HU",
        Class::Corporation,
        &["Zrt.", "Zártkörűen Működő Részvénytársaság"],
    ),
    (
        "Nyrt.",
        "HU",
        Class::Corporation,
        &["Nyrt.", "Nyilvánosan Működő Részvénytársaság"],
    ),
    (
        "d.o.o.",
        "HR,SI,RS,BA,ME,MK",
        Class::Limited,
        &["d.o.o.", "D.O.O.", "doo"],
    ),
    ("d.d.", "HR,SI,BA", Class::Corporation, &["d.d."]),
    ("EOOD", "BG", Class::Limited, &["EOOD"]),
    ("OOD", "BG", Class::Limited, &["OOD"]),
    ("UAB", "LT", Class::Limited, &["UAB"]),
    ("SIA", "LV", Class::Limited, &["SIA"]),
    ("OÜ", "EE", Class::Limited, &["OÜ"]),
    ("ehf.", "IS", Class::Limited, &["ehf."]),
    (
        "Sp. k.",
        "PL",
        Class::Partnership,
        &["Sp. k.", "spółka komandytowa"],
    ),
    (
        "Sp. j.",
        "PL",
        Class::Partnership,
        &["Sp. j.", "spółka jawna"],
    ),
    ("S.A.U.", "ES", Class::Corporation, &["S.A.U.", "SAU"]),
    (
        "S.Coop.",
        "ES",
        Class::Cooperative,
        &["S.Coop.", "S. Coop."],
    ),
    ("VZW", "BE", Class::Association, &["VZW", "v.z.w."]),
    (
        "SE & Co. KG",
        "DE",
        Class::Partnership,
        &["SE & Co. KG", "SE & Co KG", "SE u. Co. KG"],
    ),
    (
        "GmbH & Co. OHG",
        "DE,AT",
        Class::Partnership,
        &["GmbH & Co. OHG", "GmbH u. Co. OHG"],
    ),
    (
        "GmbH & Co. KGaA",
        "DE",
        Class::Partnership,
        &["GmbH & Co. KGaA"],
    ),
    ("mbH", "DE", Class::Limited, &["mbH", "m.b.H."]),
    (
        "gUG",
        "DE",
        Class::Limited,
        &["gUG", "gUG (haftungsbeschränkt)"],
    ),
    (
        "VVaG",
        "DE",
        Class::Other,
        &["VVaG", "Versicherungsverein auf Gegenseitigkeit"],
    ),
    ("G.K.", "JP", Class::Limited, &["G.K.", "Godo Kaisha"]),
    ("W.L.L.", "BH,QA,KW", Class::Limited, &["W.L.L.", "WLL"]),
    ("FZE", "AE", Class::Limited, &["FZE"]),
    ("PJSC", "AE,RU,UA,KZ", Class::Corporation, &["PJSC"]),
    ("PT", "ID", Class::Limited, &["PT", "P.T."]),
];

/// The longest legal form has this many tokens (`S.A. de C.V.`).
pub(crate) const MAX_FORM_TOKENS: usize = 6;

/// Canonical forms that are written in front of the name (`OOO Ромашка`, `PT Astra
/// International`). Only these are read at the start of a name: a leading `AB` or `AG`
/// is usually part of the name (`AB InBev`).
const PREFIX_FORMS: &[&str] = &["OOO", "AO", "PAO", "ZAO", "PT"];

/// The prefix form that opens `tokens` (the longest spelling wins), if one of
/// [`PREFIX_FORMS`] does and at least one word of the name follows it.
pub(crate) fn lookup_prefix(tokens: &[String]) -> Option<LegalForm> {
    (1..=MAX_FORM_TOKENS.min(tokens.len().saturating_sub(1)))
        .rev()
        .find_map(|k| lookup(&tokens[..k]).filter(|f| PREFIX_FORMS.contains(&f.canonical)))
}

fn index() -> &'static HashMap<String, LegalForm> {
    static INDEX: OnceLock<HashMap<String, LegalForm>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut map = HashMap::new();
        for (canonical, countries, class, variants) in TABLE {
            let form = LegalForm {
                canonical,
                countries,
                class: *class,
            };
            for v in *variants {
                map.entry(key_of(&raw_tokens(v))).or_insert(form);
            }
        }
        map
    })
}

fn key_of(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|t| form_token(t))
        .collect::<Vec<_>>()
        .concat()
}

/// The legal form spelled by exactly these tokens, if there is one.
pub(crate) fn lookup(tokens: &[String]) -> Option<LegalForm> {
    index().get(&key_of(tokens)).copied()
}

/// Every legal form known to the crate (one entry per canonical spelling).
pub fn all() -> Vec<LegalForm> {
    TABLE
        .iter()
        .map(|(canonical, countries, class, _)| LegalForm {
            canonical,
            countries,
            class: *class,
        })
        .collect()
}

impl LegalForm {
    /// Whether the form is used in the given ISO 3166 country code.
    pub fn is_used_in(&self, country: &str) -> bool {
        self.countries
            .split(',')
            .any(|c| c.eq_ignore_ascii_case(country))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(text: &str) -> Option<LegalForm> {
        lookup(&raw_tokens(text))
    }

    #[test]
    fn spellings_share_one_canonical_form() {
        for s in ["GmbH", "G.m.b.H.", "GesmbH", "gmbh"] {
            assert_eq!(form(s).unwrap().canonical, "GmbH", "{s}");
        }
        assert_eq!(form("S.r.l.").unwrap().canonical, "S.r.l.");
        assert_eq!(form("Sàrl").unwrap().canonical, "SARL");
    }

    #[test]
    fn multi_token_forms_and_connector_variants() {
        for s in ["GmbH & Co. KG", "GmbH und Co. KG", "GmbH u. Co. KG"] {
            assert_eq!(form(s).unwrap().canonical, "GmbH & Co. KG", "{s}");
        }
        assert_eq!(form("Sp. z o.o.").unwrap().canonical, "Sp. z o.o.");
        assert_eq!(form("UG (haftungsbeschränkt)").unwrap().canonical, "UG");
    }

    #[test]
    fn cyrillic_forms() {
        assert_eq!(form("ООО").unwrap().canonical, "OOO");
    }

    #[test]
    fn country_lookup() {
        assert!(form("GmbH").unwrap().is_used_in("at"));
        assert!(!form("GmbH").unwrap().is_used_in("FR"));
    }

    #[test]
    fn table_is_consistent() {
        // a variant must not be claimed by two different canonical forms
        let mut seen: HashMap<String, &str> = HashMap::new();
        for (canonical, _, _, variants) in TABLE {
            for v in *variants {
                let k = key_of(&raw_tokens(v));
                if let Some(prev) = seen.insert(k.clone(), canonical) {
                    assert_eq!(prev, *canonical, "variant {v:?} ({k}) is ambiguous");
                }
            }
        }
        assert!(TABLE
            .iter()
            .all(|r| r.3.iter().all(|v| raw_tokens(v).len() <= MAX_FORM_TOKENS)));
    }

    #[test]
    fn central_european_and_hybrid_forms_are_recognised() {
        for (text, canonical) in [
            ("Kft.", "Kft."),
            ("Zártkörűen Működő Részvénytársaság", "Zrt."),
            ("D.O.O.", "d.o.o."),
            ("d.d.", "d.d."),
            ("UAB", "UAB"),
            ("SIA", "SIA"),
            ("OÜ", "OÜ"),
            ("ehf.", "ehf."),
            ("Sp. k.", "Sp. k."),
            ("S.Coop.", "S.Coop."),
            ("SE & Co. KG", "SE & Co. KG"),
            ("SE u. Co. KG", "SE & Co. KG"),
            ("GmbH & Co. OHG", "GmbH & Co. OHG"),
            ("GmbH & Co. KGaA", "GmbH & Co. KGaA"),
            ("m.b.H.", "mbH"),
            ("gUG (haftungsbeschränkt)", "gUG"),
            ("Godo Kaisha", "G.K."),
            ("W.L.L.", "W.L.L."),
        ] {
            assert_eq!(form(text).map(|f| f.canonical), Some(canonical), "{text}");
        }
    }

    #[test]
    fn the_longest_hybrid_wins_over_its_tail() {
        use crate::Company;
        assert_eq!(
            Company::parse("Beispiel SE & Co. KG")
                .form
                .map(|f| f.canonical),
            Some("SE & Co. KG")
        );
        assert_eq!(
            Company::parse("Beispiel GmbH & Co. OHG")
                .form
                .map(|f| f.canonical),
            Some("GmbH & Co. OHG")
        );
    }

    #[test]
    fn prefix_forms_are_read_only_at_the_start() {
        let prefix = |text: &str| lookup_prefix(&raw_tokens(text)).map(|f| f.canonical);
        assert_eq!(prefix("ООО Ромашка"), Some("OOO"));
        assert_eq!(prefix("ЗАО «Ромашка»"), Some("ZAO"));
        assert_eq!(prefix("PT Astra International"), Some("PT"));
        assert_eq!(prefix("P.T. Astra"), Some("PT"));
        assert_eq!(prefix("AB InBev"), None);
        assert_eq!(prefix("Ltd Acme"), None);
        assert_eq!(prefix("OOO"), None);
        // a prefix form only in the middle does not count
        assert_eq!(prefix("Acme OOO Ромашка"), None);
    }

    #[test]
    fn indonesian_pt_is_a_form() {
        let pt = form("PT").unwrap();
        assert_eq!(pt.canonical, "PT");
        assert!(pt.is_used_in("ID"));
    }

    #[test]
    fn every_canonical_spelling_parses_to_itself() {
        for f in all() {
            assert_eq!(
                form(f.canonical).map(|x| x.canonical),
                Some(f.canonical),
                "{}",
                f.canonical
            );
        }
    }
}
