//! Definition subclassification (`:>` / `specializes`) parsing.

use crate::ast::Span;
use crate::parser::lex::{
    qualified_name, specialization_operator, starts_with_keyword, take_until_terminator,
    ws_and_comments,
};
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

fn starts_with_typing_colon(fragment: &[u8]) -> bool {
    fragment.starts_with(b":") && !fragment.starts_with(b":>")
}

fn specializes_from_header_text(header: &str) -> Option<String> {
    let trimmed = header.trim();
    if let Some(pos) = trimmed.find(":>") {
        let tail = trimmed[pos + 2..].trim();
        if !tail.is_empty() {
            return Some(tail.to_string());
        }
    }
    if let Some(pos) = trimmed
        .as_bytes()
        .windows(b"specializes".len())
        .position(|window| window.eq_ignore_ascii_case(b"specializes"))
    {
        let tail = trimmed[pos + b"specializes".len()..].trim();
        if !tail.is_empty() {
            return Some(tail.to_string());
        }
    }
    None
}

/// After `identification`, parse optional typed header and/or subclassification.
///
/// Supports both:
/// - `def Name :> Base` / `specializes Base`
/// - library shorthand `abstract connection name : Type[multiplicity] :> redefines { ... }`
pub(crate) fn parse_optional_definition_header_after_identification(
    input: Input<'_>,
) -> IResult<Input<'_>, (Option<String>, Option<Span>)> {
    let (input, _) = ws_and_comments(input)?;
    if input.fragment().starts_with(b":>") || starts_with_keyword(input.fragment(), b"specializes")
    {
        return parse_optional_definition_specialization(input);
    }
    if starts_with_typing_colon(input.fragment()) {
        let before_header = input;
        let (input, header) = take_until_terminator(input, b";{")?;
        let specializes = specializes_from_header_text(&header);
        let specializes_span = specializes
            .as_ref()
            .map(|_| span_from_to(before_header, input));
        return Ok((input, (specializes, specializes_span)));
    }
    Ok((input, (None, None)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom_locate::LocatedSpan;

    fn span_input(text: &str) -> Input<'_> {
        LocatedSpan::new(text.as_bytes())
    }

    #[test]
    fn header_after_ident_skips_typing_and_extracts_specializes() {
        let input = span_input(": Connection[0..*] nonunique :> linkObjects, parts");
        let (rest, (specializes, _)) =
            parse_optional_definition_header_after_identification(input).expect("header");
        assert!(rest.fragment().is_empty());
        assert_eq!(specializes.as_deref(), Some("linkObjects, parts"));
    }

    #[test]
    fn header_after_ident_parses_direct_specializes() {
        let input = span_input(":> Base, Other");
        let (rest, (specializes, _)) =
            parse_optional_definition_header_after_identification(input).expect("header");
        assert!(rest.fragment().is_empty());
        assert_eq!(specializes.as_deref(), Some("Base, Other"));
    }
}
