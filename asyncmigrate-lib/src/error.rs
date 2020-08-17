use failure::{Backtrace, Context, Fail};
use std::fmt::{self, Display};
use std::num;

/**
 * Data migration error
 */
#[derive(Debug)]
pub struct MigrationError {
    inner: failure::Context<MigrationErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum MigrationErrorKind {
    #[fail(display = "I/O Error")]
    IoError,
    #[fail(display = "Format Error")]
    FormatError,
    #[fail(display = "Utf8 Error")]
    Utf8Error,
    #[fail(display = "Parse Int Error")]
    ParseIntError,
    #[fail(display = "Parse Float Error")]
    ParseFloatError,
    #[fail(display = "PostgreSQL Error")]
    PostgresError,
    #[fail(display = "{}: V{}", _0, _1)]
    InconsistentMigrationError(&'static str, i32),
    #[fail(
        display = "Version mismatch: local version: {} database version: {}",
        _0, _1
    )]
    VersionMismatchError(i32, i32),
    #[fail(display = "Error: {}", _0)]
    OtherError(&'static str),
}

impl Fail for MigrationError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl MigrationError {
    pub fn kind(&self) -> MigrationErrorKind {
        *self.inner.get_context()
    }
}

impl From<MigrationErrorKind> for MigrationError {
    fn from(kind: MigrationErrorKind) -> MigrationError {
        MigrationError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<MigrationErrorKind>> for MigrationError {
    fn from(inner: Context<MigrationErrorKind>) -> MigrationError {
        MigrationError { inner }
    }
}

impl From<std::io::Error> for MigrationError {
    fn from(e: std::io::Error) -> MigrationError {
        e.context(MigrationErrorKind::IoError).into()
    }
}

impl From<std::str::Utf8Error> for MigrationError {
    fn from(e: std::str::Utf8Error) -> MigrationError {
        e.context(MigrationErrorKind::Utf8Error).into()
    }
}

impl From<num::ParseIntError> for MigrationError {
    fn from(e: num::ParseIntError) -> MigrationError {
        e.context(MigrationErrorKind::ParseIntError).into()
    }
}

impl From<num::ParseFloatError> for MigrationError {
    fn from(e: num::ParseFloatError) -> MigrationError {
        e.context(MigrationErrorKind::ParseFloatError).into()
    }
}

impl From<fmt::Error> for MigrationError {
    fn from(e: fmt::Error) -> MigrationError {
        e.context(MigrationErrorKind::FormatError).into()
    }
}

impl From<tokio_postgres::Error> for MigrationError {
    fn from(e: tokio_postgres::Error) -> MigrationError {
        e.context(MigrationErrorKind::PostgresError).into()
    }
}
