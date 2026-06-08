# Plan: P2 parser technical debt

> **Status:** Complete (June 2026). Open work: [PARSER_BACKLOG_ROADMAP.md](./PARSER_BACKLOG_ROADMAP.md).

This plan tracks the **P2** work from [PARSER_TECHNICAL_DEBT.md](./PARSER_TECHNICAL_DEBT.md). P1 (definition prefix + body helpers) is complete.

## Status (June 2026)

| Stream | Item | Status |
|--------|------|--------|
| A1 | Split `parser/mod.rs` → `recovery.rs`, `diagnostics.rs`, `collect_errors.rs`, `parse.rs` | Done |
| A1b | Sharpen generic nom diagnostic messages | Done |
| A2 | Split `tests/parser_tests.rs` → `tests/parser/*.rs` | Done |
| A3 | Split `src/ast.rs` → `src/ast/{core,kerml_fallback,mod}.rs` | Done (phase 1); further family modules optional |
| B1 | `package_body_element` keyword-group dispatch | Done |
| B2 | Action/state structured body via `parse_structured_brace_members` | Done (action + state); requirement keeps custom loop |
| B3 | Opaque bodies: dependency connect uses `advance_to_closing_brace` | Done (batch 1) |

## Regression gates

After each PR:

- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test -- --include-ignored` (validation + library gates)
- `cargo test --test bnf_compliance` after body-parser changes

## Follow-up (P3)

See [`PARSER_DEBT_P3_PLAN.md`](./PARSER_DEBT_P3_PLAN.md).

## Notes

- Recovery scope labels like `"action body"` are part of the diagnostic contract; do not rename without updating `invalid_bare_identifier_in_body_diagnostic` and `tests/recovery_actions.rs`.
