//! AST shape tests for Spec42 diagnostics parser roadmap items.

use std::fs;
use std::path::PathBuf;

use sysml_v2_parser::ast::*;
use sysml_v2_parser::parse;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);
    fs::read_to_string(path).expect("read fixture")
}

fn first_package<'a>(root: &'a RootNamespace) -> &'a Package {
    match &root.elements[0].value {
        RootElement::Package(p) => &p.value,
        other => panic!("expected package, got {other:?}"),
    }
}

fn package_body_elements(pkg: &Package) -> &[Node<PackageBodyElement>] {
    match &pkg.body {
        PackageBody::Brace { elements } => elements.as_slice(),
        _ => panic!("expected brace package body"),
    }
}

#[test]
fn transition_accept_retained_with_spans() {
    let root = parse(&fixture("transition-accept-typed.sysml")).expect("parse");
    let pkg = first_package(&root);
    let state_def = match &package_body_elements(pkg)[0].value {
        PackageBodyElement::StateDef(sd) => &sd.value,
        other => panic!("expected state def, got {other:?}"),
    };
    let transitions: Vec<&Transition> = match &state_def.body {
        StateDefBody::Brace { elements } => elements
            .iter()
            .filter_map(|e| match &e.value {
                StateDefBodyElement::Transition(t) => Some(&t.value),
                _ => None,
            })
            .collect(),
        _ => panic!("expected brace state body"),
    };
    assert_eq!(transitions.len(), 2);
    assert!(transitions[0].is_initial);
    let accept = transitions[0]
        .accept
        .as_ref()
        .expect("shorthand accept");
    match accept {
        TransitionAccept::Shorthand(expr) => {
            assert!(matches!(expr.value, Expression::FeatureRef(ref n) if n == "StartPressed"));
            assert!(expr.span.len > 0);
        }
        _ => panic!("expected shorthand accept"),
    }
    let typed = transitions[1]
        .accept
        .as_ref()
        .expect("typed accept");
    match typed {
        TransitionAccept::Payload(p) => {
            assert_eq!(p.name, "evt");
            assert_eq!(p.type_name.as_deref(), Some("StartEvent"));
            assert!(p.name_span.len > 0);
            assert!(p.type_span.is_some());
        }
        _ => panic!("expected typed accept"),
    }
}

#[test]
fn final_state_members_parsed() {
    let root = parse(&fixture("final-state.sysml")).expect("parse");
    let pkg = first_package(&root);
    let state_def = match &package_body_elements(pkg)[0].value {
        PackageBodyElement::StateDef(sd) => &sd.value,
        other => panic!("expected state def, got {other:?}"),
    };
    let finals: Vec<&FinalState> = match &state_def.body {
        StateDefBody::Brace { elements } => elements
            .iter()
            .filter_map(|e| match &e.value {
                StateDefBodyElement::FinalState(f) => Some(&f.value),
                _ => None,
            })
            .collect(),
        _ => panic!("expected brace state body"),
    };
    assert_eq!(finals.len(), 2);
    assert_eq!(finals[0].state_name, "expired");
    assert_eq!(finals[1].state_name, "completed");
    assert!(finals[0].name_span.len > 0);
}

#[test]
fn send_payload_on_control_node_action() {
    let root = parse(&fixture("send-payload.sysml")).expect("parse");
    let pkg = first_package(&root);
    let action_def = match &package_body_elements(pkg)[0].value {
        PackageBodyElement::ActionDef(ad) => &ad.value,
        other => panic!("expected action def, got {other:?}"),
    };
    let send_usage = match &action_def.body {
        ActionDefBody::Brace { elements } => elements
            .iter()
            .find_map(|e| match &e.value {
                ActionDefBodyElement::ActionUsage(a) => Some(&a.value),
                _ => None,
            })
            .expect("send action usage"),
        _ => panic!("expected brace action body"),
    };
    assert_eq!(send_usage.name, "send");
    let send = send_usage.send.as_ref().expect("send payload");
    assert_eq!(send.name, "message");
    assert_eq!(send.type_name.as_deref(), Some("AlertMessage"));
    assert!(send.name_span.len > 0);
    assert!(send.type_span.is_some());
}

