// Port of tests/txtstyle_test.py.
use regex::Regex;

use txtstyle::confparser::ConfParser;
use txtstyle::linestyle::{find_regions, get_style_map};
use txtstyle::transformer::{Style, Transformer, new_index_style, new_regex_style};

const TEST_DATA_DIR: &str = "tests/testdata";

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap()
}

fn rx_style(pattern: &str, keys: &[&str]) -> Style {
    let ks: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
    new_regex_style(pattern, &ks, false).unwrap()
}

fn rx_style_whole(pattern: &str, keys: &[&str]) -> Style {
    let ks: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
    new_regex_style(pattern, &ks, true).unwrap()
}

fn idx_style(regions: Vec<(usize, Option<usize>)>, keys: &[&str]) -> Style {
    let ks: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
    new_index_style(regions, &ks).unwrap()
}

fn sorted_regions<'a>(
    map: &[((usize, usize), &'a Style)],
) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = map.iter().map(|(r, _)| *r).collect();
    out.sort();
    out
}

fn find_region_for<'a>(
    map: &'a [((usize, usize), &'a Style)],
    key: (usize, usize),
) -> &'a Style {
    map.iter()
        .find(|(r, _)| *r == key)
        .map(|(_, s)| *s)
        .expect("region missing")
}

// ---- LineStyleProcessorTests ----

#[test]
fn test_get_style_map() {
    //       0123456789012345678901234567890123456789
    let line = "This is a long string forty chars long..";
    let s1 = rx_style("This", &["red"]);
    let s2 = rx_style("is", &["red"]);
    let s3 = rx_style("s", &["red"]);
    let styles = vec![s1, s2, s3];
    let map = get_style_map(line, &styles);
    assert_eq!(
        sorted_regions(&map),
        vec![(0, 4), (5, 7), (15, 16), (32, 33)]
    );
    assert!(std::ptr::eq(find_region_for(&map, (0, 4)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (5, 7)), &styles[1]));
    assert!(std::ptr::eq(find_region_for(&map, (15, 16)), &styles[2]));
    assert!(std::ptr::eq(find_region_for(&map, (32, 33)), &styles[2]));
}

#[test]
fn test_get_style_map_reverse_order() {
    let line = "This is a long string forty chars long..";
    let s1 = rx_style("s", &["red"]);
    let s2 = rx_style("is", &["red"]);
    let s3 = rx_style("This", &["red"]);
    let styles = vec![s1, s2, s3];
    let map = get_style_map(line, &styles);
    assert_eq!(
        sorted_regions(&map),
        vec![(3, 4), (6, 7), (15, 16), (32, 33)]
    );
    assert!(std::ptr::eq(find_region_for(&map, (3, 4)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (6, 7)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (15, 16)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (32, 33)), &styles[0]));
}

#[test]
fn test_get_style_map_index_start_equals_line_length() {
    let line = "blip";
    let region = (line.len(), Some(line.len() + 1));
    let s1 = idx_style(vec![region], &["red"]);
    let styles = vec![s1];
    let map = get_style_map(line, &styles);
    assert!(map.is_empty());
}

#[test]
fn test_get_style_map_index_end_greater_than_line_length() {
    //       01234567890123456
    let line = "a short string...";
    let region = (7, Some(20));
    assert!(region.1.unwrap() > line.len());
    let s1 = idx_style(vec![region], &["red"]);
    let styles = vec![s1];
    let map = get_style_map(line, &styles);
    assert_eq!(sorted_regions(&map), vec![(7, 17)]);
    assert!(std::ptr::eq(find_region_for(&map, (7, 17)), &styles[0]));
}

#[test]
fn test_get_style_map_index_end_is_none() {
    let line = "end is None, and therefore defaults to line length";
    let region = (0usize, None);
    let s1 = idx_style(vec![region], &["red"]);
    let styles = vec![s1];
    let map = get_style_map(line, &styles);
    let expected_end = line.len();
    assert_eq!(sorted_regions(&map), vec![(0, expected_end)]);
    assert!(std::ptr::eq(
        find_region_for(&map, (0, expected_end)),
        &styles[0]
    ));
}

