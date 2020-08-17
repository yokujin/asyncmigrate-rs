//! database migration with async support
//!
//! ## Supported database
//! * PostgreSQL
//!
//! ## License
//! Apache License 2.0
//!
//! ## Example
//!
//! ```
//! use asyncmigrate::{MigrationError, Migration};
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed)]
//! #[folder = "schema/"]
//! struct Assets;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), MigrationError> {
//! let mut connection = asyncmigrate::connect(
//!     "postgres://dbmigration-test:dbmigration-test@127.0.0.1:5432/dbmigration-test",
//! )
//! .await?;
//!
//! let changeset = asyncmigrate::MigrationChangeSets::load_asset("default", Assets)?;
//!
//! // Run migration
//! connection.migrate(&changeset, None).await?;
//!
//! // Rollback
//! connection.rollback("default", None).await?;
//! # Ok(())
//! # }
//! ```

mod changeset;
mod driver;
mod error;
pub use changeset::{ChangeSet, ChangeSetVersionName, MigrationChangeSets};
pub use driver::{connect, Connection, Migration};
pub use error::{MigrationError, MigrationErrorKind};

#[cfg(feature = "async-postgres")]
pub mod tokio_postgres;

#[cfg(test)]
mod test {
    use crate::{Migration, MigrationError};
    use rust_embed::RustEmbed;

    #[derive(RustEmbed)]
    #[folder = "schema/"]
    struct Assets;

    #[tokio::test]
    async fn test_connection() -> Result<(), MigrationError> {
        let mut connection = crate::connect(
            "postgres://dbmigration-test:dbmigration-test@127.0.0.1:5432/dbmigration-test",
        )
        .await?;
        let changeset = crate::MigrationChangeSets::load_asset("default", Assets)?;
        // Run migration
        connection.migrate(&changeset, None).await?;

        // Rollback
        connection.rollback("default", None).await?;

        Ok(())
    }
}
