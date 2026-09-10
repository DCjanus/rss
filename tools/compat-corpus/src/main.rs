use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use rss::{Category, Channel, Cloud, Enclosure, Guid, Image, Item, Source, TextInput};
use serde_json::{json, Map, Value};

const SUPPORTED_VERSIONS: &[&str] = &["0.9", "0.90", "0.91", "0.92", "1.0", "2.0"];

struct Args {
    gofeed: PathBuf,
    source_revision: String,
    crate_revision: String,
    report: PathBuf,
}

#[derive(Default)]
struct Summary {
    total: usize,
    excluded: usize,
    passed: usize,
    parse_errors: usize,
    field_mismatches: usize,
    roundtrip_errors: usize,
    roundtrip_mismatches: usize,
}

fn main() -> Result<(), String> {
    let args = parse_args()?;
    let mut fixtures = fixture_paths(&args.gofeed)?;
    fixtures.sort();
    if fixtures.is_empty() {
        return Err(format!(
            "no XML fixtures found in {}",
            args.gofeed.display()
        ));
    }

    let mut summary = Summary::default();
    let mut findings = Vec::new();
    let mut excluded = Vec::new();
    let mut inventory = Vec::new();
    let mut prior_triage = load_prior_triage(&args.report, &args.source_revision)?;

    for xml_path in fixtures {
        summary.total += 1;
        let expected_path = xml_path.with_extension("json");
        let fixture = xml_path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| format!("non-UTF-8 fixture path: {}", xml_path.display()))?;
        let expected: Value = serde_json::from_slice(
            &fs::read(&expected_path)
                .map_err(|error| format!("read {}: {error}", expected_path.display()))?,
        )
        .map_err(|error| format!("decode {}: {error}", expected_path.display()))?;
        let version = expected
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("");

        if !SUPPORTED_VERSIONS.contains(&version) {
            summary.excluded += 1;
            excluded.push(json!({
                "fixture": fixture,
                "version": version,
                "reason": "unsupported or missing RSS version",
            }));
            inventory.push(json!({
                "fixture": fixture,
                "version": version,
                "automatic_status": "excluded",
                "reason": "unsupported or missing RSS version",
            }));
            continue;
        }

        let xml =
            fs::read(&xml_path).map_err(|error| format!("read {}: {error}", xml_path.display()))?;
        let channel = match Channel::read_from(&xml[..]) {
            Ok(channel) => channel,
            Err(error) => {
                summary.parse_errors += 1;
                findings.push(json!({
                    "fixture": fixture,
                    "version": version,
                    "kind": "parse-error",
                    "details": [error.to_string()],
                }));
                inventory.push(candidate_record(
                    fixture,
                    version,
                    &findings[findings.len() - 1..],
                    prior_triage.remove(fixture),
                ));
                continue;
            }
        };

        let finding_start = findings.len();
        let mut mismatches = Vec::new();
        compare_channel(&channel, &expected, &mut mismatches);
        if !mismatches.is_empty() {
            summary.field_mismatches += 1;
            findings.push(json!({
                "fixture": fixture,
                "version": version,
                "kind": "field-mismatch",
                "details": mismatches,
            }));
        }

        let output = match channel.write_to(Vec::new()) {
            Ok(output) => output,
            Err(error) => {
                summary.roundtrip_errors += 1;
                findings.push(json!({
                    "fixture": fixture,
                    "version": version,
                    "kind": "serialize-error",
                    "details": [error.to_string()],
                }));
                inventory.push(candidate_record(
                    fixture,
                    version,
                    &findings[finding_start..],
                    prior_triage.remove(fixture),
                ));
                continue;
            }
        };
        match Channel::read_from(&output[..]) {
            Ok(reparsed) if reparsed == channel => {}
            Ok(_) => {
                summary.roundtrip_mismatches += 1;
                findings.push(json!({
                    "fixture": fixture,
                    "version": version,
                    "kind": "roundtrip-mismatch",
                    "details": ["serialized output parses but changes the Channel value"],
                }));
            }
            Err(error) => {
                summary.roundtrip_errors += 1;
                findings.push(json!({
                    "fixture": fixture,
                    "version": version,
                    "kind": "roundtrip-parse-error",
                    "details": [error.to_string()],
                }));
            }
        }

        if findings.len() == finding_start {
            summary.passed += 1;
            inventory.push(json!({
                "fixture": fixture,
                "version": version,
                "automatic_status": "pass",
            }));
        } else {
            inventory.push(candidate_record(
                fixture,
                version,
                &findings[finding_start..],
                prior_triage.remove(fixture),
            ));
        }
    }

    let report = json!({
        "schema_version": 2,
        "experiment": {
            "crate_revision": args.crate_revision,
            "note": "Internal discovery draft; candidate differences are not confirmed defects",
        },
        "source": {
            "repository": "https://github.com/mmcdole/gofeed",
            "revision": args.source_revision,
            "path": "testdata/parser/rss",
        },
        "supported_versions": SUPPORTED_VERSIONS,
        "summary": {
            "total": summary.total,
            "excluded": summary.excluded,
            "passed": summary.passed,
            "parse_errors": summary.parse_errors,
            "field_mismatches": summary.field_mismatches,
            "roundtrip_errors": summary.roundtrip_errors,
            "roundtrip_mismatches": summary.roundtrip_mismatches,
            "finding_records": findings.len(),
        },
        "findings": findings,
        "excluded_fixtures": excluded,
        "inventory": inventory,
    });

    if let Some(parent) = args.report.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::write(
        &args.report,
        serde_json::to_vec_pretty(&report).map_err(|error| format!("encode report: {error}"))?,
    )
    .map_err(|error| format!("write {}: {error}", args.report.display()))?;

    println!(
        "checked={} excluded={} passed={} parse_errors={} field_mismatches={} roundtrip_errors={} roundtrip_mismatches={}",
        summary.total,
        summary.excluded,
        summary.passed,
        summary.parse_errors,
        summary.field_mismatches,
        summary.roundtrip_errors,
        summary.roundtrip_mismatches,
    );
    Ok(())
}

