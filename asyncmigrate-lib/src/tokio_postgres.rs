use crate::{ChangeSet, ChangeSetVersionName, Migration, MigrationChangeSets};
use crate::MigrationError;
use async_trait::async_trait;
use tokio_postgres::{Client, Transaction};

#[async_trait]
impl Migration for Client {
    async fn migrate(
        &mut self,
        changesets: &MigrationChangeSets,
        count: Option<usize>,
    ) -> Result<(), MigrationError> {
        let mut transaction = self.transaction().await?;
        migrate_postgres(&mut transaction, changesets, count).await?;
        transaction.commit().await?;
        Ok(())
    }
    async fn update_rollback_sql(
        &mut self,
        changesets: &MigrationChangeSets,
    ) -> Result<(), MigrationError> {
        let mut transaction = self.transaction().await?;
        update_rollback_sql_postgres(&mut transaction, changesets).await?;
        transaction.commit().await?;
        Ok(())
    }
    async fn rollback(
        &mut self,
        group_name: &str,
        count: Option<usize>,
    ) -> Result<(), MigrationError> {
        let mut transaction = self.transaction().await?;
        rollback_postgres(&mut transaction, group_name, count).await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn load_applied_change_sets(
        &mut self,
        group_name: &str,
    ) -> Result<MigrationChangeSets, MigrationError> {
        let mut transaction = self.transaction().await?;
        let changesets = load_migration_set(&mut transaction, group_name).await?;
        transaction.commit().await?;
        Ok(changesets)
    }
}

async fn migrate_postgres(
    client: &mut Transaction<'_>,
    changesets: &MigrationChangeSets,
    count: Option<usize>,
) -> Result<(), MigrationError> {
    let db_migration_set = load_migration_set(client, &changesets.group_name).await?;
    let diff = changesets.calc_diff(&db_migration_set)?;
    let apply_diff = if let Some(count) = count {
        diff.subset(..count)
    } else {
        diff
    };
    for one in apply_diff.change_sets.iter() {
        migrate_one(client, &changesets.group_name, one).await?;
    }
    Ok(())
}

async fn rollback_postgres(
    client: &mut Transaction<'_>,
    group_name: &str,
    count: Option<usize>,
) -> Result<(), MigrationError> {
    let db_migration_set = load_migration_set(client, group_name).await?;
    let count = count.unwrap_or_else(|| db_migration_set.change_sets.len());
    if db_migration_set.change_sets.len() < count {
        return Err(MigrationError::OtherError("No change sets to revert").into());
    }
    for one in db_migration_set.change_sets.iter().rev().take(count) {
        rollback_one(client, &db_migration_set.group_name, one).await?;
    }
    Ok(())
}

async fn update_rollback_sql_postgres(
    client: &mut Transaction<'_>,
    changesets: &MigrationChangeSets,
) -> Result<(), MigrationError> {
    let db_migration_set = load_migration_set(client, &changesets.group_name).await?;
    for (local, db) in changesets
        .change_sets
        .iter()
        .zip(db_migration_set.change_sets.iter())
    {
        if local.name != db.name {
            eprintln!("version number or version name is not match");
            eprintln!("      local version: {}", local.name);
            eprintln!("   database version: {}", db.name);
            return Err(MigrationError::OtherError(
                "version number or version name is not match",
            )
            .into());
        }
        if local.down_sql != db.down_sql {
            update_rollback_sql_one(client, &changesets.group_name, local).await?;
        }
    }
    Ok(())
}

/**
 * Load migration sets from a connected database.
 */
pub async fn load_migration_set(
    client: &mut Transaction<'_>,
    group_name: &str,
) -> Result<MigrationChangeSets, MigrationError> {
    setup_table(client).await?;
    let rows = client
        .query(
            "SELECT group_name, version, name, up_sql, down_sql FROM db_migration WHERE group_name = $1 ORDER BY version",
            &[&group_name],
        )
        .await?;

    let mut change_sets = Vec::new();
    for one in rows {
        change_sets.push(ChangeSet {
            name: ChangeSetVersionName::new(one.get("version"), one.get("name")),
            up_sql: one.get("up_sql"),
            down_sql: one.get("down_sql"),
        });
    }

    Ok(MigrationChangeSets {
        group_name: group_name.to_string(),
        change_sets,
    })
}

async fn setup_table(client: &mut Transaction<'_>) -> Result<(), tokio_postgres::Error> {
    client
        .execute(
            r#"CREATE TABLE IF NOT EXISTS db_migration(
                group_name TEXT, version INTEGER,
                name TEXT NOT NULL, up_sql TEXT NOT NULL, down_sql TEXT,
                PRIMARY KEY(group_name, version));"#,
            &[],
        )
        .await?;
    Ok(())
}