#[test]
fn test_get_style_map_index_style() {
    let line = "a test string that needs to be longer than 65 characters..........";
    let s1 = idx_style(
        vec![
            (1, Some(5)),
            (4, Some(10)),
            (15, Some(20)),
            (35, Some(40)),
            (45, Some(50)),
        ],
        &["red"],
    );
    let s2 = idx_style(
        vec![
            (1, Some(3)),
            (4, Some(6)),
            (7, Some(14)),
            (41, Some(44)),
            (55, Some(60)),
        ],
        &["red"],
    );
    let s3 = idx_style(vec![(60, Some(65))], &["red"]);
    let styles = vec![s1, s2, s3];
    let map = get_style_map(line, &styles);
    assert_eq!(
        sorted_regions(&map),
        vec![
            (1, 5),
            (7, 14),
            (15, 20),
            (35, 40),
            (41, 44),
            (45, 50),
            (55, 60),
            (60, 65),
        ]
    );
    assert!(std::ptr::eq(find_region_for(&map, (1, 5)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (7, 14)), &styles[1]));
    assert!(std::ptr::eq(find_region_for(&map, (15, 20)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (35, 40)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (41, 44)), &styles[1]));
    assert!(std::ptr::eq(find_region_for(&map, (45, 50)), &styles[0]));
    assert!(std::ptr::eq(find_region_for(&map, (55, 60)), &styles[1]));
    assert!(std::ptr::eq(find_region_for(&map, (60, 65)), &styles[2]));
}

#[test]
fn test_repeated_invocation_returns_new_list() {
    let r = re("in");
    let r1 = find_regions("string", &r);
    let r2 = find_regions("string", &r);
    assert_eq!(r1, vec![(3, 5)]);
    assert_eq!(r2, vec![(3, 5)]);
}

#[test]
fn test_missing_searchstr_return_empty() {
    // An empty regex matches zero-width positions which Python's finditer
    // would normally return, but the Python guard returns [] early.
    // Our Rust port mirrors the guard.
    let empty = Regex::new("").unwrap();
    assert!(find_regions("some string", &empty).is_empty());
    assert!(find_regions("", &empty).is_empty());
}

#[test]
fn test_no_match() {
    assert!(find_regions("", &re("a")).is_empty());
    assert!(find_regions("", &re("foo")).is_empty());
    assert!(find_regions("some string", &re("foo")).is_empty());
}

#[test]
fn test_simple_cases() {
    assert_eq!(find_regions("this is...", &re("this")), vec![(0, 4)]);
    assert_eq!(find_regions("my string", &re("string")), vec![(3, 9)]);
}

#[test]
fn test_single_char_match() {
    assert_eq!(find_regions("a", &re("a")), vec![(0, 1)]);
    assert_eq!(
        find_regions("aaaaa", &re("a")),
        vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]
    );
    assert_eq!(
        find_regions("axaxa", &re("a")),
        vec![(0, 1), (2, 3), (4, 5)]
    );
    assert_eq!(find_regions("foo", &re("f")), vec![(0, 1)]);
    assert_eq!(find_regions("foo", &re("o")), vec![(1, 2), (2, 3)]);
}

#[test]
fn test_consecutive_matches() {
    assert_eq!(
        find_regions("isisis", &re("is")),
        vec![(0, 2), (2, 4), (4, 6)]
    );
    assert_eq!(
        find_regions("isisisis", &re("is")),
        vec![(0, 2), (2, 4), (4, 6), (6, 8)]
    );
    assert_eq!(find_regions("x isis", &re("is")), vec![(2, 4), (4, 6)]);
    assert_eq!(
        find_regions("this is his list isis", &re("is")),
        vec![(2, 4), (5, 7), (9, 11), (13, 15), (17, 19), (19, 21)]
    );
}

#[test]
fn test_find_regions_with_simple_regex() {
    assert_eq!(
        find_regions("x-11-11", &re(r"\d+")),
        vec![(2, 4), (5, 7)]
    );
    assert_eq!(
        find_regions("01-3456-11-11", &re(r"\d+")),
        vec![(0, 2), (3, 7), (8, 10), (11, 13)]
    );
    assert_eq!(find_regions("0123456789 nums", &re(r"\d+")), vec![(0, 10)]);
    assert_eq!(find_regions("0123456789", &re(r"\d+")), vec![(0, 10)]);
    assert_eq!(
        find_regions("some string", &re(r"\w+")),
        vec![(0, 4), (5, 11)]
    );
    assert_eq!(
        find_regions("some long string", &re("long")),
        vec![(5, 9)]
    );
    assert_eq!(find_regions("foo boo", &re("o+")), vec![(1, 3), (5, 7)]);
    assert_eq!(
        find_regions("foo boo", &re("o")),
        vec![(1, 2), (2, 3), (5, 6), (6, 7)]
    );
    assert_eq!(
        find_regions(" '192.168.99.1'", &re(r"\d+\.\d+\.\d+\.\d+")),
        vec![(2, 14)]
    );
}

