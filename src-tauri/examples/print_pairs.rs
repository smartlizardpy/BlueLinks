//! Print challenges the way the game would generate them, so the article pool
//! and the difficulty tuning can be judged without installing anything.
//!
//! cargo run --manifest-path src-tauri/Cargo.toml --example print_pairs -- ../data/articles.sqlite 20
//! cargo run --manifest-path src-tauri/Cargo.toml --example print_pairs -- ../data/articles.sqlite 20 evil

use bluelink_lib::{
    dataset::Dataset,
    randomizer::generate,
    types::{DifficultyPreset, Settings},
};
use std::{env, path::PathBuf};

fn main() {
    let mut args = env::args().skip(1);
    let path = PathBuf::from(args.next().unwrap_or_else(|| "../data/articles.sqlite".into()));
    let count: usize = args.next().and_then(|n| n.parse().ok()).unwrap_or(20);
    let evil = args.next().is_some_and(|flag| flag == "evil");

    let dataset = match Dataset::open(&path) {
        Ok(dataset) => dataset,
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            std::process::exit(1);
        }
    };

    let mut settings = Settings::default();
    if evil {
        settings.default_difficulty = DifficultyPreset::Evil;
    }

    println!("{} ({})", path.display(), if evil { "evil" } else { "normal" });
    let mut previous = None;
    for _ in 0..count {
        match generate(&dataset, previous.as_ref(), &settings) {
            Ok(challenge) => {
                println!(
                    "  {:.2}  {}  ->  {}   ({} clicks)",
                    challenge.difficulty, challenge.start.title, challenge.target.title, challenge.click_limit
                );
                previous = Some(challenge);
            }
            Err(error) => {
                eprintln!("  generation failed: {error}");
                std::process::exit(1);
            }
        }
    }
}
