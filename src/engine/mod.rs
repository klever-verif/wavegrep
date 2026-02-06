pub mod at;
pub mod changes;
pub mod info;
pub mod schema;
pub mod signals;
pub mod tree;
pub mod when;

use crate::error::WavepeekError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    Schema,
    Info,
    Tree,
    Signals,
    At,
    Changes,
    When,
}

pub fn run(command: CommandKind) -> Result<(), WavepeekError> {
    match command {
        CommandKind::Schema => schema::run(),
        CommandKind::Info => info::run(),
        CommandKind::Tree => tree::run(),
        CommandKind::Signals => signals::run(),
        CommandKind::At => at::run(),
        CommandKind::Changes => changes::run(),
        CommandKind::When => when::run(),
    }
}
