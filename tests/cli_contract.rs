use std::io;

use assert_cmd::prelude::*;
use predicates::prelude::*;

mod common;
use common::wavepeek_cmd;

const VISIBLE_TOP_LEVEL_COMMANDS: [&str; 9] = [
    "info", "scope", "signal", "value", "change", "property", "extract", "skill", "help",
];

#[cfg(feature = "fsdb")]
const EXPECTED_FSDB_FEATURE_STATUS: &str = "FSDB - enabled";
#[cfg(not(feature = "fsdb"))]
const EXPECTED_FSDB_FEATURE_STATUS: &str = "FSDB - disabled (FSDB support is currently Linux x86_64 only; reinstall with Cargo flag `--features fsdb` and provide the Synopsys Verdi FSDB Reader SDK)";

fn successful_stdout(args: &[&str]) -> Vec<u8> {
    let mut command = wavepeek_cmd();
    let assert = command.args(args).assert().success();
    let output = assert.get_output();
    assert!(
        output.stderr.is_empty(),
        "expected empty stderr for args {:?}, got: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout.clone()
}

fn successful_stdout_text(args: &[&str]) -> String {
    String::from_utf8(successful_stdout(args)).expect("stdout should be UTF-8")
}

fn assert_same_stdout(left_args: &[&str], right_args: &[&str], label: &str) {
    let left = successful_stdout(left_args);
    let right = successful_stdout(right_args);

    assert_eq!(left, right, "{label}");
}

fn command_names_from_top_level_help(help: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_commands = false;

    for line in help.lines() {
        let trimmed_line = line.trim();
        if matches!(
            trimmed_line,
            "Commands:" | "Waveform commands:" | "Helper commands:"
        ) {
            in_commands = true;
            continue;
        }

        if !in_commands {
            continue;
        }

        if trimmed_line == "Options:" {
            break;
        }

        if trimmed_line.is_empty() {
            continue;
        }

        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }

        let leading_spaces = line.len() - trimmed.len();
        if leading_spaces != 2 {
            continue;
        }

        if let Some(name) = trimmed.split_whitespace().next() {
            names.push(name.to_string());
        }
    }

    names
}

fn top_level_help_command_names() -> Vec<String> {
    let help = successful_stdout_text(&["--help"]);
    command_names_from_top_level_help(&help)
}

fn assert_legacy_subcommand_rejected(legacy_name: &str) {
    let mut command = wavepeek_cmd();

    command
        .arg(legacy_name)
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(format!(
            "unrecognized subcommand '{legacy_name}'"
        )))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}

fn assert_human_flag_rejected(args: &[&str], command_name: &str) {
    let mut command = wavepeek_cmd();

    command
        .args(args)
        .assert()
        .failure()
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--human'"))
        .stderr(predicate::str::contains(format!(
            "See 'wavepeek {command_name} --help'."
        )));
}

#[test]
fn closed_stdout_is_silent_success() {
    for args in [
        &["skill", "--help"][..],
        &["--help"][..],
        &["-V"][..],
        &["--version"][..],
    ] {
        let (reader, writer) = io::pipe().expect("pipe should open");
        drop(reader);

        let output = wavepeek_cmd()
            .args(args)
            .stdout(writer)
            .output()
            .expect("wavepeek should run");

        assert!(
            output.status.success(),
            "expected success for args {args:?}, got {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stderr.is_empty(),
            "expected empty stderr for args {args:?}, got: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn no_args_prints_top_level_help_and_exits_zero() {
    let mut command = wavepeek_cmd();

    command
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wavepeek is a machine-friendly command-line tool for RTL waveform inspection.",
        ))
        .stdout(predicate::str::contains("Usage: wavepeek"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn help_lists_expected_subcommands() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wavepeek is a machine-friendly command-line tool for RTL waveform inspection.",
        ))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("scope"))
        .stdout(predicate::str::contains("\n  modules\n").not())
        .stdout(predicate::str::contains("\n  tree\n").not())
        .stdout(predicate::str::contains("signal"))
        .stdout(predicate::str::contains("\n  signals\n").not())
        .stdout(predicate::str::contains("value"))
        .stdout(predicate::str::contains("change"))
        .stdout(predicate::str::contains("\n  changes\n").not())
        .stdout(predicate::str::contains("property"))
        .stdout(predicate::str::contains("extract"))
        .stdout(predicate::str::contains("skill"))
        .stdout(predicate::str::contains("\n  help"));
}

