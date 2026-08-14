use std::fs;
use std::io::{BufRead, BufReader};
use std::process::Stdio;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn parse_stream(stdout: &[u8], expected_command: &str) -> Vec<Value> {
    let output = std::str::from_utf8(stdout).expect("stdout should be UTF-8 JSONL");
    assert!(
        !output.is_empty(),
        "successful JSONL stdout should not be empty"
    );
    assert!(
        output.ends_with('\n'),
        "JSONL stream should end each record with a newline"
    );
    let records = output
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("line should parse as JSON"))
        .collect::<Vec<_>>();

    assert!(
        records.len() >= 2,
        "stream should contain begin and end records"
    );
    let mut data = 0usize;
    let mut diagnostics = 0usize;

    for (seq, record) in records.iter().enumerate() {
        assert_eq!(record["seq"], seq, "sequence numbers should be contiguous");
        match record["type"]
            .as_str()
            .expect("record type should be string")
        {
            "begin" => {
                assert_eq!(seq, 0, "begin must be the first record");
                assert_eq!(record["command"], expected_command);
            }
            "data" => {
                assert!(record.get("command").is_none());
                data += 1;
            }
            "diagnostic" => {
                assert!(record.get("command").is_none());
                diagnostics += 1;
            }
            "end" => {
                assert_eq!(seq, records.len() - 1, "end must be the final record");
                assert!(record.get("command").is_none());
                assert_eq!(record["records"]["data"], data);
                assert_eq!(record["records"]["diagnostics"], diagnostics);
            }
            other => panic!("unexpected JSONL record type {other}"),
        }
    }

    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(records.last().unwrap()["type"], "end");
    records
}

fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
    let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
    fs::write(fixture.path(), contents).expect("fixture should write");
    fixture
}

const PROPERTY_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  jsonl-property-test\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! sig $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "#5\n",
    "1!\n",
    "#10\n",
    "0!\n"
);

#[test]
fn change_jsonl_streams_data_with_stable_record_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--jsonl",
        ])
        .output()
        .expect("change --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "change");
    assert!(records[0].get("context").is_none());
    let items = records
        .iter()
        .filter(|record| record["type"] == "data")
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["data"]["time"], "5ns");
    assert_eq!(
        items[0]["data"]["signals"][0],
        json!({"path": "top.clk", "value": "1'h1"})
    );
}

#[test]
fn change_jsonl_reports_truncation_as_diagnostic() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--signals",
            "top.clk,top.data",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--max",
            "1",
            "--jsonl",
        ])
        .output()
        .expect("change --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "change");
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "data")
            .count(),
        1
    );
    assert!(records.iter().any(|record| {
        record["type"] == "diagnostic" && record["diagnostic"]["code"] == "WPK-W0002"
    }));
}

#[test]
fn change_jsonl_reports_empty_result_before_end() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk",
            "--on",
            "negedge clk",
            "--jsonl",
        ])
        .output()
        .expect("empty change --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "change");
    assert_eq!(records[0]["context"]["scope"], "top");
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "data")
            .count(),
        0
    );
    assert!(records.iter().any(|record| {
        record["type"] == "diagnostic" && record["diagnostic"]["code"] == "WPK-W0003"
    }));
}

#[test]
fn extract_generic_jsonl_streams_rows_with_stable_record_order() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--when",
            "1",
            "--payload",
            "data",
            "--max",
            "1",
            "--jsonl",
        ])
        .output()
        .expect("extract generic should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "extract generic");
    assert_eq!(records[0]["context"]["scope"], "top");
    let items = records
        .iter()
        .filter(|record| record["type"] == "data")
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["data"]["source"], "transfer");
    assert_eq!(items[0]["data"]["payload"][0]["path"], "top.data");
    assert_eq!(items[0]["data"]["payload"][0]["relative_path"], "data");
}

