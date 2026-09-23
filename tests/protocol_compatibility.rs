//! Cross-language compatibility tests against canonical Python fixtures.

use dataexcept::EnvelopeNode;

fn parse_fixture(name: &str, content: &str) -> EnvelopeNode {
    EnvelopeNode::from_json(content)
        .unwrap_or_else(|error| panic!("{name} should parse: {error}"))
}

#[test]
fn parses_all_python_reference_fixtures() {
    let fixtures = [
        ("ordinary-exception", include_str!("fixtures/python/ordinary-exception.json")),
        ("explicit-cause", include_str!("fixtures/python/explicit-cause.json")),
        ("failure-metadata", include_str!("fixtures/python/failure-metadata.json")),
        ("implicit-context", include_str!("fixtures/python/implicit-context.json")),
        (
            "nested-exception-group",
            include_str!("fixtures/python/nested-exception-group.json"),
        ),
        ("redaction", include_str!("fixtures/python/redaction.json")),
        ("truncation", include_str!("fixtures/python/truncation.json")),
        ("cycle", include_str!("fixtures/python/cycle.json")),
    ];

    for (name, content) in fixtures {
        let node = parse_fixture(name, content);
        assert!(node.as_exception().is_some(), "{name} root should be an exception");
    }
}

#[test]
fn parses_nested_exception_groups_recursively() {
    let node = parse_fixture(
        "nested-exception-group",
        include_str!("fixtures/python/nested-exception-group.json"),
    );
    let root = node.as_exception().expect("root should be an exception");
    let members = root.exceptions.as_ref().expect("group should have members");

    assert_eq!(members.len(), 2);
    let nested = members[0]
        .as_exception()
        .expect("first member should be nested group");
    assert_eq!(
        nested.exceptions.as_ref().expect("nested group should have members").len(),
        2
    );
}

#[test]
fn parses_truncation_marker_at_end_of_chain() {
    let node = parse_fixture(
        "truncation",
        include_str!("fixtures/python/truncation.json"),
    );
    let level_two = node.as_exception().expect("root should be exception");
    let level_one = level_two
        .cause
        .as_deref()
        .and_then(EnvelopeNode::as_exception)
        .expect("level one should be exception");
    let level_zero = level_one
        .cause
        .as_deref()
        .and_then(EnvelopeNode::as_exception)
        .expect("level zero should be exception");
    let marker = level_zero.cause.as_deref().expect("chain should end in marker");

    assert!(marker.is_truncated());
}

#[test]
fn parses_cycle_marker_at_end_of_chain() {
    let node = parse_fixture(
        "cycle",
        include_str!("fixtures/python/cycle.json"),
    );
    let outer = node.as_exception().expect("root should be exception");
    let inner = outer
        .cause
        .as_deref()
        .and_then(EnvelopeNode::as_exception)
        .expect("inner should be exception");
    let cycle = inner
        .cause
        .as_deref()
        .and_then(EnvelopeNode::as_cycle)
        .expect("chain should end in cycle marker");

    assert_eq!(cycle.error_type, "ValidationError");
    assert_eq!(cycle.message, "outer");
}

#[test]
fn reserializes_reference_fixture_without_losing_protocol_shape() {
    let node = parse_fixture(
        "cycle",
        include_str!("fixtures/python/cycle.json"),
    );
    let value = serde_json::to_value(node).expect("node should serialize");

    assert_eq!(value["cause"]["cause"]["cycle"], true);
}
