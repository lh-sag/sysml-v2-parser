# Parser technical debt overview

This document describes structural duplication and architectural gaps in `sysml-v2-parser` as of June 2026. It complements:

- [`SYSML_V2_COMPLIANCE_GAP.md`](./SYSML_V2_COMPLIANCE_GAP.md) — what is implemented vs partial vs permissive
- [`BNF_COMPLIANCE_MATRIX.md`](./BNF_COMPLIANCE_MATRIX.md) — compact grammar-family snapshot

The parser currently passes `cargo test`, the full validation suite (`cargo test -- --include-ignored`), and strict library node-shape gates (`ExtendedLibraryDecl = 0`). Technical debt here is about **maintainability and grammar depth**, not about missing CI green.

## Current architecture (summary)

The codebase is in a **broad coverage, construct-specific modules** phase:

| Layer | Pattern |
|-------|---------|
| Top-level defs | ~25 `*_def` entry points (`item_def`, `connection_def`, `port_def`, …) |
| Package dispatch | Large ordered `if let Ok` chain in `package_body_element` (~50 branches) |
| Bodies | Per-family parsers; many still use `skip_until_brace_end` for inner content |
| Fallback | `KermlSemanticDecl`, `KermlFeatureDecl`, `ExtendedLibraryDecl` when no dedicated path matches |

That layout delivered green validation and drove `ExtendedLibraryDecl` to zero in library gates. The trade-off is **grammar unity** for **incremental delivery**.

A recent example: library declarations such as `abstract connection name : Type[multiplicity] nonunique :> redefines { ... }` require skipping a **typed header** before subclassification. When `parse_optional_definition_specialization` replaced `take_until_terminator` after `identification` without handling `: Type ... :>`, several defs failed and fell through to `ExtendedLibraryDecl`. The fix was `parse_optional_definition_header_after_identification` in [`src/parser/specialization.rs`](../src/parser/specialization.rs) — a small shared primitive, not a full rewrite.

## Where duplication appears

### 1. Definition prefix boilerplate — **P1 done (June 2026)**

[`src/parser/definition_prefix.rs`](../src/parser/definition_prefix.rs) provides `parse_definition_prefix` with `DefinitionPrefixOptions` (`DefKeywordMode`, `VisibilityPrefix`, `AnnotationMode`, optional `second_keyword` for `use case`). Migrated `*_def` parsers: item, individual, interface, metadata, connection, constraint, port, requirement, state, occurrence, flow, allocation, case / analysis / verification, view / viewpoint / rendering, use case, enum, action.

**Still on local preludes (intentional):** `part_def` (usage disambiguation), `*_usage`, `alias_def`, `dependency`, `calc_def`, `attribute_def`.

### 2. Body terminators — **opaque helper done (P1)**

[`src/parser/body.rs`](../src/parser/body.rs) exports `semicolon_or_opaque_brace_body`. Used by **flow**, **allocation**, and **metadata** definitions (and allocation/flow usages that share the same terminator).

**Opaque-body families (June 2026):** flow, allocation, metadata.

**Still local:** occurrence and other modules with their own `definition_body` helpers; structured bodies unchanged.

**P2 (not done):** `semicolon_or_structured_brace_body` with member parsing and recovery.

### 3. Package dispatch (large surface, mostly intentional)

[`package_body_element`](../src/parser/package.rs) is a long ordered dispatch chain. Much of it is **disambiguation policy** (e.g. `part_def` vs `part_usage`, `attribute_def` vs `attribute_usage`). A single giant `alt(...)` is not clearly better until disambiguation rules stabilize.

**Recommended improvement (later):** sub-dispatchers grouped by keyword family (`package_body_requirement_family`, `package_body_structure_family`, …), aligned with `PACKAGE_BODY_STARTERS` in [`lex.rs`](../src/parser/lex.rs). Worth doing when adding constructs becomes painful, not preemptively.

### 4. Recovery loops (medium duplication, high value if unified)

`recover_body_element` plus `build_recovery_error_node_from_span` loops appear in `part`, `action`, `state`, `requirement`, `constraint`, `view`, and others. The shape is always: try parse member → on failure recover and skip → push `Error` node → continue.

**Recommended improvement:** a generic `parse_body_members(input, starters, parse_one)` for structured bodies. Improves language-server resilience and reduces bug surface when touching recovery.

### 5. AST shape duplication (structural, larger refactor)

Many `*Def` structs repeat `identification`, `specializes`, `specializes_span`, and `body`. This mirrors the compliance gap: the **shared KerML definition/usage layer** from the spec is not yet a single grammar layer in code.

