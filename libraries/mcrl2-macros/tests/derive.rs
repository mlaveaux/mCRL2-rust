// use mcrl2_derive::Protected;

#[test]
fn test_derive_term() {
    let t = trybuild::TestCases::new();
    t.pass("tests/input/derive_term.rs");
}