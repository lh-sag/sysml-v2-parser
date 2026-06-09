# Validation fixture tests

Each SysML file under `sysml-v2-release/sysml/src/validation/` can have a corresponding Rust test module in this directory. Populate `./sysml-v2-release/` with `scripts/fetch-sysml-v2-release.ps1` or `scripts/fetch-sysml-v2-release.sh`, or point `SYSML_V2_RELEASE_DIR` at an unpacked release tree.

## Layout

- **`tests/validation.rs`** – Shared helpers (`release_root`, `validation_fixture_path`, `assert_ast_eq`, `assert_ast_snapshot`) and module wiring.
- **`tests/validation/<name>.rs`** – One module per validation fixture.
- **`tests/validation/snapshots/`** – Normalized AST snapshots for fixtures that use `assert_ast_snapshot`.

## Adding a new validation test

1. Add a new `.rs` file in `tests/validation/`.
2. Use `super::validation_fixture_path(relative)` for the fixture path.
3. Compare with `assert_ast_eq` (hand-built expected AST) or `assert_ast_snapshot` (checked-in snapshot).
4. Wire the module in `tests/validation.rs`.

## When to regenerate snapshots

Refresh checked-in AST snapshots **in the same PR** whenever parser output changes, for example:

- new optional fields on existing AST structs (`AttributeDef.value_span`, usage header spans, …)
- new enum variants on body-element types (`MetadataAnnotation`, `MetadataKeywordUsage`, …)
- a construct now parses into a different variant (e.g. `@Tag : Type` as `MetadataAnnotation` instead of `Annotation`)
- structured brace parsing replaces `advance_to_closing_brace` / silent skip

CI always runs `cargo test -- --include-ignored`; local `cargo test` alone does **not** run these snapshot tests.

## Regenerate

All snapshot fixtures:

```powershell
$env:UPDATE_VALIDATION_AST = "1"
cargo test --test validation -- --include-ignored
Remove-Item Env:UPDATE_VALIDATION_AST
```

Or a subset (faster while iterating):

```powershell
$env:UPDATE_VALIDATION_AST = "1"
cargo test --test validation test_parse_1a_parts_tree test_parse_3a_function_based_behavior -- --include-ignored
Remove-Item Env:UPDATE_VALIDATION_AST
```

Unset `UPDATE_VALIDATION_AST` before committing. Review the `.txt` diff in `snapshots/` — only intentional AST changes should appear.
