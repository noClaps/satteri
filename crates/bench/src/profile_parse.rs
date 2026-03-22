/// Profiling binary: hammers `parser::parse` in a tight loop so perf/flamegraph
/// gets enough samples to show a meaningful call graph.
///
/// Run via: cargo flamegraph -p tryckeri-bench --bin profile_parse

fn main() {
    let src = include_str!("../fixtures/markdown.md");
    let opts = parser::ParseOptions::default();

    // Warm up to avoid cold-start noise.
    for _ in 0..100 {
        let _ = parser::parse(src, &opts);
    }

    // Profile window — enough iterations for ~5s of samples.
    for _ in 0..50_000 {
        let arena = parser::parse(src, &opts);
        std::hint::black_box(arena);
    }
}