#[test]
fn top_level_help_documents_general_conventions() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("General conventions:"))
        .stdout(predicate::str::contains(
            "Waveform-inspection commands require `--waves <FILE>`.",
        ))
        .stdout(predicate::str::contains(
            "VCD/FST input is available in every build.",
        ))
        .stdout(predicate::str::contains(
            "FSDB support is currently Linux x86_64 only and requires a build compiled with Cargo feature `fsdb` and the Synopsys Verdi FSDB Reader SDK.",
        ))
        .stdout(predicate::str::contains("Optional features:"))
        .stdout(predicate::str::contains(EXPECTED_FSDB_FEATURE_STATUS))
        .stdout(
            predicate::str::contains(
                "Waveform-inspection commands keep their primary inputs as named flags",
            )
            .not(),
        )
        .stdout(predicate::str::contains("Output is bounded by default"))
        .stdout(predicate::str::contains("Default output is human-readable"))
        .stdout(predicate::str::contains(
            "Time values require explicit units",
        ))
        .stdout(predicate::str::contains(
            "time-window flags (`--from`, `--to`) use inclusive boundaries",
        ))
        .stdout(predicate::str::contains(
            "Process-level failures follow `fatal: <category>: <message>`",
        ));
}

#[test]
fn top_level_help_describes_shipped_subcommands_without_unimplemented_markers() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Get signal values at explicit time point(s)").and(
                predicate::str::contains(
                    "Get signal values at explicit time point(s) (not implemented yet)",
                )
                .not(),
            ),
        )
        .stdout(predicate::str::contains(
            "List signal changes over a time range",
        ))
        .stdout(
            predicate::str::contains("List signal changes over a time range (not implemented yet)")
                .not(),
        )
        .stdout(predicate::str::contains(
            "Evaluate properties over a time range",
        ))
        .stdout(
            predicate::str::contains("Evaluate properties over a time range (not implemented yet)")
                .not(),
        );
}

#[test]
fn top_level_short_help_is_compact_and_points_to_next_layers() {
    let short_help = successful_stdout_text(&["-h"]);
    let long_help = successful_stdout_text(&["--help"]);

    assert!(short_help.contains("Usage: wavepeek"));
    assert!(short_help.contains("wavepeek --help"));
    assert!(short_help.contains("wavepeek help <command-path...>"));
    assert!(short_help.contains("wavepeek skill"));
    assert!(
        !short_help.contains("General conventions:"),
        "top-level short help should stay compact and omit long-form conventions"
    );
    assert!(
        !short_help.contains("Optional features:"),
        "top-level short help should stay compact and omit build feature status"
    );
    assert!(
        short_help.len() < long_help.len(),
        "top-level short help should be materially shorter than long help"
    );
}

#[test]
fn no_args_help_matches_short_help_output() {
    let no_args = successful_stdout(&[]);
    let short_help = successful_stdout(&["-h"]);

    assert_eq!(
        no_args, short_help,
        "wavepeek (no args) output must match wavepeek -h byte-for-byte"
    );
}

#[test]
fn top_level_long_help_describes_help_and_skill_entrypoints() {
    let long_help = successful_stdout_text(&["--help"]);

    assert!(long_help.contains("General conventions:"));
    assert!(long_help.contains("wavepeek help <command-path...>"));
    assert!(long_help.contains("wavepeek skill"));
    assert!(long_help.contains("Next steps:"));
}

#[test]
fn help_subcommand_matches_top_level_long_help() {
    assert_same_stdout(
        &["help"],
        &["--help"],
        "wavepeek help output must match wavepeek --help byte-for-byte",
    );
}

#[test]
fn help_subcommand_aliases_nested_long_help() {
    assert_same_stdout(
        &["help", "change"],
        &["change", "--help"],
        "wavepeek help change must match wavepeek change --help byte-for-byte",
    );
    assert_same_stdout(
        &["help", "skill"],
        &["skill", "--help"],
        "wavepeek help skill must match wavepeek skill --help byte-for-byte",
    );
}

#[test]
fn shipped_commands_list_matches_top_level_help_surface() {
    let expected: Vec<String> = VISIBLE_TOP_LEVEL_COMMANDS
        .iter()
        .map(|command_name| command_name.to_string())
        .collect();
    let actual = top_level_help_command_names();

    assert_eq!(
        actual, expected,
        "top-level help command list changed; update VISIBLE_TOP_LEVEL_COMMANDS and help contract tests"
    );
}

#[test]
fn command_name_parser_ignores_wrapped_description_lines() {
    let help = "Usage: wavepeek <COMMAND>\n\nCommands:\n  info     Show dump metadata (time unit and bounds)\n           wrapped continuation text\n  scope    List hierarchy scopes (deterministic DFS)\n\nOptions:\n  -h, --help  Print help\n";

    assert_eq!(
        command_names_from_top_level_help(help),
        vec!["info", "scope"]
    );
}

