use super::Command;
use async_trait::async_trait;
use asyncmigrate::{Migration, MigrationChangeSets, MigrationError};
use clap::{App, Arg, ArgMatches};

pub struct RedoCommand;

#[async_trait]
impl Command for RedoCommand {
    fn command_name(&self) -> &'static str {
        "redo"
    }
    fn config_subcommand(&self, app: App<'static, 'static>) -> App<'static, 'static> {
        crate::utils::common_args(app.about("Rollback database"))
            .arg(
                Arg::with_name("group_name")
                    .index(1)
                    .help("Target group name")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("count")
                    .index(2)
                    .help("# of change sets to apply")
                    .takes_value(true)
                    .default_value("1"),
            )
    }
    async fn run(&self, matches: &ArgMatches<'static>) -> Result<(), MigrationError> {
        let config = crate::utils::load_config(matches)?;
        let mut connect = crate::utils::connect(&config).await?;

        let count = matches.value_of("count").unwrap().parse()?;

        connect
            .rollback(matches.value_of("group_name").unwrap(), Some(count))
            .await?;

        for one_change_sets in config.changesets.iter() {
            if let Some(target_group_name) = matches.value_of("group_name") {
                if target_group_name != one_change_sets.group_name {
                    continue;
                }
            }
            let local_changesets = MigrationChangeSets::load_dir(
                &one_change_sets.group_name,
                &one_change_sets.directory,
            )?;

            connect.migrate(&local_changesets, Some(count)).await?;
        }

        Ok(())
    }
}
