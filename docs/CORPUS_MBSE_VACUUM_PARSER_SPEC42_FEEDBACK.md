# Parser and Spec42 improvement notes (MBSE vacuum-cleaner corpus)

This document records findings from running Spec42 (SysML language support using this parser stack) on the public example repository **MBSE_AG_vacuum-cleaner-robot-example** (`C:\Git\MBSE_AG_vacuum-cleaner-robot-example`), including a full diagnostic export (`diagnostics.txt`, ~336 items). It is meant for **sysml-v2-parser** and **Spec42** maintainers and for tracking regression tests.

Related docs in this repo:

- [`ERROR_RECOVERY.md`](ERROR_RECOVERY.md) — recovery architecture and AST error nodes
- [`LANGUAGE_SERVER_BACKLOG.md`](LANGUAGE_SERVER_BACKLOG.md) — LSP-oriented parser backlog
- [`SYSML_V2_COMPLIANCE_GAP.md`](SYSML_V2_COMPLIANCE_GAP.md) — spec coverage gaps

---

## 1. Executive summary

| Area | Finding |
|------|-----------|
| **Example model** | Several files have **real** SysML v2 textual issues (e.g. invalid action bodies, brace mismatch, concatenated `}package`). Many diagnostics are **cascades** from a small number of root errors. |
| **Parser (`sysml-v2-parser`)** | Recovery works but produces **many similar diagnostics** (`recovered_part_*`, `missing_semicolon`). **Brace mismatch** messages can read as “missing `}`” when the human issue is an **extra** `}` or earlier unclosed block. Tightening locality and message text would help (aligns with P0 item 1 in [`LANGUAGE_SERVER_BACKLOG.md`](LANGUAGE_SERVER_BACKLOG.md)). |
| **Spec42 / IDE layer** | Opportunity to **deduplicate**, **group**, or **collapse** cascade diagnostics; clarify **semantic** vs **syntax** (`source: semantic` vs `sysml`); ensure **standard library** types (`Real`, `Boolean`, `LengthValue`, …) resolve when the model imports `ScalarValues::*` so “unresolved in semantic graph” is not mistaken for invalid SysML. |

---

## 2. Corpus and diagnostic profile

- **Repository:** MBSE_AG_vacuum-cleaner-robot-example (mixed `Functions/legacy/…` and `Functions/VacuumingRoboterSystem/…`).
- **Scale:** ~336 diagnostics in one export; largest counts on monolithic `Integration.sysml` files (multiple packages in one compilation unit).
- **Dominant codes (approximate):**
  - `recovered_part_def_body_element` — parser recovered inside a `part def` body
  - `missing_semicolon` — often chained after a bad member
  - `recovered_part_usage_body_element` — same inside `part` usage bodies
  - `unexpected_keyword_in_scope` — e.g. identifiers in invalid positions in **action** bodies
  - Smaller sets: `unresolved_type_reference`, `unresolved_import_target`, `connection_endpoint_not_port`, `untyped_part_usage`, `missing_closing_brace`, etc.

**Takeaway:** Treat the export as **signal density**, not as **independent fault count**.

---

## 3. Recommended improvements — `sysml-v2-parser`

### 3.1 Brace and delimiter diagnostics

**Observation:** A file ended with an **extra** closing `}` after a well-formed package close; the tool reported **`missing_closing_brace`** at the last line. Humans expect “unexpected `}`” or “unmatched brace” when the stack depth is wrong.

**Suggestion:**

- When brace depth goes negative or EOF is reached with non-zero depth, prefer messages that distinguish **unexpected close** vs **still open**.
- Optionally report **two spans**: first mismatch site and recovery site.

**Backlog link:** [`LANGUAGE_SERVER_BACKLOG.md`](LANGUAGE_SERVER_BACKLOG.md) P0 §3 (silent / surprising recovery around delimiters).

### 3.2 Cascade control for `missing_semicolon` + `recovered_*`

**Observation:** One structural error (e.g. unmatched `{` in a shared type package) can produce long runs of `missing_semicolon` and `recovered_part_*` in **other** files in the same workspace.

**Suggestion:**

- After N consecutive `missing_semicolon` / recovery in the same body, emit one **summary** diagnostic (“parsing abandoned in this body after earlier error”) or suppress duplicates in the same span class.
- Consider tagging diagnostics with a **weak** severity or `relatedInformation` pointing to the **primary** error when known.

**Backlog link:** [`LANGUAGE_SERVER_BACKLOG.md`](LANGUAGE_SERVER_BACKLOG.md) P0 §1 (tighten diagnostics from recovery paths).

### 3.3 Nested `interface def` inside `part def`

