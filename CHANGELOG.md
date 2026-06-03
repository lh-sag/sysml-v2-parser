# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.16.0] - 2026-06-03

### Added

- **Requirement body actors**: `RequirementActorDecl` and `actor_decl` in requirement definition bodies (anonymous `actor : Type;` mirrors existing `subject : Type;`).
- **Enumeration usages in part bodies**: `EnumerationUsage`, `enum_usage` parser, and `PartDefBodyElement::EnumerationUsage` / `PartUsageBodyElement::EnumerationUsage` for `enum name : Type;` inside part definitions and usages.
- **Part definition members**: `ItemUsage` and `CalcUsage` in `part_def_body_element` (library-style `item` / `calc` usages in part defs).
- **Diagnostics taxonomy** ([`src/parser/mod.rs`](src/parser/mod.rs)): `DiagnosticCategory`, `DiagnosticSeverity` on `ParseError`; classification for invalid requirement short names (`id '…'`), bare features in part defs, invalid typing operators, and related recovery codes.
- **Editor-oriented post-processing**: cascade suppression (`recovery_cascade_suppressed`), deduplication by specificity, and `suppress_redundant_closing_brace_errors` when a line already reports an invalid `{…}` statement block.
- **Corpus-oriented checks**: `collect_implicit_attribute_in_part_def_warnings`, `collect_requirement_id_dialect_diagnostics`; Apollo regression test [`tests/apollo_regressions.rs`](tests/apollo_regressions.rs).
- **Recovery fixtures/tests**: anonymous actor in requirement, enum in part def, calc usage in part def, bare feature hint, nested part-def typed usages, requirement `id` dialect hint; glued `}package` now expected to parse cleanly.

### Changed

- **`parse_with_diagnostics`**: no longer emits `missing_statement_separator_between_members` for valid glued `}package` boundaries; stricter trailing-`}` handling at root with `unexpected_closing_brace` where appropriate.
- **Recovery**: `missing_member_name` skips anonymous `subject` / `actor` before `:` only in `"requirement body"` scope (use case `actor:` without a name still diagnosed).
- **`part_usage_body_element`**: nested `alt` to stay within nom tuple limits after new enum arm.

### Fixed

- **False positives** on spec-aligned models: `missing_member_name` on `actor : Type` in requirement bodies; `unexpected_keyword_in_scope` for `enum` in part defs; bogus separator errors at `}package`.
- **SurveillanceDrone-errors** validation expectations aligned with multi-package recovery (four root packages, three member-level errors).

### Migration (Spec42 and similar hosts)

1. Bump to `sysml-v2-parser` `0.16.0` (crates.io or tag `v0.16.0`).
2. Match on `RequirementDefBodyElement::RequirementActorDecl` (not a separate top-level `ActorDecl` in requirement bodies — use case `ActorDecl` remains distinct).
3. Handle `PartDefBodyElement::EnumerationUsage` and `PartUsageBodyElement::EnumerationUsage` in graph builders (or ignore like other usage members).
4. Remove handling for diagnostic code `missing_statement_separator_between_members` if you branched on it.
5. Re-run `cargo test` and validation fixtures after bumping.

