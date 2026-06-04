//! View, viewpoint, and rendering parsing (SysML v2 Clause 8.2.2.26).
#![allow(dead_code, unused_imports)]

use crate::ast::{
    ExposeMember, FilterMember, Node, ParseErrorNode, RenderingDef, RenderingDefBody,
    RenderingDefBodyElement, RenderingUsage, SatisfyViewMember, ViewBody, ViewBodyElement, ViewDef,
    ViewDefBody, ViewDefBodyElement, ViewRenderingUsage, ViewUsage, ViewpointDef, ViewpointUsage,
};
use crate::parser::definition_header::{parse_feature_usage_header, parse_usage_header};
use crate::parser::definition_prefix::{parse_definition_prefix, DefinitionPrefixOptions};
use crate::parser::interface::connect_body;
use crate::parser::lex::{
    identification, name, qualified_name, starts_with_any_keyword, ws1, ws_and_comments,
    VIEW_BODY_STARTERS, VIEW_DEF_BODY_STARTERS,
};
use crate::parser::requirement::{doc_comment, requirement_def_body};
use crate::parser::Input;
use crate::parser::{build_recovery_error_node_from_span, node_from_to};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{map, success};
use nom::multi::many0;
use nom::sequence::{delimited, preceded};
use nom::{IResult, Parser};

fn view_def_body_element(input: Input<'_>) -> IResult<Input<'_>, Node<ViewDefBodyElement>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, elem) = alt((
        map(doc_comment, ViewDefBodyElement::Doc),
        map(view_filter_member, ViewDefBodyElement::Filter),
        map(view_rendering_usage, ViewDefBodyElement::ViewRendering),
    ))
    .parse(input)?;
    Ok((input, node_from_to(start, input, elem)))
}

fn view_filter_member(input: Input<'_>) -> IResult<Input<'_>, Node<FilterMember>> {
    crate::parser::package::filter_member(input)
}

fn view_rendering_usage(input: Input<'_>) -> IResult<Input<'_>, Node<ViewRenderingUsage>> {
    let start = input;
    let (input, _) = preceded(ws_and_comments, tag(&b"render"[..])).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name_str) = name(input)?;
    let (input, header) = parse_feature_usage_header(input)?;
    let (input, body) = connect_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ViewRenderingUsage {
                name: name_str,
                type_name: header.type_name,
                body,
            },
        ),
    ))
}

fn view_def_body_recovery(start: Input<'_>, end: Input<'_>) -> Node<ViewDefBodyElement> {
    if starts_with_any_keyword(start.fragment(), VIEW_DEF_BODY_STARTERS) {
        let recovery = build_recovery_error_node_from_span(
            start,
            end,
            VIEW_DEF_BODY_STARTERS,
            "view definition body",
            "recovered_view_def_body_element",
        );
        let node: Node<ParseErrorNode> = node_from_to(start, end, recovery);
        return node_from_to(start, end, ViewDefBodyElement::Error(node));
    }
    let preview = String::from_utf8_lossy(&start.fragment()[..start.fragment().len().min(60)])
        .trim()
        .to_string();
    node_from_to(start, end, ViewDefBodyElement::Other(preview))
}

