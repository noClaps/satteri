//! Parse all .mdx files from the docs repo and report results.

use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let docs_dir = PathBuf::from("/home/erika/Projects/docs");
    let opts = parser::ParseOptions::mdx();

    let mut files = Vec::new();
    collect_mdx_files(&docs_dir, &mut files);
    files.sort();

    println!("Found {} .mdx files", files.len());

    let mut success = 0;
    let mut failures: Vec<(PathBuf, String)> = Vec::new();
    let mut total_bytes = 0u64;

    let start = Instant::now();

    for path in &files {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                failures.push((path.clone(), format!("read error: {e}")));
                continue;
            }
        };
        total_bytes += source.len() as u64;

        // Catch panics.
        let result = std::panic::catch_unwind(|| {
            let arena = parser::parse(&source, &opts);
            // Also try the full pipeline to HTML.
            let _html = tryckeri_hast::arena_to_html(&arena);
            arena.len()
        });

        match result {
            Ok(node_count) => {
                success += 1;
                let _ = node_count;
            }
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "unknown panic".to_string()
                };
                failures.push((path.clone(), msg));
            }
        }
    }

    let elapsed = start.elapsed();

    println!("\nResults:");
    println!("  Success: {}/{}", success, files.len());
    println!("  Failed:  {}", failures.len());
    println!("  Total:   {:.2} MB in {:.2?}", total_bytes as f64 / 1_048_576.0, elapsed);
    println!("  Speed:   {:.2} MB/s", total_bytes as f64 / 1_048_576.0 / elapsed.as_secs_f64());

    if !failures.is_empty() {
        println!("\nFailures:");
        for (path, msg) in &failures {
            let rel = path.strip_prefix(&docs_dir).unwrap_or(path);
            println!("  {} — {}", rel.display(), msg);
        }
    }
}

fn collect_mdx_files(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_mdx_files(&path, out);
        } else if path.extension().map_or(false, |e| e == "mdx") {
            out.push(path);
        }
    }
}
