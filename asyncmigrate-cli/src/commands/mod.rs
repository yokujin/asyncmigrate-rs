mod generate;
mod migrate;
mod override_undo_sql;
mod redo;
mod rollback;
mod setup;

use async_trait::async_trait;
use asyncmigrate::MigrationError;
use clap::{crate_authors, crate_version, App, ArgMatches, SubCommand};

pub(crate) const COMMANDS: &[&dyn Command] = &[
    &migrate::MigrateCommand,
    &rollback::RollbackCommand,
    &override_undo_sql::UpdateRollbackSqlCommand,
    &setup::SetupCommand,
    &redo::RedoCommand,
];

#[async_trait]
pub trait Command {
    fn cli(&self) -> App<'static, 'static> {
        self.config_subcommand(SubCommand::with_name(self.command_name()))
            .version(crate_version!())
            .author(crate_authors!())
    }
    fn command_name(&self) -> &'static str;
    fn config_subcommand(&self, app: App<'static, 'static>) -> App<'static, 'static>;
    async fn run(&self, matches: &ArgMatches<'static>) -> Result<(), MigrationError>;
}
