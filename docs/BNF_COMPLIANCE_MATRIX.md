# BNF Compliance Matrix

This is the primary compact coverage snapshot for the parser.

Reference grammar:

- `sysml-v2-release/bnf/SysML-textual-bnf.kebnf`

Machine-readable coverage gate:

- [`docs/bnf_coverage.map`](./bnf_coverage.map), validated by `cargo test --test bnf_compliance`

Status labels:

- `implemented`: dedicated AST + dedicated parser path, exercised by the current validation baseline
- `partial`: dedicated parser exists and is validated for common forms, but coverage may still rely on permissive body parsing or subset-only grammar support
- `modeled`: parsed into BNF-aligned modeled declaration nodes (`KermlSemanticDecl` / `KermlFeatureDecl` / `ExtendedLibraryDecl`) instead of a fully dedicated construct-specific AST

Important distinction:

- **Accepted grammar fragment** means strict parsing recognizes the BNF surface form and preserves the existing public AST shape where fields exist.
- **Fully structured AST body** means brace body contents are parsed into construct-specific member nodes, not consumed by `skip_until_brace_end` or generic statement-only body parsing.

The coverage gate treats `implemented` as the stronger claim. Attribute, occurrence, part, port, flow, allocation, and metadata productions remain `partial` while any associated body path still relies on opaque or statement-only parsing, even when their headers now accept more BNF forms.

## Package-level declaration families

- `package`, `library package`, `namespace`, `import`: `implemented`
- `part`, `port`: `partial`; shared usage typing/specialization fragments are accepted, but body coverage is still not a full BNF-modeled AST for every member
- `attribute`: `partial`; `:` / `defined by` / `typed by` and specialization clauses are accepted for definitions/usages, but extra usage specializations are not all public AST fields yet and brace bodies are still opaque
- `occurrence`: `partial`; `:` / `defined by` / `typed by`, `subsets`, and `redefines` are accepted on usages with current last-wins normalization
- `requirement usage`, `case usage`, `analysis case usage`, `verification case usage`, `action usage`, `state usage`, `view usage`, `rendering usage`: `implemented` (shared `usage_header` flow accepts `:` / `defined by` / `typed by` plus specialization clauses)
- `action`, `state`, `requirement`, `case`, `analysis`, `verification`, `flow`, `allocation`, `interface`, `view`, `viewpoint`, `rendering`, `metadata`, `enum`: `partial` for the broader families due to remaining body/member-depth gaps outside the promoted usage productions
- KerML semantic families (`behavior`, `function`, `datatype`, `assoc`, `struct`, `metaclass`, `class`, `classifier`, `feature`, `step`): `modeled`
- KerML feature logic families (`occurrence`, `expr`, `predicate`, `succession`): `modeled`
- Extended declaration starters (`message`, `concern` and remaining library declarations): `modeled`

## Validation gates

- `test_systems_library_strict_no_diagnostics`: required green
- `test_full_library_strict_no_diagnostics`: required green
- `test_full_library_suite`: broad integration visibility
- `test_systems_library_node_types_no_extended`: required green (**hard 0 `ExtendedLibraryDecl` for Systems Library**)
- `test_full_library_node_types_no_extended`: required green (**hard 0 `ExtendedLibraryDecl` for full std library**)
  - supports staged migration threshold via env var `FULL_LIBRARY_EXTENDED_MAX`
  - default threshold is `0` (strict)

## Current quality baseline (2026-06-02)

- `cargo test` is green.
- `cargo test --test validation -- --include-ignored` is green, including the full validation suite and full SysML library gates.
- Systems Library node-shape validation passes with `ExtendedLibraryDecl = 0`.
- Full std-library node-shape validation also passes with `ExtendedLibraryDecl = 0`.
- The main remaining work is deeper body-level modeling precision and language-server hardening, not package-level fallback elimination.
