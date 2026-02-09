pub mod at;
pub mod changes;
pub mod info;
pub mod schema;
pub mod signals;
pub mod tree;
pub mod when;

use serde::Serialize;

use crate::cli;
use crate::error::WavepeekError;

#[derive(Debug)]
pub enum Command {
    Schema(cli::schema::SchemaArgs),
    Info(cli::info::InfoArgs),
    Tree(cli::tree::TreeArgs),
    Signals(cli::signals::SignalsArgs),
    At(cli::at::AtArgs),
    Changes(cli::changes::ChangesArgs),
    When(cli::when::WhenArgs),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Info,
    Tree,
    Signals,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Tree => "tree",
            Self::Signals => "signals",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum CommandData {
    Info(info::InfoData),
    Tree(Vec<tree::TreeEntry>),
    Signals(Vec<signals::SignalEntry>),
}

#[derive(Debug, Serialize)]
pub struct CommandResult {
    #[serde(skip)]
    pub command: CommandName,
    #[serde(skip)]
    pub human: bool,
    pub data: CommandData,
    pub warnings: Vec<String>,
}

pub fn run(command: Command) -> Result<CommandResult, WavepeekError> {
    match command {
        Command::Schema(args) => schema::run(args),
        Command::Info(args) => info::run(args),
        Command::Tree(args) => tree::run(args),
        Command::Signals(args) => signals::run(args),
        Command::At(args) => at::run(args),
        Command::Changes(args) => changes::run(args),
        Command::When(args) => when::run(args),
    }
}
