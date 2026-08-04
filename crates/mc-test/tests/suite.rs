//! The three spellings of an embedded suite, and what each rejects.

use mc_test::parse_suite;

const ONE: &str = r#"{"name":"solo","checks":[{"tick":0,"expect":"quiescent"}]}"#;

#[test]
fn a_bare_case_is_a_suite_of_one() {
    let cases = parse_suite(ONE, "f").expect("parses");
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].name, "solo");
}

#[test]
fn an_array_is_a_suite() {
    let text = format!("[{ONE},{ONE}]");
    assert_eq!(parse_suite(&text, "f").expect("parses").len(), 2);
}

#[test]
fn a_format_object_is_a_suite() {
    let text = format!(r#"{{"format":1,"cases":[{ONE}]}}"#);
    assert_eq!(parse_suite(&text, "f").expect("parses").len(), 1);
}

#[test]
fn an_unknown_format_is_refused() {
    let text = format!(r#"{{"format":2,"cases":[{ONE}]}}"#);
    let err = parse_suite(&text, "f").expect_err("format 2 is the future");
    assert!(err.contains("format"), "the error must name the problem: {err}");
}

#[test]
fn an_empty_suite_is_refused() {
    for text in [r#"[]"#.to_string(), r#"{"format":1,"cases":[]}"#.to_string()] {
        let err = parse_suite(&text, "f").expect_err("empty suites pass vacuously");
        assert!(err.contains("no cases"), "{err}");
    }
}

#[test]
fn a_typo_is_named_with_its_file() {
    let err = parse_suite(r#"{"nome":"x"}"#, "door.litematic").expect_err("typo");
    assert!(err.contains("door.litematic"), "{err}");
}
