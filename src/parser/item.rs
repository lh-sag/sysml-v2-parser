//! Item definition parsing.

use crate::ast::{ItemDef, Node};
use crate::parser::attribute::attribute_body;
use crate::parser::lex::{identification, ws1, ws_and_comments};
use crate::parser::node_from_to;
use crate::parser::parse_optional_definition_header_after_identification;
use crate::parser::Input;
use nom::bytes::complete::tag;
use nom::IResult;
use nom::Parser;

/// Item definition: `item def` Identification body
pub(crate) fn item_def(input: Input<'_>) -> IResult<Input<'_>, Node<ItemDef>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, _) =
        nom::combinator::opt(nom::sequence::preceded(tag(&b"abstract"[..]), ws1)).parse(input)?;
    let (input, _) = tag(&b"item"[..]).parse(input)?;
    let (input, _) = ws1(input)?;
    let (input, _) =
        nom::combinator::opt(nom::sequence::preceded(tag(&b"def"[..]), ws1)).parse(input)?;
    let (input, identification) = identification(input)?;
    let (input, (specializes, specializes_span)) = parse_optional_definition_header_after_identification(input)?;
    let (input, body) = attribute_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            ItemDef {
                identification,
                specializes,
                specializes_span,
                body,
            },
        ),
    ))
}
