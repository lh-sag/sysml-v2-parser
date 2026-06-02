//! Shared definition body terminators (semicolon or structured brace).

use crate::ast::{DefinitionBody, DefinitionBodyElement, Node};
use crate::parser::build_recovery_error_node_from_span;
use crate::parser::lex::{
    skip_statement_or_block, skip_until_brace_end, ws_and_comments,
};
use crate::parser::node_from_to;
use crate::parser::requirement::doc_comment;
use crate::parser::Input;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

const GENERIC_DEFINITION_BODY_STARTERS: &[&[u8]] = &[
    b"doc",
    b"comment",
    b"import",
    b"metadata",
    b"filter",
    b"@",
    b"#",
];

/// Parse `{` element* `}` with recovery for unknown members.
pub(crate) fn parse_structured_brace_members<'a, E, F, G>(
    input: Input<'a>,
    _starters: &[&[u8]],
    _scope_label: &str,
    _recovery_code: &str,
    mut parse_element: F,
    mut map_recovery: G,
) -> IResult<Input<'a>, Vec<Node<E>>>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, Node<E>>,
    G: FnMut(Input<'a>, Input<'a>) -> Node<E>,
{
    let (mut input, _) = preceded(ws_and_comments, tag(&b"{"[..])).parse(input)?;
    let mut elements = Vec::new();
    loop {
        let (next, _) = ws_and_comments(input)?;
        input = next;
        if input.fragment().is_empty() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )));
        }
        if input.fragment().starts_with(b"}") {
            let (input, _) = preceded(ws_and_comments, tag(&b"}"[..])).parse(input)?;
            return Ok((input, elements));
        }
        match parse_element(input) {
            Ok((next, element)) => {
                if next.location_offset() == input.location_offset() {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        input,
                        nom::error::ErrorKind::Many0,
                    )));
                }
                elements.push(element);
                input = next;
            }
            Err(_) => {
                let start_unknown = input;
                let (after_ws, _) = ws_and_comments(input)?;
                if after_ws.fragment().starts_with(b"}") {
                    input = after_ws;
                    continue;
                }
                let Ok((next, _)) = skip_statement_or_block(input) else {
                    let (input, _) = skip_until_brace_end(input)?;
                    let (input, _) = preceded(ws_and_comments, tag(&b"}"[..])).parse(input)?;
                    return Ok((input, elements));
                };
                if next.location_offset() == start_unknown.location_offset() {
                    let (input, _) = skip_until_brace_end(input)?;
                    let (input, _) = preceded(ws_and_comments, tag(&b"}"[..])).parse(input)?;
                    return Ok((input, elements));
                }
                let (next, _) = ws_and_comments(next)?;
                if next.fragment().starts_with(b"}") {
                    input = next;
                    continue;
                }
                elements.push(map_recovery(start_unknown, next));
                input = next;
            }
        }
    }
}

fn generic_definition_body_element(
    input: Input<'_>,
) -> IResult<Input<'_>, Node<DefinitionBodyElement>> {
    let start = input;
    let (input, _) = ws_and_comments(input)?;
    let (input, elem) = map(doc_comment, DefinitionBodyElement::Doc).parse(input)?;
    Ok((input, node_from_to(start, input, elem)))
}

fn generic_definition_body_recovery(
    start: Input<'_>,
    end: Input<'_>,
    scope_label: &str,
    recovery_code: &str,
) -> Node<DefinitionBodyElement> {
    let recovery = build_recovery_error_node_from_span(
        start,
        end,
        GENERIC_DEFINITION_BODY_STARTERS,
        scope_label,
        recovery_code,
    );
    node_from_to(
        start,
        end,
        DefinitionBodyElement::Error(node_from_to(start, end, recovery)),
    )
}

/// `;` or brace body with doc (and recovered) members for flow/allocation/metadata-style defs.
pub(crate) fn semicolon_or_structured_definition_body(
    input: Input<'_>,
) -> IResult<Input<'_>, DefinitionBody> {
    let (input, _) = ws_and_comments(input)?;
    if input.fragment().starts_with(b";") {
        let (input, _) = tag(&b";"[..]).parse(input)?;
        return Ok((input, DefinitionBody::Semicolon));
    }
    let (input, elements) = parse_structured_brace_members(
        input,
        GENERIC_DEFINITION_BODY_STARTERS,
        "definition body",
        "recovered_definition_body_element",
        generic_definition_body_element,
        |start, end| {
            generic_definition_body_recovery(start, end, "definition body", "recovered_definition_body_element")
        },
    )?;
    Ok((input, DefinitionBody::Brace { elements }))
}

/// Advance through statements/blocks until the next token is `}`.
pub(crate) fn advance_to_closing_brace(mut input: Input<'_>) -> IResult<Input<'_>, ()> {
    loop {
        let (next, _) = ws_and_comments(input)?;
        if next.fragment().starts_with(b"}") {
            return Ok((next, ()));
        }
        let (next, _) = skip_statement_or_block(next)?;
        if next.location_offset() == input.location_offset() {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Many0,
            )));
        }
        input = next;
    }
}
/// none currently; keep for families that still need broad compatibility.
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
            |_| DefinitionBody::Brace { elements: vec![] },
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
    fn structured_brace_parses_doc_member() {
        let input = span_input("{ doc /* note */ }");
        let (rest, body) = semicolon_or_structured_definition_body(input).expect("body");
        assert!(matches!(
            body,
            DefinitionBody::Brace { ref elements } if !elements.is_empty()
        ));
        assert!(rest.fragment().is_empty());
    }

    #[test]
    fn statement_brace_body_consumes_statements() {
        let input = span_input("{ doc /* note */ x = y; nested { z = q; } }");
        let (rest, body) = semicolon_or_structured_definition_body(input).expect("body");
        assert!(matches!(body, DefinitionBody::Brace { .. }));
        assert!(rest.fragment().is_empty());
    }
}