#[test]
fn waveform_help_avoids_inline_envelope_or_parse_hints() {
    for command_name in ["info", "scope", "signal", "value", "change", "property"] {
        let long_help = successful_stdout_text(&[command_name, "--help"]);

        assert!(
            !long_help.contains("`data`"),
            "help for {command_name} should not inline JSON envelope field names"
        );
        assert!(
            !long_help.contains("`diagnostics`"),
            "help for {command_name} should not inline JSON envelope field names"
        );
        assert!(
            !long_help.contains("See 'wavepeek "),
            "help for {command_name} should not include repetitive parse-hint boilerplate"
        );
    }
}

#[test]
fn waveform_help_avoids_literal_error_or_warning_message_bodies() {
    for command_name in ["info", "scope", "signal", "value", "change", "property"] {
        let long_help = successful_stdout_text(&[command_name, "--help"]);

        assert!(
            !long_help.contains("error: "),
            "help for {command_name} should avoid inlining literal runtime/parser error prefixes"
        );
        assert!(
            !long_help.contains("warning: "),
            "help for {command_name} should avoid inlining literal warning-message prefixes"
        );
        assert!(
            !long_help.contains("no signal changes found in selected time range"),
            "help for {command_name} should avoid inlining concrete warning message bodies"
        );
        assert!(
            !long_help.contains("limit disabled:"),
            "help for {command_name} should avoid inlining concrete warning message bodies"
        );
    }
}

#[test]
fn change_help_is_layered_without_examples_or_see_also() {
    let short_help = successful_stdout_text(&["change", "-h"]);
    let long_help = successful_stdout_text(&["change", "--help"]);

    assert!(short_help.contains("Usage: wavepeek change"));
    assert!(!short_help.contains("Examples:"));
    assert!(!short_help.contains("See also:"));
    assert!(!long_help.contains("Examples:"));
    assert!(!long_help.contains("See also:"));
    assert!(
        short_help.len() < long_help.len(),
        "change -h should be materially shorter than change --help"
    );
}

#[test]
fn change_help_uses_aligned_summary_behavior_and_grouped_option_docs() {
    let short_help = successful_stdout_text(&["change", "-h"]);
    let long_help = successful_stdout_text(&["change", "--help"]);
    let alias_help = successful_stdout_text(&["help", "change"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides event-driven tables for selected signals.")
        );
    }

    for fragment in [
        "Prints requested signal values for each event selected by required `--on`",
        "`--row-mode dense|sparse` controls whether every sampled event or only changed samples become rows",
        "`--row-values full|delta` controls whether rows contain all requested signals or only changed signals",
        "Range boundaries are inclusive",
        "`--signals` accepts trailing static `[msb:lsb]` projections",
        "use `[n:n]` for one bit because `[n]` remains ordinary waveform path syntax",
        "wildcard comparisons use the projected values",
    ] {
        assert!(
            long_help.contains(fragment),
            "change long help should contain `{fragment}`"
        );
    }
    assert!(!long_help.contains("omitted `--on` behaves as wildcard"));
    assert!(!long_help.contains("Event triggers may use typed `iff` logical expressions."));

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST/FSDB waveform file"));
        assert!(
            help.contains("Start of inclusive time range (e.g. 1234ns; omitted means dump start)")
        );
        assert!(help.contains("End of inclusive time range (e.g. 1234ns; omitted means dump end)"));
        assert!(help.contains(
            "Canonical scope path for relative or in-scope canonical signal and trigger names"
        ));
        assert!(help.contains("Comma-separated signal paths or flat [msb:lsb] projections"));
        assert!(
            help.contains("Select whether every sampled event or only changed samples become rows")
        );
        assert!(help.contains("[default: dense]"));
        assert!(help.contains("[possible values: dense, sparse]"));
        assert!(help.contains("Select whether rows contain all values or only changed values"));
        assert!(help.contains("[default: full]"));
        assert!(help.contains("[possible values: full, delta]"));
        assert!(help.contains(
            "Maximum number of snapshot rows (`unlimited` disables truncation, value must be > 0)"
        ));
        assert!(help.contains("Print canonical paths"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("(default: 50,"));
        assert!(!help.contains("human output"));
    }
}

