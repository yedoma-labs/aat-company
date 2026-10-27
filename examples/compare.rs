//! Compares company names.
//!
//!     cargo run --example compare -- "Siemens AG" "Siemens GmbH"

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: compare NAME NAME");
        std::process::exit(2);
    }
    for name in &args {
        let c = aat::Company::parse(name);
        println!(
            "{name:?} -> key {:?}, form {:?}",
            c.key(),
            c.form.map(|f| f.canonical)
        );
    }
    let m = aat::compare(&args[0], &args[1]);
    println!(
        "{:?} (score {:.2}, legal forms: {:?})",
        m.verdict, m.score, m.forms
    );
    for r in &m.reasons {
        println!("  - {r}");
    }
}
