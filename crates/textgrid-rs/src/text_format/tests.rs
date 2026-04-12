use pretty_assertions::assert_eq;

use crate::{TEST_FIXTURES_DIR, TextGrid};

#[test]
fn test_parse() {
    for entry in TEST_FIXTURES_DIR.find("*.TextGrid").unwrap() {
        let path = entry.path();
        println!("Testing parsing of {}", path.display());

        let file = entry.as_file().unwrap();
        let contents = file.contents();

        let actual = TextGrid::parse_text_format(contents).unwrap();

        let expected_ron = TEST_FIXTURES_DIR
            .get_file(path.with_added_extension("expected.ron"))
            .unwrap();
        let expected: TextGrid = ron::from_str(expected_ron.contents_utf8().unwrap()).unwrap();

        assert_eq!(
            expected,
            actual,
            "Parsed TextGrid did not match expected value for {}",
            path.display()
        );
    }
}

#[test]
fn test_stringify_long() {
    test_stringify("long", crate::text_format::stringify_long);
}

#[test]
fn test_stringify_short() {
    test_stringify("short", crate::text_format::stringify_short);
}

fn test_stringify(
    which: &'static str,
    stringify_fn: impl Fn(&mut Vec<u8>, &TextGrid) -> std::io::Result<()>,
) {
    for entry in TEST_FIXTURES_DIR
        .find(
            // we have already tested parsing of `*.TextGrid` files in
            // `test_stringify_long`, so we just grab those expected files here.
            "*.TextGrid.expected.ron",
        )
        .unwrap()
    {
        let path = entry.path();
        println!("Testing {}-stringification of {}", which, path.display());

        let file = entry.as_file().unwrap();
        let contents = file.contents_utf8().unwrap();

        let input: TextGrid = ron::from_str(contents).unwrap();

        let mut stringified = Vec::new();
        stringify_fn(&mut stringified, &input).unwrap();
        let stringified = String::from_utf8(stringified).unwrap();

        let reparsed = TextGrid::parse_text_format_utf8(&stringified).unwrap();

        assert_eq!(
            input,
            reparsed,
            "Re-parsed TextGrid did not match original input for {}",
            path.display()
        )
    }
}
