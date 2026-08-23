use crate::cli::skill::SkillArgs;
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;

pub fn run(args: SkillArgs) -> Result<CommandResult, WavepeekError> {
    crate::skill::materialize(&args.directory)?;
    Ok(CommandResult {
        command: CommandName::Skill,
        output_mode: crate::output_mode::OutputMode::Human,
        human_options: HumanRenderOptions::default(),
        scope: None,
        summary_only: false,
        data: CommandData::Text(format!(
            "Extracted wavepeek skill to {}\n",
            args.directory.display()
        )),
        summary: None,
        diagnostics: Vec::new(),
    })
}