#[test]
fn property_help_uses_aligned_summary_behavior_and_grouped_option_docs() {
    let short_help = successful_stdout_text(&["property", "-h"]);
    let long_help = successful_stdout_text(&["property", "--help"]);
    let alias_help = successful_stdout_text(&["help", "property"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides timestamps where the specified property holds over event triggers.")
        );
    }

    for fragment in [
        "Evaluates `--eval` at timestamps selected by `--on`",
        "Level capture (`--capture match`) reports a match at every selected timestamp",
        "Edge capture (`--capture switch`, `assert`, or `deassert`) reports transitions: no match to match, or match to no match.",
        "Empty-result, truncation, and explicitly disabled-limit conditions emit coded diagnostics.",
        "Remotely similar to a SystemVerilog assert, but without temporal expressions.",
    ] {
        assert!(
            long_help.contains(fragment),
            "property long help should contain `{fragment}`"
        );
    }
    assert!(!long_help.contains("Omitted `--on` behaves as wildcard"));
    assert!(!long_help.contains("See also:"));

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST/FSDB waveform file"));
        assert!(
            help.contains("Start of inclusive time range (e.g. 1234ns; omitted means dump start)")
        );
        assert!(help.contains("End of inclusive time range (e.g. 1234ns; omitted means dump end)"));
        assert!(help.contains(
            "Canonical scope path for relative or in-scope canonical event and expression names"
        ));
        assert!(help.contains("Logical expression evaluated at selected event timestamps"));
        assert!(
            help.contains("Capture mode: level (`match`) or edge (`switch`, `assert`, `deassert`)")
        );
        assert!(help.contains(
            "Maximum number of property rows (`unlimited` disables truncation, value must be > 0)"
        ));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("(`--eval` is required)"));
        assert!(!help.contains("Capture mode (`match`, `switch`, `assert`, `deassert`)"));
        assert!(!help.contains("Capture mode: match, switch, assert, or deassert"));
    }
}

#[test]
fn extract_command_without_subcommand_prints_help() {
    let no_args = successful_stdout_text(&["extract"]);
    let short_help = successful_stdout_text(&["extract", "-h"]);
    let long_help = successful_stdout_text(&["extract", "--help"]);
    let alias_help = successful_stdout_text(&["help", "extract"]);

    for help in [&no_args, &short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Extract row-oriented waveform data.")
        );
        assert!(help.contains("Usage: wavepeek extract"));
        assert!(help.contains("Commands:"));
        assert!(help.contains("axi"));
        assert!(help.contains("axistream"));
        assert!(help.contains("generic"));
        assert!(!help.contains("fatal: args:"));
    }

    assert_eq!(
        no_args, short_help,
        "wavepeek extract should show short help"
    );
    assert!(
        short_help.len() < long_help.len(),
        "extract -h should be materially shorter than extract --help"
    );
}

#[test]
fn info_help_uses_aligned_summary_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["info", "-h"]);
    let long_help = successful_stdout_text(&["info", "--help"]);
    let alias_help = successful_stdout_text(&["help", "info"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Reports metadata for the selected waveform dump.")
        );
    }

    assert!(long_help.contains("Behavior:\n- Prints available metadata (e.g. time unit, start/end times, etc.) in free form\n- `--json` emits the standard machine-readable envelope."));
    assert!(!short_help.contains("See also:"));
    for help in [&long_help, &alias_help] {
        assert!(!help.contains("See also:"));
    }
    for help in [&short_help, &long_help, &alias_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST/FSDB waveform file"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
    }
}

#[test]
fn scope_help_uses_aligned_summary_behavior_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["scope", "-h"]);
    let long_help = successful_stdout_text(&["scope", "--help"]);
    let alias_help = successful_stdout_text(&["help", "scope"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides deterministic hierarchy traversal over scope paths.")
        );
    }

    assert!(long_help.contains(
        "Behavior:\n- Finds all scopes matching `--filter` and displays scope name, depth, and kind."
    ));
    assert!(long_help.contains(
        "Includes stable scope kind aliases from hierarchy data (not only modules); excluded backend-specific spellings are normalized to the stable contract surface."
    ));
    assert!(!long_help.contains("Includes parser-native scope kinds"));
    assert!(!short_help.contains("See also:"));
    for help in [&long_help, &alias_help] {
        assert!(!help.contains("See also:"));
    }
    assert!(!long_help.contains("human output"));

    for help in [&short_help, &long_help, &alias_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST/FSDB waveform file"));
        assert!(help.contains(
            "Maximum number of entries (`unlimited` disables truncation, value must be > 0)"
        ));
        assert!(help.contains("Maximum traversal depth (`unlimited` disables depth truncation)"));
        assert!(help.contains("Regex filter for full scope path"));
        assert!(help.contains("Render hierarchy as an indented tree"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("(default:"));
        assert!(!help.contains("invalid regex is rejected as an argument error"));
    }
}

