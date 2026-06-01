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

### 1. Definition prefix boilerplate (high duplication, low risk to unify)

Many definition parsers share the same skeleton:

1. `ws_and_comments`
2. Optional `abstract`
3. Family keyword (`item`, `connection`, `interface`, …)
4. Optional or required `def`
5. `identification(input)`
6. `parse_optional_definition_header_after_identification(input)`
7. Family-specific body parser
8. `node_from_to` into a typed `*Def` AST node

Example ([`src/parser/item.rs`](../src/parser/item.rs)):

```rust
pub(crate) fn item_def(input: Input<'_>) -> IResult<Input<'_>, Node<ItemDef>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = opt(preceded(tag("abstract"), ws1)).parse(input)?;
    let (input, _) = tag("item").parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, _) = opt(preceded(tag("def"), ws1)).parse(input)?;
    let (input, identification) = identification(input)?;
    let (input, (specializes, specializes_span)) =
        parse_optional_definition_header_after_identification(input)?;
    let (input, body) = attribute_body(input)?;
    // ...
}
```

Families differ mainly by keyword, whether `def` is optional, and which body parser runs next.

**Recommended improvement:** extract `parse_definition_prefix(keyword, DefOptions { abstract_ok, def_required })` and reuse across simple defs. The header-after-ident step is already centralized; the prelude is the next low-risk consolidation.

### 2. Body terminators (medium duplication)

The pattern “`;` or `{ ... }`” is repeated across modules (`flow`, `allocation`, `metadata`, `occurrence`, …), often as local `definition_body` helpers. Some use structured member parsing; others use `skip_until_brace_end` and accept opaque brace content.

**Recommended improvement:** shared helpers in `lex.rs` or a small `body.rs`:

- `semicolon_or_structured_brace_body(parse_member)` — real `*_body_element` loops
- `semicolon_or_opaque_brace()` — intentional shell bodies; document which families still use this

This does not remove per-family modules; it stops copy-pasting the same `alt(semicolon, delimited("{", skip, "}"))` blocks.

### 3. Package dispatch (large surface, mostly intentional)

[`package_body_element`](../src/parser/package.rs) is a long ordered dispatch chain. Much of it is **disambiguation policy** (e.g. `part_def` vs `part_usage`, `attribute_def` vs `attribute_usage`). A single giant `alt(...)` is not clearly better until disambiguation rules stabilize.

**Recommended improvement (later):** sub-dispatchers grouped by keyword family (`package_body_requirement_family`, `package_body_structure_family`, …), aligned with `PACKAGE_BODY_STARTERS` in [`lex.rs`](../src/parser/lex.rs). Worth doing when adding constructs becomes painful, not preemptively.

### 4. Recovery loops (medium duplication, high value if unified)

`recover_body_element` plus `build_recovery_error_node_from_span` loops appear in `part`, `action`, `state`, `requirement`, `constraint`, `view`, and others. The shape is always: try parse member → on failure recover and skip → push `Error` node → continue.

**Recommended improvement:** a generic `parse_body_members(input, starters, parse_one)` for structured bodies. Improves language-server resilience and reduces bug surface when touching recovery.

### 5. AST shape duplication (structural, larger refactor)

Many `*Def` structs repeat `identification`, `specializes`, `specializes_span`, and `body`. This mirrors the compliance gap: the **shared KerML definition/usage layer** from the spec is not yet a single grammar layer in code.

**Recommended improvement (larger):** an internal `DefinitionDecl { keyword, prefixes, identification, header, body }` mapped to typed AST variants for downstream consumers. Drive this from grammar work, not from deduplication alone.

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

## Prioritized improvements

| Priority | Change | Effort | Benefit |
|----------|--------|--------|---------|
| **P1** | `parse_definition_prefix` + `DefOptions` per keyword | Small | Fewer header bugs; one place for `abstract` / `def` / keyword prelude |
| **P1** | Shared `semicolon_or_*_body` helpers | Small | Less body boilerplate; explicit list of opaque-body families |
| **P2** | Generic structured body loop with recovery | Medium | Less recovery duplication; better editor behavior |
| **P2** | Split `package_body_element` into keyword-group sub-dispatchers | Medium | Easier extension without reordering dozens of branches |
| **P3** | Unified definition/usage header (typing, multiplicity, subsets, redefines) | Large | Spec-aligned; fixes whole classes of library edge cases |
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
| Best next step? | **P1** shared definition prefix and body terminators, without AST redesign. |
| Largest long-term gap? | **Unified definition/usage/specialization grammar** plus deeper body parsing, not more top-level `*_def` files. |

The validation CI regression fixed in 2026 (typed library headers after `identification`) illustrates the preferred direction: **extract shared grammar fragments** as they are discovered, keep construct modules, and let library node-shape gates enforce that dedicated parsers stay on the happy path.