// ---- ConfParserTests ----

fn load_conf() -> ConfParser {
    let content = std::fs::read_to_string(format!("{}/test.txts.conf", TEST_DATA_DIR))
        .expect("read test.txts.conf");
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    ConfParser::new(lines)
}

fn assert_regex_style(
    style: &Style,
    pattern: &str,
    transform_keys: &[&str],
    apply_to_whole_line: bool,
) {
    assert_eq!(style.pattern(), Some(pattern), "pattern mismatch");
    let expected = build_expected_transforms(transform_keys);
    assert_eq!(style.transforms(), expected, "transforms mismatch");
    assert_eq!(
        style.apply_to_whole_line(),
        apply_to_whole_line,
        "apply_to_whole_line mismatch for {:?}",
        style
    );
}

fn assert_index_style(
    style: &Style,
    regions: &[(usize, Option<usize>)],
    transform_keys: &[&str],
) {
    assert_eq!(style.regions(), Some(regions));
    let expected = build_expected_transforms(transform_keys);
    assert_eq!(style.transforms(), expected);
}

fn build_expected_transforms(keys: &[&str]) -> String {
    let v: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
    txtstyle::transformer::build_transforms(&v).unwrap()
}

#[test]
fn test_example_style() {
    let styles = load_conf().get_styles("example").unwrap();
    assert_regex_style(&styles[0], r"error", &["red"], true);
    assert_regex_style(&styles[1], r"evil\.org", &["red"], false);
    assert_regex_style(&styles[2], r"\d{4}-\d\d-\d\d", &["green"], false);
    assert_regex_style(&styles[3], r"\d\d:\d\d:\d\d", &["green", "bold"], false);
    assert_regex_style(
        &styles[4],
        r"\d+\.\d+\.\d+\.\d+(:\d+)?",
        &["yellow", "underline"],
        false,
    );
    assert_regex_style(&styles[5], r"\[samplesession\]", &["magenta"], false);
    assert_regex_style(&styles[6], r"\[[^\]]+\]", &["blue"], false);
    assert_regex_style(&styles[7], r"\b\d+\b", &["cyan", "bold"], false);
    assert_eq!(styles.len(), 8);
}

#[test]
fn test_get_first() {
    let styles = load_conf().get_styles("first").unwrap();
    assert_regex_style(&styles[0], r"some error", &["red"], false);
    assert_regex_style(&styles[1], r"\d\d-\d\d-\d\d\d\d", &["blue"], false);
    assert_regex_style(&styles[2], r"some pattern", &["green"], false);
    assert_regex_style(&styles[3], r"\[(xyz.*x+y?z+)\]", &["underline"], false);
    assert_eq!(styles.len(), 4);
}

#[test]
fn test_get_second() {
    let styles = load_conf().get_styles("second").unwrap();
    assert_regex_style(&styles[0], r"\w+", &["blue"], false);
    assert_eq!(styles.len(), 1);
}

#[test]
fn test_get_third() {
    let styles = load_conf().get_styles("third").unwrap();
    assert_regex_style(&styles[0], r":on-red : \d+", &["on-red"], false);
    assert_regex_style(
        &styles[1],
        r"\\:\\[\s+]foo.*(foo).*bar\\\\",
        &["grey"],
        false,
    );
    assert_regex_style(&styles[2], r": double: quotes", &["yellow"], false);
    assert_eq!(styles.len(), 3);
}

#[test]
fn test_get_fourth() {
    let styles = load_conf().get_styles("fourth").unwrap();
    assert!(styles.is_empty());
}

#[test]
fn test_get_fifth() {
    let err = load_conf().get_styles("fifth").unwrap_err();
    assert_eq!(
        err,
        "Invalid style definition: green regex(\"some pattern\")"
    );
}

#[test]
fn test_get_sixth() {
    let err = load_conf().get_styles("sixth").unwrap_err();
    assert_eq!(err, "Invalid style key: \"some-bad-key\"");
}

