#[test]
fn test_derive_term() {
    let t = trybuild::TestCases::new();
    t.pass("examples/derive_term.rs");
}

#[test]
fn test_lifetimes() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/input/aterm_lifetime.rs");
}
