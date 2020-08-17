use super::Command;
use async_trait::async_trait;
use asyncmigrate::{Migration, MigrationError};
use clap::{App, Arg, ArgMatches};

pub struct StatusCommand;

#[async_trait]
impl Command for StatusCommand {
    fn command_name(&self) -> &'static str {
        "status"
    }
    fn config_subcommand(&self, app: App<'static, 'static>) -> App<'static, 'static> {
        crate::utils::common_args(app.about("Rollback database")).arg(
            Arg::with_name("group_name")
                .index(1)
                .help("Target group name")
                .takes_value(true)
                .required(true),
        )
    }
    async fn run(&self, matches: &ArgMatches<'static>) -> Result<(), MigrationError> {
        let config = crate::utils::load_config(matches)?;
        let mut connect = crate::utils::connect(&config).await?;

        connect
            .rollback(
                matches.value_of("group_name").unwrap(),
                Some(matches.value_of("count").unwrap().parse()?),
            )
            .await?;
        Ok(())
    }
}
