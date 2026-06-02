use crate::ast::{AllocationDef, AllocationUsage, Node};
use crate::parser::body::semicolon_or_statement_brace_body;
use crate::parser::definition_prefix::{parse_definition_prefix, DefinitionPrefixOptions};
use crate::parser::expr::expression;
use crate::parser::lex::{name, qualified_name, take_until_terminator, ws1, ws_and_comments};
use crate::parser::node_from_to;
use crate::parser::Input;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

pub(crate) fn allocation_def(input: Input<'_>) -> IResult<Input<'_>, Node<AllocationDef>> {
    let start = input;
    let (input, prefix) = parse_definition_prefix(
        input,
        DefinitionPrefixOptions::new(b"allocation")
            .def_required()
            .no_abstract(),
    )?;
    let (input, body) = semicolon_or_statement_brace_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            AllocationDef {
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}

pub(crate) fn allocation_usage(input: Input<'_>) -> IResult<Input<'_>, Node<AllocationUsage>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = nom::combinator::opt(preceded(tag(&b"abstract"[..]), ws1)).parse(input)?;
    let (input, _) = tag(&b"allocation"[..]).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name_str) = name(input)?;
    let (input, type_name) = opt(preceded(
        preceded(ws_and_comments, tag(&b":"[..])),
        preceded(ws_and_comments, qualified_name),
    ))
    .parse(input)?;
    let (input, source) = opt(preceded(
        preceded(ws_and_comments, tag(&b"allocate"[..])),
        preceded(ws1, expression),
    ))
    .parse(input)?;
    let (input, target) = match source {
        Some(_) => {
            let (input, _) = preceded(ws_and_comments, tag(&b"to"[..])).parse(input)?;
            let (input, target) = preceded(ws1, expression).parse(input)?;
            (input, Some(target))
        }
        None => (input, None),
    };
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = take_until_terminator(input, b";{")?;
    let (input, body) = semicolon_or_statement_brace_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            AllocationUsage {
                name: name_str,
                type_name,
                source,
                target,
                body,
            },
        ),
    ))
}

pub(crate) fn allocate_usage(input: Input<'_>) -> IResult<Input<'_>, Node<AllocationUsage>> {
    let start = input;
    let (input, _) = preceded(ws_and_comments, tag(&b"allocate"[..])).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, source) = expression(input)?;
    let (input, _) = preceded(ws_and_comments, tag(&b"to"[..])).parse(input)?;
    let (input, target) = preceded(ws1, expression).parse(input)?;
    let (input, body) = semicolon_or_statement_brace_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            AllocationUsage {
                name: String::new(),
                type_name: None,
                source: Some(source),
                target: Some(target),
                body,
            },
        ),
    ))
}
