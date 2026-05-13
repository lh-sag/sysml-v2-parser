use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::panic::{catch_unwind, AssertUnwindSafe};

use sysml_v2_parser::parse_with_diagnostics;

fn collect_sysml_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sysml_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "sysml") {
            files.push(path);
        }
    }
}

#[test]
#[ignore = "requires MBSE_VACUUM_EXAMPLE_DIR to point at the MBSE vacuum-cleaner corpus"]
fn mbse_vacuum_corpus_parse_with_diagnostics_is_bounded_and_panic_free() {
    let Some(root) = std::env::var_os("MBSE_VACUUM_EXAMPLE_DIR").map(PathBuf::from) else {
        eprintln!("MBSE_VACUUM_EXAMPLE_DIR is not set; skipping optional corpus regression");
        return;
    };
    assert!(
        root.is_dir(),
        "MBSE_VACUUM_EXAMPLE_DIR should be a directory: {}",
        root.display()
    );

    let mut files = Vec::new();
    collect_sysml_files(&root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "expected at least one .sysml file under {}",
        root.display()
    );

    let mut primary_codes = BTreeMap::<String, usize>::new();
    for path in files {
        let input = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        let result = catch_unwind(AssertUnwindSafe(|| parse_with_diagnostics(&input)))
            .unwrap_or_else(|_| panic!("parse_with_diagnostics panicked for {}", path.display()));
        assert!(
            result.errors.len() <= 100,
            "diagnostic count should stay bounded for {}: {}",
            path.display(),
            result.errors.len()
        );
        for err in result.errors {
            let code = err.code.unwrap_or_else(|| "unknown".to_string());
            *primary_codes.entry(code).or_default() += 1;
        }
    }

    eprintln!("MBSE vacuum corpus primary diagnostic codes: {primary_codes:#?}");
}