fn parse_args() -> Result<Args, String> {
    let mut gofeed = None;
    let mut source_revision = None;
    let mut crate_revision = None;
    let mut report = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {arg}"))?;
        match arg.as_str() {
            "--gofeed" => gofeed = Some(PathBuf::from(value)),
            "--source-revision" => source_revision = Some(value),
            "--crate-revision" => crate_revision = Some(value),
            "--report" => report = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    Ok(Args {
        gofeed: gofeed.ok_or("missing --gofeed")?,
        source_revision: source_revision.ok_or("missing --source-revision")?,
        crate_revision: crate_revision.ok_or("missing --crate-revision")?,
        report: report.ok_or("missing --report")?,
    })
}

fn load_prior_triage(
    report: &Path,
    source_revision: &str,
) -> Result<BTreeMap<String, Value>, String> {
    if !report.exists() {
        return Ok(BTreeMap::new());
    }
    let previous: Value = serde_json::from_slice(
        &fs::read(report).map_err(|error| format!("read {}: {error}", report.display()))?,
    )
    .map_err(|error| format!("decode {}: {error}", report.display()))?;
    let previous_revision = previous
        .pointer("/source/revision")
        .and_then(Value::as_str)
        .unwrap_or("");
    if previous_revision != source_revision {
        return Err(format!(
            "{} records source revision {previous_revision}; refusing to reuse triage state for {source_revision}",
            report.display()
        ));
    }

    let mut triage = BTreeMap::new();
    for record in previous
        .get("inventory")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let (Some(fixture), Some(state)) = (
            record.get("fixture").and_then(Value::as_str),
            record.get("triage"),
        ) {
            triage.insert(fixture.to_owned(), state.clone());
        }
    }
    Ok(triage)
}

fn candidate_record(
    fixture: &str,
    version: &str,
    findings: &[Value],
    prior_triage: Option<Value>,
) -> Value {
    json!({
        "fixture": fixture,
        "version": version,
        "automatic_status": "candidate",
        "automatic_findings": findings,
        "triage": prior_triage.unwrap_or_else(|| json!({
            "status": "pending",
            "reason": null,
            "issue": null,
        })),
    })
}

fn fixture_paths(root: &Path) -> Result<Vec<PathBuf>, String> {
    fs::read_dir(root)
        .map_err(|error| format!("read {}: {error}", root.display()))?
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().extension() == Some(OsStr::new("xml")) => {
                Some(Ok(entry.path()))
            }
            Ok(_) => None,
            Err(error) => Some(Err(format!("read {}: {error}", root.display()))),
        })
        .collect()
}

