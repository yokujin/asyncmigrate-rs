use thiserror::Error;

/**
 * Data migration error
 */
#[derive(Debug, Error)]
pub enum MigrationError {
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  FormatError(#[from] std::fmt::Error),
  #[error(transparent)]
  Utf8Error(#[from] std::str::Utf8Error),
  #[error(transparent)]
  ParseIntError(#[from] std::num::ParseIntError),
  #[error(transparent)]
  ParseFloatError(#[from] std::num::ParseFloatError),
  #[error(transparent)]
  PostgresError(#[from] tokio_postgres::Error),
  #[error("{0}: V{1}")]
  InconsistentMigrationError(&'static str, i32),
  #[error("Version mismatch: local version: {0} database version: {1}")]
  VersionMismatchError(i32, i32),
  #[error("Error: {0}")]
  OtherError(&'static str),
}
