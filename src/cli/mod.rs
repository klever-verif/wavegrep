pub mod change;
pub mod extract;
pub mod info;
pub mod limits;
pub mod property;
pub mod sampling;
pub mod scope;
pub mod signal;
pub mod skill;
pub mod value;

use std::ffi::{OsStr, OsString};
use std::io::Write;

use clap::error::ErrorKind;
use clap::parser::ValueSource;
use clap::{Arg, ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::engine::{self, Command as EngineCommand};
use crate::error::WavepeekError;
use crate::output::{self, JsonlWriter};
use crate::output_mode::OutputMode;

#[derive(Debug, Parser)]
#[command(
    name = "wavepeek",
    disable_version_flag = true,
    about = "wavepeek is a machine-friendly command-line tool for RTL waveform inspection.\nSee more with '--help'",
    long_about = r#"wavepeek is a machine-friendly command-line tool for RTL waveform inspection.
See more with '--help'

General conventions:
- Waveform-inspection commands require `--waves <FILE>`.
- VCD/FST input is available in every build.
- FSDB support is currently Linux x86_64 only and requires a build compiled with Cargo feature `fsdb` and the Synopsys Verdi FSDB Reader SDK.
- Output is bounded by default (e.g. with `--max` or similar) and recursive traversals are depth-bounded.
- Default output is human-readable for waveform commands; `--json` enables machine-readable output documented in the packaged `references/machine-output.md`.
- Time values require explicit units (`zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, `s`) and integer magnitudes.
- Parsed times are normalized to dump `time_unit`; time-window flags (`--from`, `--to`) use inclusive boundaries.
- Human process-level failures follow `fatal: <category>: <message>`; `--json` and `--jsonl` use typed fatal records documented in `references/machine-output.md`."#,
    after_help = "Next steps:\n  wavepeek --help\n  wavepeek help <command-path...>\n  wavepeek skill <DIRECTORY>",
    help_template = "{about-with-newline}\nUsage: {usage}\n\nWaveform commands:\n  info      Show waveform metadata\n  scope     Explore hierarchy scopes\n  signal    Explore signals within scope\n  value     Get signal values at explicit time point(s)\n  change    List signal changes over a time range\n  property  Evaluate properties over a time range\n  extract   Extract event rows from waveform signals\n\nHelper commands:\n  skill     Extract the packaged agent skill\n  help      Show help for the given subcommand(s)\n\nOptions:\n{options}{after-help}"
)]
pub struct Cli {
    /// Print semver version
    #[arg(short = 'V', action = ArgAction::SetTrue)]
    version_semver: bool,
    /// Print full version
    #[arg(long = "version", action = ArgAction::SetTrue)]
    version_full: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(flatten, next_help_heading = "Waveform commands")]
    Waveform(WaveformCommand),
    #[command(flatten, next_help_heading = "Helper commands")]
    Helper(HelperCommand),
}

#[derive(Debug, Subcommand)]
enum WaveformCommand {
    #[command(
        about = "Reports metadata for the selected waveform dump.",
        long_about = r#"Reports metadata for the selected waveform dump.

Behavior:
- Prints available metadata (e.g. time unit, start/end times, etc.) in free form
- `--json` emits the standard machine-readable envelope."#
    )]
    Info(info::InfoArgs),
    #[command(
        about = "Provides deterministic hierarchy traversal over scope paths.",
        long_about = r#"Provides deterministic hierarchy traversal over scope paths.

Behavior:
- Finds all scopes matching `--filter` and displays scope name, depth, and kind.
- Traversal order is stable: pre-order depth-first, with lexicographic child ordering.
- Includes stable scope kind aliases from hierarchy data (not only modules); excluded backend-specific spellings are normalized to the stable contract surface.
- `--tree` switches from flat list to visual hierarchy rendering and includes filtered matches' ancestors up to the root.
- Truncation emits a coded diagnostic; without `--summary`, an empty valid human result prints a short message on stdout.
- `--json` emits the standard machine-readable envelope.

Use this command to explore hierarchy shape before narrowing to signal-level queries."#
    )]
    Scope(scope::ScopeArgs),
    #[command(
        about = "Provides scope-local signal listings.",
        long_about = r#"Provides scope-local signal listings.

Behavior:
- Finds all signals matching `--filter` within the selected scope and displays name, kind, and available metadata (for example width).
- Default mode lists only direct signals in the selected scope.
- Recursive mode walks child scopes depth-first in stable lexicographic order; `--max-depth` limits recursion when set.
- Includes stable signal kind aliases (not only wires); excluded backend-specific VHDL spellings are normalized to the stable contract surface.
- Ambiguous FSDB signal paths are omitted with a coded diagnostic; no backing record is selected.
- Truncation emits a coded diagnostic; without `--summary`, an empty valid human result prints a short message on stdout.
- `--json` emits the standard machine-readable envelope.

Use this command after `scope` to inspect available signals in a target scope."#
    )]
    Signal(signal::SignalArgs),
    #[command(
        about = "Provides point sampling for selected signals.",
        long_about = r#"Provides point sampling for selected signals.

Behavior:
- Prints values for the requested signals at each selected time point.
- By default, signal names are top-related canonical paths (e.g. `top.cpu.state`).
- For deep hierarchies, set `--scope` once with a canonical scope path; signal references may be relative to it or canonical paths inside it.
- Relative and canonical references inside the selected scope may be mixed in one request.
- A trailing static `[msb:lsb]` projects a flat integral signal's normalized sampled value; use `[n:n]` for one bit.
- Exact waveform paths win before projection parsing, and `[n]` remains ordinary waveform path syntax.
- `--at` accepts one explicit time token or a comma-separated list in one argument.
- Output preserves the input order from `--at` and `--signals`, including duplicates.
- Human output emits one `@<time>` row per requested time with `display=value` fields, matching `change`.
- When following up a `change` or `property` JSON row, prefer that row's `sample_time` field for `--at`; in `pre-edge` mode, `time` is the selected trigger timestamp and `sample_time` is where values were sampled.
- Time tokens must include explicit units and align to dump precision.
- Values are emitted as Verilog literals (`<width>'h<digits>` with `x`/`z` support).
- Fails fast if any requested signal cannot be resolved or if any selected time point is more precise than dump resolution.
- `--json` emits the standard machine-readable envelope.

Use this command for deterministic spot checks at specific timestamps."#
    )]
    Value(value::ValueArgs),
    #[command(
        about = "Provides event-driven tables for selected signals.",
        long_about = r#"Provides event-driven tables for selected signals.

