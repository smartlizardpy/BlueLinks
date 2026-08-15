// Pair inspection is intentionally a development-only command.
// `cargo test` exercises scoring; use the Python dataset tool's --print-pairs option
// for a quick metadata-level sample without launching the application.
fn main() {
    println!(
        "Generate data/articles.sqlite with tools/build_dataset.py --development --print-pairs"
    );
}
