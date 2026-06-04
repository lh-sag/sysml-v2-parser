use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CoverageStatus {
    Implemented,
    Partial,
    Opaque,
    Fallback,
    Untested,
    NotSupported,
}

impl CoverageStatus {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "implemented" => Some(Self::Implemented),
            "partial" => Some(Self::Partial),
            "opaque" => Some(Self::Opaque),
            "fallback" => Some(Self::Fallback),
            "untested" => Some(Self::Untested),
            "not_supported" => Some(Self::NotSupported),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Grammar {
    SysML,
    KerML,
    Any,
}

impl Grammar {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "SysML" => Some(Self::SysML),
            "KerML" => Some(Self::KerML),
            "*" => Some(Self::Any),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct CoverageRule {
    grammar: Grammar,
    pattern: String,
    status: CoverageStatus,
    line: usize,
}

impl CoverageRule {
    fn matches(&self, grammar: Grammar, production: &str) -> bool {
        if self.grammar != Grammar::Any && self.grammar != grammar {
            return false;
        }
        pattern_matches(&self.pattern, production)
    }

    fn specificity(&self) -> usize {
        let non_wildcard = self.pattern.chars().filter(|ch| *ch != '*').count();
        let grammar_bonus = usize::from(self.grammar != Grammar::Any) * 1_000;
        let exact_bonus = usize::from(!self.pattern.contains('*')) * 10_000;
        exact_bonus + grammar_bonus + non_wildcard
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn release_root() -> PathBuf {
    if let Some(path) = std::env::var_os("SYSML_V2_RELEASE_DIR") {
        return PathBuf::from(path);
    }

    manifest_dir().join("sysml-v2-release")
}

fn extract_productions(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read BNF file {}: {err}", path.display()));
    let mut productions = Vec::new();
    for line in text.lines() {
        let Some(first) = line.as_bytes().first().copied() else {
            continue;
        };
        if !first.is_ascii_alphabetic() {
            continue;
        }
        let name_len = line
            .bytes()
            .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            .count();
        if name_len == 0 {
            continue;
        }
        let rest = line[name_len..].trim_start();
        if rest.starts_with('=') || rest.contains(" =") {
            productions.push(line[..name_len].to_string());
        }
    }
    productions.sort();
    productions.dedup();
    productions
}

fn parse_coverage_rules(path: &Path) -> Vec<CoverageRule> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read coverage map {}: {err}", path.display()));
    let mut rules = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let fields = trimmed.split_whitespace().collect::<Vec<_>>();
        assert_eq!(
            fields.len(),
            3,
            "invalid coverage rule at {}:{line_no}: expected 3 fields",
            path.display()
        );
        let grammar = Grammar::parse(fields[0]).unwrap_or_else(|| {
            panic!(
                "invalid grammar '{}' at {}:{line_no}",
                fields[0],
                path.display()
            )
        });
        let status = CoverageStatus::parse(fields[2]).unwrap_or_else(|| {
            panic!(
                "invalid coverage status '{}' at {}:{line_no}",
                fields[2],
                path.display()
            )
        });
        rules.push(CoverageRule {
            grammar,
            pattern: fields[1].to_string(),
            status,
            line: line_no,
        });
    }
    rules
}

fn pattern_matches(pattern: &str, value: &str) -> bool {
    match (pattern.starts_with('*'), pattern.ends_with('*')) {
        (false, false) => pattern == value,
        (false, true) => value.starts_with(&pattern[..pattern.len() - 1]),
        (true, false) => value.ends_with(&pattern[1..]),
        (true, true) => {
            let needle = &pattern[1..pattern.len() - 1];
            !needle.is_empty() && value.contains(needle)
        }
    }
}

fn classify<'a>(
    rules: &'a [CoverageRule],
    grammar: Grammar,
    production: &str,
) -> Result<&'a CoverageRule, String> {
    let matches = rules
        .iter()
        .filter(|rule| rule.matches(grammar, production))
        .collect::<Vec<_>>();
    let Some(best_specificity) = matches.iter().map(|rule| rule.specificity()).max() else {
        return Err(format!("no coverage rule for {grammar:?}.{production}"));
    };
    let best = matches
        .into_iter()
        .filter(|rule| rule.specificity() == best_specificity)
        .collect::<Vec<_>>();
    let first = best[0];
    if best.iter().any(|rule| rule.status != first.status) {
        let lines = best
            .iter()
            .map(|rule| rule.line.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!(
            "conflicting equally-specific coverage rules for {grammar:?}.{production} at lines {lines}"
        ));
    }
    Ok(first)
}