Behavior:
- Prints requested signal values for each event selected by required `--on`.
- `--signals` accepts trailing static `[msb:lsb]` projections; sparse, delta, and wildcard comparisons use the projected values.
- Exact waveform paths win before projection parsing; use `[n:n]` for one bit because `[n]` remains ordinary waveform path syntax.
- `--row-mode dense|sparse` controls whether every sampled event or only changed samples become rows; the default is `dense`.
- Pre-edge events without a representable earlier sample point are skipped.
- `--row-values full|delta` controls whether rows contain all requested signals or only changed signals; the default is `full`, and the first delta row is always full.
- Range boundaries are inclusive. Dense mode can emit a matching event at `--from`; sparse mode uses `--from` only as its comparison baseline.
- Value sampling defaults to pre-edge sampling: displayed values are sampled just before edge-only triggers while row timestamps stay at the trigger edge.
- Use `--sample-mode native` for raw wildcard or plain-signal triggers such as `--on '*'`.
- JSON and JSONL rows include both `time` (selected event timestamp) and `sample_time` (where values were sampled); text output shows `sample@<time>` only when it differs from `time`.
- Truncation emits a coded diagnostic; without `--summary`, an empty valid human result prints a short message on stdout.
- `--json` emits the standard machine-readable envelope.

Use this command to inspect event-aligned values or value transitions over bounded time windows."#
    )]
    Change(change::ChangeArgs),
    #[command(
        about = "Provides timestamps where the specified property holds over event triggers.",
        long_about = r#"Provides timestamps where the specified property holds over event triggers.

Behavior:
- Evaluates `--eval` at timestamps selected by `--on` and prints time plus metadata when the property holds.
- Level capture (`--capture match`) reports a match at every selected timestamp where the property holds.
- Edge capture (`--capture switch`, `assert`, or `deassert`) reports transitions: no match to match, or match to no match.
- `--on` is required. Use explicit clock edges such as `--on 'posedge clk'` for RTL-style sampling.
- Value sampling defaults to pre-edge sampling: `--eval` reads values just before edge-only triggers while row timestamps stay at the trigger edge.
- Use `--sample-mode native` for raw wildcard or plain-signal triggers such as `--on '*'`.
- JSON and JSONL rows include both `time` (selected event timestamp) and `sample_time` (where `--eval` was sampled); text output shows `sample@<time>` only when it differs from `time`.
- Truncation emits a coded diagnostic; without `--summary`, an empty valid human result prints a short message on stdout.
- Remotely similar to a SystemVerilog assert, but without temporal expressions.
- `--json` emits the standard machine-readable envelope.

