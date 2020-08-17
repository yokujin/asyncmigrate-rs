use super::Command;
use async_trait::async_trait;
use asyncmigrate::{Migration, MigrationChangeSets, MigrationError, MigrationErrorKind};
use clap::{App, Arg, ArgMatches};
use failure::ResultExt;

pub struct MigrateCommand;

#[async_trait]
impl Command for MigrateCommand {
    fn command_name(&self) -> &'static str {
        "migrate"
    }
    fn config_subcommand(&self, app: App<'static, 'static>) -> App<'static, 'static> {
        crate::utils::common_args(app.about("Migration database"))
            .arg(
                Arg::with_name("group_name")
                    .index(1)
                    .help("Target group name")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("count")
                    .index(2)
                    .help("# of change sets to apply")
                    .takes_value(true),
            )
    }
    async fn run(&self, matches: &ArgMatches<'static>) -> Result<(), MigrationError> {
        let config = crate::utils::load_config(matches)
            .context(MigrationErrorKind::OtherError("Failed to load config"))?;
        let mut connect = crate::utils::connect(&config).await?;

        for one_change_sets in config.changesets.iter() {
            if let Some(target_group_name) = matches.value_of("group_name") {
                if target_group_name != one_change_sets.group_name {
                    continue;
                }
            }
            println!("loading {}", one_change_sets.directory);
            let local_changesets = MigrationChangeSets::load_dir(
                &one_change_sets.group_name,
                &one_change_sets.directory,
            )
            .context(MigrationErrorKind::OtherError(
                "Failed to load migration SQLs",
            ))?;
            //println!("Processing {}", one_change_sets.group_name);

            connect
                .migrate(
                    &local_changesets,
                    matches.value_of("count").map(|x| x.parse().unwrap()),
                )
                .await?;
        }

        Ok(())
    }
}
