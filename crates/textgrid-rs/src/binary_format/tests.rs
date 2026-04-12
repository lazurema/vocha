use pretty_assertions::assert_eq;

use crate::{TEST_FIXTURES_DIR, TextGrid, TextGridInterval, TextGridIntervalTier, TextGridTier};

#[test]
fn test_deserialize() {
    for entry in TEST_FIXTURES_DIR.find("*.TextGrid").unwrap() {
        let path = entry.path();
        let bin_path = path.with_added_extension("pympi-bin");
        println!("Testing deserialization of {}", bin_path.display());

        let file = TEST_FIXTURES_DIR.get_file(&bin_path).unwrap();
        let contents = file.contents();

        let actual = TextGrid::deserialize_binary_format(contents).unwrap();

        let expected_path = path.with_added_extension("expected.ron");
        let expected_ron = TEST_FIXTURES_DIR.get_file(&expected_path).unwrap();
        let mut expected: TextGrid = ron::from_str(expected_ron.contents_utf8().unwrap()).unwrap();

        patch("expected", &expected_path, &mut expected);

        assert_eq!(
            expected,
            actual,
            "Deserialized TextGrid did not match expected value for {}",
            bin_path.display()
        );
    }
}

#[test]
fn test_serialize() {
    for base_entry in TEST_FIXTURES_DIR.find("*.TextGrid").unwrap() {
        let base_path = base_entry.path();
        let path = base_path.with_added_extension("expected.ron");
        println!("Testing serialization of {}", path.display());

        let file = TEST_FIXTURES_DIR.get_file(&path).unwrap();
        let contents = file.contents_utf8().unwrap();

        let mut input: TextGrid = ron::from_str(contents).unwrap();
        patch("input", &path, &mut input);
        let actual = input.serialize_binary_format().unwrap();

        let expected_path = base_path.with_added_extension("pympi-bin");
        let expected = TEST_FIXTURES_DIR
            .get_file(&expected_path)
            .unwrap()
            .contents();
        assert_eq!(
            expected,
            actual,
            "Serialized TextGrid did not match expected value for {}",
            path.display()
        );
    }
}

fn patch(what: &str, path: &std::path::Path, expected: &mut TextGrid) {
    match path.to_str() {
        Some("praatIO/bobby_phones_elan.TextGrid.expected.ron") => {
            println!(
                "Patching {} TextGrid for {}: {}",
                what,
                path.display(),
                "pympi prepended an extra interval, which is not present in the original TextGrid file."
            );
            let Some(TextGridTier::IntervalTier(TextGridIntervalTier { intervals, .. })) =
                expected.tiers.first_mut()
            else {
                unreachable!()
            };
            let mut new_intervals = vec![TextGridInterval {
                xmin: 0.0,
                xmax: 0.0124716553288,
                text: "".to_owned(),
            }];
            new_intervals.extend_from_slice(intervals);
            *intervals = new_intervals;
        }

        Some("nltk_contrib/demo_data2.TextGrid.expected.ron") => {
            println!(
                "Patching {} TextGrid for {}: {}",
                what,
                path.display(),
                "pympi appended an extra interval, which is not present in the original TextGrid file."
            );
            let Some(TextGridTier::IntervalTier(TextGridIntervalTier { intervals, .. })) =
                expected.tiers.last_mut()
            else {
                unreachable!()
            };
            intervals.push(TextGridInterval {
                xmin: 2.341428074708195,
                xmax: 2.8,
                text: "".to_owned(),
            });
        }
        _ => {}
    }
}
