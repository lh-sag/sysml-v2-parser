//! Port definition and port usage parsing.
#![allow(dead_code, unused_imports)]

use crate::ast::{Node, PortBody, PortDef, PortDefBody, PortDefBodyElement, PortUsage};
use crate::parser::action::in_out_decl;
use crate::parser::attribute::{attribute_def, attribute_usage};
use crate::parser::expr::expression;
use crate::parser::lex::{
    identification, name, qualified_name, skip_until_brace_end, specialization_operator,
    take_until_terminator, ws1, ws_and_comments,
};
use crate::parser::definition_prefix::{parse_definition_prefix, DefinitionPrefixOptions};
use crate::parser::node_from_to;
use crate::parser::requirement::doc_comment;
use crate::parser::usage::{multiplicity, optional_typings, specialization_clauses};
use crate::parser::with_span;
use crate::parser::Input;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

/// Port body: ';' or '{' PortUsage* '}' (nested ports) or '{' skip '}' for Brace (e.g. in/out ends).
fn port_body(input: Input<'_>) -> IResult<Input<'_>, PortBody> {
    let (input, _) = ws_and_comments(input)?;
    alt((
        map(tag(&b";"[..]), |_| PortBody::Semicolon),
        port_body_brace,
    ))
    .parse(input)
}

/// Brace port body: '{' ( PortUsage* | skip to '}' ) '}'.
fn port_body_brace(input: Input<'_>) -> IResult<Input<'_>, PortBody> {
    let (input, _) = tag(&b"{"[..]).parse(input)?;
    let (input, _) = ws_and_comments(input)?;
    let (input, elements) = many0(preceded(ws_and_comments, port_usage)).parse(input)?;
    let (input, _) = if elements.is_empty() {
        skip_until_brace_end(input)?
    } else {
        (input, ())
    };
    let (input, _) = preceded(ws_and_comments, tag(&b"}"[..])).parse(input)?;
    Ok((
        input,
        if elements.is_empty() {
            PortBody::Brace
        } else {
            PortBody::BraceWithPorts { elements }
        },
    ))
}

/// Port usage: 'port' ( ':>>' name | name ) ( ':' type )? multiplicity? ( ':>' ... )? ( 'redefines' ... )? body
pub(crate) fn port_usage(input: Input<'_>) -> IResult<Input<'_>, Node<PortUsage>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = tag(&b"port"[..]).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, prefix_redefines) = opt(preceded(ws_and_comments, tag(&b":>>"[..])))
        .parse(input)
        .map(|(i, o)| (i, o.is_some()))?;
    let (input, _) = ws_and_comments(input)?;
    let (input, (name_span, name_str)) = with_span(name).parse(input)?;
    let (input, type_result) = optional_typings(input)?;
    let (type_ref_span, type_name) = type_result
        .map(|(span, name)| (Some(span), Some(name)))
        .unwrap_or((None, None));
    let (input, multiplicity) = opt(multiplicity).parse(input)?;
    let (input, clauses) = specialization_clauses(input)?;
    let redefines = clauses.redefines.or_else(|| {
        if prefix_redefines {
            Some(name_str.clone())
        } else {
            None
        }
    });
    let (input, body) = port_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            PortUsage {
                name: name_str,
                type_name,
                multiplicity,
                subsets: clauses.subsets,
                redefines,
                body,
                name_span: Some(name_span),
                type_ref_span,
            },
        ),
    ))
}

/// Port def body: ';' or '{' PortDefBodyElement* '}' (or skip to '}' when body is unparseable).
fn port_def_body(input: Input<'_>) -> IResult<Input<'_>, PortDefBody> {
    let (input, _) = ws_and_comments(input)?;
    alt((
        map(tag(&b";"[..]), |_| PortDefBody::Semicolon),
        port_def_body_brace,
    ))
    .parse(input)
}

fn port_def_body_brace(input: Input<'_>) -> IResult<Input<'_>, PortDefBody> {
    let (input, _) = tag(&b"{"[..]).parse(input)?;
    let (input, _) = ws_and_comments(input)?;
    let (input, elements) = many0(preceded(ws_and_comments, port_def_body_element)).parse(input)?;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = if input.fragment().starts_with(b"}") {
        (input, ())
    } else {
        skip_until_brace_end(input)?
    };
    let (input, _) = preceded(ws_and_comments, tag(&b"}"[..])).parse(input)?;
    Ok((input, PortDefBody::Brace { elements }))
}

fn port_def_body_element(input: Input<'_>) -> IResult<Input<'_>, Node<PortDefBodyElement>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, elem) = nom::branch::alt((
        map(in_out_decl, PortDefBodyElement::InOutDecl),
        map(doc_comment, PortDefBodyElement::Doc),
        map(|i| attribute_def(i, true), PortDefBodyElement::AttributeDef),
        map(attribute_usage, PortDefBodyElement::AttributeUsage),
        map(port_usage, PortDefBodyElement::PortUsage),
    ))
    .parse(input)?;
    Ok((input, node_from_to(start, input, elem)))
}

/// Port definition: 'port' 'def' Identification ( (':>' | 'specializes') qualified_name )? body
pub(crate) fn port_def(input: Input<'_>) -> IResult<Input<'_>, Node<PortDef>> {
    let start = input;
    let (input, prefix) =
        parse_definition_prefix(input, DefinitionPrefixOptions::new(b"port"))?;
    let (input, body) = port_def_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            PortDef {
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}