**Recommended improvement (larger):** an internal `DefinitionDecl { keyword, prefixes, identification, header, body }` mapped to typed AST variants for downstream consumers. Drive this from grammar work, not from deduplication alone.

### 6. Shared usage grammar fragments — **started**

[`src/parser/usage.rs`](../src/parser/usage.rs) now centralizes small `UsageDeclaration` / `FeatureSpecializationPart` fragments: multiplicity, `TypedBy` (`:` / `defined by`), subsetting, and redefinition. `port_usage` and `part_usage` have been migrated first, including `defined by` and multiple typing targets. Next candidates are `attribute_usage`, occurrence usages, and requirement/case usages that still parse these fragments locally.

## What is not wasteful duplication

| Pattern | Why it stays |
|---------|----------------|
| Separate modules per SysML family (`part.rs`, `requirement.rs`, …) | Clear ownership, targeted tests, incremental BNF alignment |
| Per-fixture validation tests under `tests/validation/` | Catches regressions the aggregate suite might miss |
| `ExtendedLibraryDecl` as last resort | Safety net; library gates require count = 0 on the happy path |
| Ordered dispatch in `package_body_element` | Reflects real keyword disambiguation, not arbitrary repetition |

## Relationship to compliance gaps

From [`SYSML_V2_COMPLIANCE_GAP.md`](./SYSML_V2_COMPLIANCE_GAP.md):

1. **Generic definition/usage/specialization** — still distributed across construct-specific parsers instead of one unified layer (largest architectural gap).
2. **Permissive bodies** — `skip_until_brace_end` and related helpers in metadata, occurrence, alias, import, and parts of other modules.
3. **Expression subset** — `expr.rs` is precedence-aware but not full `OwnedExpression`.
4. **Recovery / LSP** — solid baseline; more specific diagnostics and coverage still wanted.

Duplication in code and “partial grammar” in the spec sense overlap: the same missing shared header/body grammar shows up as copy-pasted parsers *and* as `ExtendedLibraryDecl` or opaque bodies when a shortcut fails.

## Implementation plan (P1)

**Status: complete (June 2026).** Spec and checklist: [`PARSER_DEBT_P1_PLAN.md`](./PARSER_DEBT_P1_PLAN.md). Code: [`definition_prefix.rs`](../src/parser/definition_prefix.rs), [`body.rs`](../src/parser/body.rs).

## Prioritized improvements

| Priority | Change | Effort | Benefit |
|----------|--------|--------|---------|
| ~~**P1**~~ | ~~`parse_definition_prefix` + options per keyword~~ | Done | Central prelude for migrated defs |
| ~~**P1**~~ | ~~`semicolon_or_opaque_brace_body`~~ | Done | flow / allocation / metadata |
| **P2** | Generic structured body loop with recovery | Medium | Less recovery duplication; better editor behavior |
| **P2** | Split `package_body_element` into keyword-group sub-dispatchers | Medium | Easier extension without reordering dozens of branches |
| **P3** | Unified definition/usage header (typing, multiplicity, subsets, redefines) | Started | Spec-aligned; fixes whole classes of library edge cases |
| **P3** | Replace `skip_until_brace_end` in high-traffic bodies | Large | Deeper AST; significant work per module |

## What to avoid

- **Monolithic “parser framework” rewrite** while validation and library gates are green — high risk of re-breaking `ExtendedLibraryDecl` and strict diagnostics tests.
- **Dedup-only refactors** without grammar tests — merging code paths without fixture coverage tends to hide regressions until the full library suite runs.
- **Removing fallback nodes prematurely** — keep `ExtendedLibraryDecl` at zero via dedicated parsers, not by deleting the fallback.

## Recommended workflow for refactors

1. Introduce a small shared primitive (like `parse_optional_definition_header_after_identification`).
2. Add or extend unit tests on the primitive and one representative family parser.
3. Migrate similar families in a single PR; run `cargo test -- --include-ignored`.
4. Document any family that still uses opaque bodies in this file or in module-level comments.

## Summary

| Question | Answer |
|----------|--------|
| Is there a lot of duplication? | **Yes** — especially definition prefixes, body terminators, and recovery loops. |
| Is the codebase unmaintainable? | **No** — modules and tests are coherent; debt is known and gated. |
| Best next step? | **P2** structured body recovery loop; optional package sub-dispatch. |
| Largest long-term gap? | **Unified definition/usage/specialization grammar** plus deeper body parsing, not more top-level `*_def` files. |

The validation CI regression fixed in 2026 (typed library headers after `identification`) illustrates the preferred direction: **extract shared grammar fragments** as they are discovered, keep construct modules, and let library node-shape gates enforce that dedicated parsers stay on the happy path.
