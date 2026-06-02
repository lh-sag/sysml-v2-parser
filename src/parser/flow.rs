use crate::ast::{FlowDef, FlowUsage, Node};
use crate::parser::body::semicolon_or_structured_definition_body;
use crate::parser::definition_prefix::{parse_definition_prefix, DefinitionPrefixOptions};
use crate::parser::expr::expression;
use crate::parser::lex::{name, ws1, ws_and_comments};
use crate::parser::node_from_to;
use crate::parser::usage::feature_usage_header;
use crate::parser::Input;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

pub(crate) fn flow_def(input: Input<'_>) -> IResult<Input<'_>, Node<FlowDef>> {
    let start = input;
    let (input, prefix) =
        parse_definition_prefix(input, DefinitionPrefixOptions::new(b"flow").def_required())?;
    let (input, body) = semicolon_or_structured_definition_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            FlowDef {
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}

pub(crate) fn flow_usage(input: Input<'_>) -> IResult<Input<'_>, Node<FlowUsage>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = nom::combinator::opt(preceded(tag(&b"abstract"[..]), ws1)).parse(input)?;
    let (input, _) = alt((tag(&b"flow"[..]), tag(&b"message"[..]))).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name_str) = name(input)?;
    let (input, header) = feature_usage_header(input)?;
    let type_name = header.type_name;
    let (input, from) = opt(preceded(
        preceded(ws_and_comments, tag(&b"from"[..])),
        preceded(ws1, expression),
    ))
    .parse(input)?;
    let (input, to) = match from {
        Some(_) => {
            let (input, _) = preceded(ws_and_comments, tag(&b"to"[..])).parse(input)?;
            let (input, to) = preceded(ws1, expression).parse(input)?;
            (input, Some(to))
        }
        None => (input, None),
    };
    let (input, _) = ws_and_comments(input)?;
    let (input, body) = semicolon_or_structured_definition_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            FlowUsage {
                name: name_str,
                type_name,
                from,
                to,
                body,
            },
        ),
    ))
}
