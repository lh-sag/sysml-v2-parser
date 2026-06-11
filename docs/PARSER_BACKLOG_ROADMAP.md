# Parser backlog & roadmap

**Single entry point** for open work on `sysml-v2-parser` and the Spec42 diagnostics integration. Historical plans remain as references; this document is updated when items open or close.

**Last updated:** 2026-06-09

## How to use this document

| If you want to… | Start here |
| ---------------- | ---------- |
| See **all open work** in one place | This file — sections below |
| Understand **what the parser already ships** for Spec42 | [§ Completed — Spec42 parser wave](#completed--spec42-parser-wave-june-2026) |
| Wire **Spec42 graph builders / collectors** | [§ 1 — Spec42 cross-repo](#1-spec42-cross-repo-follow-up) (**done** in Spec42 0.29.0) |
| Improve **editor / LSP** behavior | [§ 3 — Language server](#3-language-server--recovery) |
| Go deeper on **grammar fidelity** | [§ 4 — Grammar & compliance](#4-grammar-depth--compliance) |
| Read the **original Spec42 parser spec** | [SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md](./SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md) |

### Regression gates (every parser PR)

- `cargo test`
- `cargo test --test validation -- --include-ignored`
- `test_systems_library_strict_no_diagnostics` / `test_full_library_strict_no_diagnostics` (validation suite)
- `ExtendedLibraryDecl = 0` in library node-shape gates

### AST snapshot refresh (when AST shape changes)

CI runs the full validation suite (`cargo test -- --include-ignored`). Several fixtures compare against checked-in AST text under [`tests/validation/snapshots/`](../tests/validation/snapshots/). **Any PR that changes AST shape must refresh those snapshots in the same PR** — do not rely on the default `cargo test` alone (snapshot tests are `#[ignore]` unless `--include-ignored`).

Regenerate after changes such as:

- new or renamed AST fields (e.g. `value_span`, `MetadataAnnotation` variants)
- new body-element enum variants or different parse classification (e.g. `@` metadata vs generic `Annotation`)
- structured-body parsing replacing silent skip

```powershell
$env:UPDATE_VALIDATION_AST = "1"
cargo test --test validation -- --include-ignored
Remove-Item Env:UPDATE_VALIDATION_AST
```

See [`tests/validation/README.md`](../tests/validation/README.md) for layout and per-fixture commands. Review the snapshot diff before committing — it should reflect intentional parser output only.

---

## Status snapshot

| Area | State |
| ---- | ----- |
| CI & library gates | Green |
| BNF coverage map | 640/640 productions classified `implemented` |
| Spec42 diagnostics **parser AST** (P0–P2 wave) | **Largely done** — see completed table; partial items listed in § 2 |
| Spec42 **semantic** diagnostics (§1 wave) | **Done** in Spec42 0.29.0 — partial §2 items remain parser-side |
| Deep body fidelity | **Open** — many `advance_to_closing_brace` call sites remain |
| Full `OwnedExpression` | **Open** — operator enums added; full KerML expression family not modeled |
| Unified definition/usage grammar layer | **Open** — P5+ architectural work |

```mermaid
flowchart TB
  subgraph done [Parser done June 2026]
    ast[AST fields + spans]
    fixtures[spec42_diagnostics_ast tests]
  end
  subgraph open_parser [Parser open]
    partial[Partial items §2]
    bodies[Opaque bodies §2.3]
    expr[Expression depth §2.4]
  end
  subgraph open_spec42 [Spec42 open]
    graph[Graph builders]
    coll[Diagnostic collectors]
  end
  ast --> graph --> coll
  partial --> graph
```

---

## 1. Spec42 cross-repo follow-up

Parser changes unlock diagnostics only after Spec42 projects new AST fields. **Highest ROI** after the June 2026 parser wave.

| Diagnostic / theme | Parser AST (ready?) | Spec42 work | Spec42 doc |
| ------------------ | ------------------- | ----------- | ---------- |
| `accept_payload_incompatible` on transitions | Yes — `Transition.accept`, `PayloadClause` | Graph: transition trigger `payloadType`; collector | [DIAGNOSTIC-CHECKS-ROADMAP](https://github.com/spec42/spec42/blob/main/docs/engineering/DIAGNOSTIC-CHECKS-ROADMAP.md) |
| `send_payload_incompatible` | Yes — `ActionUsage.send` | Graph: control-node send payload | same |
| Final-state cardinality (`multiple_final_states`, …) | Yes — `FinalState` member | Graph: final-state edges; drop sink heuristics | same |
| `metadata_keyword_unresolved` (`#Tag`) | Yes — `MetadataKeywordUsage` (simple `#name`) | Walk new node in part/state/requirement builders | [AST-SEMANTIC-COVERAGE](https://github.com/spec42/spec42/blob/main/docs/engineering/AST-SEMANTIC-COVERAGE.md) |
| `viewpoint_reference_unresolved` (stakeholder/purpose) | Yes — `StakeholderMember`, `PurposeMember` | Extend collector for new ref spans | same |
| `viewpoint_rep_language_unresolved` | Partial — `rep` in requirement body | Wire `TextualRep` + `language_span` in graph | same |
| `transition_guard_non_boolean`, filters, assignments | **Done** — `Expression::Classification`, `exprClass` AST walk (0.23.0) | — | same |
| Typed `stakeholder name : Type` | **Done** — `StakeholderMember` with optional typing (0.23.0) | Graph: `stakeholderType` + typing edge | same |
| `assignment_value_incompatible` in case bodies | **Done** — `AttributeDef.value_span` + verification graph (0.23.0) | — | same |
| Initial state via `first` | Yes — `Transition.is_initial` | Align with `ThenStmt` initial edges | same |

**Release train:** parser release → Spec42 graph_builder PR → collector + catalog entry → move item **Deferred → Done** in Spec42 roadmap.

---

## 2. Parser — open & partial (Spec42 wave)

Items from [SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md](./SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md) that are **not fully closed** in the parser.

### 2.1 Metadata & annotations

| Item | Status | Remaining work |
| ---- | ------ | -------------- |
| `#keyword` in bodies | **Done** (simple `#Tag;`) | — |
| Extended `#refinement dependency …` | **Done** (opaque `Annotation`) | — |
| User-defined **declaration** keywords (`metadata def` short name as header starter) | **Not started** (deferred 1.5b) | Dynamic dispatch in `feature_decl` / `classifier_decl`; package-local metadata def index |
| `MetadataAnnotation` in all bodies | **Partial** | Part **def**, state, requirement, part usage, action bodies; constraint `@` still generic `Annotation` in some paths |
| `head_span` on all annotation usages | **Partial** | Wired on parse paths; not all body enums expose `MetadataAnnotation` variant |

### 2.2 TextualRepresentation (`rep`)

| Item | Status | Remaining work |
| ---- | ------ | -------------- |
| `rep` in requirement / viewpoint body | **Done** | Fixture: `tests/fixtures/requirement-rep-language.sysml` |
| `rep` in frame, concern, package-adjacent bodies | **Partial** | Package-level `TextualRep` exists; frame/concern may need explicit wiring |
| `language_span` | **Done** on parse path | — |
| Parser diagnostics `missing_rep_language` / `invalid_rep_language` | **Catalog only** | Constants in [`diagnostic_catalog.rs`](../src/parser/diagnostic_catalog.rs); not emitted by `textual_representation()` yet |

### 2.3 Opaque brace-body skipping

**Problem:** Unmodeled inner regions are invisible to Spec42 and the LSP.

| Module | `advance_to_closing_brace` uses (approx.) | Priority |
| ------ | ---------------------------------------- | -------- |
| `action.rs` | 0 (was 7) | High — behavior / control nodes |
| `requirement.rs` | 0 (was 4) | High |
| `state.rs` | 0 (was 2) | Medium (transition connect bodies unified) |
| `part/usage.rs` | 0 (was 3) | Medium |
| `usecase.rs` | 0 (was 2) | Lower — structured case bodies + return-ref expressions |

**Direction:** Per construct family, replace silent skip with `ParseErrorNode` + partial member lists ([LANGUAGE_SERVER_BACKLOG.md](./LANGUAGE_SERVER_BACKLOG.md) P0). One family per PR; track remaining sites here.

### 2.4 Expression AST

| Item | Status | Remaining work |
| ---- | ------ | -------------- |
| Operator classification | **Done** — `BinaryOperator`, `UnaryOperator` | — |
| `@Metaclass` classification | **Done** — `Expression::Classification` | Spec42 `exprClass` on filter/guard nodes |
| `istype` / `hastype` / `as` | **Done** — `Expression::TypeCheck` | Filter/guard contexts |
| `select` / `collect` | **Done** — `Expression::Select` / `Collect` | — |

### 2.5 Case & verification bodies

| Item | Status | Remaining work |
| ---- | ------ | -------------- |
| `AttributeDef.name_span` in case bodies | **Done** | — |
| `value_span` on `AttributeDef` | **Done** | Populated on parse path; verification/analysis graph builders project local attributes |
| Verdict / return forms, `:>>` in analysis bodies | **Partial** — `ReturnRef.return_expression`, structured `ref :>>` bodies | Typed objective + remaining library `:>>` nesting |

### 2.6 Parser diagnostic contract

| Item | Status | Remaining work |
| ---- | ------ | -------------- |
| `diagnostic_catalog.rs` | **Done** (registry file) | Wire constants into `diagnostics.rs` / `recovery.rs` instead of string literals |
| Range-text regression tests | **Partial** | `recovery_diagnostics_integration.rs` exists; add transition/import/type range matrix |
| Scope labels (`"state body"`, …) | **Done** in major bodies | Extend to nested families per § 2.3 |

---

## 3. Language server & recovery

Consolidated from [LANGUAGE_SERVER_BACKLOG.md](./LANGUAGE_SERVER_BACKLOG.md). **Not duplicated** — see that file for narrative detail.

| Priority | Theme | Open? |
| -------- | ----- | ----- |
| P0 | Tighten recovery diagnostics (`expected` / `suggestion` precision) | Yes |
| P0 | Expand `ParseErrorNode` to view/constraint/calc nested scopes | Yes |
| P0 | Remove silent reshaping on malformed input | Yes |
| P0 | Recovery tests per construct (codes + ranges + siblings) | Partial — good baseline, gaps in views/constraints |
| P1 | Normalize recovery loops across modules | Partial — `parse_structured_brace_members` exists |
| P1 | Finer grammar-aware sync helpers | Yes |
| P1 | Span robustness under recovery | Yes |
| P2 | Strict vs resilient parse path separation (internal) | Yes |
| P2 | Richer error infrastructure (`nom-supreme`, custom state) | Investigate |

---

## 4. Grammar depth & compliance

Consolidated from [SYSML_V2_COMPLIANCE_GAP.md](./SYSML_V2_COMPLIANCE_GAP.md) and [PARSER_TECHNICAL_DEBT.md](./PARSER_TECHNICAL_DEBT.md).

| Theme | Priority | Notes |
| ----- | -------- | ----- |
| Unified definition / usage / specialization grammar layer | **P5+** | Largest architectural gap; do not big-bang rewrite |
| `take_until_terminator` header scraping → structured headers | Medium | Per-family as library fixtures expose gaps |
| `part_def` prelude unify with `definition_prefix` | Low | Intentionally local for disambiguation |
| `package_body_element` sub-dispatchers | **Done** (P2) | Maintain when adding keywords |
| AST shape dedup (`DefinitionDecl` internal) | P5+ | Drive from grammar work |
| Semantic conformance (types, resolution) | Out of scope | Spec42 / other tools |

---

## Completed — Spec42 parser wave (June 2026)

Parser-side delivery for [SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md](./SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md).

| # | Item | Key types / files | Test |
| - | ---- | ----------------- | ---- |
| P0 §1 | Transition `accept` | `TransitionAccept`, `PayloadClause`, [`payload.rs`](../src/parser/payload.rs) | `transition_accept_retained_with_spans` |
| P0 §2 | Final state | `FinalState`, `StateDefBodyElement::FinalState` | `final_state_members_parsed` |
| P0 §3 | Send payload | `ActionUsage.send`, `control_node_action_usage` | `send_payload_on_control_node_action` |
| P0 §4 | `#keyword` (bodies) | `MetadataKeywordUsage` | `metadata_keyword_usage_in_part_body` |
| P0 §5 | Stakeholder / purpose | `StakeholderMember`, `PurposeMember` | `viewpoint_stakeholder_and_purpose_members` |
| P1 §6 | Expression operators | `BinaryOperator`, `UnaryOperator` | `expression_parses_implies_lower_than_or` |
| P1 §7 | Case attribute spans | `AttributeDef.name_span` | `verification_local_attribute_has_name_span` |
| P1 §8 | `rep` in requirement body | `TextualRep`, `language_span` | `requirement_body_rep_language_parsed` |
| P1 §9 | Annotations | `head_span` on `Annotation` / `MetadataAnnotation`; use-case bodies | apollo_regressions (rationale) |
| P2 §10 | Brace skipping (increment) | Transition uses `connect_body` | — |
| P2 §11 | Diagnostic catalog | [`diagnostic_catalog.rs`](../src/parser/diagnostic_catalog.rs) | `diagnostic_catalog_documents_stable_codes` |
| P2 §12 | `first` = initial | `Transition.is_initial` | `transition_first_sets_is_initial_flag` |

**Fixtures:** [tests/fixtures/](../tests/fixtures/) (`transition-accept-typed.sysml`, `final-state.sysml`, `send-payload.sysml`, `metadata-keyword-usage.sysml`, `viewpoint-stakeholder-purpose.sysml`, `verification-local-attribute.sysml`, `requirement-rep-language.sysml`)

**Integration test file:** [tests/spec42_diagnostics_ast.rs](../tests/spec42_diagnostics_ast.rs)

---

## Completed — technical debt tranches (reference)

| Plan | Status | Doc |
| ---- | ------ | --- |
| P1 definition prefix + opaque bodies | Complete | [PARSER_DEBT_P1_PLAN.md](./PARSER_DEBT_P1_PLAN.md) |
| P2 structured body loops | Complete | [PARSER_DEBT_P2_PLAN.md](./PARSER_DEBT_P2_PLAN.md) |
| P3 AST split, action/requirement bodies | Complete | [PARSER_DEBT_P3_PLAN.md](./PARSER_DEBT_P3_PLAN.md) |
| P4 view/part bodies, implies, part split | Complete | [PARSER_DEBT_P4_PLAN.md](./PARSER_DEBT_P4_PLAN.md) |

---

## Suggested execution order

1. **Spec42 graph builders** for completed P0 AST (§ 1) — unlocks user-visible diagnostics.
2. **Parser partials** that block Spec42 (§ 2.1 declaration keywords, § 2.2 rep diagnostics, § 2.3 action/requirement bodies).
3. **LSP P0** (§ 3) in parallel with body fidelity.
4. **Expression depth** (§ 2.4) and **P5 grammar layer** (§ 4) as longer horizons.

---

## Document map

| Document | Role |
| -------- | ---- |
| **This file** | Open backlog & roadmap (maintain here) |
| [SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md](./SPEC42-DIAGNOSTICS-PARSER-IMPROVEMENTS.md) | Spec42-facing parser requirements & fixture index |
| [LANGUAGE_SERVER_BACKLOG.md](./LANGUAGE_SERVER_BACKLOG.md) | LSP/recovery detail |
| [SYSML_V2_COMPLIANCE_GAP.md](./SYSML_V2_COMPLIANCE_GAP.md) | Grammar fidelity narrative |
| [PARSER_TECHNICAL_DEBT.md](./PARSER_TECHNICAL_DEBT.md) | Duplication & architecture notes |
| [BNF_COMPLIANCE_MATRIX.md](./BNF_COMPLIANCE_MATRIX.md) | Compact grammar-family snapshot |
| [ERROR_RECOVERY.md](./ERROR_RECOVERY.md) | Recovery behavior reference |
| PARSER_DEBT_P1–P4_PLAN.md | Completed implementation checklists |
