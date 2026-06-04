# Plan: P4 parser technical debt

Tracks **P4** work from [PARSER_TECHNICAL_DEBT.md](./PARSER_TECHNICAL_DEBT.md). P1–P3 are complete.

## Status

| Stream | Item | Status |
|--------|------|--------|
| 0 | Docs + P3 marked complete | Done |
| C | `view_body`, `part_def_body_brace` → structured loop | Done |
| B | Port/attribute usage AST (`references`/`crosses`, multi-target) | Done |
| E | Recovery tests + diagnostics sharpen | Done |
| A | `definition_header.rs` + pilot defs | Done |
| D | `OwnedExpression` tranche 1 (`implies`) | Done |
| F | `part.rs` module split | Done |

## Regression gates

After each PR:

- `cargo fmt`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo test -- --include-ignored`
- `cargo test --test bnf_compliance` (body/expr/header changes)
- `cargo test --test recovery_views` (view body)
- `cargo test --test recovery_body_scopes` (part body)

## Scope labels (diagnostic contract)

- `"view body"` / `recovered_view_body_element`
- `"part definition body"` / `recovered_part_def_body_element`

## Notes

- `part_def_body_brace` keeps `part`→`part_usage` retry before recovery (not in generic loop).
- P5 (out of scope): semantic layer, full BNF bodies, `part_def` prelude unify.