fn compare_channel(channel: &Channel, expected: &Value, out: &mut Vec<String>) {
    compare_str(
        out,
        "channel.title",
        Some(channel.title()),
        expected,
        "title",
    );
    compare_str(out, "channel.link", Some(channel.link()), expected, "link");
    compare_str(
        out,
        "channel.description",
        Some(channel.description()),
        expected,
        "description",
    );
    compare_str(
        out,
        "channel.language",
        channel.language(),
        expected,
        "language",
    );
    compare_str(
        out,
        "channel.copyright",
        channel.copyright(),
        expected,
        "copyright",
    );
    compare_str(
        out,
        "channel.managingEditor",
        channel.managing_editor(),
        expected,
        "managingEditor",
    );
    compare_str(
        out,
        "channel.webMaster",
        channel.webmaster(),
        expected,
        "webMaster",
    );
    compare_str(
        out,
        "channel.pubDate",
        channel.pub_date(),
        expected,
        "pubDate",
    );
    compare_str(
        out,
        "channel.lastBuildDate",
        channel.last_build_date(),
        expected,
        "lastBuildDate",
    );
    compare_str(
        out,
        "channel.generator",
        channel.generator(),
        expected,
        "generator",
    );
    compare_str(out, "channel.docs", channel.docs(), expected, "docs");
    compare_str(out, "channel.ttl", channel.ttl(), expected, "ttl");
    compare_str(out, "channel.rating", channel.rating(), expected, "rating");
    compare_string_array(
        out,
        "channel.skipHours",
        channel.skip_hours(),
        expected,
        "skipHours",
    );
    compare_string_array(
        out,
        "channel.skipDays",
        channel.skip_days(),
        expected,
        "skipDays",
    );
    compare_categories(
        out,
        "channel.categories",
        channel.categories(),
        expected.get("categories"),
    );
    compare_optional_object(
        out,
        "channel.image",
        channel.image(),
        expected.get("image"),
        compare_image,
    );
    compare_optional_object(
        out,
        "channel.cloud",
        channel.cloud(),
        expected.get("cloud"),
        compare_cloud,
    );
    compare_optional_object(
        out,
        "channel.textInput",
        channel.text_input(),
        expected.get("textInput"),
        compare_text_input,
    );

    if let Some(items) = expected.get("items").and_then(Value::as_array) {
        if channel.items().len() != items.len() {
            mismatch(
                out,
                "channel.items.length",
                channel.items().len(),
                items.len(),
            );
        }
        for (index, (actual, expected)) in channel.items().iter().zip(items).enumerate() {
            compare_item(actual, expected, index, out);
        }
    }
}

fn compare_item(item: &Item, expected: &Value, index: usize, out: &mut Vec<String>) {
    let base = format!("channel.items[{index}]");
    compare_str(
        out,
        &format!("{base}.title"),
        item.title(),
        expected,
        "title",
    );
    compare_str(out, &format!("{base}.link"), item.link(), expected, "link");
    compare_str(
        out,
        &format!("{base}.description"),
        item.description(),
        expected,
        "description",
    );
    compare_str(
        out,
        &format!("{base}.content"),
        item.content(),
        expected,
        "content",
    );
    compare_str(
        out,
        &format!("{base}.author"),
        item.author(),
        expected,
        "author",
    );
    compare_str(
        out,
        &format!("{base}.comments"),
        item.comments(),
        expected,
        "comments",
    );
    compare_str(
        out,
        &format!("{base}.pubDate"),
        item.pub_date(),
        expected,
        "pubDate",
    );
    compare_categories(
        out,
        &format!("{base}.categories"),
        item.categories(),
        expected.get("categories"),
    );
    compare_optional_object(
        out,
        &format!("{base}.enclosure"),
        item.enclosure(),
        expected.get("enclosure"),
        compare_enclosure,
    );
    compare_optional_object(
        out,
        &format!("{base}.guid"),
        item.guid(),
        expected.get("guid"),
        compare_guid,
    );
    compare_optional_object(
        out,
        &format!("{base}.source"),
        item.source(),
        expected.get("source"),
        compare_source,
    );
}

fn compare_image(actual: &Image, expected: &Map<String, Value>, base: &str, out: &mut Vec<String>) {
    compare_map_str(
        out,
        &format!("{base}.url"),
        Some(actual.url()),
        expected,
        "url",
    );
    compare_map_str(
        out,
        &format!("{base}.link"),
        Some(actual.link()),
        expected,
        "link",
    );
    compare_map_str(
        out,
        &format!("{base}.title"),
        Some(actual.title()),
        expected,
        "title",
    );
    compare_map_str(
        out,
        &format!("{base}.width"),
        actual.width(),
        expected,
        "width",
    );
    compare_map_str(
        out,
        &format!("{base}.height"),
        actual.height(),
        expected,
        "height",
    );
    compare_map_str(
        out,
        &format!("{base}.description"),
        actual.description(),
        expected,
        "description",
    );
}

fn compare_cloud(actual: &Cloud, expected: &Map<String, Value>, base: &str, out: &mut Vec<String>) {
    compare_map_str(
        out,
        &format!("{base}.domain"),
        Some(actual.domain()),
        expected,
        "domain",
    );
    compare_map_str(
        out,
        &format!("{base}.port"),
        Some(actual.port()),
        expected,
        "port",
    );
    compare_map_str(
        out,
        &format!("{base}.path"),
        Some(actual.path()),
        expected,
        "path",
    );
    compare_map_str(
        out,
        &format!("{base}.registerProcedure"),
        Some(actual.register_procedure()),
        expected,
        "registerProcedure",
    );
    compare_map_str(
        out,
        &format!("{base}.protocol"),
        Some(actual.protocol()),
        expected,
        "protocol",
    );
}