**Observation:** OMG SysML v2 Part 1 textual notation lists `InterfaceDefinition` under `DefinitionElement`, so **nesting `interface def` under `part def` is spec-allowed**. If the parser still flags some of these as “unexpected token in part definition body”, add a **minimal fixture** and either fix the grammar path or document the intentional subset restriction in [`SYSML_V2_COMPLIANCE_GAP.md`](SYSML_V2_COMPLIANCE_GAP.md).

**Suggested fixture name:** `tests/fixtures/part-def-nested-interface-def.sysml` (minimal, passes strict parse if conformant).

### 3.4 Action usage / state entry bodies

**Observation:** Patterns such as `action act : ComputeBatteryInfo { batCap; maxBatCap; computedColor; }` produce **`unexpected_keyword_in_scope`** (identifiers in an action body where the grammar expects structured statements / performs / bindings).

**Suggestion:**

- Prefer a diagnostic code or message like **invalid bare identifier in action body** with a short **hint** toward valid forms (`perform`, `in`/`out` bindings, etc.), instead of labeling user identifiers as “keyword”.
- Add fixtures under `tests/fixtures/` mirroring invalid vs valid minimal snippets for editor tests.

**Backlog link:** [`LANGUAGE_SERVER_BACKLOG.md`](LANGUAGE_SERVER_BACKLOG.md) P0 §4 (recovery-focused tests for action bodies); [`ERROR_RECOVERY.md`](ERROR_RECOVERY.md) action usage recovery.

### 3.5 Regression suite from the corpus

**Suggestion:** Add an **optional** integration test (ignored by default, like validation against the release tree) that parses the MBSE repo path when `MBSE_VACUUM_EXAMPLE_DIR` is set, and asserts:

- no parser panic
- bounded diagnostic count or snapshot of **primary** codes only

This avoids licensing/size issues if the corpus is not vendored.

### 3.6 Shared usage-header parsing (Spec42 sync point)

`sysml-v2-parser` now routes additional usage families through shared usage-header parsing (`:` / `defined by` / `typed by` + specialization clauses) for requirement/case/action/state/view/rendering usages.

**Spec42 alignment notes:**

- Keep consuming normalized usage typing from existing `type_name` fields (same public field names, broader accepted syntax).
- For requirement usage, `subsets` continues to be exposed as a single normalized target (`last-wins` when multiple clauses are present).
- Treat broader acceptance of typed/specialized headers as parser coverage growth, not as a semantic model change by itself.

---

## 4. Recommended improvements — Spec42 (LSP / semantic layer)

Spec42 is not vendored in this repository; this section is for the **consumer** of `sysml-v2-parser` diagnostics and semantic graph.

### 4.1 Diagnostic presentation

- **Group** `recovered_*` + following `missing_semicolon` under a single parent diagnostic where possible (VS Code `relatedInformation` or code actions “show cascade”).
- **Deduplicate** identical message + range in one publish cycle.
- Map **`severity: 8`** to the correct LSP severity consistently (Error vs Warning) per code.

### 4.2 Semantic graph and standard library

- **`unresolved_type_reference` for `Real`, `Boolean`, `LengthValue`** often indicates the **semantic graph** did not load KerML/SysML standard libraries for the compilation unit, not that the type names are illegal.
- **Improvement:** Auto-attach standard library packages for textual models that reference `ScalarValues`, `ISQ`, etc., or document required **project configuration** so users do not chase false “model errors.”

### 4.3 Multi-package single file

- Large files with `}package … {` sequences stress both parser recovery and **outline** UX.
- **Improvement:** Validate **root namespace** / file scope rules in the spec and ensure the language server **outline** and **go-to-definition** remain stable on these files.

---

## 5. What to fix in the example model (not the parser)

For a fair parser benchmark, the following are **authoritative model fixes** (still useful as negative fixtures if copied into `tests/fixtures/`):

- Remove or fix **extra** closing braces and ensure each `package` / `part def` / `interface def` block matches.
- Replace **invalid action bodies** with conformant behavior (explicit steps / bindings per SysML v2 / KerML).
- Avoid **concatenated** `}package` without clear member separation if the toolchain requires explicit terminators in that context.
- Fix typos in type names (`Bumber` vs `Bumper`, `EeArchitecture`, etc.) to reduce semantic noise.

---

## 6. Traceability

| Symptom | Likely owner |
|---------|----------------|
| Wrong/misleading **brace** message | Parser |
| Long **cascade** of `missing_semicolon` / `recovered_*` | Parser (recovery policy) + Spec42 (presentation) |
| **`unexpected_keyword_in_scope`** in action/state for bare names | Parser message + user model |
| **`unresolved_type_reference`** for std types | Spec42 / library loading |
| **`connection_endpoint_not_port`**, port typing | Semantic layer + model |

---

## 7. Changelog of this document

| Date | Change |
|------|--------|
| 2026-05-13 | Initial version from MBSE vacuum example + Spec42 diagnostic export review. |
