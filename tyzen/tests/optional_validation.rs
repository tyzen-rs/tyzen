#[test]
fn tyzen_optional_requires_option_field() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/optional_non_option.rs");
}
