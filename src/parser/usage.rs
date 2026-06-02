//! Shared usage grammar fragments from `UsageDeclaration` / `FeatureSpecializationPart`.

use crate::ast::{Expression, Node, Span};
use crate::parser::expr::expression;
use crate::parser::lex::{
    name, qualified_name, redefine_operator, starts_with_keyword, subset_operator,
    typed_by_operator, ws_and_comments,
};
use crate::parser::{span_from_to, Input};
use nom::bytes::complete::{tag, take_until};
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::preceded;
use nom::IResult;
use nom::Parser;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct SpecializationClauses {
    pub subsets: Option<(String, Option<Node<Expression>>)>,
    pub redefines: Option<String>,
    pub had_any: bool,
}

/// Multiplicity part: '[' ... ']'.
pub(crate) fn multiplicity(input: Input<'_>) -> IResult<Input<'_>, String> {
    let (input, _) = ws_and_comments(input)?;
    let (input, _) = tag(&b"["[..]).parse(input)?;
    let (input, content) = take_until(&b"]"[..]).parse(input)?;
    let (input, _) = tag(&b"]"[..]).parse(input)?;
    Ok((
        input,
        format!("[{}]", String::from_utf8_lossy(content.fragment()).trim()),
    ))
}

/// Typings: `:` / `defined by` one or more qualified names, with optional conjugated `~`.
pub(crate) fn typings(input: Input<'_>) -> IResult<Input<'_>, (Span, String)> {
    let before = input;
    let (input, _) = preceded(ws_and_comments, typed_by_operator).parse(input)?;
    let (input, first) = preceded(ws_and_comments, conjugated_qualified_name).parse(input)?;
    let (input, rest) = many0(preceded(
        preceded(ws_and_comments, tag(&b","[..])),
        preceded(ws_and_comments, conjugated_qualified_name),
    ))
    .parse(input)?;
    let mut names = vec![first];
    names.extend(rest);
    Ok((input, (span_from_to(before, input), names.join(", "))))
}

/// Optional typings that remain strict once a typing starter is present.
pub(crate) fn optional_typings(input: Input<'_>) -> IResult<Input<'_>, Option<(Span, String)>> {
    let (peek, _) = ws_and_comments(input)?;
    let fragment = peek.fragment();
    if (fragment.starts_with(b":") && !fragment.starts_with(b":>") && !fragment.starts_with(b":>>"))
        || starts_with_keyword(fragment, b"defined")
        || starts_with_keyword(fragment, b"typed")
    {
        let (input, typing) = typings(input)?;
        return Ok((input, Some(typing)));
    }
    Ok((input, None))
}

fn conjugated_qualified_name(input: Input<'_>) -> IResult<Input<'_>, String> {
    let (input, conjugated) = opt(tag(&b"~"[..])).parse(input)?;
    let (input, name) = qualified_name(input)?;
    Ok((
        input,
        if conjugated.is_some() {
            format!("~{name}")
        } else {
            name
        },
    ))
}

fn specialization_target(input: Input<'_>) -> IResult<Input<'_>, String> {
    let (input, base) = qualified_name(input)?;
    let (input, dotted) = many0(preceded(
        preceded(ws_and_comments, tag(&b"."[..])),
        preceded(ws_and_comments, name),
    ))
    .parse(input)?;
    if dotted.is_empty() {
        return Ok((input, base));
    }
    Ok((input, format!("{base}.{}", dotted.join("."))))
}

fn specialization_targets(input: Input<'_>) -> IResult<Input<'_>, String> {
    let (input, first) = specialization_target(input)?;
    let (input, rest) = many0(preceded(
        preceded(ws_and_comments, tag(&b","[..])),
        preceded(ws_and_comments, specialization_target),
    ))
    .parse(input)?;
    if rest.is_empty() {
        return Ok((input, first));
    }
    let mut targets = vec![first];
    targets.extend(rest);
    Ok((input, targets.join(", ")))
}

/// Subsettings: `:>` / `subsets` target, with optional `= expression` value.
pub(crate) fn subsetting(
    input: Input<'_>,
) -> IResult<Input<'_>, (String, Option<Node<Expression>>)> {
    let (input, _) = preceded(ws_and_comments, subset_operator).parse(input)?;
    preceded(
        ws_and_comments,
        (
            specialization_targets,
            opt(preceded(
                preceded(ws_and_comments, tag(&b"="[..])),
                preceded(ws_and_comments, expression),
            )),
        ),
    )
    .parse(input)
}

/// Redefinitions: `:>>` / `redefines` target.
pub(crate) fn redefinition(input: Input<'_>) -> IResult<Input<'_>, String> {
    preceded(
        preceded(ws_and_comments, redefine_operator),
        preceded(ws_and_comments, specialization_targets),
    )
    .parse(input)
}

enum SpecializationClause {
    Subsets((String, Option<Node<Expression>>)),
    Redefines(String),
}

/// Parse zero or more subsetting/redefinition clauses in any order.
///
/// When multiple clauses of the same kind are present, the last one wins.
pub(crate) fn specialization_clauses(
    input: Input<'_>,
) -> IResult<Input<'_>, SpecializationClauses> {
    let (input, clauses) = many0(preceded(
        ws_and_comments,
        nom::branch::alt((
            nom::combinator::map(subsetting, SpecializationClause::Subsets),
            nom::combinator::map(redefinition, SpecializationClause::Redefines),
        )),
    ))
    .parse(input)?;
    let mut out = SpecializationClauses::default();
    for clause in clauses {
        match clause {
            SpecializationClause::Subsets(value) => out.subsets = Some(value),
            SpecializationClause::Redefines(value) => out.redefines = Some(value),
        }
    }
    out.had_any = out.subsets.is_some() || out.redefines.is_some();
    Ok((input, out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom_locate::LocatedSpan;

    fn span_input(text: &str) -> Input<'_> {
        LocatedSpan::new(text.as_bytes())
    }

    #[test]
    fn typings_accepts_defined_by_and_multiple_targets() {
        let input = span_input("defined by ~Ports::Fuel, Ports::Command ;");
        let (rest, (_, typing)) = typings(input).expect("typings");
        assert_eq!(typing, "~Ports::Fuel, Ports::Command");
        assert!(rest.fragment().trim_ascii_start().starts_with(b";"));
    }

    #[test]
    fn typings_accepts_typed_by_keyword_alias() {
        let input = span_input("typed by ~Ports::Fuel, Ports::Command ;");
        let (rest, (_, typing)) = typings(input).expect("typings");
        assert_eq!(typing, "~Ports::Fuel, Ports::Command");
        assert!(rest.fragment().trim_ascii_start().starts_with(b";"));
    }

    #[test]
    fn subsetting_accepts_keyword_alias_with_value() {
        let input = span_input("subsets wheel = rearWheel[1];");
        let (_, (target, value)) = subsetting(input).expect("subsetting");
        assert_eq!(target, "wheel");
        assert!(value.is_some());
    }

    #[test]
    fn specialization_clauses_accepts_multiple_mixed_clauses() {
        let input = span_input("subsets base redefines old :> latest :>> newest ;");
        let (rest, clauses) = specialization_clauses(input).expect("specialization clauses");
        assert_eq!(
            clauses.subsets.as_ref().map(|(name, _)| name.as_str()),
            Some("latest")
        );
        assert_eq!(clauses.redefines.as_deref(), Some("newest"));
        assert!(rest.fragment().trim_ascii_start().starts_with(b";"));
    }

    #[test]
    fn specialization_clauses_accept_dotted_feature_chain_targets() {
        let input = span_input(":> electricGrid.outlets :>> Vehicle::mass.value ;");
        let (rest, clauses) = specialization_clauses(input).expect("specialization clauses");
        assert_eq!(
            clauses.subsets.as_ref().map(|(name, _)| name.as_str()),
            Some("electricGrid.outlets")
        );
        assert_eq!(clauses.redefines.as_deref(), Some("Vehicle::mass.value"));
        assert!(rest.fragment().trim_ascii_start().starts_with(b";"));
    }

    #[test]
    fn specialization_clauses_accept_multiple_targets() {
        let input = span_input(":> CoordinateTransformation, List {");
        let (rest, clauses) = specialization_clauses(input).expect("specialization clauses");
        assert_eq!(
            clauses.subsets.as_ref().map(|(name, _)| name.as_str()),
            Some("CoordinateTransformation, List")
        );
        assert!(rest.fragment().trim_ascii_start().starts_with(b"{"));
    }
}
