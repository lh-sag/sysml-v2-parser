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

Regenerate snapshots after intentional parser output changes:

```powershell
$env:UPDATE_VALIDATION_AST = "1"
cargo test --test validation -- --include-ignored parts_tree_1a functional_allocation_4a function_based_behavior_3a
Remove-Item Env:UPDATE_VALIDATION_AST
```
