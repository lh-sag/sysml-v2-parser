//! Shared definition body terminators (semicolon or opaque brace).

use crate::ast::DefinitionBody;
use crate::parser::lex::{skip_statement_or_block, skip_until_brace_end, ws_and_comments};
use crate::parser::Input;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

/// Families using opaque brace bodies (inner content skipped, not structured):
/// none currently; keep this helper for families that still need broad compatibility.
#[allow(dead_code)]
pub(crate) fn semicolon_or_opaque_brace_body(
    input: Input<'_>,
) -> IResult<Input<'_>, DefinitionBody> {
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

/// `;` or brace body parsed as a sequence of statements/blocks.
///
/// This is more grammar-aware than fully opaque skipping: it still doesn't build
/// a structured AST for inner members, but it does parse through each inner
/// statement or nested block boundary.
pub(crate) fn semicolon_or_statement_brace_body(
    input: Input<'_>,
) -> IResult<Input<'_>, DefinitionBody> {
    let (input, _) = ws_and_comments(input)?;
    if let Ok((input, _)) = tag::<_, _, nom::error::Error<_>>(&b";"[..]).parse(input) {
        return Ok((input, DefinitionBody::Semicolon));
    }

    let (mut input, _) = tag(&b"{"[..]).parse(input)?;
    loop {
        let (next, _) = ws_and_comments(input)?;
        let fragment = next.fragment();
        if fragment.is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                next,
                nom::error::ErrorKind::Tag,
            )));
        }
        if fragment.starts_with(b"}") {
            let (input, _) = tag(&b"}"[..]).parse(next)?;
            return Ok((input, DefinitionBody::Brace));
        }
        let (next, _) = skip_statement_or_block(next)?;
        input = next;
    }
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

    #[test]
    fn statement_brace_body_consumes_statements() {
        let input = span_input("{ doc /* note */ x = y; nested { z = q; } }");
        let (rest, body) = semicolon_or_statement_brace_body(input).expect("body");
        assert!(matches!(body, DefinitionBody::Brace));
        assert!(rest.fragment().is_empty());
    }
}