fn view_def_body(input: Input<'_>) -> IResult<Input<'_>, ViewDefBody> {
    let (input, _) = ws_and_comments(input)?;
    if input.fragment().starts_with(b";") {
        let (input, _) = tag(&b";"[..]).parse(input)?;
        return Ok((input, ViewDefBody::Semicolon));
    }
    let (input, elements) = crate::parser::body::parse_structured_brace_members(
        input,
        VIEW_DEF_BODY_STARTERS,
        "view definition body",
        "recovered_view_def_body_element",
        view_def_body_element,
        view_def_body_recovery,
    )?;
    Ok((input, ViewDefBody::Brace { elements }))
}

pub(crate) fn view_def(input: Input<'_>) -> IResult<Input<'_>, Node<ViewDef>> {
    let start = input;
    let (input, prefix) =
        parse_definition_prefix(input, DefinitionPrefixOptions::new(b"view").def_required())?;
    let (input, body) = view_def_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ViewDef {
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}

pub(crate) fn viewpoint_def(input: Input<'_>) -> IResult<Input<'_>, Node<ViewpointDef>> {
    let start = input;
    let (input, prefix) = parse_definition_prefix(
        input,
        DefinitionPrefixOptions::new(b"viewpoint").def_required(),
    )?;
    let (input, body) = requirement_def_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ViewpointDef {
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}

fn rendering_def_body_element(
    input: Input<'_>,
) -> IResult<Input<'_>, Node<RenderingDefBodyElement>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, elem) = alt((
        map(doc_comment, RenderingDefBodyElement::Doc),
        map(view_filter_member, RenderingDefBodyElement::Filter),
        map(view_rendering_usage, RenderingDefBodyElement::ViewRendering),
    ))
    .parse(input)?;
    Ok((input, node_from_to(start, input, elem)))
}

fn rendering_def_body_recovery(start: Input<'_>, end: Input<'_>) -> Node<RenderingDefBodyElement> {
    let recovery = build_recovery_error_node_from_span(
        start,
        end,
        VIEW_DEF_BODY_STARTERS,
        "rendering definition body",
        "recovered_rendering_def_body_element",
    );
    node_from_to(
        start,
        end,
        RenderingDefBodyElement::Error(node_from_to(start, end, recovery)),
    )
}

fn rendering_def_body(input: Input<'_>) -> IResult<Input<'_>, RenderingDefBody> {
    let (input, _) = ws_and_comments(input)?;
    if input.fragment().starts_with(b";") {
        let (input, _) = tag(&b";"[..]).parse(input)?;
        return Ok((input, RenderingDefBody::Semicolon));
    }
    let (input, elements) = crate::parser::body::parse_structured_brace_members(
        input,
        VIEW_DEF_BODY_STARTERS,
        "rendering definition body",
        "recovered_rendering_def_body_element",
        rendering_def_body_element,
        rendering_def_body_recovery,
    )?;
    Ok((input, RenderingDefBody::Brace { elements }))
}

pub(crate) fn rendering_def(input: Input<'_>) -> IResult<Input<'_>, Node<RenderingDef>> {
    let start = input;
    let (input, prefix) = parse_definition_prefix(
        input,
        DefinitionPrefixOptions::new(b"rendering").def_required(),
    )?;
    let (input, body) = rendering_def_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            RenderingDef {
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}

fn view_body_element(input: Input<'_>) -> IResult<Input<'_>, Node<ViewBodyElement>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, elem) = alt((
        map(doc_comment, ViewBodyElement::Doc),
        map(view_filter_member, ViewBodyElement::Filter),
        map(view_rendering_usage, ViewBodyElement::ViewRendering),
        map(expose_member, ViewBodyElement::Expose),
        map(satisfy_view_member, ViewBodyElement::Satisfy),
    ))
    .parse(input)?;
    Ok((input, node_from_to(start, input, elem)))
}

/// expose (MembershipImport | NamespaceImport) RelationshipBody
/// MembershipImport = QualifiedName (::**)?
/// NamespaceImport = QualifiedName :: * (::**)?
fn expose_member(input: Input<'_>) -> IResult<Input<'_>, Node<ExposeMember>> {
    let start = input;
    let (input, _) = preceded(ws_and_comments, tag(&b"expose"[..])).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, first) = qualified_name.parse(input)?;
    let (input, target) = alt((
        // ::*::** (try before ::* since * would consume first char of **)
        map(
            (
                preceded(ws_and_comments, tag(&b"::"[..])),
                preceded(ws_and_comments, tag(&b"*"[..])),
                preceded(ws_and_comments, tag(&b"::"[..])),
                preceded(ws_and_comments, tag(&b"**"[..])),
            ),
            |_| format!("{}::*::**", first),
        ),
        // ::** (try before ::*)
        map(
            (
                preceded(ws_and_comments, tag(&b"::"[..])),
                preceded(ws_and_comments, tag(&b"**"[..])),
            ),
            |_| format!("{}::**", first),
        ),
        // ::*
        map(
            (
                preceded(ws_and_comments, tag(&b"::"[..])),
                preceded(ws_and_comments, tag(&b"*"[..])),
            ),
            |_| format!("{}::*", first),
        ),
        // plain
        map(success(()), |_| first.clone()),
    ))
    .parse(input)?;
    let (peek, _) = ws_and_comments(input)?;
    if peek.fragment().first() == Some(&b'.') {
        return Err(nom::Err::Error(nom::error::Error::new(
            peek,
            nom::error::ErrorKind::Tag,
        )));
    }
    // Optional filter [ expr ] - skip content to reach body
    let (input, _) = nom::combinator::opt(nom::sequence::delimited(
        preceded(ws_and_comments, tag(&b"["[..])),
        nom::bytes::complete::take_until(&b"]"[..]),
        preceded(ws_and_comments, tag(&b"]"[..])),
    ))
    .parse(input)?;
    let (input, body) = connect_body(input)?;
    Ok((
        input,
        node_from_to(start, input, ExposeMember { target, body }),
    ))
}

/// satisfy QualifiedName RelationshipBody (simplified form in view body)
fn satisfy_view_member(input: Input<'_>) -> IResult<Input<'_>, Node<SatisfyViewMember>> {
    let start = input;
    let (input, _) = preceded(ws_and_comments, tag(&b"satisfy"[..])).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, viewpoint_ref) = qualified_name.parse(input)?;
    let (input, body) = connect_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            SatisfyViewMember {
                viewpoint_ref,
                body,
            },
        ),
    ))
}

fn view_body_recovery(start: Input<'_>, end: Input<'_>) -> Node<ViewBodyElement> {
    if starts_with_any_keyword(start.fragment(), VIEW_BODY_STARTERS) {
        let recovery = build_recovery_error_node_from_span(
            start,
            end,
            VIEW_BODY_STARTERS,
            "view body",
            "recovered_view_body_element",
        );
        let node: Node<ParseErrorNode> = node_from_to(start, end, recovery);
        return node_from_to(start, end, ViewBodyElement::Error(node));
    }
    let preview = String::from_utf8_lossy(&start.fragment()[..start.fragment().len().min(60)])
        .trim()
        .to_string();
    node_from_to(start, end, ViewBodyElement::Other(preview))
}

fn view_body(input: Input<'_>) -> IResult<Input<'_>, ViewBody> {
    let (input, _) = ws_and_comments(input)?;
    if input.fragment().starts_with(b";") {
        let (input, _) = tag(&b";"[..]).parse(input)?;
        return Ok((input, ViewBody::Semicolon));
    }
    let (input, elements) = crate::parser::body::parse_structured_brace_members_with_skip(
        input,
        VIEW_BODY_STARTERS,
        "view body",
        "recovered_view_body_element",
        view_body_element,
        view_body_recovery,
        crate::parser::body::BraceMemberSkip::BodyElementRecover,
    )?;
    Ok((input, ViewBody::Brace { elements }))
}

pub(crate) fn view_usage(input: Input<'_>) -> IResult<Input<'_>, Node<ViewUsage>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = nom::combinator::opt(preceded(tag(&b"abstract"[..]), ws1)).parse(input)?;
    let (input, _) = tag(&b"view"[..]).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name_str) = name(input)?;
    let (input, header) = parse_feature_usage_header(input)?;
    let (input, body) = view_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ViewUsage {
                name: name_str,
                type_name: header.type_name,
                body,
            },
        ),
    ))
}

