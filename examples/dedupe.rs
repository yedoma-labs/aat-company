//! Groups company names read from stdin, one per line.
//!
//!     printf 'Siemens AG\nSIEMENS Aktiengesellschaft\nApple Inc\n' | cargo run --example dedupe

use std::io::BufRead;

fn main() {
    let names: Vec<String> = std::io::stdin()
        .lock()
        .lines()
        .map_while(Result::ok)
        .collect();
    for cluster in aat::dedupe(&names, aat::Verdict::Same) {
        println!("keep: {}", names[cluster.canonical]);
        for &i in &cluster.members {
            if i != cluster.canonical {
                println!("  duplicate: {}", names[i]);
            }
        }
    }
}