Use this command to check event-driven property matches and transitions over bounded time windows."#
    )]
    Property(property::PropertyArgs),
    #[command(
        subcommand,
        about = "Extract row-oriented waveform data.",
        long_about = r#"Extract row-oriented waveform data.

Use nested extractors for protocol-neutral or protocol-specific event rows. The generic extractor selects edge events, evaluates a predicate at the pre-edge sample point, and emits ordered payload values."#
    )]
    Extract(extract::ExtractCommand),
}

#[derive(Debug, Subcommand)]
enum HelperCommand {
    #[command(
        about = "Extract the packaged agent skill into a directory.",
        long_about = "Extract the packaged agent skill into a directory."
    )]
    Skill(skill::SkillArgs),
}

struct OptionalFeatureHelp {
    name: &'static str,
    enabled: bool,
    disabled_hint: &'static str,
}

const OPTIONAL_FEATURES: &[OptionalFeatureHelp] = &[OptionalFeatureHelp {
    name: "FSDB",
    enabled: cfg!(feature = "fsdb"),
    disabled_hint: "FSDB support is currently Linux x86_64 only; reinstall with Cargo flag `--features fsdb` and provide the Synopsys Verdi FSDB Reader SDK",
}];

const ROOT_NEXT_STEPS: &str = "Next steps:\n  wavepeek --help\n  wavepeek help <command-path...>\n  wavepeek skill <DIRECTORY>";
const BROWSER_LONG_ABOUT: &str = r#"wavepeek is a machine-friendly command-line tool for RTL waveform inspection.

Browser conventions:
- Waveform-inspection commands require `--waves <FILE>` matching the active demo or local file.
- VCD/FST input is processed locally in this browser worker.
- FSDB input, `skill`, and extraction `--source <FILE>` options are not supported.
- Output and time conventions match the native WavePeek CLI."#;
const BROWSER_HELP_TEMPLATE: &str = "{about-with-newline}\nUsage: {usage}\n\nWaveform commands:\n  info      Show waveform metadata\n  scope     Explore hierarchy scopes\n  signal    Explore signals within scope\n  value     Get signal values at explicit time point(s)\n  change    List signal changes over a time range\n  property  Evaluate properties over a time range\n  extract   Extract event rows from waveform signals\n\nHelper commands:\n  help      Show help for the given subcommand(s)\n\nOptions:\n{options}{after-help}";
const BROWSER_NEXT_STEPS: &str =
    "Browser limits: FSDB input, skill, and extraction --source files are unavailable.";

fn root_after_long_help() -> String {
    let mut help = String::from("Optional features:\n");
    for feature in OPTIONAL_FEATURES {
        if feature.enabled {
            help.push_str("- ");
            help.push_str(feature.name);
            help.push_str(" - enabled\n");
        } else {
            help.push_str("- ");
            help.push_str(feature.name);
            help.push_str(" - disabled (");
            help.push_str(feature.disabled_hint);
            help.push_str(")\n");
        }
    }
    help.push('\n');
    help.push_str(ROOT_NEXT_STEPS);
    help
}

#[derive(Clone, Copy)]
struct OutputSelection {
    mode: OutputMode,
    json: usize,
    jsonl: usize,
}

pub(crate) struct CliFailure {
    pub(crate) error: WavepeekError,
    pub(crate) reported: bool,
}

impl From<WavepeekError> for CliFailure {
    fn from(error: WavepeekError) -> Self {
        Self {
            error,
            reported: false,
        }
    }
}

