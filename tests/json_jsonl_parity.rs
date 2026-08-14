use serde_json::Value;

mod common;
use common::{fixture_path, wavepeek_cmd};

fn assert_json_jsonl_parity(args: &[String]) {
    let run = |mode| {
        let output = wavepeek_cmd()
            .args(args)
            .arg(mode)
            .output()
            .expect("wavepeek should execute");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        output.stdout
    };

    let json: Value = serde_json::from_slice(&run("--json")).expect("valid JSON");
    let stream = String::from_utf8(run("--jsonl")).expect("UTF-8 JSONL");
    assert!(stream.ends_with('\n'));
    let records = stream
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("valid JSONL record"))
        .collect::<Vec<_>>();

    assert_eq!(json["type"], "result");
    assert_eq!(records.first().unwrap()["type"], "begin");
    assert_eq!(records.last().unwrap()["type"], "end");
    for (seq, record) in records.iter().enumerate() {
        assert_eq!(record["seq"], seq);
        if seq == 0 {
            assert_eq!(record["command"], json["command"]);
        } else {
            assert!(record.get("command").is_none());
        }
    }

    assert_eq!(json.get("context"), records[0].get("context"));
    let data = records
        .iter()
        .filter(|record| record["type"] == "data")
        .map(|record| record["data"].clone())
        .collect::<Vec<_>>();
    let diagnostics = records
        .iter()
        .filter(|record| record["type"] == "diagnostic")
        .map(|record| record["diagnostic"].clone())
        .collect::<Vec<_>>();
    assert_eq!(json["data"], Value::Array(data.clone()));
    assert_eq!(json["diagnostics"], Value::Array(diagnostics.clone()));
    assert_eq!(records.last().unwrap()["records"]["data"], data.len());
    assert_eq!(
        records.last().unwrap()["records"]["diagnostics"],
        diagnostics.len()
    );
}

#[test]
fn every_command_has_matching_json_and_jsonl_payloads() {
    let fixture = |name: &str| fixture_path(name).to_string_lossy().into_owned();
    let m2 = fixture("m2_core.vcd");
    let cases = vec![
        vec!["info", "--waves", &m2],
        vec!["scope", "--waves", &m2, "--max", "1"],
        vec!["signal", "--waves", &m2, "--scope", "top"],
        vec![
            "value",
            "--waves",
            &m2,
            "--at",
            "5ns",
            "--signals",
            "top.clk,top.data",
        ],
        vec![
            "change",
            "--waves",
            &m2,
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
            "--row-values",
            "delta",
        ],
        vec![
            "property",
            "--waves",
            &m2,
            "--scope",
            "top",
            "--on",
            "*",
            "--sample-mode",
            "native",
            "--eval",
            "data == 8'h0f",
            "--capture",
            "match",
        ],
        vec![
            "extract",
            "generic",
            "--waves",
            &m2,
            "--scope",
            "top",
            "--on",
            "posedge clk",
            "--when",
            "1",
            "--payload",
            "data",
        ],
    ];
    for args in cases {
        assert_json_jsonl_parity(&args.into_iter().map(String::from).collect::<Vec<_>>());
    }

    let ahb = fixture("extract_ahb_lite.vcd");
    let apb = fixture("extract_apb3.vcd");
    let atb = fixture("extract_atb.vcd");
    let axi = fixture("extract_axi3_w.vcd");
    let axistream = fixture("extract_axistream.vcd");
    let protocol_cases = vec![
        vec![
            "extract",
            "ahb",
            "--waves",
            &ahb,
            "--scope",
            "top",
            "--map",
            "hclk=clk",
            "--include",
            "^ahb_lite_.*",
        ],
        vec![
            "extract",
            "apb",
            "--waves",
            &apb,
            "--scope",
            "top",
            "--profile",
            "apb3",
            "--include",
            "^apb3_",
        ],
        vec![
            "extract",
            "atb",
            "--waves",
            &atb,
            "--scope",
            "top",
            "--include",
            "^trace_",
        ],
        vec![
            "extract",
            "axi",
            "--waves",
            &axi,
            "--scope",
            "top",
            "--profile",
            "axi3",
            "--map",
            "aclk=clk",
            "--include",
            "^axi_w",
        ],
        vec![
            "extract",
            "axistream",
            "--waves",
            &axistream,
            "--scope",
            "top",
            "--map",
            "aclk=clk",
            "--map",
            "aresetn=rst_n",
            "--include",
            "^s_axis_",
        ],
    ];
    for args in protocol_cases {
        assert_json_jsonl_parity(&args.into_iter().map(String::from).collect::<Vec<_>>());
    }
}
