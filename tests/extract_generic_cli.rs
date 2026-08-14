use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).expect("stdout should be valid json")
}

fn write_fixture(contents: &str, suffix: &str) -> NamedTempFile {
    let fixture = NamedTempFile::with_suffix(suffix).expect("temp fixture should create");
    fs::write(fixture.path(), contents).expect("fixture should write");
    fixture
}

fn write_source(contents: &str) -> NamedTempFile {
    let source = NamedTempFile::with_suffix("extract-sources.json").expect("source should create");
    fs::write(source.path(), contents).expect("source should write");
    source
}

fn parse_stream(stdout: &[u8]) -> Vec<Value> {
    let output = std::str::from_utf8(stdout).expect("stdout should be UTF-8 JSONL");
    assert!(output.ends_with('\n'));
    output
        .lines()
        .map(|line| {
            let record: Value = serde_json::from_str(line).expect("JSONL line should parse");
            record
        })
        .collect()
}

const IFF_EVENT_TIME_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  wavepeek-extract-iff-event-time\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" rst_n $end\n",
    "$var wire 1 # valid $end\n",
    "$var wire 8 $ data $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "0\"\n",
    "0#\n",
    "b00000000 $\n",
    "#4\n",
    "1#\n",
    "b10101010 $\n",
    "#5\n",
    "1!\n",
    "1\"\n",
    "#10\n",
    "0!\n"
);

const MULTI_CLOCK_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  wavepeek-extract-multi-clock\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! wclk $end\n",
    "$var wire 1 \" rclk $end\n",
    "$var wire 1 # wvalid $end\n",
    "$var wire 1 $ wready $end\n",
    "$var wire 8 % wdata $end\n",
    "$var wire 1 & rvalid $end\n",
    "$var wire 1 ' rready $end\n",
    "$var wire 8 ( rdata $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "0\"\n",
    "0#\n",
    "1$\n",
    "b00000000 %\n",
    "0&\n",
    "1'\n",
    "b00000000 (\n",
    "#4\n",
    "1#\n",
    "b10101010 %\n",
    "#5\n",
    "1!\n",
    "#6\n",
    "1&\n",
    "b01010101 (\n",
    "#7\n",
    "1\"\n",
    "#10\n",
    "0!\n",
    "0\"\n"
);

const HANDSHAKE_VCD: &str = concat!(
    "$date\n  today\n$end\n",
    "$version\n  wavepeek-extract-generic-test\n$end\n",
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 1 \" rst_n $end\n",
    "$var wire 1 # valid $end\n",
    "$var wire 1 $ ready $end\n",
    "$var wire 8 % data $end\n",
    "$var wire 1 & last $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n",
    "0!\n",
    "1\"\n",
    "0#\n",
    "1$\n",
    "b00000000 %\n",
    "0&\n",
    "#4\n",
    "1#\n",
    "b10101010 %\n",
    "1&\n",
    "#5\n",
    "1!\n",
    "0#\n",
    "b11111111 %\n",
    "0&\n",
    "#10\n",
    "0!\n",
    "#14\n",
    "1#\n",
    "b10101010 %\n",
    "1&\n",
    "#15\n",
    "1!\n",
    "#20\n",
    "0!\n"
);

#[test]
fn extract_generic_json_preserves_repeated_identical_payload_rows() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-handshake.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge clk iff rst_n",
            "--when",
            "valid && ready",
            "--payload",
            "data,last",
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value = parse_json(&output.stdout);
    assert_eq!(value["command"], "extract generic");
    assert_eq!(
        value["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "4ns",
                "source": "transfer",
                "payload": [
                    {"path": "top.data", "relative_path": "data", "value": "8'haa"},
                    {"path": "top.last", "relative_path": "last", "value": "1'h1"}
                ]
            },
            {
                "time": "15ns",
                "sample_time": "14ns",
                "source": "transfer",
                "payload": [
                    {"path": "top.data", "relative_path": "data", "value": "8'haa"},
                    {"path": "top.last", "relative_path": "last", "value": "1'h1"}
                ]
            }
        ])
    );
    assert_eq!(value["diagnostics"], json!([]));
}

