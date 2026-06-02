//! Metadata definition parsing (BNF MetadataDefinition).

use crate::ast::{MetadataDef, Node};
use crate::parser::body::semicolon_or_structured_definition_body;
use crate::parser::definition_prefix::{parse_definition_prefix, DefinitionPrefixOptions};
use crate::parser::node_from_to;
use crate::parser::Input;
use nom::IResult;

/// Metadata definition: `metadata def` Identification body (optional `abstract` prefix).
pub(crate) fn metadata_def(input: Input<'_>) -> IResult<Input<'_>, Node<MetadataDef>> {
    let start = input;
    let (input, prefix) = parse_definition_prefix(
        input,
        DefinitionPrefixOptions::new(b"metadata").def_required(),
    )?;
    let (input, body) = semicolon_or_structured_definition_body(input)?;
    Ok((
        input,
        node_from_to(
            start,
            input,
            MetadataDef {
                is_abstract: prefix.is_abstract,
                identification: prefix.identification,
                specializes: prefix.specializes,
                specializes_span: prefix.specializes_span,
                body,
            },
        ),
    ))
}
