//! R4 + D7: alloy types must not spread beyond the boundary.
//!
//! Run: `cargo test -p etherscanlite --test isolation`
//!
//! This is the mechanical enforcement the pre-mortem asks for. Convention alone loses:
//! the first `use alloy_primitives::B256;` added to `main.rs` "just for a test" normalises
//! the next one, and by then the type families have merged.

use std::fs;
use std::path::Path;

/// Only these may name an alloy crate.
///
/// `client.rs` is on the list because the provider call sites unavoidably touch alloy
/// types — that is precisely why the RPC calls live there and not in `main.rs`.
const BOUNDARY_FILES: &[&str] = &["boundary.rs", "client.rs"];

/// Matching on the crate names rather than `alloy::` catches the umbrella spelling too,
/// should someone re-add that dependency.
const ALLOY_MARKERS: &[&str] = &[
    "alloy_primitives",
    "alloy_rpc_types_eth",
    "alloy_consensus",
    "alloy_provider",
    "alloy::",
];

#[test]
fn no_alloy_types_outside_the_boundary() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    for entry in fs::read_dir(&src).expect("src/ is readable") {
        let path = entry.expect("readable entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if BOUNDARY_FILES.contains(&name.as_str()) {
            continue;
        }

        let contents = fs::read_to_string(&path).expect("readable source");
        for (lineno, line) in contents.lines().enumerate() {
            // Skip comments: the modules legitimately *discuss* alloy.
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for marker in ALLOY_MARKERS {
                if line.contains(marker) {
                    violations.push(format!("{name}:{}: {}", lineno + 1, line.trim()));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "alloy types escaped the boundary — convert them in {BOUNDARY_FILES:?} instead:\n{}",
        violations.join("\n")
    );
}

#[test]
fn the_boundary_files_still_exist() {
    // Guards the test above from silently passing if a file is renamed: with no boundary
    // files present, "no violations" would be vacuously true.
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for name in BOUNDARY_FILES {
        assert!(src.join(name).exists(), "missing boundary file: {name}");
    }
}