async fn update_rollback_sql_one(
    client: &mut Transaction<'_>,
    group_name: &str,
    changeset: &ChangeSet,
) -> Result<(), MigrationError> {
    println!("update rollback SQL: {}", changeset.name);
    client
        .execute(
            r#"UPDATE db_migration SET down_sql = $1
            WHERE group_name = $2 AND version = $3"#,
            &[&changeset.down_sql, &group_name, &changeset.name.version],
        )
        .await?;
    Ok(())
}

async fn migrate_one(
    client: &mut Transaction<'_>,
    group_name: &str,
    changeset: &ChangeSet,
) -> Result<(), MigrationError> {
    setup_table(client).await?;
    client.batch_execute(&changeset.up_sql).await?;
    println!("migrate: {}", changeset.name);
    client.execute(
        "INSERT INTO db_migration(group_name, version, name, up_sql, down_sql) VALUES($1, $2, $3, $4, $5)",
        &[
            &group_name,
            &changeset.name.version,
            &changeset.name.name,
            &changeset.up_sql,
            &changeset.down_sql

        ]).await?;
    Ok(())
}

async fn rollback_one(
    client: &mut Transaction<'_>,
    group_name: &str,
    changeset: &ChangeSet,
) -> Result<(), MigrationError> {
    println!("revert: {}", changeset.name);
    client
        .execute(
            "DELETE FROM db_migration VALUES WHERE group_name = $1 AND version = $2",
            &[&group_name, &changeset.name.version],
        )
        .await?;
    if let Some(down_sql) = changeset.down_sql.as_ref() {
        client.batch_execute(down_sql).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_load_migration_set() {
        let (mut client, connection) = ::tokio_postgres::connect(
            "postgres://dbmigration-test:dbmigration-test@127.0.0.1:5432/dbmigration-test",
            ::tokio_postgres::NoTls,
        )
        .await
        .unwrap();
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let mut transaction = client.transaction().await.unwrap();
        let migration_set = load_migration_set(&mut transaction, "generic")
            .await
            .unwrap();
        assert_eq!(migration_set.change_sets.len(), 0);
        transaction.rollback().await.unwrap();
        //transaction.commit().await.unwrap();
    }

    #[tokio::test]
    async fn test_apply() {
        let (mut client, connection) = ::tokio_postgres::connect(
            "postgres://dbmigration-test:dbmigration-test@127.0.0.1:5432/dbmigration-test",
            ::tokio_postgres::NoTls,
        )
        .await
        .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        let mut transaction = client.transaction().await.unwrap();
        let change_sets = MigrationChangeSets::load_dir("generic", "./schema").unwrap();

        migrate_postgres(&mut transaction, &change_sets, None)
            .await
            .unwrap();
        transaction
            .execute("SELECT * FROM db_migration", &[])
            .await
            .unwrap();
        transaction
            .execute("SELECT * FROM new_table", &[])
            .await
            .unwrap();
        transaction
            .execute("SELECT * FROM minor_table", &[])
            .await
            .unwrap();
        transaction
            .execute("SELECT * FROM base_table", &[])
            .await
            .unwrap();

        // run undo
        rollback_postgres(&mut transaction, "generic", Some(2))
            .await
            .unwrap();
        let migration_row = transaction
            .query_one("SELECT count(*) cx FROM db_migration", &[])
            .await
            .unwrap();
        assert_eq!(migration_row.get::<_, i64>("cx"), 2);
        // transaction
        //     .execute("SELECT * FROM new_table", &[])
        //     .await
        //     .unwrap_err();
        transaction
            .execute("SELECT * FROM minor_table", &[])
            .await
            .unwrap();
        transaction
            .execute("SELECT * FROM base_table", &[])
            .await
            .unwrap();

        // run undo
        rollback_postgres(&mut transaction, "generic", Some(1))
            .await
            .unwrap();
        let migration_row = transaction
            .query_one("SELECT count(*) cx FROM db_migration", &[])
            .await
            .unwrap();
        assert_eq!(migration_row.get::<_, i64>("cx"), 1);
        // transaction
        //     .execute("SELECT * FROM new_table", &[])
        //     .await
        //     .unwrap_err();
        // transaction
        //     .execute("SELECT * FROM minor_table", &[])
        //     .await
        //     .unwrap_err();
        transaction
            .execute("SELECT * FROM base_table", &[])
            .await
            .unwrap();

        // run undo
        rollback_postgres(&mut transaction, "generic", Some(1))
            .await
            .unwrap();
        let migration_row = transaction
            .query_one("SELECT count(*) cx FROM db_migration", &[])
            .await
            .unwrap();
        assert_eq!(migration_row.get::<_, i64>("cx"), 0);
        // transaction
        //     .execute("SELECT * FROM new_table", &[])
        //     .await
        //     .unwrap_err();
        // transaction
        //     .execute("SELECT * FROM minor_table", &[])
        //     .await
        //     .unwrap_err();
        // transaction
        //     .execute("SELECT * FROM base_table", &[])
        //     .await
        //     .unwrap_err();
        transaction.rollback().await.unwrap();
        //transaction.commit().await.unwrap();
    }
}
