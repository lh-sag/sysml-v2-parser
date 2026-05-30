//! Definition subclassification (`:>` / `specializes`) parsing.

use crate::ast::Span;
use crate::parser::lex::{qualified_name, specialization_operator, ws_and_comments};
use crate::parser::{span_from_to, Input};
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

/// Optional definition subclassification: `:> Base` or `specializes Base`, with optional `, Base2`.
pub(crate) fn parse_optional_definition_specialization(
    input: Input<'_>,
) -> IResult<Input<'_>, (Option<String>, Option<Span>)> {
    let before_specializes = input;
    let (input, opt_first) = opt((
        preceded(ws_and_comments, specialization_operator),
        preceded(ws_and_comments, qualified_name),
    ))
    .parse(input)?;
    let Some((_, first)) = opt_first else {
        return Ok((input, (None, None)));
    };
    let (input, rest) = many0(preceded(
        preceded(ws_and_comments, tag(&b","[..])),
        preceded(ws_and_comments, qualified_name),
    ))
    .parse(input)?;
    let specializes = if rest.is_empty() {
        first
    } else {
        let mut bases = vec![first];
        bases.extend(rest);
        bases.join(", ")
    };
    Ok((
        input,
        (
            Some(specializes),
            Some(span_from_to(before_specializes, input)),
        ),
    ))
}