#[test]
fn extract_generic_human_uses_relative_or_absolute_payload_paths() {
    let fixture = fixture_path("signal_recursive_depth.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let relative = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top.cpu",
            "--on",
            "posedge valid",
            "--when",
            "1",
            "--payload",
            "top.cpu.valid,core.execute",
        ])
        .output()
        .expect("extract should execute");
    let absolute = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top.cpu",
            "--on",
            "posedge valid",
            "--when",
            "1",
            "--payload",
            "top.cpu.valid,core.execute",
            "--abs",
        ])
        .output()
        .expect("extract should execute");

    assert!(relative.status.success());
    assert!(absolute.status.success());
    assert_eq!(
        String::from_utf8(relative.stdout).expect("stdout should be UTF-8"),
        "@5ns sample@4ns valid=1'h0 core.execute=1'h0\n"
    );
    assert_eq!(
        String::from_utf8(absolute.stdout).expect("stdout should be UTF-8"),
        "@5ns sample@4ns top.cpu.valid=1'h0 top.cpu.core.execute=1'h0\n"
    );
}

#[test]
fn extract_generic_from_bounds_apply_to_event_time_not_sample_time() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-from-boundary.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
            "valid && ready",
            "--payload",
            "data",
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(output.status.success());
    assert_eq!(
        parse_json(&output.stdout)["data"],
        json!([{
            "time": "5ns",
            "sample_time": "4ns",
            "source": "transfer",
            "payload": [{"path": "top.data", "relative_path": "data", "value": "8'haa"}]
        }])
    );
}

#[test]
fn extract_generic_iff_uses_event_time_while_when_uses_sample_time() {
    let fixture = write_fixture(IFF_EVENT_TIME_VCD, "extract-generic-iff.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let iff_output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--on",
            "posedge clk iff rst_n",
            "--when",
            "valid",
            "--payload",
            "data",
            "--json",
        ])
        .output()
        .expect("extract should execute");
    let when_output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--on",
            "posedge clk",
            "--when",
            "rst_n",
            "--payload",
            "data",
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(iff_output.status.success());
    assert_eq!(
        parse_json(&iff_output.stdout)["data"],
        json!([{
            "time": "5ns",
            "sample_time": "4ns",
            "source": "transfer",
            "payload": [{"path": "top.data", "relative_path": "data", "value": "8'haa"}]
        }])
    );
    assert!(when_output.status.success());
    assert_eq!(parse_json(&when_output.stdout)["data"], json!([]));
    assert_eq!(
        parse_json(&when_output.stdout)["diagnostics"][0]["code"],
        "WPK-W0003"
    );
}

#[test]
fn extract_generic_source_file_preserves_declaration_order_in_jsonl() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-source.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();
    let source = write_source(
        r#"{
  "kind": "extract.generic.sources",
  "sources": [
    {"name": "beat.a", "on": "posedge clk", "when": "valid && ready", "payload": ["data"]},
    {"name": "beat.b", "on": "posedge clk", "when": "valid && ready", "payload": ["last"]}
  ]
}
"#,
    );
    let source = source.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            source.as_str(),
            "--from",
            "5ns",
            "--to",
            "5ns",
            "--jsonl",
        ])
        .output()
        .expect("extract should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let records = parse_stream(&output.stdout);
    assert_eq!(records[0]["type"], "begin");
    assert_eq!(records[1]["data"]["source"], "beat.a");
    assert_eq!(records[2]["data"]["source"], "beat.b");
    assert_eq!(records.last().unwrap()["records"]["data"], 2);
}

#[test]
fn extract_generic_source_file_collects_independent_clock_sources() {
    let fixture = write_fixture(MULTI_CLOCK_VCD, "extract-generic-multi-clock.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();
    let source = write_source(
        r#"{
  "kind": "extract.generic.sources",
  "sources": [
    {"name": "write", "on": "posedge wclk", "when": "wvalid && wready", "payload": ["wdata"]},
    {"name": "read", "on": "posedge rclk", "when": "rvalid && rready", "payload": ["rdata"]}
  ]
}
"#,
    );
    let source = source.path().to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            source.as_str(),
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(output.status.success());
    assert_eq!(
        parse_json(&output.stdout)["data"],
        json!([
            {
                "time": "5ns",
                "sample_time": "4ns",
                "source": "write",
                "payload": [{"path": "top.wdata", "relative_path": "wdata", "value": "8'haa"}]
            },
            {
                "time": "7ns",
                "sample_time": "6ns",
                "source": "read",
                "payload": [{"path": "top.rdata", "relative_path": "rdata", "value": "8'h55"}]
            }
        ])
    );
}