#[test]
fn signal_help_uses_aligned_summary_behavior_and_simple_option_docs() {
    let short_help = successful_stdout_text(&["signal", "-h"]);
    let long_help = successful_stdout_text(&["signal", "--help"]);
    let alias_help = successful_stdout_text(&["help", "signal"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides scope-local signal listings.")
        );
    }

    assert!(long_help.contains(
        "Behavior:\n- Finds all signals matching `--filter` within the selected scope and displays name, kind, and available metadata (for example width)."
    ));
    assert!(long_help.contains("Recursive mode walks child scopes depth-first in stable lexicographic order; `--max-depth` limits recursion when set."));
    assert!(long_help.contains(
        "Includes stable signal kind aliases (not only wires); excluded backend-specific VHDL spellings are normalized to the stable contract surface."
    ));
    assert!(!long_help.contains("human output"));

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST/FSDB waveform file"));
        assert!(help.contains("Exact scope path (e.g. top.cpu)"));
        assert!(help.contains(
            "Maximum number of entries (`unlimited` disables truncation, value must be > 0)"
        ));
        assert!(help.contains("Regex filter for signal name"));
        assert!(
            help.contains(
                "Maximum recursion depth below --scope (`unlimited` disables this limit)"
            )
        );
        assert!(help.contains("[default: 5]"));
        assert!(help.contains("Show canonical signal paths"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("`--scope` is required"));
        assert!(!help.contains("(default:"));
        assert!(!help.contains("invalid regex is rejected as an argument error"));
    }
}

#[test]
fn value_help_uses_aligned_summary_behavior_and_grouped_option_docs() {
    let short_help = successful_stdout_text(&["value", "-h"]);
    let long_help = successful_stdout_text(&["value", "--help"]);
    let alias_help = successful_stdout_text(&["help", "value"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Provides point sampling for selected signals.")
        );
    }

    for fragment in [
        "Prints values for the requested signals at each selected time point.",
        "`--at` accepts one explicit time token or a comma-separated list in one argument.",
        "Output preserves the input order from `--at` and `--signals`, including duplicates.",
        "By default, signal names are top-related canonical paths",
        "set `--scope` once with a canonical scope path",
        "Relative and canonical references inside the selected scope may be mixed",
        "A trailing static `[msb:lsb]` projects a flat integral signal's normalized sampled value",
        "`[n]` remains ordinary waveform path syntax",
        "more precise than dump resolution",
    ] {
        assert!(
            long_help.contains(fragment),
            "value long help should contain `{fragment}`"
        );
    }

    for help in [&short_help, &long_help] {
        assert!(help.contains("Input options:"));
        assert!(help.contains("Selection options:"));
        assert!(help.contains("Output options:"));
        assert!(help.contains("Other options:"));
        assert!(help.contains("Path to VCD/FST/FSDB waveform file"));
        assert!(help.contains("Time point(s) with explicit units (e.g. 1337ns or 10ns,20ns)"));
        assert!(
            help.contains("Canonical scope path for relative or in-scope canonical signal names")
        );
        assert!(help.contains("Comma-separated signal paths or flat [msb:lsb] projections"));
        assert!(help.contains("Show canonical signal paths"));
        assert!(help.contains("Machine-readable JSON output"));
        assert!(!help.contains("(`--waves <FILE>` is required)"));
        assert!(!help.contains("for example"));
        assert!(!help.contains("bare numbers are rejected as argument errors"));
        assert!(!help.contains("human output"));
    }
}

#[test]
fn skill_help_surfaces_are_aligned_and_trimmed() {
    let short_help = successful_stdout_text(&["skill", "-h"]);
    let long_help = successful_stdout_text(&["skill", "--help"]);
    let alias_help = successful_stdout_text(&["help", "skill"]);

    for help in [&short_help, &long_help, &alias_help] {
        assert_eq!(
            help.lines().next(),
            Some("Extract the packaged agent skill into a directory.")
        );
        assert!(!help.contains("Behavior:"));
        assert!(!help.contains("--json"));
    }
    assert_eq!(long_help, alias_help);
}

#[test]
fn shipped_commands_help_is_self_descriptive() {
    let command_contracts: [(&str, &[&str]); 7] = [
        (
            "info",
            &[
                "Reports metadata for the selected waveform dump.",
                "Prints available metadata",
                "time unit, start/end times",
            ],
        ),
        (
            "scope",
            &[
                "deterministic hierarchy traversal",
                "pre-order depth-first",
                "lexicographic child ordering",
                "Truncation and disabled-limit conditions emit coded diagnostics",
            ],
        ),
        (
            "signal",
            &[
                "Provides scope-local signal listings.",
                "Finds all signals matching `--filter`",
                "depth-first in stable lexicographic order",
                "Includes stable signal kind aliases",
            ],
        ),
        (
            "value",
            &[
                "point sampling",
                "input order from `--at` and `--signals`",
                "display=value",
                "align to dump precision",
                "Verilog literals",
            ],
        ),
        (
            "change",
            &[
                "event-driven tables",
                "Prints requested signal values for each event selected by required `--on`",
                "--row-mode dense|sparse",
                "--row-values full|delta",
                "Empty-result, truncation, and explicitly disabled-limit conditions emit coded diagnostics",
            ],
        ),
        (
            "property",
            &[
                "specified property holds over event triggers",
                "Evaluates `--eval` at timestamps selected by `--on`",
                "Level capture (`--capture match`) reports a match",
                "Edge capture (`--capture switch`, `assert`, or `deassert`) reports transitions",
            ],
        ),
        (
            "skill",
            &["Extract the packaged agent skill into a directory."],
        ),
    ];

    for (command_name, fragments) in command_contracts {
        let long_help = successful_stdout_text(&[command_name, "--help"]);
        for fragment in fragments {
            assert!(
                long_help.contains(fragment),
                "help for {command_name} must include self-descriptive fragment `{fragment}`"
            );
        }
    }
}

