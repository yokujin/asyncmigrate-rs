use asyncmigrate::{Connection, MigrationError, MigrationErrorKind};
use clap::{App, Arg, ArgMatches};
use failure::ResultExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub fn common_args(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
        Arg::with_name("url")
            .short("u")
            .long("url")
            .takes_value(true)
            .help("Database connection URL"),
    )
    .arg(
        Arg::with_name("config")
            .short("c")
            .long("config")
            .takes_value(true)
            .help("Configuration file path"),
    )
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MigrationConfig {
    pub database_url: Option<String>,
    pub changesets: Vec<MigrationConfigSet>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MigrationConfigSet {
    pub group_name: String,
    pub directory: String,
}

pub fn load_config(matches: &ArgMatches<'static>) -> Result<MigrationConfig, MigrationError> {
    let mut loaded_config = load_config_try(matches)?;
    if let Some(db_url) = matches.value_of("url") {
        loaded_config.database_url = Some(db_url.to_string())
    }
    Ok(loaded_config)
}

fn load_config_try(matches: &ArgMatches<'static>) -> Result<MigrationConfig, MigrationError> {
    let (config_path, mut migration_config) = if let Some(path) = matches.value_of("config") {
        //println!("loading config from {}", path);
        (
            path,
            serde_json::from_reader(fs::File::open(path)?)
                .context(MigrationErrorKind::OtherError("Cannot parse config"))?,
        )
    } else if let Ok(file) = fs::File::open("dbmigration.json") {
        //println!("loading config from current directory");
        (
            "./dbmigration.json",
            serde_json::from_reader(file)
                .context(MigrationErrorKind::OtherError("Cannot parse config"))?,
        )
    } else {
        //println!("config file is not found");
        (
            "./dbmigration.json",
            MigrationConfig {
                database_url: None,
                changesets: vec![],
            },
        )
    };

    let base_path = Path::new(config_path)
        .parent()
        .unwrap_or_else(|| Path::new("/"));

    for x in migration_config.changesets.iter_mut() {
        let new_path = base_path.join(&x.directory);
        x.directory = new_path.to_str().unwrap().to_string();
    }

    Ok(migration_config)
}

pub async fn connect(config: &MigrationConfig) -> Result<Connection, MigrationError> {
    let url = config
        .database_url
        .as_ref()
        .ok_or(MigrationErrorKind::OtherError("No connection URL"))?;
    asyncmigrate::connect(url).await
}