#[test]
fn extract_generic_source_file_reports_malformed_literals() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-bad-literal.vcd");

    for (on, when, rendered_source, caret) in [
        (
            "posedge clk iff data == 0x10",
            "valid",
            "source: posedge clk iff data == 0x10",
            "                                ^^^^",
        ),
        (
            "posedge clk",
            "data == 64h10",
            "source: data == 64h10",
            "                ^^^^^",
        ),
    ] {
        let source = write_source(
            serde_json::json!({
                "kind": "extract.generic.sources",
                "sources": [{"name": "bad", "on": on, "when": when, "payload": ["data"]}]
            })
            .to_string()
            .as_str(),
        );

        wavepeek_cmd()
            .args([
                "extract",
                "generic",
                "--waves",
                fixture.path().to_str().unwrap(),
                "--scope",
                "top",
                "--source",
                source.path().to_str().unwrap(),
            ])
            .assert()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(
                predicate::str::starts_with("fatal: expr:")
                    .and(predicate::str::contains("EXPR-PARSE-LOGICAL-LITERAL"))
                    .and(predicate::str::contains(rendered_source))
                    .and(predicate::str::contains(caret)),
            );
    }
}

#[test]
fn extract_generic_reports_limit_diagnostics() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-limit.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    let truncated = wavepeek_cmd()
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
            "valid && ready",
            "--payload",
            "data",
            "--max",
            "1",
            "--json",
        ])
        .output()
        .expect("extract should execute");
    let unlimited = wavepeek_cmd()
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
            "valid && ready",
            "--payload",
            "data",
            "--max",
            "unlimited",
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(truncated.status.success());
    assert_eq!(
        parse_json(&truncated.stdout)["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        parse_json(&truncated.stdout)["diagnostics"][0]["code"],
        "WPK-W0002"
    );
    assert!(unlimited.status.success());
    assert_eq!(
        parse_json(&unlimited.stdout)["diagnostics"][0]["code"],
        "WPK-W0001"
    );
}

#[test]
fn extract_generic_reports_empty_result_diagnostic() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-empty.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

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
            "valid && !ready",
            "--payload",
            "data",
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert_eq!(value["context"]["scope"], "top");
    assert_eq!(value["data"], json!([]));
    assert_eq!(value["diagnostics"][0]["code"], "WPK-W0003");
}

#[test]
fn extract_generic_scope_mixes_relative_and_canonical_names() {
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
            "posedge top.cpu.valid",
            "--when",
            "!mem.ready && !top.mem.ready",
            "--payload",
            "cpu.valid,top.mem.ready",
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value = parse_json(&output.stdout);
    assert_eq!(value["context"]["scope"], "top");
    assert_eq!(value["data"][0]["time"], "5ns");
    assert_eq!(value["data"][0]["payload"][0]["path"], "top.cpu.valid");
    assert_eq!(value["data"][0]["payload"][0]["relative_path"], "cpu.valid");
    assert_eq!(value["data"][0]["payload"][1]["path"], "top.mem.ready");
    assert_eq!(value["data"][0]["payload"][1]["relative_path"], "mem.ready");

    wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--on",
            "posedge cpu.valid",
            "--when",
            "1",
            "--payload",
            "cpu.valid,top.cpu.valid",
        ])
        .assert()
        .success()
        .stdout(predicate::eq(
            "@5ns sample@4ns cpu.valid=1'h0 cpu.valid=1'h0\n",
        ));
}

#[test]
fn extract_generic_unscoped_output_omits_scope_and_relative_path() {
    let fixture = fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.cpu.valid",
            "--when",
            "1",
            "--payload",
            "top.mem.ready",
            "--max",
            "1",
            "--json",
        ])
        .output()
        .expect("extract should execute");

    assert!(output.status.success());
    let value = parse_json(&output.stdout);
    assert!(value.get("context").is_none());
    assert_eq!(value["data"][0]["payload"][0]["path"], "top.mem.ready");
    assert!(
        value["data"][0]["payload"][0]
            .get("relative_path")
            .is_none()
    );
}