#[test]
fn extract_ahb_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "ahb", "--help"]);
    for fragment in [
        "Extract manager-facing AHB pipeline events.",
        "Supports AHB-Lite and AHB5 profiles from Arm IHI 0033C, Issue C.",
        "Tracks one accepted address phase so real data completions remain distinct from idle clocks.",
        "--include-stall, --include-idle, and --include-busy independently expose cycle-level events.",
        "Uses manager-facing HREADY; HREADYOUT, HSELx, and parity/check signals are outside this interface.",
        "Signal mapping combines explicit STD_NAME=WAVES_NAME maps with include-regex auto-mapping; explicit maps win.",
        "[default: ahb-lite]",
        "[possible values: ahb-lite, ahb5]",
        "Source-file fields and behavior are documented in the corresponding protocol topic.",
        "JSON output includes Issue C context, initial pipeline state, mappings, and ordered event rows.",
        "Does not reconstruct bursts, aggregate transactions, or join address and data phases.",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract ahb long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_apb_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "apb", "--help"]);
    for fragment in [
        "Extract APB Setup and Access event rows.",
        "Supports APB3, APB4, and APB5 profiles from Arm IHI 0024E; APB4 is the default.",
        "Emits setup and access-complete rows by default; --include-wait adds one access-wait row per waited Access cycle.",
        "Mapped PREADY mode requires pready; implicit-high mode forbids pready and wait capture.",
        "Signal mapping combines explicit STD_NAME=WAVES_NAME maps with include-regex auto-mapping; explicit maps win.",
        "Maps one concrete Completer PSELx as canonical psel.",
        "Source files accept canonical lowercase profile and PREADY-mode values only.",
        "[default: apb4]",
        "[possible values: apb3, apb4, apb5]",
        "[default: mapped]",
        "[possible values: mapped, implicit-high]",
        "In source-file mode, --source provides profile, PREADY mode, wait capture, name, includes, and maps",
        "Source-file fields and behavior are documented in the corresponding protocol topic.",
        "JSON output includes APB metadata, mappings, and event rows.",
        "Reports independent sampled events only; it does not correlate or validate transactions.",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract apb long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_atb_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "atb", "--help"]);
    for fragment in [
        "Extract ATB transfer, flush, and synchronization-request events.",
        "Supports ATB-A, ATB-B, and ATB-C profiles from Arm IHI 0032C Issue C; ATB-C is the default.",
        "Profile aliases are atb_a, atb_b, atb_c, atbv1.0, and atbv1.1; source files accept canonical hyphenated profile names only.",
        "Builds independent sources for complete ATVALID/ATREADY and AFVALID/AFREADY handshakes.",
        "Mapping SYNCREQ on ATB-B or ATB-C automatically adds a synchronization-request source.",
        "Emits same-edge events in transfer, flush, then sync-request order.",
        "Preserves raw mapped ATBYTES, ATDATA, and ATID values without trace decoding.",
        "[default: atb-c]",
        "[possible values: atb-a, atb-b, atb-c]",
        "Source-file fields and behavior are documented in the corresponding protocol topic.",
        "Reports stateless sampled events only; it does not reconstruct packets, stalls, flush episodes, or synchronization episodes.",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract atb long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_axi_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "axi", "--help"]);
    for fragment in [
        "Extract AXI ready/valid transfer rows.",
        "AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 profiles use Arm IHI 0022H.c.",
        "AXI5, AXI5-Lite, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP profiles use Arm IHI 0022L ready/valid transport.",
        "Supports AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP profiles.",
        "ACE5-Lite aliases are ace5_lite; ACE5-LiteDVM aliases are ace5-litedvm, ace5_litedvm, and ace5_lite_dvm; ACE5-LiteACP aliases are ace5-liteacp, ace5_liteacp, and ace5_lite_acp.",
        "Source files accept canonical hyphenated profile names only.",
        "Signal mapping combines explicit STD_NAME=WAVES_NAME maps with include-regex auto-mapping; explicit maps win.",
        "[default: axi4]",
        "[possible values: axi3, axi4, axi4-lite, axi5, axi5-lite, ace, ace-lite, ace5, ace5-lite, ace5-lite-dvm, ace5-lite-acp]",
        "Builds one extraction source per complete ready/valid channel.",
        "AXI5 and ACE5-LiteDVM can add DVM ac and cr channels but do not add cd.",
        "In source-file mode, --source provides profile, name, includes, and maps",
        "Source-file fields and behavior are documented in the corresponding protocol topic.",
        "JSON output includes AXI metadata, mappings, and transfer rows.",
        "Reports channel transfers only; it does not reconstruct bursts, ordering, or outstanding request state.",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract axi long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_axistream_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "axistream", "--help"]);
    for fragment in [
        "Extract AXI-Stream transfer rows.",
        "Supports AXI4-Stream and AXI5-Stream profiles from Arm IHI 0051B Issue B.",
        "The default profile is AXI4-Stream.",
        "Signal mapping combines explicit STD_NAME=WAVES_NAME maps with include-regex auto-mapping; explicit maps win.",
        "Mapped TREADY mode requires tvalid and tready; implicit-high mode explicitly declares that physical TREADY is omitted.",
        "Samples reset, handshake predicates, and payload values at the pre-edge sample point for posedge aclk.",
        "One invocation maps one stream interface and emits one row per completed transfer without a synthetic channel.",
        "AXI5-Stream wake-up and parity/check signals are outside this transfer extractor.",
        "[default: axi4-stream]",
        "[possible values: axi4-stream, axi5-stream]",
        "[default: mapped]",
        "[possible values: mapped, implicit-high]",
        "Source-file fields and behavior are documented in the corresponding protocol topic.",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract axistream long help should contain `{fragment}`"
        );
    }
}

