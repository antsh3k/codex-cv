#[test]
fn derive_subagent_compiles() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/basic.rs");
}