pub(crate) fn run(report_machine_errors: bool) -> Result<(), CliFailure> {
    run_from(
        std::env::args_os().collect(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
        report_machine_errors,
        false,
    )
}

pub(crate) fn run_from(
    argv: Vec<OsString>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    report_machine_errors: bool,
    browser: bool,
) -> Result<(), CliFailure> {
    let (selection, argv) = extract_output_selection(argv);

    match execute(
        argv,
        selection,
        stdout,
        stderr,
        report_machine_errors,
        browser,
    ) {
        Ok(()) => Ok(()),
        Err(CliFailure {
            error: WavepeekError::BrokenPipe,
            ..
        }) if report_machine_errors => Ok(()),
        Err(failure) if failure.reported || !report_machine_errors => Err(failure),
        Err(failure) => report_failure(selection.mode, failure.error, stdout),
    }
}

fn execute(
    argv: Vec<OsString>,
    selection: OutputSelection,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    report_machine_errors: bool,
    browser: bool,
) -> Result<(), CliFailure> {
    if selection.mode != OutputMode::Human {
        if selection.json > 1 {
            return Err(WavepeekError::Args(
                "the argument '--json' cannot be used multiple times".to_string(),
            )
            .into());
        }
        if selection.jsonl > 1 {
            return Err(WavepeekError::Args(
                "the argument '--jsonl' cannot be used multiple times".to_string(),
            )
            .into());
        }
        if selection.json > 0 && selection.jsonl > 0 {
            return Err(WavepeekError::Args(
                "the argument '--json' cannot be used with '--jsonl'".to_string(),
            )
            .into());
        }
    }

    let parse_argv = if argv.len() == 1 && selection.mode == OutputMode::Human {
        vec![argv[0].clone(), "-h".into()]
    } else if argv.len() == 2
        && selection.mode == OutputMode::Human
        && matches!(argv[1].to_str(), Some("extract"))
    {
        vec![argv[0].clone(), argv[1].clone(), "-h".into()]
    } else {
        argv
    };

    let matches = match build_cli_command_for(browser).try_get_matches_from(parse_argv) {
        Ok(matches) => matches,
        Err(error) => return handle_parse_error(error, stdout).map_err(Into::into),
    };

    if change_tune_overrides_requested(&matches) && !is_debug_mode_enabled() {
        return Err(WavepeekError::Args(
            "internal tuning overrides (--tune-*) require DEBUG=1. Set DEBUG=1 only for local diagnostics or CI debugging."
                .to_string(),
        )
        .into());
    }

    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => return handle_parse_error(error, stdout).map_err(Into::into),
    };

    if cli.version_semver {
        output::write_to(stdout, env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }

    if cli.version_full {
        output::write_to(stdout, concat!("wavepeek v", env!("CARGO_PKG_VERSION")))?;
        return Ok(());
    }

    let Some(command) = cli.command else {
        return Err(WavepeekError::Args("a waveform command is required".to_string()).into());
    };

    dispatch(
        command,
        selection.mode,
        stdout,
        stderr,
        report_machine_errors,
    )
}

fn report_failure(
    mode: OutputMode,
    error: WavepeekError,
    stdout: &mut dyn Write,
) -> Result<(), CliFailure> {
    let reported = match mode {
        OutputMode::Human => return Err(error.into()),
        OutputMode::Json => output::write_json_fatal_to(&error, stdout),
        OutputMode::Jsonl => output::write_jsonl_fatal_to(&error, stdout),
    };

    match reported {
        Ok(()) => Err(CliFailure {
            error,
            reported: true,
        }),
        Err(WavepeekError::BrokenPipe) => Ok(()),
        Err(write_error) => Err(write_error.into()),
    }
}

fn extract_output_selection(argv: Vec<OsString>) -> (OutputSelection, Vec<OsString>) {
    let command = build_cli_command();
    let mut json = 0;
    let mut jsonl = 0;
    let mut options_ended = false;
    let mut parse_argv = Vec::with_capacity(argv.len());

    for (index, arg) in argv.iter().enumerate() {
        if index == 0 {
            parse_argv.push(arg.clone());
            continue;
        }
        if arg == OsStr::new("--") {
            options_ended = true;
            parse_argv.push(arg.clone());
            continue;
        }
        if !options_ended && !is_known_option_value(&command, &argv, index) {
            if arg == OsStr::new("--json") {
                json += 1;
                continue;
            }
            if arg == OsStr::new("--jsonl") {
                jsonl += 1;
                continue;
            }
        }
        parse_argv.push(arg.clone());
    }

    let mode = OutputMode::from_json_flags(json > 0, jsonl > 0);
    (OutputSelection { mode, json, jsonl }, parse_argv)
}

fn is_known_option_value(command: &clap::Command, argv: &[OsString], index: usize) -> bool {
    let Some(previous) = index.checked_sub(1).and_then(|index| argv[index].to_str()) else {
        return false;
    };
    let Some(name) = previous.strip_prefix("--") else {
        return false;
    };
    let active_command = argv[1..index].iter().fold(command, |command, arg| {
        arg.to_str()
            .and_then(|name| command.find_subcommand(name))
            .unwrap_or(command)
    });
    !name.contains('=')
        && active_command.get_arguments().any(|arg| {
            arg.get_long() == Some(name)
                && arg.get_action().takes_values()
                && arg.is_allow_hyphen_values_set()
        })
}

fn is_debug_mode_enabled() -> bool {
    std::env::var("DEBUG")
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn change_tune_overrides_requested(matches: &clap::ArgMatches) -> bool {
    let Some(("change", change_matches)) = matches.subcommand() else {
        return false;
    };

    is_command_line_override(change_matches, "tune_engine")
        || is_command_line_override(change_matches, "tune_candidates")
        || is_command_line_override(change_matches, "tune_edge_fast_force")
}

fn is_command_line_override(matches: &clap::ArgMatches, arg: &str) -> bool {
    matches!(matches.value_source(arg), Some(ValueSource::CommandLine))
}

fn build_cli_command() -> clap::Command {
    build_cli_command_for(false)
}

fn build_cli_command_for(browser: bool) -> clap::Command {
    let mut command = Cli::command().after_long_help(root_after_long_help());
    if let Some(help) = command.find_subcommand_mut("help") {
        *help = help.clone().about("Show help for the given subcommand(s)");
    }
    for command_name in ["info", "scope", "signal", "value", "change", "property"] {
        if let Some(subcommand) = command.find_subcommand_mut(command_name) {
            *subcommand = with_other_help_options(subcommand.clone());
        }
    }
    if let Some(extract) = command.find_subcommand_mut("extract") {
        for name in ["ahb", "apb", "atb", "axi", "axistream", "generic"] {
            if let Some(subcommand) = extract.find_subcommand_mut(name) {
                let mut updated = with_other_help_options(subcommand.clone());
                if browser {
                    updated = updated.mut_arg("source", |arg| arg.hide(true));
                    if let Some(help) = updated.get_long_about() {
                        let help = help
                            .to_string()
                            .lines()
                            .filter(|line| {
                                let line = line.to_ascii_lowercase();
                                !line.contains("source file")
                                    && !line.contains("source-file")
                                    && !line.contains("--source")
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        updated = updated.long_about(help);
                    }
                }
                *subcommand = updated;
            }
        }
    }
    if browser {
        if let Some(skill) = command.find_subcommand_mut("skill") {
            *skill = skill.clone().hide(true);
        }
        command = command
            .long_about(BROWSER_LONG_ABOUT)
            .help_template(BROWSER_HELP_TEMPLATE)
            .after_help(BROWSER_NEXT_STEPS)
            .after_long_help(BROWSER_NEXT_STEPS);
    }
    command
}

fn with_other_help_options(command: clap::Command) -> clap::Command {
    command
        .disable_help_flag(true)
        .arg(
            Arg::new("help_short")
                .short('h')
                .action(ArgAction::HelpShort)
                .help("Print help (see more with '--help')")
                .help_heading("Other options"),
        )
        .arg(
            Arg::new("help_long")
                .long("help")
                .action(ArgAction::HelpLong)
                .help("Print help (see a summary with '-h')")
                .help_heading("Other options"),
        )
}

fn handle_parse_error(error: clap::Error, stdout: &mut dyn Write) -> Result<(), WavepeekError> {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => stdout
            .write_all(error.render().to_string().as_bytes())
            .map_err(output::map_stdout_io_error),
        _ => Err(WavepeekError::Args(normalize_clap_error(&error))),
    }
}

fn normalize_clap_error(error: &clap::Error) -> String {
    let rendered = error.render().to_string();
    let detail = clap_error_detail(rendered.as_str());
    let hint = help_hint_for_rendered_clap_error(rendered.as_str());

    format!("{detail} {hint}")
}

fn clap_error_detail(rendered: &str) -> String {
    let lines: Vec<&str> = rendered.lines().collect();
    if let Some(start_index) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("error:"))
    {
        let mut chunks = Vec::new();

        for (index, line) in lines.iter().enumerate().skip(start_index) {
            let trimmed = line.trim();
            if index > start_index
                && (trimmed.starts_with("Usage:") || trimmed.starts_with("For more information"))
            {
                break;
            }

            if index == start_index {
                if let Some(rest) = trimmed.strip_prefix("error:") {
                    let rest = rest.trim();
                    if !rest.is_empty() {
                        chunks.push(rest.to_string());
                    }
                }
                continue;
            }

            if !trimmed.is_empty() {
                chunks.push(trimmed.to_string());
            }
        }

        if !chunks.is_empty() {
            return chunks.join(" ");
        }
    }

    for line in lines {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("error:") {
            return rest.trim().to_string();
        }
    }

    rendered
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| "invalid arguments".to_string())
}

fn help_hint_for_rendered_clap_error(rendered: &str) -> String {
    let usage_line = rendered
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("Usage:"));

    let Some(usage_line) = usage_line else {
        return "See 'wavepeek --help'.".to_string();
    };

    let usage = usage_line.trim_start_matches("Usage:").trim();
    let mut parts = usage.split_whitespace();
    let Some(command_name) = parts.next() else {
        return "See 'wavepeek --help'.".to_string();
    };
    if command_name != "wavepeek" {
        return "See 'wavepeek --help'.".to_string();
    }

    let mut path_tokens = Vec::new();
    for token in parts {
        if token.starts_with('[') || token.starts_with('<') || token.starts_with('-') {
            break;
        }
        path_tokens.push(token);
    }

    if path_tokens.is_empty() {
        return "See 'wavepeek --help'.".to_string();
    }

    format!("See 'wavepeek {} --help'.", path_tokens.join(" "))
}

