//! Parser test for `01-Parts Tree/1a-Parts Tree.sysml`.

use sysml_v2_parser::parse;

#[test]
#[ignore = "requires SysML v2 release fixtures; run with: cargo test --test validation -- --include-ignored"]
fn test_parse_1a_parts_tree() {
    super::init_log();
    let path = super::validation_fixture_path("01-Parts Tree").join("1a-Parts Tree.sysml");
    let input = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {}", path.display(), e));
    let result = parse(&input).expect("parse should succeed for 1a-Parts Tree.sysml");
    super::assert_ast_snapshot(
        &result,
        "parts_tree_1a",
        "parsed AST should match expected for 1a-Parts Tree.sysml",
    );
}
