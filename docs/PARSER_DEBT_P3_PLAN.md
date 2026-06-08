# Plan: P3 parser technical debt

> **Status:** Complete (June 2026). Open work: [PARSER_BACKLOG_ROADMAP.md](./PARSER_BACKLOG_ROADMAP.md).

This plan tracks **P3** work from [PARSER_TECHNICAL_DEBT.md](./PARSER_TECHNICAL_DEBT.md). P1 and P2 are complete.

## Status

| Stream | Item | Status |
|--------|------|--------|
| G | `PARSER_DEBT_P3_PLAN.md`, P2 marked complete, compliance gap sync | Done |
| A | AST `mod.rs` → `common`, `structure`, `behavior`, `requirement`, `view`, `package`, `root` | Done |
| B | Requirement def body → `parse_structured_brace_members` (library Other/Error) | Done |
| C | `body.rs` fallback: `advance_to_closing_brace` in structured loop | Done |
| D | Action usage body → shared structured loop | Done |
| E | Usage header AST fidelity (`references` / `crosses` on `UsageHeader` / `OccurrenceUsage`) | Done |
| F | LSP: constraint/calc/view def bodies → shared structured loop + error nodes | Done |

## Regression gates

After each PR:

- `cargo fmt`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo test -- --include-ignored`
- `cargo test --test bnf_compliance` after body/recovery changes

## Notes

- Recovery scope labels (`"action body"`, `"state body"`, `"requirement body"`) are diagnostic contracts.
- Requirement bodies keep library-tolerant `Other` vs `Error` policy when migrating to the shared loop.
- P4 (not P3): unified `DefinitionDecl` layer, full `OwnedExpression`.