fn dispatch(
    command: Command,
    output_mode: OutputMode,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    report_machine_errors: bool,
) -> Result<(), CliFailure> {
    let mut engine_command = into_engine_command(command);
    if matches!(engine_command, EngineCommand::Skill(_)) && output_mode != OutputMode::Human {
        return Err(WavepeekError::Args(
            "--json and --jsonl are available only for waveform commands".to_string(),
        )
        .into());
    }
    engine_command.set_output_mode(output_mode);

    if output_mode == OutputMode::Jsonl {
        let mut writer = JsonlWriter::new(stdout, engine_command.name());
        return match engine::run_jsonl(engine_command, &mut writer) {
            Ok(()) => Ok(()),
            Err(WavepeekError::BrokenPipe) => Err(WavepeekError::BrokenPipe.into()),
            Err(error) if !report_machine_errors => Err(error.into()),
            Err(error) => match writer.fatal(&error) {
                Ok(()) => Err(CliFailure {
                    error,
                    reported: true,
                }),
                Err(write_error) => Err(write_error.into()),
            },
        };
    }

    let result = engine::run(engine_command)?;
    output::write_result_to(result, stdout, stderr).map_err(Into::into)
}

fn into_engine_command(command: Command) -> EngineCommand {
    match command {
        Command::Waveform(command) => match command {
            WaveformCommand::Info(args) => EngineCommand::Info(args),
            WaveformCommand::Scope(args) => EngineCommand::Scope(args),
            WaveformCommand::Signal(args) => EngineCommand::Signal(args),
            WaveformCommand::Value(args) => EngineCommand::Value(args),
            WaveformCommand::Change(args) => EngineCommand::Change(args),
            WaveformCommand::Property(args) => EngineCommand::Property(args),
            WaveformCommand::Extract(command) => match command {
                extract::ExtractCommand::Ahb(args) => EngineCommand::ExtractAhb(*args),
                extract::ExtractCommand::Apb(args) => EngineCommand::ExtractApb(*args),
                extract::ExtractCommand::Atb(args) => EngineCommand::ExtractAtb(*args),
                extract::ExtractCommand::Axi(args) => EngineCommand::ExtractAxi(*args),
                extract::ExtractCommand::AxiStream(args) => EngineCommand::ExtractAxiStream(*args),
                extract::ExtractCommand::Generic(args) => EngineCommand::ExtractGeneric(*args),
            },
        },
        Command::Helper(command) => match command {
            HelperCommand::Skill(args) => EngineCommand::Skill(args),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::{CommandFactory, Parser};

    use crate::cli::limits::LimitArg;

    use super::{
        Cli, EngineCommand, build_cli_command, change_tune_overrides_requested, clap_error_detail,
        handle_parse_error, help_hint_for_rendered_clap_error, into_engine_command,
        normalize_clap_error,
    };

    #[test]
    fn info_dispatch_keeps_json_and_waves_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "info",
            "--waves",
            "fixtures/sample.vcd",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Info(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert!(args.json);
            }
            other => panic!("expected info command, got {other:?}"),
        }
    }

    #[test]
    fn scope_dispatch_keeps_bounded_query_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "scope",
            "--waves",
            "fixtures/sample.vcd",
            "--max",
            "12",
            "--max-depth",
            "3",
            "--filter",
            "^top\\..*",
            "--tree",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Scope(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, LimitArg::Numeric(12));
                assert_eq!(args.max_depth, LimitArg::Numeric(3));
                assert_eq!(args.filter, "^top\\..*");
                assert!(args.tree);
                assert!(args.json);
            }
            other => panic!("expected scope command, got {other:?}"),
        }
    }

    #[test]
    fn scope_dispatch_accepts_unlimited_limit_literals() {
        let cli = Cli::parse_from([
            "wavepeek",
            "scope",
            "--waves",
            "fixtures/sample.vcd",
            "--max",
            "unlimited",
            "--max-depth",
            "unlimited",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Scope(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.max, LimitArg::Unlimited);
                assert_eq!(args.max_depth, LimitArg::Unlimited);
            }
            other => panic!("expected scope command, got {other:?}"),
        }
    }

    #[test]
    fn signal_dispatch_keeps_recursive_and_max_depth_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "signal",
            "--waves",
            "fixtures/sample.vcd",
            "--scope",
            "top.cpu",
            "--recursive",
            "--max-depth",
            "3",
            "--max",
            "7",
            "--filter",
            ".*clk.*",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Signal(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.scope, "top.cpu");
                assert!(args.recursive);
                assert_eq!(args.max_depth, LimitArg::Numeric(3));
                assert_eq!(args.max, LimitArg::Numeric(7));
                assert_eq!(args.filter, ".*clk.*");
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected signal command, got {other:?}"),
        }
    }

    #[test]
    fn value_dispatch_keeps_scope_signals_abs_and_json_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "value",
            "--waves",
            "fixtures/sample.vcd",
            "--at",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Value(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.at, "10ns");
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.signals, vec!["clk", "data"]);
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected value command, got {other:?}"),
        }
    }

    #[test]
    fn change_dispatch_keeps_on_abs_and_limits() {
        let cli = Cli::parse_from([
            "wavepeek",
            "change",
            "--waves",
            "fixtures/sample.vcd",
            "--from",
            "1ns",
            "--to",
            "10ns",
            "--scope",
            "top",
            "--signals",
            "clk,data",
            "--on",
            "posedge clk",
            "--max",
            "12",
            "--abs",
            "--json",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Change(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.from.as_deref(), Some("1ns"));
                assert_eq!(args.to.as_deref(), Some("10ns"));
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.signals, vec!["clk", "data"]);
                assert_eq!(args.on, "posedge clk");
                assert_eq!(args.sample_mode, crate::cli::sampling::SampleMode::PreEdge);
                assert_eq!(args.row_mode, crate::cli::change::RowMode::Dense);
                assert_eq!(args.row_values, crate::cli::change::RowValues::Full);
                assert_eq!(args.max, LimitArg::Numeric(12));
                assert!(args.abs);
                assert!(args.json);
            }
            other => panic!("expected change command, got {other:?}"),
        }
    }

    #[test]
    fn property_dispatch_parses_capture_default() {
        let cli = Cli::parse_from([
            "wavepeek",
            "property",
            "--waves",
            "fixtures/sample.vcd",
            "--on",
            "posedge top.clk",
            "--eval",
            "1",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Property(args) => {
                assert_eq!(args.on, "posedge top.clk");
                assert_eq!(args.sample_mode, crate::cli::sampling::SampleMode::PreEdge);
                assert_eq!(args.eval, "1");
                assert_eq!(args.capture, crate::cli::property::CaptureMode::Switch);
                assert_eq!(args.max, LimitArg::Numeric(50));
            }
            other => panic!("expected property command, got {other:?}"),
        }
    }

    #[test]
    fn property_dispatch_accepts_unlimited_max() {
        let cli = Cli::parse_from([
            "wavepeek",
            "property",
            "--waves",
            "fixtures/sample.vcd",
            "--on",
            "posedge top.clk",
            "--eval",
            "1",
            "--max",
            "unlimited",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::Property(args) => {
                assert_eq!(args.max, LimitArg::Unlimited);
            }
            other => panic!("expected property command, got {other:?}"),
        }
    }

    #[test]
    fn extract_ahb_dispatch_keeps_pipeline_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "extract",
            "ahb",
            "--waves",
            "fixtures/sample.vcd",
            "--profile",
            "ahb5",
            "--scope",
            "top",
            "--name",
            "dmem",
            "--map",
            "hclk=clk",
            "--include",
            "^dmem_",
            "--include-stall",
            "--include-idle",
            "--include-busy",
            "--max",
            "unlimited",
            "--abs",
            "--jsonl",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::ExtractAhb(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.profile.as_str(), "ahb5");
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.name.as_deref(), Some("dmem"));
                assert_eq!(args.maps, ["hclk=clk"]);
                assert_eq!(args.includes, ["^dmem_"]);
                assert!(args.include_stall);
                assert!(args.include_idle);
                assert!(args.include_busy);
                assert_eq!(args.max, LimitArg::Unlimited);
                assert!(args.abs);
                assert!(args.jsonl);
            }
            other => panic!("expected extract ahb command, got {other:?}"),
        }
    }

    #[test]
    fn extract_generic_dispatch_keeps_single_source_args() {
        let cli = Cli::parse_from([
            "wavepeek",
            "extract",
            "generic",
            "--waves",
            "fixtures/sample.vcd",
            "--scope",
            "top",
            "--name",
            "rx.beat",
            "--on",
            "posedge clk",
            "--when",
            "valid && ready",
            "--payload",
            "data,last",
            "--max",
            "unlimited",
            "--abs",
            "--jsonl",
        ]);

        let command = into_engine_command(cli.command.expect("parsed command"));
        match command {
            EngineCommand::ExtractGeneric(args) => {
                assert_eq!(args.waves, PathBuf::from("fixtures/sample.vcd"));
                assert_eq!(args.scope.as_deref(), Some("top"));
                assert_eq!(args.name.as_deref(), Some("rx.beat"));
                assert_eq!(args.on.as_deref(), Some("posedge clk"));
                assert_eq!(args.when.as_deref(), Some("valid && ready"));
                assert_eq!(
                    args.payload,
                    Some(vec!["data".to_string(), "last".to_string()])
                );
                assert_eq!(args.max, LimitArg::Unlimited);
                assert!(args.abs);
                assert!(args.jsonl);
            }
            other => panic!("expected extract generic command, got {other:?}"),
        }
    }

    #[test]
    fn clap_errors_are_normalized_to_single_line_message() {
        let error = Cli::try_parse_from(["wavepeek", "info", "--unknown"])
            .expect_err("unknown argument should fail");

        let normalized = normalize_clap_error(&error);
        assert!(normalized.contains("unexpected argument '--unknown'"));
        assert!(normalized.contains("See 'wavepeek info --help'."));
        assert!(!normalized.contains("Usage:"));
    }

    #[test]
    fn clap_error_detail_preserves_missing_argument_names() {
        let rendered = "error: the following required arguments were not provided:\n  --waves <FILE>\n\nUsage: wavepeek info --waves <FILE>\n\nFor more information, try '--help'.\n";

        let normalized = clap_error_detail(rendered);
        assert!(normalized.contains("the following required arguments were not provided"));
        assert!(normalized.contains("--waves <FILE>"));
        assert!(!normalized.contains("Usage:"));
    }

    #[test]
    fn help_hint_uses_global_help_for_top_level_parse_failures() {
        let rendered = "error: unexpected argument '--wat' found\n\nUsage: wavepeek [OPTIONS] <COMMAND>\n\nFor more information, try '--help'.\n";
        let hint = help_hint_for_rendered_clap_error(rendered);
        assert_eq!(hint, "See 'wavepeek --help'.");
    }

    #[test]
    fn help_hint_uses_subcommand_help_for_subcommand_parse_failures() {
        let rendered = "error: unexpected argument '--wat' found\n\nUsage: wavepeek info --waves <FILE>\n\nFor more information, try '--help'.\n";
        let hint = help_hint_for_rendered_clap_error(rendered);
        assert_eq!(hint, "See 'wavepeek info --help'.");
    }

    #[test]
    fn cli_helper_functions_exercise_override_detection_and_fallback_paths() {
        let command = build_cli_command();
        let info = command
            .find_subcommand("info")
            .expect("info subcommand should exist")
            .clone();
        assert!(info.get_arguments().any(|arg| arg.get_id() == "help_short"));
        assert!(info.get_arguments().any(|arg| arg.get_id() == "help_long"));

        let matches = Cli::command()
            .try_get_matches_from([
                "wavepeek",
                "change",
                "--waves",
                "fixtures/sample.vcd",
                "--signals",
                "clk",
                "--on",
                "*",
                "--sample-mode",
                "native",
                "--tune-engine",
                "fused",
            ])
            .expect("change command should parse");
        assert!(change_tune_overrides_requested(&matches));

        let matches = Cli::command()
            .try_get_matches_from([
                "wavepeek",
                "change",
                "--waves",
                "fixtures/sample.vcd",
                "--signals",
                "clk",
                "--on",
                "*",
                "--sample-mode",
                "native",
            ])
            .expect("change command should parse");
        assert!(!change_tune_overrides_requested(&matches));

        let help_error = Cli::command()
            .try_get_matches_from(["wavepeek", "info", "--help"])
            .expect_err("--help should short-circuit through clap");
        handle_parse_error(help_error, &mut Vec::new())
            .expect("display-help errors should print cleanly");

        assert_eq!(
            clap_error_detail("plain fallback detail"),
            "plain fallback detail"
        );
        assert_eq!(clap_error_detail("\n\n   \n"), "invalid arguments");
        assert_eq!(
            help_hint_for_rendered_clap_error("Usage: nope info --waves <FILE>\n"),
            "See 'wavepeek --help'."
        );
    }
}
