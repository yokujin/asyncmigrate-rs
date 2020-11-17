use crate::MigrationChangeSets;
use crate::MigrationError;
use async_trait::async_trait;

#[async_trait]
pub trait Migration {
    /**
     * Run migration with change sets.
     *
     * equivalence with
     * ```ignore
     * async fn migrate(
     *    &mut self,
     *    changesets: &MigrationChangeSets,
     *    count: Option<usize>,
     * ) -> Result<(), MigrationError>;
     * ```
     */
    async fn migrate(
        &mut self,
        changesets: &MigrationChangeSets,
        count: Option<usize>,
    ) -> Result<(), MigrationError>;

    /**
     * Update rollback SQL schema without downgrading
     *
     * equivalence with
     * ```ignore
     * async fn update_rollback_sql(
     *     &mut self,
     *     changesets: &MigrationChangeSets,
     * ) -> Result<(), MigrationError>;
     * ```
     */
    async fn update_rollback_sql(
        &mut self,
        changesets: &MigrationChangeSets,
    ) -> Result<(), MigrationError>;

    /**
     * Rollback SQL schema
     *
     * equivalence with
     * ```ignore
     * async fn rollback(
     *     &mut self,
     *     group_name: &str,
     *     count: Option<usize>,
     * ) -> Result<(), MigrationError>;
     * ```
     */
    async fn rollback(
        &mut self,
        group_name: &str,
        count: Option<usize>,
    ) -> Result<(), MigrationError>;

    /**
     * Load applied change sets from database.
     *
     * equivalence with
     * ```ignore
     * async fn load_applied_change_sets(
     *     &mut self,
     *     group_name: &str,
     * ) -> Result<MigrationChangeSets, MigrationError>;
     * ```
     */
    async fn load_applied_change_sets(
        &mut self,
        group_name: &str,
    ) -> Result<MigrationChangeSets, MigrationError>;
}

/**
 * Connection.
 */
pub enum Connection {
    /**
     * tokio postgres connection
     */
    #[cfg(feature = "async-postgres")]
    TokioPostgres(tokio_postgres::Client),

    /**
     * Async MySQL connection (not implemented)
     */
    #[cfg(feature = "async-mysql")]
    MySQL(mysql_async::Conn),
}

/**
 * Connect to a database with database URL
 */
pub async fn connect(url: &str) -> Result<Connection, MigrationError> {
    if url.starts_with("postgres://") {
        let (client, connection) = tokio_postgres::connect(url, tokio_postgres::NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        Ok(Connection::TokioPostgres(client))
    } else {
        Err(MigrationError::OtherError("unknown database protocol").into())
    }
}

#[async_trait]
impl Migration for Connection {
    async fn migrate(
        &mut self,
        change_sets: &MigrationChangeSets,
        count: Option<usize>,
    ) -> Result<(), MigrationError> {
        match self {
            #[cfg(feature = "async-postgres")]
            Connection::TokioPostgres(c) => c.migrate(change_sets, count).await,
            #[cfg(feature = "async-mysql")]
            Connection::MySQL(c) => unimplemented!(),
        }
    }
    async fn update_rollback_sql(
        &mut self,
        changesets: &MigrationChangeSets,
    ) -> Result<(), MigrationError> {
        match self {
            #[cfg(feature = "async-postgres")]
            Connection::TokioPostgres(c) => c.update_rollback_sql(changesets).await,
            #[cfg(feature = "async-mysql")]
            Connection::MySQL(c) => unimplemented!(),
        }
    }

    async fn rollback(
        &mut self,
        group_name: &str,
        count: Option<usize>,
    ) -> Result<(), MigrationError> {
        match self {
            #[cfg(feature = "async-postgres")]
            Connection::TokioPostgres(c) => c.rollback(group_name, count).await,
            #[cfg(feature = "async-mysql")]
            Connection::MySQL(c) => unimplemented!(),
        }
    }
    async fn load_applied_change_sets(
        &mut self,
        group_name: &str,
    ) -> Result<MigrationChangeSets, MigrationError> {
        match self {
            #[cfg(feature = "async-postgres")]
            Connection::TokioPostgres(c) => c.load_applied_change_sets(group_name).await,
            #[cfg(feature = "async-mysql")]
            Connection::MySQL(c) => unimplemented!(),
        }
    }
}
