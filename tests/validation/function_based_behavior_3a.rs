//! Parser test for `03-Function-based Behavior/3a-Function-based Behavior-1.sysml`.

use sysml_v2_parser::parse;

#[test]
#[ignore = "requires SysML v2 release fixtures; run with: cargo test --test validation -- --include-ignored"]
fn test_parse_3a_function_based_behavior() {
    super::init_log();
    let path = super::validation_fixture_path("03-Function-based Behavior")
        .join("3a-Function-based Behavior-1.sysml");
    let input = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {}", path.display(), e));
    let result =
        parse(&input).expect("parse should succeed for 3a-Function-based Behavior-1.sysml");
    super::assert_ast_snapshot(
        &result,
        "function_based_behavior_3a",
        "parsed AST should match expected for 3a-Function-based Behavior-1.sysml",
    );
}
