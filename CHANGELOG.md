# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Transition names vs transition keywords**: optional transition names such as `docked` are no longer dropped when the name shares a prefix with `first`, `if`, `do`, or `then`. The parser now uses whole-keyword detection (`starts_with_keyword`) so `transition docked first docking then charging;` parses correctly.

## [0.10.0] - 2026-05-13

### Breaking

- **`PartDefBodyElement`**: added new variant `InterfaceDef(Node<InterfaceDef>)` so that nested `interface def` declarations inside a `part def` body are surfaced in the AST. Any exhaustive `match` on `PartDefBodyElement` must add an arm for `InterfaceDef`.
- **`parse_root` strict mode**: a stray trailing `}` after a well-formed root namespace is now reported as `unexpected_closing_brace` instead of being silently accepted. Inputs that previously parsed under `parse_root` but contained extra closing braces will now return an error (these inputs already produced diagnostics from `parse_with_diagnostics`).

### Added

- **Nested `interface def` in part definitions**: `part def` bodies now accept nested `interface def` (and continue to accept `interface` usages), matching the OMG SysML v2 Part 1 textual grammar which lists `InterfaceDefinition` under `DefinitionElement`. New fixtures cover the nested form and assert no recovery diagnostics.
- **Diagnostic code `invalid_bare_identifier_in_action_body` / `invalid_bare_identifier_in_state_body`**: bare identifiers in action and state bodies (e.g. `action a { batCap; maxBatCap; }`) now produce a targeted message naming valid forms (`perform`, `bind`, `in`/`out`, `entry`, `transition`, `then`, …) instead of the generic `unexpected_keyword_in_scope`.
- **Diagnostic code `recovery_cascade_suppressed`**: after three consecutive `missing_semicolon` or `recovered_*` diagnostics in the same body region, a single warning-severity summary replaces the remaining cascade entries, pointing back to the first error to fix.
- **Diagnostic code `recovered_root_body`**: when a root-level `package` / `library` / `standard package` / `namespace` body fails to parse, the recovery path emits one root-scoped error and skips to the next root element, preventing cascades across unrelated definitions in the same file.
- **Docs**: new `docs/CORPUS_MBSE_VACUUM_PARSER_SPEC42_FEEDBACK.md` capturing findings from running the parser/Spec42 stack against the public MBSE vacuum-cleaner robot example, plus a documentation index in `README.md`.

### Fixed

- **`interface` usage with no whitespace before `:`**: `interface : Foo;` (and similar forms without a space between the keyword and the colon) is now accepted.
- **`comment` annotation prefixes**: `comment` annotations tolerate arbitrary tokens between the optional name/about clauses and the opening `/* … */` comment body, matching real-world inputs that include extra metadata before the comment.
- **Part / state body recovery**: classification codes `invalid_bare_identifier_in_action_body`, `invalid_bare_identifier_in_state_body`, `unexpected_keyword_in_scope`, `missing_semicolon`, and `missing_body_or_semicolon` now produce `Other` placeholder elements in `PartDefBody` / `StateDefBody` so downstream tooling can see the skipped span.

### Reliability

- Cascade suppression and the root-body recovery error together significantly reduce diagnostic volume on large real-world corpora where a single structural error previously fanned out into hundreds of follow-up `missing_semicolon` / `recovered_*` entries.

### Migration (Spec42 and similar hosts)

1. Bump the `sysml-v2-parser` dependency to `0.10.0` (or the matching git revision).
2. If you exhaustively match on `PartDefBodyElement`, add an arm for the new `InterfaceDef` variant (carry-through is usually identical to the existing `InterfaceUsage` arm).
3. Diagnostic consumers can opt to treat `recovery_cascade_suppressed` as informational (it carries `severity: Warning`) and to display `recovered_root_body` as the primary error for affected root scopes.

[0.10.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.9.0...v0.10.0

## [0.9.0] - 2026-05-04

### Breaking

- **`AttributeDef`**: added optional field `value: Option<Node<Expression>>` for default / value parts after `=`, `:=`, or `default =` on attribute definitions (e.g. `attribute n: Integer = 0;`). Update any exhaustive matches or struct literals that construct `AttributeDef`.
- **Expression `Span` for parenthesized grouping**: a single expression in parentheses `( expr )` now uses a node span covering the full `(` … `)` in the source (not only the inner expression). Tools that slice source by `Span` (e.g. joining `require constraint` text) may see different byte ranges than in 0.8.x for the same logical tree.
- **Numeric literal parsing**: `literal_only` tries `literal_real` before `literal_integer`, so decimals such as `0.9` parse as reals instead of integer `0` with a stray `.9`. Rare integer-vs-real edge cases in malformed or unusual inputs may produce a different AST than before.

### Fixed

- **Quantity literals**: bracket units such as `[m/s]` or library-style names with `::` inside `[` … `]` parse more reliably into `LiteralWithUnit`.
- **Constraint and calc brace bodies**: optional terminating `;` after each body item is accepted, so chained expressions split with `;` (e.g. `(a <= b); and (c <= d);`) map to multiple `Expression` elements instead of falling through to `Other`.
- **Recovery**: `inout` is included in constraint/calc body recovery keyword lists alongside `in` / `out`.

### Reliability

- Slightly longer preview text for `Other` placeholders in constraint/calc recovery paths (diagnostics).

### Migration (Spec42 and similar hosts)

1. Bump the `sysml-v2-parser` dependency to `0.9.0` (or the matching git revision).
2. Add `value: None` (or the parsed value) wherever you construct `AttributeDef` manually; re-run tests that assert on expression source spans inside parentheses or on joined constraint text.

**Local smoke (optional):** In a Spec42 checkout, add to `.cargo/config.toml` a `[patch."https://github.com/elan8/sysml-v2-parser"]` entry with `sysml-v2-parser = { path = "../sysml-v2-parser" }`, then run `cargo update -p sysml-v2-parser` and `cargo check -p kernel`. Remove the patch afterward unless you intend to keep developing against a local parser build.

[0.9.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.8.0...v0.9.0
