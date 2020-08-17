use super::Command;
use async_trait::async_trait;
use asyncmigrate::{MigrationError, MigrationErrorKind};
use clap::{App, Arg, ArgMatches};
use failure::ResultExt;
use rustyline::Editor;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct SetupCommand;

#[async_trait]
impl Command for SetupCommand {
    fn command_name(&self) -> &'static str {
        "setup"
    }
    fn config_subcommand(&self, app: App<'static, 'static>) -> App<'static, 'static> {
        app.about("initialize dbmigration config file").arg(
            Arg::with_name("directory")
                .index(1)
                .help("initialize target directory")
                .takes_value(true),
        )
    }
    async fn run(&self, matches: &ArgMatches<'static>) -> Result<(), MigrationError> {
        let mut rl = Editor::<()>::new();
        let host = rl
            .readline_with_initial("PostgreSQL host: ", ("localhost", ""))
            .context(MigrationErrorKind::OtherError("Cannot get host"))?;
        rl.clear_history();
        let port = rl
            .readline_with_initial("PostgreSQL port: ", ("5432", ""))
            .context(MigrationErrorKind::OtherError("Cannot get host"))?;
        rl.clear_history();
        let dbname = rl
            .readline_with_initial("PostgreSQL database name: ", ("postgres", ""))
            .context(MigrationErrorKind::OtherError("Cannot get database name"))?;
        rl.clear_history();
        let user = rl
            .readline_with_initial("PostgreSQL user: ", ("postgres", ""))
            .context(MigrationErrorKind::OtherError("Cannot get user"))?;
        let pass = rpassword::read_password_from_tty(Some("Password: "))
            .context(MigrationErrorKind::OtherError("Cannot get password"))?;
        let connection_url = format!("postgres://{}:{}@{}:{}/{}", user, pass, host, port, dbname);

        rl.clear_history();
        let group_name = rl
            .readline_with_initial("Migration group: ", ("default", ""))
            .context(MigrationErrorKind::OtherError("Cannot get migration group"))?;

        let initialize_directory = PathBuf::from(matches.value_of("directory").unwrap_or("."));
        let mut migration_group_path = initialize_directory.clone();
        migration_group_path.push(&group_name);
        let mut config_path = initialize_directory;
        config_path.push("dbmigration.json");

        fs::create_dir_all(&migration_group_path)?;
        let config_writer = fs::File::create(&config_path)?;

        let config = crate::utils::MigrationConfig {
            database_url: Some(connection_url),
            changesets: vec![crate::utils::MigrationConfigSet {
                directory: format!("./{}", group_name),
                group_name,
            }],
        };

        serde_json::to_writer_pretty(config_writer, &config)
            .context(MigrationErrorKind::OtherError("serialize config error"))?;

        fs::File::create(migration_group_path.join("1__start__up.sql"))?
            .write_all(b"CREATE TABLE start_table(id INTEGER PRIMARY KEY);\n")?;
        fs::File::create(migration_group_path.join("1__start__down.sql"))?
            .write_all(b"DROP TABLE start_table;\n")?;

        Ok(())
    }
}