#[test]
fn test_get_seventh() {
    let styles = load_conf().get_styles("seventh").unwrap();
    assert_regex_style(
        &styles[0],
        r#":.*\d\s\'\""#,
        &["blue", "on-white"],
        false,
    );
    assert_regex_style(&styles[1], r#"\""#, &["125", "on-245"], false);
    assert_eq!(styles.len(), 2);
}

#[test]
fn test_get_eighth() {
    let styles = load_conf().get_styles("eighth").unwrap();
    assert_regex_style(&styles[0], r"org.[\w+|\.]+", &["red"], false);
    assert_eq!(styles.len(), 1);
}

#[test]
fn test_get_ninth() {
    let styles = load_conf().get_styles("ninth").unwrap();
    assert_regex_style(&styles[0], r"error", &["red"], true);
    assert_regex_style(&styles[1], r"another error", &["red", "bold"], true);
    assert_eq!(styles.len(), 2);
}

#[test]
fn test_get_tenth() {
    let err = load_conf().get_styles("tenth").unwrap_err();
    assert_eq!(
        err,
        "Invalid style definition: red: regex(\"bad\") # can't comment here"
    );
}

#[test]
fn test_get_eleventh() {
    let styles = load_conf().get_styles("eleventh").unwrap();
    assert_index_style(&styles[0], &[(0, Some(8))], &["green"]);
    assert_index_style(&styles[1], &[(9, Some(13))], &["160", "bold"]);
    assert_index_style(&styles[2], &[(15, Some(18))], &["215"]);
    assert_index_style(&styles[3], &[(20, Some(24))], &["115"]);
    assert_index_style(&styles[4], &[(26, Some(31))], &["162"]);
    assert_index_style(&styles[5], &[(65, Some(200))], &["48"]);
    assert_eq!(styles.len(), 6);
}

#[test]
fn test_get_twelfth() {
    let styles = load_conf().get_styles("twelfth").unwrap();
    assert_index_style(&styles[0], &[(0, Some(8))], &["18", "on-45"]);
    assert_index_style(
        &styles[1],
        &[(13, Some(18)), (20, Some(22))],
        &["yellow"],
    );
    assert_eq!(styles.len(), 2);
}

#[test]
fn test_get_thirteenth() {
    let err = load_conf().get_styles("thirteenth").unwrap_err();
    assert_eq!(err, "Invalid style definition: blue: index()");
}

#[test]
fn test_get_undefined() {
    let err = load_conf().get_styles("FOO").unwrap_err();
    assert_eq!(err, "Style \"FOO\" is not defined");
}

// ---- TransformerTests ----

fn assert_styled(styles: Vec<Style>, input: &str, expected: &str) {
    let t = Transformer::new(&styles);
    assert_eq!(t.style(input), expected);
}

#[test]
fn test_substring_style() {
    let input = "some text...";
    // <red>some<default> text...<default>
    let expected = "\x1b[31msome\x1b[m text...\x1b[m";
    assert_styled(
        vec![idx_style(vec![(0, Some(4))], &["red"])],
        input,
        expected,
    );
    assert_styled(vec![rx_style("some", &["red"])], input, expected);
}

#[test]
fn test_whole_line_style() {
    let input = "some text...";
    let expected = "\x1b[31msome text...\x1b[m";
    assert_styled(
        vec![rx_style_whole("some", &["red"])],
        input,
        expected,
    );
    assert_styled(
        vec![rx_style("some text...", &["red"])],
        input,
        expected,
    );
    assert_styled(
        vec![idx_style(vec![(0, Some(input.len()))], &["red"])],
        input,
        expected,
    );
    // if end > line length, default to line length
    assert_styled(
        vec![idx_style(vec![(0, Some(99999))], &["red"])],
        input,
        expected,
    );
}

#[test]
fn test_removing_styles_is_equal_to_original_line() {
    // Style a line, then strip escape sequences and compare to original.
    let styles = vec![
        rx_style(r"http:[\w+|/+|:]+", &["red"]),
        rx_style(r"^\w\w\w \d\d\s?", &["white", "on-magenta"]),
        rx_style(r"\d\d:\d\d:\d\d", &["bold", "on-blue"]),
        rx_style(r".*<warn>.*", &["yellow"]),
        rx_style(r"\((.*)\)", &["red", "on-white"]),
        rx_style(r"\[(.*)\]", &["grey", "bold"]),
    ];
    let t = Transformer::new(&styles);
    let content = std::fs::read_to_string(format!("{}/test-log", TEST_DATA_DIR)).unwrap();
    for original in content.lines() {
        let styled = t.style(original);
        let unstyled = strip_ansi(&styled);
        assert_eq!(original, unstyled, "line: {:?}", original);
    }
}

fn strip_ansi(s: &str) -> String {
    // Remove ESC[...m sequences.
    let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
}