#[test]
fn extract_generic_rejects_unsupported_trigger_forms() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-bad-on.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    for trigger in ["*", "valid", "valid or posedge clk"] {
        wavepeek_cmd()
            .args([
                "extract",
                "generic",
                "--waves",
                fixture.as_str(),
                "--scope",
                "top",
                "--on",
                trigger,
                "--when",
                "valid",
                "--payload",
                "data",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(
                "requires --on with only edge event terms",
            ));
    }
}

#[test]
fn extract_generic_rejects_bad_sources_and_paths_outside_scope() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-invalid.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();
    let ambiguous_fixture = fixture_path("change_scope_ambiguous.vcd");
    let ambiguous_fixture = ambiguous_fixture.to_string_lossy().into_owned();
    let duplicate_names = write_source(
        r#"{"kind":"extract.generic.sources","sources":[{"name":"dup","on":"posedge clk","when":"valid","payload":["data"]},{"name":"dup","on":"posedge clk","when":"valid","payload":["last"]}]}"#,
    );
    let empty_sources = write_source(r#"{"kind":"extract.generic.sources","sources":[]}"#);

    wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            duplicate_names.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("duplicate source name 'dup'"));
    wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--scope",
            "top",
            "--source",
            empty_sources.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must contain at least one source"));
    wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            ambiguous_fixture.as_str(),
            "--scope",
            "top.top",
            "--on",
            "negedge clk",
            "--when",
            "1",
            "--payload",
            "top.clk",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("fatal: signal:"))
        .stderr(predicate::str::contains(
            "signal 'top.clk' not found under scope 'top.top'\nclosest query names:\n  clk",
        ));
}

#[test]
fn extract_generic_does_not_suggest_unusable_payload_signals() {
    let fixture = fixture_path("change_property_real_output.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
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
            "clk",
            "--payload",
            "tem",
        ])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "no dumped signal with basename 'tem'",
        ))
        .stderr(predicate::str::contains("closest query names:").not());
}

#[test]
fn extract_generic_rejects_zero_max() {
    let fixture = write_fixture(HANDSHAKE_VCD, "extract-generic-zero-max.vcd");
    let fixture = fixture.path().to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.clk",
            "--when",
            "top.valid",
            "--payload",
            "top.data",
            "--max",
            "0",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--max must be greater than 0"));
}

const FLAT_PROJECTION_VCD: &str = concat!(
    "$timescale 1ns $end\n",
    "$scope module top $end\n",
    "$var wire 1 ! clk $end\n",
    "$var wire 8 \" data $end\n",
    "$upscope $end\n",
    "$enddefinitions $end\n",
    "#0\n0!\nb00000000 \"\n",
    "#4\nb00001111 \"\n",
    "#5\n1!\n",
);

#[test]
fn extract_generic_projects_and_preserves_duplicate_payloads_from_cli_and_source() {
    let fixture = write_fixture(FLAT_PROJECTION_VCD, "extract-flat-projection.vcd");
    let source = write_source(
        r#"{
  "kind": "extract.generic.sources",
  "sources": [
    {"name": "transfer", "on": "posedge clk", "when": "1", "payload": ["data[7:4]", "data[7:4]", "data[5:2]", "data"]}
  ]
}
"#,
    );
    let expected = json!([{
        "time": "5ns",
        "sample_time": "4ns",
        "source": "transfer",
        "payload": [
            {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'h0"},
            {"path": "top.data[7:4]", "relative_path": "data[7:4]", "value": "4'h0"},
            {"path": "top.data[5:2]", "relative_path": "data[5:2]", "value": "4'h3"},
            {"path": "top.data", "relative_path": "data", "value": "8'h0f"},
        ]
    }]);

    let cli = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.path().to_str().unwrap(),
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--when",
            "1",
            "--payload",
            "data[7:4],data[7:4],data[5:2],data",
            "--json",
        ])
        .output()
        .expect("CLI extract should execute");
    assert!(
        cli.status.success(),
        "{}",
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(parse_json(&cli.stdout)["data"], expected);

    let from_source = wavepeek_cmd()
        .args([
            "extract",
            "generic",
            "--waves",
            fixture.path().to_str().unwrap(),
            "--scope",
            "top",
            "--source",
            source.path().to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("source-file extract should execute");
    assert!(
        from_source.status.success(),
        "{}",
        String::from_utf8_lossy(&from_source.stderr)
    );
    assert_eq!(parse_json(&from_source.stdout)["data"], expected);
}
