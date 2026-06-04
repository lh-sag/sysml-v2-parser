//! Parser tests: behavior

use sysml_v2_parser::ast::*;
use sysml_v2_parser::{parse, parse_with_diagnostics};

#[test]
fn test_state_def_body_parses_members() {
    let input =
        "package P { state def S { then Ready; state Running : Mode; transition t then Ready; } }";
    let result = parse(input).expect("parse should succeed");
    let pkg = match &result.elements[0].value {
        RootElement::Package(p) => &p.value,
        _ => panic!("expected package"),
    };
    let elements = match &pkg.body {
        PackageBody::Brace { elements } => elements,
        _ => panic!("expected brace body"),
    };
    let state_def = match &elements[0].value {
        PackageBodyElement::StateDef(sd) => &sd.value,
        _ => panic!("expected StateDef"),
    };
    let body_elements = match &state_def.body {
        sysml_v2_parser::ast::StateDefBody::Brace { elements } => elements,
        _ => panic!("expected state brace body"),
    };
    assert!(body_elements
        .iter()
        .any(|e| matches!(e.value, sysml_v2_parser::ast::StateDefBodyElement::Then(_))));
    assert!(body_elements.iter().any(|e| matches!(
        e.value,
        sysml_v2_parser::ast::StateDefBodyElement::StateUsage(_)
    )));
    assert!(body_elements.iter().any(|e| matches!(
        e.value,
        sysml_v2_parser::ast::StateDefBodyElement::Transition(_)
    )));
}

#[test]
fn test_constraint_and_calc_bodies_parse_members() {
    let input = "package P { constraint def C { in x : Real; out y : Real; x >= y; } calc def K { in x : Real; return r : Real; x; } }";
    let result = parse(input).expect("parse should succeed");
    let pkg = match &result.elements[0].value {
        RootElement::Package(p) => &p.value,
        _ => panic!("expected package"),
    };
    let elements = match &pkg.body {
        PackageBody::Brace { elements } => elements,
        _ => panic!("expected brace body"),
    };
    let constraint_def = match &elements[0].value {
        PackageBodyElement::ConstraintDef(cd) => &cd.value,
        _ => panic!("expected ConstraintDef"),
    };
    let constraint_elements = match &constraint_def.body {
        sysml_v2_parser::ast::ConstraintDefBody::Brace { elements } => elements,
        _ => panic!("expected constraint brace body"),
    };
    assert!(
        !constraint_elements.is_empty(),
        "constraint body should not be empty"
    );
    let calc_def = match &elements[1].value {
        PackageBodyElement::CalcDef(cd) => &cd.value,
        _ => panic!("expected CalcDef"),
    };
    let calc_elements = match &calc_def.body {
        sysml_v2_parser::ast::CalcDefBody::Brace { elements } => elements,
        _ => panic!("expected calc brace body"),
    };
    assert!(!calc_elements.is_empty(), "calc body should not be empty");
}

#[test]
fn test_perform_action_decl_body_parses_bindings() {
    let input = "package P { part def Carrier { perform action run : Runner { in speed = speedInput; out torque = torqueOutput; } } }";
    let result = parse(input).expect("parse should succeed");
    let pkg = match &result.elements[0].value {
        RootElement::Package(p) => &p.value,
        _ => panic!("expected package"),
    };
    let elements = match &pkg.body {
        PackageBody::Brace { elements } => elements,
        _ => panic!("expected brace body"),
    };
    let part_def = match &elements[0].value {
        PackageBodyElement::PartDef(pd) => &pd.value,
        _ => panic!("expected PartDef"),
    };
    let part_body = match &part_def.body {
        sysml_v2_parser::ast::PartDefBody::Brace { elements } => elements,
        _ => panic!("expected part def brace body"),
    };
    let perform = match &part_body[0].value {
        sysml_v2_parser::ast::PartDefBodyElement::Perform(p) => &p.value,
        _ => panic!("expected perform action declaration"),
    };
    assert!(
        matches!(&perform.body, sysml_v2_parser::ast::PerformBody::Brace { elements } if !elements.is_empty()),
        "perform action brace body should retain parsed in/out bindings"
    );
}

#[test]
fn test_action_def_body_allows_doc_and_nested_action_usages_without_semicolon_after_doc() {
    let input = r#"package P {
action def ExecutePatrol {
  in route : String;
  out status : String;
  doc /* Execute patrol/overwatch mission along route. */

  action validateRoute { out validationStatus : String; };
  action startMission { out missionStarted : String; };

  first validateRoute then startMission;
  bind status = startMission::missionStarted;
}
}"#;

    let result = parse_with_diagnostics(input);
    assert!(
        result.is_ok(),
        "action def with doc + nested actions should parse without recovery diagnostics: {:?}",
        result.errors
    );
    assert!(
        !result
            .errors
            .iter()
            .any(|e| e.code.as_deref() == Some("missing_semicolon")),
        "should not report missing_semicolon around doc/nested action usages: {:?}",
        result.errors
    );
}

#[test]
fn test_action_usage_body_allows_untyped_out_pin_decl() {
    // Common SysML v2 shorthand in action usage bodies: `out foo;` (no `: Type`)
    // to reference the corresponding typed parameter on the referenced action definition.
    let input = r#"package P {
action def CaptureVideo { out videoStream : String; }
action def ExecutePatrol {
  action capture : CaptureVideo { out videoStream; };
  first capture then capture;
}
}"#;

    let result = parse_with_diagnostics(input);
    assert!(
        result.is_ok(),
        "untyped out pin decl in action usage body should not trigger recovery diagnostics: {:?}",
        result.errors
    );
}

#[test]
fn test_action_def_accepts_specializes_keyword_as_specialization() {
    let input = r#"package P {
action def Run specializes BaseAction;
}"#;
    let result = parse(input).expect("parse should succeed");
    let pkg = match &result.elements[0].value {
        RootElement::Package(p) => p,
        other => panic!("expected package, got {:?}", other),
    };
    let elements = match &pkg.value.body {
        PackageBody::Brace { elements } => elements,
        other => panic!("expected brace body, got {:?}", other),
    };
    let action_def = match &elements[0].value {
        PackageBodyElement::ActionDef(p) => p,
        other => panic!("expected action def, got {:?}", other),
    };
    assert_eq!(action_def.value.specializes.as_deref(), Some("BaseAction"));
}

#[test]
fn test_action_def_preserves_multiple_specializes_targets() {
    let input = r#"package P {
action def Run :> BaseAction, LoggedAction;
}"#;
    let result = parse(input).expect("parse should succeed");
    let pkg = match &result.elements[0].value {
        RootElement::Package(p) => p,
        other => panic!("expected package, got {:?}", other),
    };
    let elements = match &pkg.value.body {
        PackageBody::Brace { elements } => elements,
        other => panic!("expected brace body, got {:?}", other),
    };
    let action_def = match &elements[0].value {
        PackageBodyElement::ActionDef(action) => action,
        other => panic!("expected action definition, got {:?}", other),
    };
    assert_eq!(
        action_def.value.specializes.as_deref(),
        Some("BaseAction, LoggedAction")
    );
    assert!(action_def.value.specializes_span.is_some());
}
