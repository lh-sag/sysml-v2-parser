//! Shared definition body terminators (semicolon or opaque brace).

use crate::ast::DefinitionBody;
use crate::parser::lex::{skip_until_brace_end, ws_and_comments};
use crate::parser::Input;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

/// Families using opaque brace bodies (inner content skipped, not structured):
/// `flow`, `allocation`, `metadata` (as of P1 migration).
pub(crate) fn semicolon_or_opaque_brace_body(input: Input<'_>) -> IResult<Input<'_>, DefinitionBody> {
    let (input, _) = ws_and_comments(input)?;
    alt((
        map(tag(&b";"[..]), |_| DefinitionBody::Semicolon),
        map(
            nom::sequence::delimited(
                tag(&b"{"[..]),
                skip_until_brace_end,
                preceded(ws_and_comments, tag(&b"}"[..])),
            ),
            |_| DefinitionBody::Brace,
        ),
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom_locate::LocatedSpan;

    fn span_input(text: &str) -> Input<'_> {
        LocatedSpan::new(text.as_bytes())
    }

    #[test]
    fn semicolon_body() {
        let input = span_input(";");
        let (rest, body) = semicolon_or_opaque_brace_body(input).expect("body");
        assert!(matches!(body, DefinitionBody::Semicolon));
        assert!(rest.fragment().is_empty());
    }

    #[test]
    fn opaque_brace_skips_inner_doc() {
        let input = span_input("{ doc /* note */ part x; }");
        let (rest, body) = semicolon_or_opaque_brace_body(input).expect("body");
        assert!(matches!(body, DefinitionBody::Brace));
        assert!(rest.fragment().is_empty());
    }
}
