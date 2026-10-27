# aat-company

<picture>
  <source media="(max-width: 640px)" srcset="https://raw.githubusercontent.com/yedoma-labs/assets/main/resized/banner-resized-mobile.png">
  <img src="https://raw.githubusercontent.com/yedoma-labs/assets/main/resized/banner-resized.png" alt="Project Header">
</picture>

> **Company name normalization and matching for Rust** — legal-form aware, explainable, dependency-free

[![CI](https://github.com/yedoma-labs/aat-company/actions/workflows/ci.yml/badge.svg)](https://github.com/yedoma-labs/aat-company/actions/workflows/ci.yml)
[![Security Audit](https://github.com/yedoma-labs/aat-company/actions/workflows/audit.yml/badge.svg)](https://github.com/yedoma-labs/aat-company/actions/workflows/audit.yml)
[![Crates.io](https://img.shields.io/crates/v/aat-company.svg)](https://crates.io/crates/aat-company)
[![Documentation](https://docs.rs/aat-company/badge.svg)](https://docs.rs/aat-company)
[![License](https://img.shields.io/crates/l/aat-company.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-stable-blue.svg)](https://www.rust-lang.org)
[![No Unsafe](https://img.shields.io/badge/unsafe-forbidden-success)](src/lib.rs)
[![Dependencies](https://img.shields.io/badge/dependencies-0-success)](Cargo.toml)
[![Conventional Commits](https://img.shields.io/badge/commits-conventional-fe5196?logo=conventionalcommits&logoColor=white)](https://www.conventionalcommits.org/)

*aat* (аат; Yakutian/Sakha) = "name"

## Why aat-company?

**The Problem:** The same company is written a hundred ways: `Müller & Söhne GmbH`, `MUELLER und SOEHNE G.m.b.H.`, `Müller u. Söhne`. A generic fuzzy matcher scores `Siemens AG` and `Siemens GmbH` as nearly identical, although they are two legal entities, and it scores `Hotel Adler 1` and `Hotel Adler 2` as one hotel. For KYC, CRM de-duplication, invoice matching and supplier lists, both mistakes cost money.

**The Solution:** Split the legal form off the name, compare the cores, and treat a *conflicting* legal form as evidence against a match. Every result says why.

```rust
use aat::{compare, Verdict};

let m = compare("Müller & Söhne GmbH", "MUELLER und SOEHNE G.m.b.H.");
assert_eq!(m.verdict, Verdict::Identical);

let m = compare("Siemens AG", "Siemens GmbH");
assert_eq!(m.verdict, Verdict::Possible);        // never "same"
println!("{}", m.reasons.join("; "));
// same name core after normalisation (...); legal forms differ (AG vs GmbH): probably separate legal entities
```

## What makes it unique

| | |
|---|---|
| **Legal-form aware** | 90 legal forms from more than 50 countries (GmbH, UG, AG, KG, GmbH & Co. KG, e.K., eG, e.V., Ltd, LLC, Inc, PLC, B.V., N.V., SARL, SAS, S.L., S.r.l., S.p.A., AB, A/S, Oy, Sp. z o.o., s.r.o., OOO, K.K., ...), every spelling (`G.m.b.H.`, `GesmbH`, `Gesellschaft mit beschränkter Haftung`) mapped to one canonical form. The longest spelling wins, so `GmbH & Co. KG` is not read as `KG`. |
| **Conflict is evidence** | Two different legal forms never merge, in `compare` or in `dedupe`. A missing form does not contradict. |
| **Numbers and short names are guarded** | `Hotel Adler 1` vs `2` is capped below a match; names under four letters are not fuzzy-matched (`ABC` is not `ABD`). |
| **German-first folding** | `ä ö ü ß` become `ae oe ue ss` (`Müller` = `Mueller`), `&`/`und`/`u.`/`and`/`et` are connectors, `A.P. Møller` and `AP Moller` meet. Other Latin diacritics and Cyrillic forms work too. |
| **Acronyms and descriptors** | `IBM` ↔ `International Business Machines Corp`; `Acme Holding` ↔ `ACME Group`. |
| **Explainable** | `Match` carries a score, a `Verdict` (`Different < Possible < Likely < Same < Identical`), the legal-form relation and reasons in plain sentences. `dedupe` returns the comparisons that merged each cluster. |
| **Dependency-free, deterministic** | Zero dependencies, `#![forbid(unsafe_code)]`, same input → same output. `dedupe` uses blocking, so it does not compare all pairs. |

## Usage

```toml
[dependencies]
aat-company = "0.1"
```

The library is imported as `aat`.

```rust
use aat::{dedupe, Company, Verdict};

let c = Company::parse("Beispiel Verwaltungs GmbH & Co. KG");
assert_eq!(c.form.unwrap().canonical, "GmbH & Co. KG");
assert_eq!(c.key(), "beispiel verwaltungs");
assert_eq!(c.loose_key(), "beispiel");               // without descriptive words

let names = ["Siemens AG", "SIEMENS Aktiengesellschaft", "Siemens GmbH", "Apple Inc"];
for cluster in dedupe(&names, Verdict::Same) {
    println!("keep {:?}, duplicates {:?}", names[cluster.canonical], cluster.members);
}
```

Looking up one name in a known list, for example before creating a customer record:

```rust
use aat::{Index, Verdict};

let crm = Index::new(&["Müller & Söhne GmbH", "Siemens AG", "Siemens Energy AG"]);
let hit = crm.best("MUELLER und SOEHNE G.m.b.H.", Verdict::Likely).unwrap();
assert_eq!(hit.name, "Müller & Söhne GmbH");
```

```bash
cargo run --example compare -- "Siemens AG" "Siemens GmbH"
printf 'Siemens AG\nSIEMENS Aktiengesellschaft\nApple Inc\n' | cargo run --example dedupe
```

## Limitations

- **Names only.** It does not know that two names with nothing in common belong to one group (`Google LLC` / `Alphabet Inc`). Use registry identifiers (LEI, Handelsregister number, VAT ID) where you have them; this crate is for when you don't.
- **Legal form at the end.** Prefix forms (`AB Volvo`, `PT Astra`) are not split off; the prefix stays in the name.
- **Latin and Cyrillic scripts.** Other scripts are compared as written, without transliteration.
- **Transitive clusters.** `dedupe` merges A~B and B~C into one cluster. Use `Verdict::Same` or higher to avoid chains of near-matches.
- **Blocking.** `dedupe` compares names that share the start of their words, their initials or their first letters without spaces. A pair that differs in all of these (typo in the first letters of every word) is not found; `compare` still scores it if you ask.
- **Thresholds are heuristics.** Tune `Config`, and review `Possible` and `Likely` results by hand for anything legally binding.
- **Novelty claim.** We searched crates.io (2026-10) for company, legal-form and organisation-name crates and found no equivalent; the Python library `cleanco` has no Rust counterpart that we know of. We cannot prove none exists.

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
git config core.hooksPath .githooks   # Conventional Commits
```

See [CONTRIBUTING.md](CONTRIBUTING.md). New legal forms: add a row to the table in `src/forms.rs`; the consistency test rejects spellings claimed by two forms.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