type ClassifyAllResult = (
    BTreeMap<CoverageStatus, usize>,
    BTreeMap<CoverageStatus, Vec<String>>,
    Vec<String>,
);

fn classify_all(
    grammar: Grammar,
    productions: &[String],
    rules: &[CoverageRule],
) -> ClassifyAllResult {
    let mut counts = BTreeMap::<CoverageStatus, usize>::new();
    let mut productions_by_status = BTreeMap::<CoverageStatus, Vec<String>>::new();
    let mut errors = Vec::new();
    for production in productions {
        match classify(rules, grammar, production) {
            Ok(rule) => {
                *counts.entry(rule.status).or_insert(0) += 1;
                productions_by_status
                    .entry(rule.status)
                    .or_default()
                    .push(production.clone());
            }
            Err(err) => errors.push(err),
        }
    }
    (counts, productions_by_status, errors)
}

fn assert_all_productions_are_classified(
    grammar: Grammar,
    productions: &[String],
    rules: &[CoverageRule],
) {
    let (counts, productions_by_status, errors) = classify_all(grammar, productions, rules);

    eprintln!("{grammar:?} BNF coverage counts: {counts:?}");
    for (status, productions) in &productions_by_status {
        eprintln!(
            "{grammar:?} {status:?} productions: {}",
            productions.join(", ")
        );
    }

    assert!(
        errors.is_empty(),
        "unclassified or ambiguous BNF productions:\n{}",
        errors.join("\n")
    );
}

fn load_bnf_productions() -> (Vec<String>, Vec<String>, Vec<CoverageRule>) {
    let root = release_root();
    let sysml_bnf = root.join("bnf").join("SysML-textual-bnf.kebnf");
    let kerml_bnf = root.join("bnf").join("KerML-textual-bnf.kebnf");
    let rules = parse_coverage_rules(&manifest_dir().join("docs").join("bnf_coverage.map"));
    (
        extract_productions(&sysml_bnf),
        extract_productions(&kerml_bnf),
        rules,
    )
}

#[test]
fn textual_bnf_productions_are_covered_by_status_map() {
    let root = release_root();
    let sysml_bnf = root.join("bnf").join("SysML-textual-bnf.kebnf");
    let kerml_bnf = root.join("bnf").join("KerML-textual-bnf.kebnf");
    assert!(
        sysml_bnf.exists(),
        "SysML textual BNF not found at {}",
        sysml_bnf.display()
    );
    assert!(
        kerml_bnf.exists(),
        "KerML textual BNF not found at {}",
        kerml_bnf.display()
    );

    let rules = parse_coverage_rules(&manifest_dir().join("docs").join("bnf_coverage.map"));
    assert!(!rules.is_empty(), "coverage map must contain rules");

    let sysml = extract_productions(&sysml_bnf);
    let kerml = extract_productions(&kerml_bnf);
    assert_eq!(
        sysml.len(),
        350,
        "unexpected SysML textual BNF production count"
    );
    assert_eq!(
        kerml.len(),
        290,
        "unexpected KerML textual BNF production count"
    );

    assert_all_productions_are_classified(Grammar::SysML, &sysml, &rules);
    assert_all_productions_are_classified(Grammar::KerML, &kerml, &rules);
}

/// Wildcard map rules for Flow/Allocation/Metadata must not claim `implemented` while bodies
/// still use opaque skipping. Exact production names (e.g. `FlowDefinition`) are allowed.
#[test]
fn implemented_wildcard_patterns_do_not_target_opaque_body_helper_families() {
    let rules = parse_coverage_rules(&manifest_dir().join("docs").join("bnf_coverage.map"));
    let opaque_families = ["Flow", "Allocation", "Metadata"];
    let implemented_opaque_rules = rules
        .iter()
        .filter(|rule| rule.status == CoverageStatus::Implemented)
        .filter(|rule| rule.pattern.contains('*'))
        .filter(|rule| {
            opaque_families
                .iter()
                .any(|family| pattern_matches(&rule.pattern, family))
        })
        .collect::<Vec<_>>();

    assert!(
        implemented_opaque_rules.is_empty(),
        "opaque helper wildcard families must not be marked implemented: {implemented_opaque_rules:?}"
    );
}

