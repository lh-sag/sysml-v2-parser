//! Parser tests: occurrence-style DefinitionBody members on flow/allocation defs.

use sysml_v2_parser::ast::*;

fn parse_package(input: &str) -> Package {
    let result = sysml_v2_parser::parse(input).expect("parse should succeed");
    match result.elements[0].value.clone() {
        RootElement::Package(package) => package.value,
        _ => panic!("expected package"),
    }
}

fn brace_package_elements(pkg: &Package) -> &[Node<PackageBodyElement>] {
    match &pkg.body {
        PackageBody::Brace { elements } => elements,
        _ => panic!("expected brace package body"),
    }
}

fn brace_definition_elements(body: &DefinitionBody) -> &[Node<DefinitionBodyElement>] {
    match body {
        DefinitionBody::Brace { elements } => elements,
        _ => panic!("expected brace definition body"),
    }
}

fn has_occurrence_attribute_member(elements: &[Node<DefinitionBodyElement>], name: &str) -> bool {
    elements.iter().any(|element| {
        matches!(
            &element.value,
            DefinitionBodyElement::OccurrenceMember(member)
                if matches!(
                    &member.value,
                    OccurrenceBodyElement::AttributeUsage(attribute)
                        if attribute.value.name == name
                )
        )
    })
}

fn has_occurrence_part_member(elements: &[Node<DefinitionBodyElement>], name: &str) -> bool {
    elements.iter().any(|element| {
        matches!(
            &element.value,
            DefinitionBodyElement::OccurrenceMember(member)
                if matches!(
                    &member.value,
                    OccurrenceBodyElement::PartUsage(part)
                        if part.value.name == name
                )
        )
    })
}

fn has_occurrence_doc_member(elements: &[Node<DefinitionBodyElement>]) -> bool {
    elements.iter().any(|element| {
        matches!(
            &element.value,
            DefinitionBodyElement::OccurrenceMember(member)
                if matches!(&member.value, OccurrenceBodyElement::Doc(_))
        )
    })
}

#[test]
fn flow_def_body_parses_inner_attribute() {
    let pkg = parse_package("package P { flow def Power { attribute rate : Real; } }");
    let flow = match &brace_package_elements(&pkg)[0].value {
        PackageBodyElement::FlowDef(flow) => flow,
        _ => panic!("expected FlowDef"),
    };
    let elements = brace_definition_elements(&flow.value.body);
    assert!(
        has_occurrence_attribute_member(elements, "rate"),
        "expected attribute rate in flow def body"
    );
}

#[test]
fn flow_def_body_parses_nested_part() {
    let pkg = parse_package("package P { part def Wheel; flow def Event { part wheel : Wheel; } }");
    let elements = brace_package_elements(&pkg);
    let flow = match &elements[1].value {
        PackageBodyElement::FlowDef(flow) => flow,
        _ => panic!("expected FlowDef"),
    };
    let body_elements = brace_definition_elements(&flow.value.body);
    assert!(
        has_occurrence_part_member(body_elements, "wheel"),
        "expected part wheel in flow def body"
    );
}

#[test]
fn flow_usage_brace_body_parses_attribute() {
    let pkg = parse_package("package P { item def Payload; flow cargo : Payload { attribute weight : Real; } }");
    let flow = brace_package_elements(&pkg)
        .iter()
        .find_map(|element| match &element.value {
            PackageBodyElement::FlowUsage(flow) => Some(flow),
            _ => None,
        })
        .expect("expected FlowUsage");
    let elements = brace_definition_elements(&flow.body);
    assert!(
        has_occurrence_attribute_member(elements, "weight"),
        "expected attribute weight in flow usage body"
    );
}

#[test]
fn allocation_def_body_parses_attribute() {
    let pkg = parse_package("package P { allocation def Map { attribute id : String; } }");
    let alloc = match &brace_package_elements(&pkg)[0].value {
        PackageBodyElement::AllocationDef(alloc) => alloc,
        _ => panic!("expected AllocationDef"),
    };
    let elements = brace_definition_elements(&alloc.value.body);
    assert!(
        has_occurrence_attribute_member(elements, "id"),
        "expected attribute id in allocation def body"
    );
}

#[test]
fn flow_def_doc_only_body_regression() {
    let pkg = parse_package("package P { flow def Power { doc /* note */ } }");
    let flow = match &brace_package_elements(&pkg)[0].value {
        PackageBodyElement::FlowDef(flow) => flow,
        _ => panic!("expected FlowDef"),
    };
    let elements = brace_definition_elements(&flow.value.body);
    assert!(
        has_occurrence_doc_member(elements),
        "expected doc member in flow def body"
    );
}
