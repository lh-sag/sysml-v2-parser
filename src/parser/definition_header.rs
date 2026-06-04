//! Internal definition and usage header parsing (P4).
//!
//! Consolidates typed headers and specialization without changing public AST shapes.

use crate::ast::Span;
use crate::parser::specialization::parse_optional_definition_header_after_identification;
use crate::parser::usage::{feature_usage_header, usage_header, UsageHeader};
use crate::parser::Input;
use nom::IResult;

/// Parsed subclassification / typed header after `identification`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DefinitionHeaderParts {
    pub specializes: Option<String>,
    pub specializes_span: Option<Span>,
}

/// Parse optional definition header after identification (typing + `:>` / `specializes`).
pub(crate) fn parse_definition_header_after_ident(
    input: Input<'_>,
) -> IResult<Input<'_>, DefinitionHeaderParts> {
    let (input, (specializes, specializes_span)) =
        parse_optional_definition_header_after_identification(input)?;
    Ok((
        input,
        DefinitionHeaderParts {
            specializes,
            specializes_span,
        },
    ))
}

/// Feature usage header parts (typing, subsets, redefines, references, crosses).
pub(crate) type FeatureHeaderParts = UsageHeader;

/// Library-style feature usage header (multiplicity, typing, specialization, intersects).
pub(crate) fn parse_feature_usage_header(input: Input<'_>) -> IResult<Input<'_>, FeatureHeaderParts> {
    feature_usage_header(input)
}

/// Usage header with specialization before or after typing.
pub(crate) fn parse_usage_header(input: Input<'_>) -> IResult<Input<'_>, FeatureHeaderParts> {
    usage_header(input)
}