#[test]
fn extract_generic_help_is_self_descriptive() {
    let long_help = successful_stdout_text(&["extract", "generic", "--help"]);
    for fragment in [
        "Extract protocol-neutral event rows from waveform signals.",
        "Selects edge-only event timestamps with --on.",
        "Always samples --when and --payload at the pre-edge sample point.",
        "Payload entries accept flat trailing [msb:lsb] projections and preserve order and duplicates.",
        "Use [n:n] for one bit; exact waveform paths win and [n] remains path syntax.",
        "Comma-separated payload paths or flat [msb:lsb] projections",
        "Canonical scope path for relative or in-scope canonical event, predicate, and payload names",
        "In source-file mode, --source provides one or more sources",
        "Source-file fields and behavior are documented in the corresponding protocol topic.",
        "JSON and JSONL rows include time, sample_time, source, and ordered payload values.",
    ] {
        assert!(
            long_help.contains(fragment),
            "extract generic long help should contain `{fragment}`"
        );
    }
    assert!(!long_help.contains("commands/extract-generic"));
}

#[test]
fn version_flags_print_version_to_stdout() {
    let mut short_command = wavepeek_cmd();

    short_command
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d+\.\d+\.\d+\n$").unwrap())
        .stderr(predicate::str::is_empty());

    let mut long_command = wavepeek_cmd();

    long_command
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^wavepeek v\d+\.\d+\.\d+\n$").unwrap())
        .stderr(predicate::str::is_empty());
}

#[test]
fn subcommand_help_uses_extended_prd_descriptions() {
    let mut scope_command = wavepeek_cmd();

    scope_command
        .args(["scope", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "deterministic hierarchy traversal",
        ))
        .stdout(predicate::str::contains("pre-order depth-first"))
        .stdout(predicate::str::contains(
            "Truncation and disabled-limit conditions emit coded diagnostics",
        ));

    let mut property_command = wavepeek_cmd();

    property_command
        .args(["property", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Edge capture (`--capture switch`, `assert`, or `deassert`) reports transitions",
        ))
        .stdout(predicate::str::contains("--capture"));
}

#[test]
fn signal_help_documents_recursive_and_max_depth_flags() {
    let mut command = wavepeek_cmd();

    command
        .args(["signal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--recursive"))
        .stdout(predicate::str::contains("--max-depth"))
        .stdout(predicate::str::contains("limits recursion when set"));
}

