use lazy_static::lazy_static;
use regex::Regex;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{self, prelude::*};
use std::path::Path;
use std::str;

use crate::{MigrationError, MigrationErrorKind};

/**
 * change sets for migration
 */
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationChangeSets {
    /**
     * change set group name
     */
    pub group_name: String,

    /**
     * List of change sets
     */
    pub change_sets: Vec<ChangeSet>,
}

impl MigrationChangeSets {
    /**
     * Load change set from directory
     */
    pub fn load_dir<P: AsRef<Path>>(
        name: &str,
        path: P,
    ) -> Result<MigrationChangeSets, MigrationError> {
        MigrationChangeSets::load(
            name,
            fs::read_dir(path)?.flat_map(|x| x.ok()).flat_map(|x| {
                if let Ok(m) = x.metadata() {
                    if m.is_dir() {
                        None
                    } else if let Some(y) = x.path().to_str() {
                        Some(Cow::Owned(y.to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }),
            |x| match fs::File::open(x).map(|mut y| {
                let mut buffer = Vec::new();
                y.read_to_end(&mut buffer)?;
                Ok(buffer)
            }) {
                Ok(Ok(x)) => Ok(Cow::Owned(x)),
                Ok(Err(e)) => Err(e),
                Err(e) => Err(e),
            },
        )
    }

    /**
     * Load change sets from a RustEmbed asset.
     *
     * ```rust
     * use rust_embed::RustEmbed;
     * use asyncmigrate::MigrationChangeSets;
     * #[derive(RustEmbed)]
     * #[folder = "schema/"]
     * struct Assets;
     *
     * let change_sets = MigrationChangeSets::load_asset("generic", Assets)
     * .unwrap();
     * ```
     */
    pub fn load_asset<A: RustEmbed>(
        name: &str,
        _asset: A,
    ) -> Result<MigrationChangeSets, MigrationError> {
        MigrationChangeSets::load(name, A::iter(), |x| {
            A::get(x).ok_or_else(|| io::ErrorKind::NotFound.into())
        })
    }

    /**
     * Load change sets from filename iterator and file data read function.
     */
    fn load<I, G>(
        name: &str,
        filenames: I,
        get_data: G,
    ) -> Result<MigrationChangeSets, MigrationError>
    where
        I: Iterator<Item = Cow<'static, str>>,
        G: Fn(&str) -> Result<Cow<'static, [u8]>, io::Error>,
    {
        let mut up_sql: HashMap<i32, (ChangeSetVersionName, String)> = HashMap::new();
        let mut down_sql: HashMap<i32, (ChangeSetVersionName, String)> = HashMap::new();
        for entry in filenames {
            if let Some(filename) = Path::new(entry.as_ref())
                .file_name()
                .map(|x| x.to_str())
                .flatten()
            {
                match ParsedName::parse(filename) {
                    Some(ParsedName::Up(x)) => {
                        let data = get_data(entry.as_ref())?;
                        up_sql.insert(
                            x.version,
                            (x, str::from_utf8(data.as_ref()).unwrap().to_string()),
                        );
                    }
                    Some(ParsedName::Down(x)) => {
                        let data = get_data(entry.as_ref())?;
                        down_sql.insert(
                            x.version,
                            (x, str::from_utf8(data.as_ref()).unwrap().to_string()),
                        );
                    }
                    None => (),
                }
            }
        }

        let mut change_sets: Vec<_> = up_sql
            .into_iter()
            .map(|(k, v)| ChangeSet {
                name: v.0,
                up_sql: v.1,
                down_sql: down_sql.remove(&k).map(|x| x.1),
            })
            .collect();
        change_sets.sort();

        Ok(MigrationChangeSets {
            group_name: name.to_string(),
            change_sets,
        })
    }

    /**
     * Create subset of this change sets.
     */
    pub fn subset<R: std::slice::SliceIndex<[ChangeSet], Output = [ChangeSet]>>(
        &self,
        range: R,
    ) -> MigrationChangeSets {
        MigrationChangeSets {
            group_name: self.group_name.to_string(),
            change_sets: self.change_sets[range].to_vec(),
        }
    }

    /**
     * Calculate a difference from the other change sets.
     */
    pub fn calc_diff(
        &self,
        original_sets: &MigrationChangeSets,
    ) -> Result<MigrationChangeSets, MigrationError> {
        for one in self
            .change_sets
            .iter()
            .zip(original_sets.change_sets.iter())
        {
            if one.0.name.version != one.1.name.version {
                return Err(MigrationErrorKind::VersionMismatchError(
                    one.0.name.version,
                    one.1.name.version,
                )
                .into());
            }
            if one.0.name != one.1.name {
                return Err(MigrationErrorKind::InconsistentMigrationError(
                    "Mismatch name",
                    one.0.name.version,
                )
                .into());
            }
            if one.0.up_sql != one.1.up_sql {
                return Err(MigrationErrorKind::InconsistentMigrationError(
                    "Up SQL mismatch",
                    one.0.name.version,
                )
                .into());
            }
            if one.0.down_sql != one.1.down_sql {
                return Err(MigrationErrorKind::InconsistentMigrationError(
                    "Down SQL mismatch",
                    one.0.name.version,
                )
                .into());
            }
        }
        if self.change_sets.len() < original_sets.change_sets.len() {
            return Err(MigrationErrorKind::InconsistentMigrationError(
                "Some migration is not found in local files",
                original_sets.change_sets[self.change_sets.len()]
                    .name
                    .version,
            )
            .into());
        }
        Ok(MigrationChangeSets {
            group_name: self.group_name.to_string(),
            change_sets: self.change_sets[original_sets.change_sets.len()..].to_vec(),
        })
    }
}

/**
 * a change set with upgrade SQL and downgrade SQL
 */
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct ChangeSet {
    pub name: ChangeSetVersionName,
    pub up_sql: String,
    pub down_sql: Option<String>,
}

/**
 * A change set version and a name
 */
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct ChangeSetVersionName {
    pub version: i32,
    pub name: String,
}

impl fmt::Display for ChangeSetVersionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V{} {}", self.version, self.name)
    }
}

impl ChangeSetVersionName {
    pub fn new(version: i32, name: &str) -> ChangeSetVersionName {
        ChangeSetVersionName {
            version,
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ParsedName {
    Up(ChangeSetVersionName),
    Down(ChangeSetVersionName),
}

lazy_static! {
    static ref UP_NAME: Regex = Regex::new(r"^(\d+)__(.+)__up.sql$").unwrap();
    static ref DOWN_NAME: Regex = Regex::new(r"^(\d+)__(.*)__down.sql$").unwrap();
}

impl ParsedName {
    fn parse(name: &str) -> Option<ParsedName> {
        if let Some(cap) = UP_NAME.captures(name) {
            let version = cap.get(1).unwrap().as_str().parse().unwrap();
            let name = cap.get(2).unwrap().as_str();
            Some(ParsedName::Up(ChangeSetVersionName {
                version,
                name: name.to_string(),
            }))
        } else if let Some(cap) = DOWN_NAME.captures(name) {
            let version = cap.get(1).unwrap().as_str().parse().unwrap();
            let name = cap.get(2).unwrap().as_str();
            Some(ParsedName::Down(ChangeSetVersionName {
                version,
                name: name.to_string(),
            }))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_change_set() -> MigrationChangeSets {
        MigrationChangeSets {
            group_name: "generic".to_string(),
            change_sets: vec![
                ChangeSet {
                    name: ChangeSetVersionName::new(1, "setup"),
                    up_sql: include_str!("../schema/001__setup__up.sql").to_string(),
                    down_sql: Some(include_str!("../schema/001__setup__down.sql").to_string()),
                },
                ChangeSet {
                    name: ChangeSetVersionName::new(10, "minor_change"),
                    up_sql: include_str!("../schema/010__minor_change__up.sql").to_string(),
                    down_sql: Some(
                        include_str!("../schema/010__minor_change__down.sql").to_string(),
                    ),
                },
                ChangeSet {
                    name: ChangeSetVersionName::new(11, "patch_change"),
                    up_sql: include_str!("../schema/011__patch_change__up.sql").to_string(),
                    down_sql: None,
                },
                ChangeSet {
                    name: ChangeSetVersionName::new(200, "major_change"),
                    up_sql: include_str!("../schema/200__major_change__up.sql").to_string(),
                    down_sql: Some(
                        include_str!("../schema/200__major_change__down.sql").to_string(),
                    ),
                },
            ],
        }
    }

    #[derive(RustEmbed)]
    #[folder = "schema/"]
    struct Assets;

    #[test]
    fn test_load_assets() {
        let change_sets = MigrationChangeSets::load_asset("generic", Assets).unwrap();
        assert_eq!(change_sets, expected_change_set());
    }

    #[test]
    fn test_load_change_set() {
        let change_sets = MigrationChangeSets::load_dir("generic", "./schema").unwrap();
        assert_eq!(change_sets, expected_change_set());
    }

    #[test]
    fn test_version_compare() {
        assert!(ChangeSetVersionName::new(111, "foo") < ChangeSetVersionName::new(200, "bar"));
        assert!(ChangeSetVersionName::new(101, "foo") < ChangeSetVersionName::new(110, "bar"));
        assert!(ChangeSetVersionName::new(100, "foo") < ChangeSetVersionName::new(101, "bar"));
        assert!(ChangeSetVersionName::new(100, "foo") > ChangeSetVersionName::new(100, "bar"));
        assert!(ChangeSetVersionName::new(100, "foo") == ChangeSetVersionName::new(100, "foo"));
    }

    #[test]
    fn test_parse_name() {
        assert_eq!(
            ParsedName::parse("1__setup__up.sql").unwrap(),
            ParsedName::Up(ChangeSetVersionName {
                version: 1,
                name: "setup".to_string()
            },)
        );
        assert_eq!(
            ParsedName::parse("10__minor_change__up.sql").unwrap(),
            ParsedName::Up(ChangeSetVersionName {
                version: 10,
                name: "minor_change".to_string()
            },)
        );
        assert_eq!(
            ParsedName::parse("11__patch_change__up.sql").unwrap(),
            ParsedName::Up(ChangeSetVersionName {
                version: 11,
                name: "patch_change".to_string()
            },)
        );
        assert_eq!(
            ParsedName::parse("001__setup__down.sql").unwrap(),
            ParsedName::Down(ChangeSetVersionName {
                version: 1,
                name: "setup".to_string()
            },)
        );
    }
}