[0.16.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.15.0...v0.16.0

## [0.15.0] - 2026-06-03

### Breaking

- **`PortBody`**: removed variant `BraceWithPorts { elements: Vec<Node<PortUsage>> }`. Nested port bodies now use `PortBody::Brace { elements: Vec<Node<PortBodyElement>> }` with structured members (`PortUsage`, `InOutDecl`, `Error`, `Other`). Update exhaustive matches and any code that assumed nested ports were only `PortUsage` nodes.
- **`AttributeBody`**: brace bodies are now `AttributeBody::Brace { elements: Vec<Node<AttributeBodyElement>> }` instead of an opaque skipped brace. Members include nested attributes, doc comments, annotations, and recovery `Error` nodes.
- **`DefinitionBody` / `RenderingDefBody`**: generic definition and rendering definition brace bodies now expose structured `DefinitionBodyElement` / `RenderingDefBodyElement` lists (doc, occurrence members, recovery errors) rather than opaque skipped content for occurrence, rendering, flow, allocation, and metadata families.

### Added

- **BNF compliance gate (100% `implemented`)**: machine-readable map [`docs/bnf_coverage.map`](docs/bnf_coverage.map) and [`tests/bnf_compliance.rs`](tests/bnf_compliance.rs) classify all 640 SysML/KerML textual productions; new tests assert zero `partial` map rules and full production coverage. See [`docs/BNF_COVERAGE.md`](docs/BNF_COVERAGE.md) and [`docs/BNF_COMPLIANCE_MATRIX.md`](docs/BNF_COMPLIANCE_MATRIX.md).
- **Shared usage grammar** ([`src/parser/usage.rs`](src/parser/usage.rs)): `usage_header`, `feature_usage_header`, `specialization_clauses`, `subsetting` / `redefinition`, plus `references` (`::>`) and `crosses` (`=>`) operators; supports `defined by`, `typed by`, conjugated types (`~`), and multiple specialization clauses (last-wins where the AST stores a single target).
- **Structured body parsing** ([`src/parser/body.rs`](src/parser/body.rs)): `parse_structured_brace_members` and `advance_to_closing_brace` replace opaque `skip_until_brace_end` in many high-traffic modules (attribute, part, port, occurrence, rendering, flow, allocation, metadata, connection, interface, import, alias, enumeration, constraint, use case).
- **Expression surface** ([`src/parser/expr.rs`](src/parser/expr.rs)): `select` (`.?`), `collect` (`.**`), and parenthesized sequence expressions; precedence-aware binary/unary chain unchanged as the main `expression()` entry point.
- **BNF surface helpers** ([`src/parser/bnf_surface.rs`](src/parser/bnf_surface.rs)): shared entry points and unit tests for lexical terminals, empty productions, and usage/definition declaration fragments.
- **Lexical operators** ([`src/parser/lex.rs`](src/parser/lex.rs)): `references_operator`, `crosses_operator`, `decimal_value_text`, `string_value`, plus lexical BNF unit tests.
- **Action control nodes**: action definition bodies recognize `accept`, `decision`, `fork`, `join`, `send`, `terminate`, `while`, and `if` starters as control-node action usages.
- **CI**: workflow fetches the pinned SysML v2 release tree and runs `cargo test` with `SYSML_V2_RELEASE_DIR` so the BNF gate and default tests run against normative fixtures on every push.
- **Docs**: updated [`docs/SYSML_V2_COMPLIANCE_GAP.md`](docs/SYSML_V2_COMPLIANCE_GAP.md), [`docs/PARSER_TECHNICAL_DEBT.md`](docs/PARSER_TECHNICAL_DEBT.md), and validation README/snapshots for structured parsing regressions.

### Changed

- **Part and port definitions/usages**: brace bodies parse structured member AST with recovery (`PartDefBody`, `PortDefBody`, `PortBodyElement`) instead of swallowing inner grammar.
- **Action and state definitions**: definition-level bodies use structured member loops with `skip_statement_or_block` recovery (no `skip_until_brace_end` on promoted top-level defs guarded by `bnf_compliance`).
- **Requirement, case, view, and usage families**: migrated to shared `usage_header` / `feature_usage_header` where applicable (requirement/case/analysis/verification/action/state/view/rendering/viewpoint/use-case usages, concern usage, calc definition prefix).
- **Specialization targets**: subsetting and redefinition accept qualified names with dotted feature chains and comma-separated target lists.
- **Validation tests**: `parts_tree_1a`, `parts_interconnection_2a`, `function_based_behavior_3a`, and `functional_allocation_4a` refactored to snapshot-based checks aligned with structured AST shapes.

### Fixed

- **Port nested bodies**: `port` usages inside `port` brace bodies (e.g. left/right redefinitions) parse into `PortBodyElement::PortUsage` instead of a separate `BraceWithPorts` shape.
- **Library typing headers**: `defined by` and `typed by` accepted alongside `:` on usage headers; multiple `:>` / `:>>` / `subsets` / `redefines` clauses parse without spurious recovery on common stdlib patterns.
- **Part `ref` lines**: optional comments and formatting around `ref part` assignments tolerated in part usage bodies.

### Migration (Spec42 and similar hosts)

1. Bump the `sysml-v2-parser` dependency to `0.15.0` (or the matching git revision / path).
2. Replace `PortBody::BraceWithPorts` matches with `PortBody::Brace` and handle `PortBodyElement` (nested ports are `PortBodyElement::PortUsage`).
3. If you read attribute or generic definition brace bodies as opaque text, switch to iterating `AttributeBodyElement` / `DefinitionBodyElement` (or keep using span recovery `Error` / `Other` members for unsupported inner forms).
4. For usage typing and specialization, prefer `usage_header` semantics: `references` / `crosses` may appear in the same clause stream as subsets (stored via the shared specialization path where the AST has a single subsets slot).
5. Run `cargo test`, `cargo test --test bnf_compliance`, and `cargo test --test validation -- --include-ignored` with `SYSML_V2_RELEASE_DIR` pointing at the release BNF tree.

[0.15.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.14.0...v0.15.0

## [0.14.0] - 2026-06-02

### Added

- **Qualified package identifiers**: package and namespace declarations now accept qualified names in the identification position (e.g. `package AstronomyReference::Domain { ... }`) and keep the full qualified path in the AST.
- **`ref part` assignment forms**: part usage bodies now parse `ref part` declarations with optional typing and optional value binding (e.g. `ref part centralBody = sun;`, `ref part orbitingBody : Body = earth;`) without recovery diagnostics.

### Fixed

- **Reference usage grammar coverage**: `ref part` declarations that omit explicit typing are no longer forced into a `:` parse path, aligning parser behavior with SysML v2 reference-usage notation.

### Migration (Spec42 and similar hosts)

1. Bump the `sysml-v2-parser` dependency to `0.14.0` (or the matching git revision / path).
2. If downstream code assumes `package`/`namespace` names are unqualified, update it to handle `::`-qualified identifiers in `Identification.name`.
3. Re-run parser and semantic smoke tests that cover `ref part` declarations with and without type annotations.

[0.14.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.13.0...v0.14.0

## [0.13.0] - 2026-06-01

### Breaking

- **Definition subclassification on AST nodes**: many `*Def` types now include `specializes: Option<String>` and `specializes_span: Option<Span>` when a declaration uses `:>` / `specializes` or a library-style typed header before subclassification (e.g. `abstract connection name : Connection[0..*] :> linkObjects, parts`). Affected types include (among others) `ItemDef`, `IndividualDef`, `InterfaceDef`, `ConnectionDef`, `PortDef`, `RequirementDef`, `ConstraintDef`, `StateDef`, `ActionDef`, `FlowDef`, `AllocationDef`, `MetadataDef`, `OccurrenceDef`, `EnumDef`, and the case/view/use-case definition families. Any manual struct literals or exhaustive construction must set these fields (`None` when absent).

### Added

- **Shared definition prelude** ([`src/parser/definition_prefix.rs`](src/parser/definition_prefix.rs)): `parse_definition_prefix` with `DefinitionPrefixOptions` centralizes `abstract`, optional `private`, optional `#` annotation, keyword/`def`, and header-after-ident parsing for migrated definition parsers.
- **Shared opaque body terminator** ([`src/parser/body.rs`](src/parser/body.rs)): `semicolon_or_opaque_brace_body` for `;` or brace bodies whose inner content is skipped (`flow`, `allocation`, `metadata`, and related usages).
- **Header helper** ([`src/parser/specialization.rs`](src/parser/specialization.rs)): `parse_optional_definition_header_after_identification` handles direct `:>` / `specializes` and typed headers (`: Type[multiplicity] … :> bases`) after `identification`.
- **Docs**: [`docs/PARSER_TECHNICAL_DEBT.md`](docs/PARSER_TECHNICAL_DEBT.md) and [`docs/PARSER_DEBT_P1_PLAN.md`](docs/PARSER_DEBT_P1_PLAN.md) document parser duplication, P1 consolidation (complete), and follow-up P2/P3 work.

### Changed

- **Internal refactor (P1)**: eighteen `*_def` entry points (item, individual, interface, metadata, connection, constraint, port, requirement, state, occurrence, flow, allocation, case/analysis/verification, view/viewpoint/rendering, use case, enum, action) delegate their prelude to `parse_definition_prefix`. `part_def`, `calc_def`, usages, `alias_def`, and `dependency` remain on local preludes by design.
- **Numeric literals**: decimal and scientific-notation forms are parsed more consistently in expression paths.

### Fixed

- **Systems / full library gates**: declarations such as `abstract connection … : Type[…] :> …` and `private abstract constraint def …` map to dedicated definition nodes again (`ExtendedLibraryDecl` count stays at zero with `cargo test -- --include-ignored`).
- **Calc and constraint bodies**: `return` expressions in calculation definitions and constraint bodies parse without swallowing following members.
- **Definition prefix modifier order**: `private` is accepted before `abstract` (stdlib `private abstract constraint def`).

### Migration (Spec42 and similar hosts)

1. Bump the `sysml-v2-parser` dependency to `0.13.0` (or the matching git revision / path).
2. Update any manual `*Def` struct literals to include `specializes` and `specializes_span` (use `None` when not modeled).
3. When building semantics from definitions, read `specializes` / `specializes_span` for subclassification edges; typed library headers populate `specializes` from the `:>` clause after the skipped typing fragment.
4. Re-run `cargo test --test validation -- --include-ignored` after upgrading.

[0.13.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.12.0...v0.13.0

## [0.12.0] - 2026-05-28

### Breaking

- **`AttributeUsage`**: added `typing: Option<String>` and `typing_span: Option<Span>` for the type after `:` or `:>` on attribute usages (e.g. `attribute totalMassKg : MassValue`). Any struct literals or manual construction of `AttributeUsage` must set these fields (use `None` when untyped).

### Fixed

- **Typed attribute usages in usage bodies**: `attribute` name followed by `:` or `:>` and a qualified type name now parses as `AttributeUsage` with `typing` populated, including inside `part` usage bodies. Previously the parser rejected this form in usage contexts (recovery / wrong classification). This matches OMG SysML v2 `AttributeUsage = UsagePrefix 'attribute' Usage`, where typing is part of the usage, not only of `attribute def`.
- **Attribute def vs usage disambiguation**: in definition bodies (`part def`, `port def`, `requirement def`), the parser tries `attribute def` before `attribute usage` so typed declaration members such as `attribute mass :> ISQ::mass` remain `AttributeDef`. Untyped value assignments (`attribute actualMass = measuredMass`), `redefines` / `:>>` forms, and prefix redefinitions (`attribute :>> propellantMass = …`) still parse as `AttributeUsage`. Package- and use-case-level attributes are unchanged (`attribute x = expr` stays `AttributeDef`). Fixes validation fixture `1a-Parts Tree.sysml` and similar library models.
- **`:>` vs `:>>` on attributes**: attribute typing no longer treats `:>>` as a `:>` prefix. Prefix-redefine usages (`attribute :>> currentTime : TimeInstantValue`) accept an optional `: Type` after the redefine target; a following `:>` is left for subsetting (e.g. `attribute :>> outlet :> electricGrid.outlets`). `attribute def` requires a declared name so bare `attribute :>> …` is not misclassified.

### Migration (Spec42 and similar hosts)

1. Bump the `sysml-v2-parser` dependency to `0.12.0` (or the matching git revision).
2. Update `AttributeUsage` struct literals to include `typing` and `typing_span`.
3. When building semantics from attribute usages, read `AttributeUsage::typing` for type edges in **usage** bodies (e.g. nested `part` usages).
4. In **definition** bodies, typed members without `def` (e.g. `attribute massActual: MassValue` in `requirement def`) continue to surface as `AttributeDef`; do not assume every typed `attribute` is an `AttributeUsage`.
5. Re-run `cargo test --test validation -- --include-ignored` after upgrading; the full validation and std-library gates should be green.

[0.12.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.11.0...v0.12.0

## [0.11.0] - 2026-05-28

### Breaking

- **`UseCaseDefBodyElement`**: added new variant `AttributeDef(Node<AttributeDef>)` so that `attribute` definitions inside a `use case def` body are surfaced in the AST. Any exhaustive `match` on `UseCaseDefBodyElement` must add an arm for `AttributeDef`.

### Fixed

- **Transition names vs transition keywords**: optional transition names such as `docked` are no longer dropped when the name shares a prefix with `first`, `if`, `do`, or `then`. The parser now uses whole-keyword detection (`starts_with_keyword`) so `transition docked first docking then charging;` parses correctly.

### Migration (Spec42 and similar hosts)

1. Bump the `sysml-v2-parser` dependency to `0.11.0` (or the matching git revision).
2. If you exhaustively match on `UseCaseDefBodyElement`, add an arm for the new `AttributeDef` variant (carry-through is usually identical to the existing `AttributeDef` arms on `PartDefBodyElement` or `RequirementDefBodyElement`).

[0.11.0]: https://github.com/elan8/sysml-v2-parser/compare/v0.10.0...v0.11.0

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
