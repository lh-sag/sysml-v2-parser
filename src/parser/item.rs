//! Item definition and usage parsing.

use crate::ast::{ItemDef, ItemUsage, Node};
use crate::parser::attribute::attribute_body;
use crate::parser::definition_prefix::{parse_definition_prefix, DefinitionPrefixOptions};
use crate::parser::lex::{name, ws1, ws_and_comments};
use crate::parser::node_from_to;
use crate::parser::usage::{feature_usage_header, multiplicity};
use crate::parser::Input;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::Parser;
use nom::IResult;

/// Item definition: `item def` Identification body
pub(crate) fn item_def(input: Input<'_>) -> IResult<Input<'_>, Node<ItemDef>> {
    let start = input;
    let (input, prefix) = parse_definition_prefix(input, DefinitionPrefixOptions::new(b"item"))?;
    let (input, body) = attribute_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ItemDef {
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}

/// Item usage in a part definition body: `item` name multiplicity? (`:` type)? body.
pub(crate) fn item_usage(input: Input<'_>) -> IResult<Input<'_>, Node<ItemUsage>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = tag(&b"item"[..]).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, name) = name(input)?;
    let (input, multiplicity) = opt(multiplicity).parse(input)?;
    let (input, header) = feature_usage_header(input)?;
    let (input, body) = attribute_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ItemUsage {
                name,
                type_name: header.type_name,
                multiplicity,
                body,
            },
        ),
    ))
}
