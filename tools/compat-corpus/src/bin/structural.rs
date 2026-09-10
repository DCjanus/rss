use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use rss::validation::Validate;
use rss::Channel;
use serde_json::{json, Value};

struct Args {
    root: PathBuf,
    includes: Vec<PathBuf>,
    mode: String,
    source_repository: String,
    source_revision: String,
    crate_revision: String,
    report: PathBuf,
}

fn main() -> Result<(), String> {
    let args = parse_args()?;
    let prior_triage = load_prior_triage(&args.report, &args.source_revision)?;
    let mut paths = Vec::new();
    for include in &args.includes {
        collect_xml(&args.root.join(include), &mut paths)?;
    }
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        return Err("no XML fixtures matched the selected paths".to_owned());
    }

    let mut passed = 0;
    let mut candidates = 0;
    let mut parse_errors = 0;
    let mut roundtrip_errors = 0;
    let mut inventory = Vec::new();

    for path in paths {
        let fixture = path
            .strip_prefix(&args.root)
            .map_err(|error| format!("relativize {}: {error}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let xml = fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let text = String::from_utf8_lossy(&xml);
        let description = metadata_field(&text, "Description");
        let expectation = metadata_field(&text, "Expect");
        let mut findings = Vec::new();
        let mut validation = None;

        match Channel::read_from(&xml[..]) {
            Ok(channel) => {
                validation = Some(match channel.validate() {
                    Ok(()) => json!({"status": "ok"}),
                    Err(error) => json!({"status": "error", "details": error.to_string()}),
                });

                if args.mode == "feedvalidator"
                    && expectation
                        .as_deref()
                        .is_some_and(|value| value.starts_with("SAXError"))
                {
                    findings.push(json!({
                        "kind": "accepted-expected-sax-error",
                        "details": "fixture expects the validator XML parser to report SAXError",
                    }));
                }

                match channel.write_to(Vec::new()) {
                    Ok(output) => match Channel::read_from(&output[..]) {
                        Ok(reparsed) if reparsed == channel => {}
                        Ok(reparsed) => {
                            roundtrip_errors += 1;
                            findings.push(json!({
                                "kind": "roundtrip-mismatch",
                                "details": "serialized output parses but changes the Channel value",
                                "before": format!("{channel:#?}"),
                                "after": format!("{reparsed:#?}"),
                                "serialized": String::from_utf8_lossy(&output),
                            }));
                        }
                        Err(error) => {
                            roundtrip_errors += 1;
                            findings.push(json!({
                                "kind": "roundtrip-parse-error",
                                "details": error.to_string(),
                                "serialized": String::from_utf8_lossy(&output),
                            }));
                        }
                    },
                    Err(error) => {
                        roundtrip_errors += 1;
                        findings.push(json!({
                            "kind": "serialize-error",
                            "details": error.to_string(),
                        }));
                    }
                }
            }
            Err(error) => {
                parse_errors += 1;
                let expected_sax_error = args.mode == "feedvalidator"
                    && expectation
                        .as_deref()
                        .is_some_and(|value| value.starts_with("SAXError"));
                if !expected_sax_error {
                    findings.push(json!({
                        "kind": "parse-error",
                        "details": error.to_string(),
                    }));
                }
            }
        }

        if findings.is_empty() {
            passed += 1;
            inventory.push(json!({
                "fixture": fixture,
                "automatic_status": "pass",
                "description": description,
                "source_expectation": expectation,
                "validation": validation,
            }));
        } else {
            candidates += 1;
            inventory.push(json!({
                "fixture": fixture,
                "automatic_status": "candidate",
                "description": description,
                "source_expectation": expectation,
                "validation": validation,
                "automatic_findings": findings,
                "triage": prior_triage.get(&fixture).cloned().unwrap_or_else(|| json!({
                    "status": "pending",
                    "reason": null,
                    "issue": null,
                })),
            }));
        }
    }

    let report = json!({
        "schema_version": 1,
        "experiment": {
            "crate_revision": args.crate_revision,
            "mode": args.mode,
            "note": "Internal discovery draft; candidate differences are not confirmed defects",
        },
        "source": {
            "repository": args.source_repository,
            "revision": args.source_revision,
            "include_paths": args.includes,
        },
        "summary": {
            "total": inventory.len(),
            "passed": passed,
            "candidates": candidates,
            "parse_errors_observed": parse_errors,
            "roundtrip_errors": roundtrip_errors,
        },
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
        "checked={} passed={} candidates={} parse_errors_observed={} roundtrip_errors={}",
        report["summary"]["total"], passed, candidates, parse_errors, roundtrip_errors
    );
    Ok(())
}

fn parse_args() -> Result<Args, String> {
    let mut root = None;
    let mut includes = Vec::new();
    let mut mode = None;
    let mut source_repository = None;
    let mut source_revision = None;
    let mut crate_revision = None;
    let mut report = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {arg}"))?;
        match arg.as_str() {
            "--root" => root = Some(PathBuf::from(value)),
            "--include" => includes.push(PathBuf::from(value)),
            "--mode" => mode = Some(value),
            "--source-repository" => source_repository = Some(value),
            "--source-revision" => source_revision = Some(value),
            "--crate-revision" => crate_revision = Some(value),
            "--report" => report = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }
    if includes.is_empty() {
        return Err("at least one --include is required".to_owned());
    }
    let mode = mode.ok_or("missing --mode")?;
    if !matches!(mode.as_str(), "feedparser-wellformed" | "feedvalidator") {
        return Err(format!("unsupported mode: {mode}"));
    }
    Ok(Args {
        root: root.ok_or("missing --root")?,
        includes,
        mode,
        source_repository: source_repository.ok_or("missing --source-repository")?,
        source_revision: source_revision.ok_or("missing --source-revision")?,
        crate_revision: crate_revision.ok_or("missing --crate-revision")?,
        report: report.ok_or("missing --report")?,
    })
}

fn collect_xml(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension() == Some(OsStr::new("xml")) {
            out.push(path.to_owned());
        }
        return Ok(());
    }
    for entry in fs::read_dir(path).map_err(|error| format!("read {}: {error}", path.display()))? {
        let entry = entry.map_err(|error| format!("read {}: {error}", path.display()))?;
        collect_xml(&entry.path(), out)?;
    }
    Ok(())
}

fn metadata_field(text: &str, name: &str) -> Option<String> {
    let marker = format!("{name}:");
    text.lines()
        .find_map(|line| line.split_once(&marker).map(|(_, value)| value))
        .map(|value| value.trim().trim_end_matches("-->").trim().to_owned())
        .filter(|value| !value.is_empty())
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

    Ok(previous
        .get("inventory")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|record| {
            Some((
                record.get("fixture")?.as_str()?.to_owned(),
                record.get("triage")?.clone(),
            ))
        })
        .collect())
}
