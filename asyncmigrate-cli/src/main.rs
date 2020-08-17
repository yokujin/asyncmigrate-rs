mod commands;
pub mod utils;

use clap::{crate_authors, crate_version, App, AppSettings};

#[tokio::main]
async fn main() -> Result<(), asyncmigrate::MigrationError> {
    let matches = App::new("database migration")
        .version(crate_version!())
        .author(crate_authors!())
        .subcommands(crate::commands::COMMANDS.iter().map(|x| {
            x.cli()
                .setting(AppSettings::ColorAuto)
                .setting(AppSettings::ColoredHelp)
        }))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .get_matches();

    for one_command in commands::COMMANDS {
        if let Some(matches) = matches.subcommand_matches(one_command.command_name()) {
            one_command.run(matches).await?;
            return Ok(());
        }
    }
    unreachable!()
}
