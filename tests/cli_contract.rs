use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn wavepeek_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_wavepeek"))
}

#[test]
fn help_lists_expected_subcommands() {
    let mut command = wavepeek_cmd();

    command
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "wavepeek is a command-line tool for RTL waveform inspection.",
        ))
        .stdout(predicate::str::contains("schema"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("tree"))
        .stdout(predicate::str::contains("signals"))
        .stdout(predicate::str::contains("at"))
        .stdout(predicate::str::contains("changes"))
        .stdout(predicate::str::contains("when"))
        .stdout(predicate::str::contains("\n  help\n").not());
}

#[test]
fn subcommand_help_uses_extended_prd_descriptions() {
    let mut tree_command = wavepeek_cmd();

    tree_command
        .args(["tree", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "recursively traversing the hierarchy",
        ))
        .stdout(predicate::str::contains("bounded by --max and --max-depth"));

    let mut when_command = wavepeek_cmd();

    when_command
        .args(["when", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "boolean expression evaluates to true",
        ))
        .stdout(predicate::str::contains("first N, or last N matches"));
}

#[test]
fn waveform_commands_require_waves_flag() {
    let mut command = wavepeek_cmd();

    command
        .arg("info")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--waves <FILE>"));
}

#[test]
fn schema_does_not_accept_waves_flag() {
    let mut command = wavepeek_cmd();

    command
        .args(["schema", "--waves", "dump.vcd"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--waves'"));
}

#[test]
fn positional_arguments_are_rejected() {
    let mut command = wavepeek_cmd();

    command
        .args(["info", "--waves", "dump.vcd", "extra"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument 'extra'"));
}
