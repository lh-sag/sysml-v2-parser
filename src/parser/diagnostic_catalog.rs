//! Stable parser diagnostic code registry for Spec42 and LSP consumers.

/// Parser-owned diagnostic codes emitted by recovery and validation paths.
pub const MISSING_MEMBER_NAME: &str = "missing_member_name";
pub const MISSING_TYPE_REFERENCE: &str = "missing_type_reference";
pub const INVALID_TYPING_OPERATOR: &str = "invalid_typing_operator";
pub const MISSING_EXPRESSION_AFTER_OPERATOR: &str = "missing_expression_after_operator";
pub const INVALID_UNIT_REFERENCE: &str = "invalid_unit_reference";
pub const INVALID_BARE_IDENTIFIER_IN_STATE_BODY: &str = "invalid_bare_identifier_in_state_body";
pub const RECOVERY_CASCADE_SUPPRESSED: &str = "recovery_cascade_suppressed";
pub const RECOVERED_ROOT_BODY: &str = "recovered_root_body";
pub const MISSING_CLOSING_BRACE: &str = "missing_closing_brace";
pub const UNEXPECTED_CLOSING_BRACE: &str = "unexpected_closing_brace";
pub const MISSING_SEMICOLON: &str = "missing_semicolon";
pub const MISSING_BODY_OR_SEMICOLON: &str = "missing_body_or_semicolon";
pub const MISSING_REP_LANGUAGE: &str = "missing_rep_language";
pub const INVALID_REP_LANGUAGE: &str = "invalid_rep_language";

/// All stable codes documented for cross-repo contracts.
pub const DOCUMENTED_CODES: &[&str] = &[
    MISSING_MEMBER_NAME,
    MISSING_TYPE_REFERENCE,
    INVALID_TYPING_OPERATOR,
    MISSING_EXPRESSION_AFTER_OPERATOR,
    INVALID_UNIT_REFERENCE,
    INVALID_BARE_IDENTIFIER_IN_STATE_BODY,
    RECOVERY_CASCADE_SUPPRESSED,
    RECOVERED_ROOT_BODY,
    MISSING_CLOSING_BRACE,
    UNEXPECTED_CLOSING_BRACE,
    MISSING_SEMICOLON,
    MISSING_BODY_OR_SEMICOLON,
    MISSING_REP_LANGUAGE,
    INVALID_REP_LANGUAGE,
];
