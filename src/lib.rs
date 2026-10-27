//! # aat-company
//!
//! *aat* (аат; Sakha) = "name".
//!
//! Company name normalization and matching that knows legal forms. Business names are
//! written a hundred ways (`Müller & Söhne GmbH`, `MUELLER und SOEHNE G.m.b.H.`), and the
//! legal form decides *which* company it is (`Siemens AG` is not `Siemens GmbH`). Generic
//! fuzzy matching knows neither. This crate does, and says why it decided as it did.
//!
//! ```
//! use aat::{compare, Verdict};
//!
//! let m = compare("Müller & Söhne GmbH", "MUELLER und SOEHNE G.m.b.H.");
//! assert_eq!(m.verdict, Verdict::Identical);
//!
//! let m = compare("Siemens AG", "Siemens GmbH");
//! assert_eq!(m.verdict, Verdict::Possible); // never "same": the legal forms differ
//! println!("{}", m.reasons.join("; "));
//! ```
//!
//! - [`Company::parse`] splits a name into core words and [`LegalForm`] (91 forms
//!   in more than 40 countries, all spellings mapped to one canonical form; a few, such as
//!   `OOO` and `PT`, are read at the start of a name)
//! - [`compare`] scores two names and returns a [`Verdict`] with reasons
//! - [`dedupe()`](fn@dedupe) groups a list of names, with the evidence for every merge
//!
//! No dependencies, no `unsafe`, deterministic.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod company;
pub mod dedupe;
mod fold;
pub mod forms;
pub mod index;
pub mod similarity;

pub use company::Company;
pub use dedupe::{dedupe, Cluster, Link};
pub use forms::{Class, LegalForm};
pub use index::{Hit, Index};
pub use similarity::{compare, compare_parsed, jaro_winkler, Config, FormRelation, Match, Verdict};