#[test]
fn property_jsonl_streams_capture_rows() {
    let fixture = write_fixture(PROPERTY_VCD, ".property-jsonl.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "sig",
            "--capture",
            "switch",
            "--jsonl",
        ])
        .output()
        .expect("property --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "property");
    let kinds = records
        .iter()
        .filter(|record| record["type"] == "data")
        .map(|record| record["data"]["kind"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(kinds, vec!["assert", "deassert"]);
}

#[test]
fn property_jsonl_reports_truncation_as_diagnostic() {
    let fixture = write_fixture(PROPERTY_VCD, ".property-jsonl-truncated.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "sig",
            "--capture",
            "switch",
            "--max",
            "1",
            "--jsonl",
        ])
        .output()
        .expect("property --jsonl should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout, "property");
    assert_eq!(
        records
            .iter()
            .filter(|record| record["type"] == "data")
            .count(),
        1
    );
    assert!(records.iter().any(|record| {
        record["type"] == "diagnostic" && record["diagnostic"]["code"] == "WPK-W0002"
    }));
}

#[test]
fn info_scope_signal_and_value_jsonl_emit_representative_data() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let info = wavepeek_cmd()
        .args(["info", "--waves", fixture.as_str(), "--jsonl"])
        .output()
        .expect("info --jsonl should execute");
    assert!(info.status.success());
    let info_records = parse_stream(&info.stdout, "info");
    assert_eq!(info_records[1]["data"]["time_unit"], "1ns");

    let scope = wavepeek_cmd()
        .args(["scope", "--waves", fixture.as_str(), "--jsonl"])
        .output()
        .expect("scope --jsonl should execute");
    assert!(scope.status.success());
    let scope_records = parse_stream(&scope.stdout, "scope");
    assert!(
        scope_records
            .iter()
            .any(|record| { record["type"] == "data" && record["data"]["path"] == "top" })
    );

    let signal_fixture = fixture_path("signal_recursive_depth.vcd");
    let signal_fixture = signal_fixture.to_string_lossy().into_owned();
    let signal = wavepeek_cmd()
        .args([
            "signal",
            "--waves",
            signal_fixture.as_str(),
            "--scope",
            "top.cpu",
            "--recursive",
            "--jsonl",
        ])
        .output()
        .expect("signal --jsonl should execute");
    assert!(signal.status.success());
    let signal_records = parse_stream(&signal.stdout, "signal");
    assert_eq!(signal_records[0]["context"]["scope"], "top.cpu");
    assert!(signal_records.iter().any(|record| {
        record["type"] == "data"
            && record["data"]["path"] == "top.cpu.core.execute"
            && record["data"]["relative_path"] == "core.execute"
    }));

    let value = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.as_str(),
            "--at",
            "5ns",
            "--signals",
            "top.clk,top.data",
            "--jsonl",
        ])
        .output()
        .expect("value --jsonl should execute");
    assert!(value.status.success());
    let value_records = parse_stream(&value.stdout, "value");
    assert!(value_records[0].get("context").is_none());
    assert_eq!(value_records[1]["data"]["time"], "5ns");
    assert_eq!(
        value_records[1]["data"]["signals"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn json_and_jsonl_flags_conflict_on_waveform_commands() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.clk",
            "--json",
            "--jsonl",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("cannot be used with"))
        .stderr(predicate::str::contains("--json"))
        .stderr(predicate::str::contains("--jsonl"));
}

#[test]
fn helper_commands_do_not_accept_jsonl_output_mode() {
    wavepeek_cmd()
        .args(["skill", "unused", "--jsonl"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("unexpected argument '--jsonl'"));
}

#[test]
fn jsonl_broken_pipe_from_early_consumer_is_silent_success() {
    let mut vcd = String::from(
        "$date\n  today\n$end\n$version\n  jsonl-broken-pipe-test\n$end\n$timescale 1ns $end\n$scope module top $end\n$var wire 1 ! sig $end\n$upscope $end\n$enddefinitions $end\n#0\n0!\n",
    );
    for index in 1..=20_000 {
        vcd.push_str(&format!("#{}\n{}!\n", index, index % 2));
    }
    let fixture = write_fixture(&vcd, ".jsonl-broken-pipe.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let mut child = wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.sig",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--max",
            "unlimited",
            "--jsonl",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("change --jsonl should spawn");

    let stdout = child.stdout.take().expect("child stdout should be piped");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    for _ in 0..4 {
        line.clear();
        let bytes = reader
            .read_line(&mut line)
            .expect("should read early JSONL line");
        assert!(bytes > 0, "expected an early JSONL line");
    }
    drop(reader);

    let output = child.wait_with_output().expect("child should finish");
    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "broken pipe should not print stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn value_jsonl_emits_projected_paths_and_widths() {
    let fixture = fixture_path("m2_core.vcd");
    let output = wavepeek_cmd()
        .args([
            "value",
            "--waves",
            fixture.to_str().unwrap(),
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "data[3:0]",
            "--jsonl",
        ])
        .output()
        .expect("value JSONL should execute");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let records = parse_stream(&output.stdout, "value");
    let data = records
        .iter()
        .find(|record| record["type"] == "data")
        .expect("data record should exist");
    assert_eq!(
        data["data"]["signals"][0],
        json!({"path": "top.data[3:0]", "relative_path": "data[3:0]", "value": "4'hf"})
    );
}
