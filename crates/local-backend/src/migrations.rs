//! Embedded migration runner.
//!
//! Migrations live in `migrations/*.sql` and are applied transactionally
//! at database open. `schema_migrations(version INTEGER PK, applied_at TEXT)`
//! tracks applied versions.

use libsql::Transaction;

use crate::db::Db;
use crate::error::{LocalError, LocalResult};

/// The current migration version (matches the highest numbered migration file).
const CURRENT_VERSION: i64 = 1;

/// Embedded migration SQL (included at compile time).
const MIGRATION_0001: &str = include_str!("../migrations/0001_init.sql");

/// Run all pending migrations inside a transaction.
pub(crate) async fn run_migrations(db: &Db) -> LocalResult<()> {
    let applied = get_applied_versions(db).await?;

    if applied.contains(&CURRENT_VERSION) {
        return Ok(());
    }

    db.transaction(|tx| {
        Box::pin(async move {
            // Apply migration 0001
            if !applied.contains(&1) {
                apply_migration(&tx, 1, MIGRATION_0001).await?;
            }

            // Record the highest version as applied
            record_version(&tx, CURRENT_VERSION).await?;

            tx.commit()
                .await
                .map_err(|e| LocalError::Migration(format!("commit failed: {}", e)))
        })
    })
    .await
}

async fn get_applied_versions(db: &Db) -> LocalResult<Vec<i64>> {
    // Ensure the schema_migrations table exists (it's created by the first migration,
    // but we need it to check versions — the migration itself also has CREATE TABLE IF NOT EXISTS).
    db.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version     INTEGER PRIMARY KEY,
            applied_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
        )",
        (),
    )
    .await?;

    let rows = db
        .query_all("SELECT version FROM schema_migrations ORDER BY version", ())
        .await?;

    let mut versions = Vec::new();
    for row in rows {
        let v = row
            .get::<Option<i64>>(0)
            .map_err(|e| LocalError::Query(format!("version read failed: {}", e)))?
            .unwrap_or(0);
        versions.push(v);
    }
    Ok(versions)
}

async fn apply_migration(tx: &Transaction, version: i64, sql: &str) -> LocalResult<()> {
    // Execute the migration SQL
    tx.execute_batch(sql)
        .await
        .map_err(|e| LocalError::Migration(format!("migration {} failed: {}", version, e)))?;

    record_version(tx, version).await
}

async fn record_version(tx: &Transaction, version: i64) -> LocalResult<()> {
    tx.execute(
        "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?)",
        [version],
    )
    .await
    .map_err(|e| LocalError::Migration(format!("record version {} failed: {}", version, e)))?;
    Ok(())
}