#[test]
fn help_documents_unlimited_limit_literals_for_all_affected_commands() {
    wavepeek_cmd()
        .args(["scope", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--max-depth"))
        .stdout(predicate::str::contains("unlimited"));

    wavepeek_cmd()
        .args(["signal", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("--max-depth"))
        .stdout(predicate::str::contains("unlimited"));

    wavepeek_cmd()
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--max"))
        .stdout(predicate::str::contains("unlimited"));

    wavepeek_cmd()
        .args(["property", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--capture <MODE>"))
        .stdout(predicate::str::contains("--max <MAX>"))
        .stdout(predicate::str::contains("unlimited"))
        .stdout(predicate::str::contains("switch"));
}

#[test]
fn property_accepts_capture_flag_in_cli_then_runs() {
    let fixture = common::fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    let output = wavepeek_cmd()
        .args([
            "property",
            "--waves",
            fixture.as_str(),
            "--on",
            "posedge top.clk",
            "--eval",
            "1",
            "--capture",
            "switch",
            "--json",
        ])
        .output()
        .expect("property should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid json");
    assert_eq!(value["command"], "property");
    assert!(
        value["data"].is_array(),
        "property json output should use an array payload"
    );
}

#[test]
fn unimplemented_subcommands_disclose_status_in_help() {
    let mut value_command = wavepeek_cmd();
    value_command
        .args(["value", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());

    let mut change_command = wavepeek_cmd();
    change_command
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());

    let mut property_command = wavepeek_cmd();
    property_command
        .args(["property", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Execution is not implemented yet.").not());
}

#[test]
fn change_help_documents_on_trigger_and_does_not_expose_clk() {
    let mut command = wavepeek_cmd();

    command
        .args(["change", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--on"))
        .stdout(predicate::str::contains("--clk").not())
        .stdout(predicate::str::contains("--tune-engine").not())
        .stdout(predicate::str::contains("--tune-candidates").not())
        .stdout(predicate::str::contains("--tune-edge-fast-force").not())
        .stdout(predicate::str::contains("--perf-engine").not())
        .stdout(predicate::str::contains("--perf-candidates").not())
        .stdout(predicate::str::contains("--perf-edge-fast-force").not());
}

#[test]
fn change_rejects_legacy_when_flag_without_alias() {
    let fixture = common::fixture_path("m2_core.vcd");
    let fixture = fixture.to_string_lossy().into_owned();

    wavepeek_cmd()
        .args([
            "change",
            "--waves",
            fixture.as_str(),
            "--signals",
            "top.clk",
            "--when",
            "*",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--when'"))
        .stderr(predicate::str::contains("See 'wavepeek change --help'."));
}

#[test]
fn waveform_commands_require_waves_flag() {
    let mut command = wavepeek_cmd();

    command
        .arg("info")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--waves <FILE>"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn legacy_subcommands_are_rejected_without_alias() {
    for legacy_name in [
        "tree", "modules", "signals", "changes", "when", "at", "docs",
    ] {
        assert_legacy_subcommand_rejected(legacy_name);
    }
}

#[test]
fn value_rejects_legacy_time_flag_without_alias() {
    let mut command = wavepeek_cmd();

    command
        .args([
            "value",
            "--waves",
            "dump.vcd",
            "--time",
            "1ns",
            "--signals",
            "sig",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--time'"))
        .stderr(predicate::str::contains("See 'wavepeek value --help'."));
}

#[test]
fn positional_arguments_are_rejected() {
    let mut command = wavepeek_cmd();

    command
        .args(["info", "--waves", "dump.vcd", "extra"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument 'extra'"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn unknown_flags_are_normalized_to_args_category() {
    let mut command = wavepeek_cmd();

    command
        .args(["info", "--waves", "dump.vcd", "--wat"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--wat'"))
        .stderr(predicate::str::contains("See 'wavepeek info --help'."));
}

#[test]
fn all_commands_reject_human_flag() {
    let cases: &[(&[&str], &str)] = &[
        (&["info", "--waves", "dump.vcd", "--human"], "info"),
        (&["scope", "--waves", "dump.vcd", "--human"], "scope"),
        (
            &["signal", "--waves", "dump.vcd", "--scope", "top", "--human"],
            "signal",
        ),
        (
            &[
                "value",
                "--waves",
                "dump.vcd",
                "--at",
                "1ns",
                "--signals",
                "sig",
                "--human",
            ],
            "value",
        ),
        (
            &[
                "change",
                "--waves",
                "dump.vcd",
                "--signals",
                "sig",
                "--human",
            ],
            "change",
        ),
        (
            &[
                "property", "--waves", "dump.vcd", "--on", "*", "--eval", "1", "--human",
            ],
            "property",
        ),
    ];

    for (args, command_name) in cases {
        assert_human_flag_rejected(args, command_name);
    }
}

#[test]
fn unknown_top_level_flag_uses_global_help_hint() {
    let mut command = wavepeek_cmd();

    command
        .args(["--wat"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::starts_with("fatal: args:"))
        .stderr(predicate::str::contains("unexpected argument '--wat'"))
        .stderr(predicate::str::contains("See 'wavepeek --help'."));
}