#[test]
fn viewpoint_stakeholder_and_purpose_members() {
    let root = parse(&fixture("viewpoint-stakeholder-purpose.sysml")).expect("parse");
    let pkg = first_package(&root);
    let vp = match &package_body_elements(pkg)[0].value {
        PackageBodyElement::ViewpointDef(v) => &v.value,
        other => panic!("expected viewpoint def, got {other:?}"),
    };
    let body = match &vp.body {
        RequirementDefBody::Brace { elements } => elements,
        _ => panic!("expected brace viewpoint body"),
    };
    assert!(body.iter().any(|e| matches!(
        e.value,
        RequirementDefBodyElement::Stakeholder(_)
    )));
    assert!(body.iter().any(|e| matches!(
        e.value,
        RequirementDefBodyElement::Purpose(_)
    )));
}

#[test]
fn metadata_keyword_usage_in_part_body() {
    let root = parse(&fixture("metadata-keyword-usage.sysml")).expect("parse");
    let pkg = first_package(&root);
    let part_def = package_body_elements(pkg)
        .iter()
        .find_map(|e| match &e.value {
            PackageBodyElement::PartDef(pd) => Some(&pd.value),
            _ => None,
        })
        .expect("part def");
    let keyword = match &part_def.body {
        PartDefBody::Brace { elements } => elements
            .iter()
            .find_map(|e| match &e.value {
                PartDefBodyElement::MetadataKeywordUsage(k) => Some(&k.value),
                _ => None,
            })
            .expect("metadata keyword usage"),
        _ => panic!("expected brace part body"),
    };
    assert_eq!(keyword.keyword, "Tag");
    assert!(keyword.keyword_span.len > 0);
}

#[test]
fn verification_local_attribute_has_name_span() {
    let root = parse(&fixture("verification-local-attribute.sysml")).expect("parse");
    let pkg = first_package(&root);
    let verification = match &package_body_elements(pkg)[0].value {
        PackageBodyElement::VerificationCaseDef(v) => &v.value,
        other => panic!("expected verification def, got {other:?}"),
    };
    let attr = match &verification.body {
        UseCaseDefBody::Brace { elements } => elements
            .iter()
            .find_map(|e| match &e.value {
                UseCaseDefBodyElement::AttributeDef(a) => Some(&a.value),
                _ => None,
            })
            .expect("attribute def"),
        _ => panic!("expected brace verification body"),
    };
    assert_eq!(attr.name, "count");
    assert!(attr.name_span.is_some());
    assert_eq!(attr.typing.as_deref(), Some("Integer"));
}

#[test]
fn requirement_body_rep_language_parsed() {
    let root = parse(&fixture("requirement-rep-language.sysml")).expect("parse");
    let pkg = first_package(&root);
    let req = match &package_body_elements(pkg)[0].value {
        PackageBodyElement::RequirementDef(r) => &r.value,
        other => panic!("expected requirement def, got {other:?}"),
    };
    let rep = match &req.body {
        RequirementDefBody::Brace { elements } => elements
            .iter()
            .find_map(|e| match &e.value {
                RequirementDefBodyElement::TextualRep(t) => Some(&t.value),
                _ => None,
            })
            .expect("textual rep"),
        _ => panic!("expected brace requirement body"),
    };
    assert_eq!(rep.language, "sysml");
    assert!(rep.language_span.is_some());
}

#[test]
fn diagnostic_catalog_documents_stable_codes() {
    use sysml_v2_parser::parser::diagnostic_catalog::DOCUMENTED_CODES;
    assert!(DOCUMENTED_CODES.contains(&"missing_member_name"));
    assert!(DOCUMENTED_CODES.contains(&"missing_closing_brace"));
}

#[test]
fn transition_first_sets_is_initial_flag() {
    let input = "package P { state def S { transition t first idle then running; } }";
    let root = parse(input).expect("parse");
    let pkg = first_package(&root);
    let state_def = match &package_body_elements(pkg)[0].value {
        PackageBodyElement::StateDef(sd) => &sd.value,
        _ => panic!("expected state def"),
    };
    let transition = match &state_def.body {
        StateDefBody::Brace { elements } => elements
            .iter()
            .find_map(|e| match &e.value {
                StateDefBodyElement::Transition(t) => Some(&t.value),
                _ => None,
            })
            .expect("transition"),
        _ => panic!("expected brace body"),
    };
    assert!(transition.is_initial);
    assert!(transition.source.is_some());
}