pub(crate) fn viewpoint_usage(input: Input<'_>) -> IResult<Input<'_>, Node<ViewpointUsage>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = nom::combinator::opt(preceded(tag(&b"abstract"[..]), ws1)).parse(input)?;
    let (input, _) = tag(&b"viewpoint"[..]).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name_str) = name(input)?;
    let (input, header) = parse_feature_usage_header(input)?;
    let (input, body) = requirement_def_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ViewpointUsage {
                name: name_str,
                type_name: header.type_name.unwrap_or_default(),
                body,
            },
        ),
    ))
}

pub(crate) fn rendering_usage(input: Input<'_>) -> IResult<Input<'_>, Node<RenderingUsage>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = nom::combinator::opt(preceded(tag(&b"abstract"[..]), ws1)).parse(input)?;
    let (input, _) = tag(&b"rendering"[..]).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name_str) = name(input)?;
    let (input, header) = parse_feature_usage_header(input)?;
    let (input, body) = connect_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            RenderingUsage {
                name: name_str,
                type_name: header.type_name,
                body,
            },
        ),
    ))
}

#[cfg(test)]
mod expose_diagnostic_tests {
    use crate::parse_with_diagnostics;

    #[test]
    fn invalid_expose_dot_produces_separator_diagnostic() {
        let input = "package Views { view structure: GeneralView { expose SurveillanceDrone.SurveillanceQuadrotorDrone; } }";
        let result = parse_with_diagnostics(input);
        assert!(
            !result.is_ok(),
            "expected diagnostics, got {:?}",
            result.errors
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| { e.code.as_deref() == Some("invalid_qualified_name_separator") }),
            "codes: {:?}",
            result.errors.iter().map(|e| &e.code).collect::<Vec<_>>()
        );
    }
}
