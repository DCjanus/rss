//! Confirmed compatibility regressions discovered by `tools/compat-corpus`.
//!
//! These tests intentionally remain ignored until the corresponding bug is fixed.

use std::io::Cursor;

use rss::validation::Validate;
use rss::{CategoryBuilder, Channel, CloudBuilder, ImageBuilder, Item, TextInputBuilder};

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/5"]
fn rss1_core_elements_are_matched_by_namespace_uri() {
    let input = r#"<rdf:RDF
        xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
        xmlns:r="http://purl.org/rss/1.0/">
        <r:channel>
            <r:title>Feed</r:title>
            <r:link>https://example.com/</r:link>
            <r:description>Description</r:description>
        </r:channel>
    </rdf:RDF>"#;

    let channel = input
        .parse::<Channel>()
        .expect("RSS 1.0 core elements may use any bound prefix");
    assert_eq!(channel.title(), "Feed");
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/6"]
fn category_domain_accepts_the_rss_specification_example() {
    let category = CategoryBuilder::default()
        .name("1765")
        .domain(Some("Syndic8".to_owned()))
        .build();

    category
        .validate()
        .expect("category domain is an arbitrary taxonomy identifier");
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/7"]
fn cloud_domain_accepts_the_rss_specification_example() {
    let cloud = CloudBuilder::default()
        .domain("rpc.sys.com")
        .port("80")
        .path("/RPC2")
        .register_procedure("pingMe")
        .protocol("soap")
        .build();

    cloud
        .validate()
        .expect("cloud domain is a host name, not an absolute URL");
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/8"]
fn channel_validation_rejects_missing_required_text() {
    let mut channel = Channel::default();
    channel.set_link("https://example.com/");

    assert!(channel.validate().is_err());
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/8"]
fn item_validation_requires_title_or_description() {
    assert!(Item::default().validate().is_err());
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/8"]
fn image_validation_rejects_missing_title() {
    let image = ImageBuilder::default()
        .url("https://example.com/image.png")
        .title("")
        .link("https://example.com/")
        .build();

    assert!(image.validate().is_err());
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/8"]
fn text_input_validation_rejects_missing_required_text() {
    let text_input = TextInputBuilder::default()
        .title("")
        .description("")
        .name("")
        .link("https://example.com/search")
        .build();

    assert!(text_input.validate().is_err());
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/9"]
fn declared_internal_general_entities_are_expanded() {
    let input = r#"<?xml version="1.0"?>
        <!DOCTYPE rss [<!ENTITY project "RSS">]>
        <rss version="2.0"><channel>
            <title>&project;</title>
            <link>https://example.com/</link>
            <description>Description</description>
        </channel></rss>"#;

    let channel = input.parse::<Channel>().unwrap();
    assert_eq!(channel.title(), "RSS");
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/9"]
fn undeclared_general_entities_are_rejected() {
    let input = r#"<rss version="2.0"><channel>
        <title>&notDeclared;</title>
        <link>https://example.com/</link>
        <description>Description</description>
    </channel></rss>"#;

    assert!(input.parse::<Channel>().is_err());
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/10"]
fn serialization_rejects_xml_1_0_illegal_characters() {
    let mut channel = Channel::default();
    channel.set_title("bad\u{8}title");
    channel.set_link("https://example.com/");
    channel.set_description("Description");
    let mut output = Cursor::new(Vec::new());

    assert!(channel.write_to(&mut output).is_err());
}

#[test]
#[ignore = "confirmed bug: https://github.com/DCjanus/rss/issues/11"]
fn nested_dublin_core_content_survives_roundtrip() {
    let input = r#"<rdf:RDF
        xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
        xmlns="http://purl.org/rss/1.0/"
        xmlns:dc="http://purl.org/dc/elements/1.1/"
        xmlns:foaf="http://xmlns.com/foaf/0.1/">
        <channel rdf:about="https://example.com/feed">
            <title>Feed</title>
            <link>https://example.com/</link>
            <description>Description</description>
            <dc:creator><foaf:name>Example Author</foaf:name></dc:creator>
        </channel>
    </rdf:RDF>"#;

    let channel = input.parse::<Channel>().unwrap();
    let mut output = Cursor::new(Vec::new());
    channel.write_to(&mut output).unwrap();
    let output = String::from_utf8(output.into_inner()).unwrap();

    assert!(output.contains("<foaf:name>Example Author</foaf:name>"));
}