#[test]
fn coverage_map_rules_use_no_partial_status() {
    let rules = parse_coverage_rules(&manifest_dir().join("docs").join("bnf_coverage.map"));
    let partial_rules: Vec<_> = rules
        .iter()
        .filter(|rule| rule.status == CoverageStatus::Partial)
        .map(|rule| format!("line {}: {:?} {}", rule.line, rule.grammar, rule.pattern))
        .collect();
    assert!(
        partial_rules.is_empty(),
        "bnf_coverage.map must not contain partial rules:\n{}",
        partial_rules.join("\n")
    );
}

#[test]
fn all_textual_bnf_productions_are_implemented() {
    let (sysml, kerml, rules) = load_bnf_productions();
    assert_eq!(sysml.len(), 350);
    assert_eq!(kerml.len(), 290);

    for grammar in [Grammar::SysML, Grammar::KerML] {
        let productions = if grammar == Grammar::SysML {
            &sysml
        } else {
            &kerml
        };
        let (counts, _, errors) = classify_all(grammar, productions, &rules);
        assert!(
            errors.is_empty(),
            "{grammar:?} classification errors: {errors:?}"
        );
        assert_eq!(
            counts.get(&CoverageStatus::Partial).copied().unwrap_or(0),
            0,
            "{grammar:?} still has partial productions"
        );
        assert_eq!(
            counts
                .get(&CoverageStatus::Implemented)
                .copied()
                .unwrap_or(0),
            productions.len(),
            "{grammar:?} implemented count must equal production count"
        );
    }
}

#[test]
fn implemented_productions_do_not_use_skip_or_statement_only_bodies() {
    let rules = parse_coverage_rules(&manifest_dir().join("docs").join("bnf_coverage.map"));
    let guarded_productions = [
        ("AttributeDefinition", "src/parser/attribute.rs"),
        ("AttributeUsage", "src/parser/attribute.rs"),
        ("OccurrenceDefinition", "src/parser/occurrence.rs"),
        ("OccurrenceUsage", "src/parser/occurrence.rs"),
        ("PartDefinition", "src/parser/part.rs"),
        ("PartUsage", "src/parser/part.rs"),
        ("PortDefinition", "src/parser/port.rs"),
        ("PortUsage", "src/parser/port.rs"),
        ("ConnectionDefinition", "src/parser/connection.rs"),
        ("InterfaceDefinition", "src/parser/interface.rs"),
        ("EnumerationDefinition", "src/parser/enumeration.rs"),
        ("RenderingDefinition", "src/parser/view.rs"),
        ("FlowDefinition", "src/parser/flow.rs"),
        ("FlowUsage", "src/parser/flow.rs"),
        ("AllocationDefinition", "src/parser/allocation.rs"),
        ("AllocationUsage", "src/parser/allocation.rs"),
        ("MetadataDefinition", "src/parser/metadata.rs"),
        ("MetadataUsage", "src/parser/metadata.rs"),
        ("ActionDefinition", "src/parser/action.rs"),
        ("StateDefinition", "src/parser/state.rs"),
        ("RequirementDefinition", "src/parser/requirement.rs"),
    ];

    let mut violations = Vec::new();
    for (production, parser_path) in guarded_productions {
        let rule = classify(&rules, Grammar::SysML, production)
            .unwrap_or_else(|err| panic!("guarded production must be classified: {err}"));
        if rule.status != CoverageStatus::Implemented {
            continue;
        }

        let path = manifest_dir().join(parser_path);
        let parser = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read parser file {}: {err}", path.display()));
        for forbidden in [
            "skip_until_brace_end",
            "semicolon_or_statement_brace_body",
            "take_until_terminator(input, b\";{\")",
        ] {
            if parser.contains(forbidden) {
                violations.push(format!(
                    "SysML.{production} is implemented by rule line {} but {} still contains {forbidden}",
                    rule.line,
                    path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "implemented productions must not rely on opaque or statement-only body parsing:\n{}",
        violations.join("\n")
    );
}
