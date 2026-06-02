//! Parser test for `04-Functional Allocation/4a-Functional Allocation.sysml`.

use sysml_v2_parser::parse;

#[test]
#[ignore = "requires SysML v2 release fixtures; run with: cargo test --test validation -- --include-ignored"]
fn test_parse_4a_functional_allocation() {
    super::init_log();
    let path = super::validation_fixture_path("04-Functional Allocation")
        .join("4a-Functional Allocation.sysml");
    let input = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {}", path.display(), e));
    let result =
        parse(&input).expect("parse should succeed for 4a-Functional Allocation.sysml");
    super::assert_ast_snapshot(
        &result,
        "functional_allocation_4a",
        "parsed AST should match expected for 4a-Functional Allocation.sysml",
    );
}