fn compare_text_input(
    actual: &TextInput,
    expected: &Map<String, Value>,
    base: &str,
    out: &mut Vec<String>,
) {
    compare_map_str(
        out,
        &format!("{base}.title"),
        Some(actual.title()),
        expected,
        "title",
    );
    compare_map_str(
        out,
        &format!("{base}.description"),
        Some(actual.description()),
        expected,
        "description",
    );
    compare_map_str(
        out,
        &format!("{base}.name"),
        Some(actual.name()),
        expected,
        "name",
    );
    compare_map_str(
        out,
        &format!("{base}.link"),
        Some(actual.link()),
        expected,
        "link",
    );
}

fn compare_enclosure(
    actual: &Enclosure,
    expected: &Map<String, Value>,
    base: &str,
    out: &mut Vec<String>,
) {
    compare_map_str(
        out,
        &format!("{base}.url"),
        Some(actual.url()),
        expected,
        "url",
    );
    compare_map_str(
        out,
        &format!("{base}.length"),
        Some(actual.length()),
        expected,
        "length",
    );
    compare_map_str(
        out,
        &format!("{base}.type"),
        Some(actual.mime_type()),
        expected,
        "type",
    );
}

fn compare_guid(actual: &Guid, expected: &Map<String, Value>, base: &str, out: &mut Vec<String>) {
    compare_map_str(
        out,
        &format!("{base}.value"),
        Some(actual.value()),
        expected,
        "value",
    );
    let permalink = actual.is_permalink().to_string();
    compare_map_str(
        out,
        &format!("{base}.isPermalink"),
        Some(&permalink),
        expected,
        "isPermalink",
    );
}

fn compare_source(
    actual: &Source,
    expected: &Map<String, Value>,
    base: &str,
    out: &mut Vec<String>,
) {
    compare_map_str(
        out,
        &format!("{base}.title"),
        actual.title(),
        expected,
        "title",
    );
    compare_map_str(
        out,
        &format!("{base}.url"),
        Some(actual.url()),
        expected,
        "url",
    );
}

fn compare_str(
    out: &mut Vec<String>,
    path: &str,
    actual: Option<&str>,
    expected: &Value,
    key: &str,
) {
    if let Some(expected) = expected.get(key).and_then(Value::as_str) {
        let actual = actual.unwrap_or("");
        if actual != expected {
            mismatch(out, path, actual, expected);
        }
    }
}

fn compare_map_str(
    out: &mut Vec<String>,
    path: &str,
    actual: Option<&str>,
    expected: &Map<String, Value>,
    key: &str,
) {
    if let Some(expected) = expected.get(key).and_then(Value::as_str) {
        let actual = actual.unwrap_or("");
        if actual != expected {
            mismatch(out, path, actual, expected);
        }
    }
}

fn compare_string_array(
    out: &mut Vec<String>,
    path: &str,
    actual: &[String],
    expected: &Value,
    key: &str,
) {
    let Some(expected) = expected.get(key).and_then(Value::as_array) else {
        return;
    };
    let expected: Vec<_> = expected.iter().filter_map(Value::as_str).collect();
    let actual: Vec<_> = actual.iter().map(String::as_str).collect();
    if actual != expected {
        mismatch(out, path, format!("{actual:?}"), format!("{expected:?}"));
    }
}

fn compare_categories(
    out: &mut Vec<String>,
    path: &str,
    actual: &[Category],
    expected: Option<&Value>,
) {
    let Some(expected) = expected.and_then(Value::as_array) else {
        return;
    };
    if actual.len() != expected.len() {
        mismatch(out, &format!("{path}.length"), actual.len(), expected.len());
    }
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        compare_str(
            out,
            &format!("{path}[{index}].value"),
            Some(actual.name()),
            expected,
            "value",
        );
        compare_str(
            out,
            &format!("{path}[{index}].domain"),
            actual.domain(),
            expected,
            "domain",
        );
    }
}

fn compare_optional_object<T>(
    out: &mut Vec<String>,
    path: &str,
    actual: Option<&T>,
    expected: Option<&Value>,
    compare: fn(&T, &Map<String, Value>, &str, &mut Vec<String>),
) {
    let Some(expected) = expected.and_then(Value::as_object) else {
        return;
    };
    match actual {
        Some(actual) => compare(actual, expected, path, out),
        None => out.push(format!("{path}: actual=<missing> expected=<object>")),
    }
}

fn mismatch(
    out: &mut Vec<String>,
    path: &str,
    actual: impl std::fmt::Display,
    expected: impl std::fmt::Display,
) {
    out.push(format!("{path}: actual={actual} expected={expected}"));
}
