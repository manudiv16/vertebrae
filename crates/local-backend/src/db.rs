//! Thin internal wrapper around `libsql::Database`.
//!
//! All SQL lives behind this wrapper so the engine can be swapped
//! (e.g. rusqlite fallback) without touching the service implementations.

use libsql::{Database, Rows, Statement, Value};
use std::path::Path;
use std::sync::Arc;

use crate::error::{LocalError, LocalResult};

/// Internal database handle wrapping a libsql connection.
#[derive(Clone)]
pub(crate) struct Db {
    db: Database,
}

impl Db {
    /// Open a local libsql database at `path`, applying PRAGMAs.
    pub(crate) async fn open(path: &Path) -> LocalResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LocalError::Database(format!("cannot create data directory {}: {}", parent.display(), e))
            })?;
        }

        let db = libsql::Builder::new_local(path)
            .build()
            .await
            .map_err(|e| LocalError::Database(format!("libsql open failed: {}", e)))?;

        let conn = db
            .connect()
            .map_err(|e| LocalError::Database(format!("libsql connect failed: {}", e)))?;

        // Apply PRAGMAs
        conn.execute("PRAGMA journal_mode=WAL", ()).await?;
        conn.execute("PRAGMA busy_timeout=5000", ()).await?;
        conn.execute("PRAGMA foreign_keys=ON", ()).await?;

        Ok(Self { db })
    }

    /// Get a connection for query execution.
    pub(crate) fn connect(&self) -> LocalResult<libsql::Connection> {
        self.db
            .connect()
            .map_err(|e| LocalError::Database(format!("libsql connect failed: {}", e)))
    }

    /// Execute a statement and return the number of affected rows.
    pub(crate) async fn execute(
        &self,
        sql: &str,
        params: impl libsql::params::Params,
    ) -> LocalResult<u64> {
        let conn = self.connect()?;
        conn.execute(sql, params)
            .await
            .map_err(|e| LocalError::Query(format!("execute failed: {}", e)))
    }

    /// Execute a query and return the first row (if any).
    pub(crate) async fn query_one(
        &self,
        sql: &str,
        params: impl libsql::params::Params,
    ) -> LocalResult<Option<libsql::Row>> {
        let conn = self.connect()?;
        let mut rows = conn
            .query(sql, params)
            .await
            .map_err(|e| LocalError::Query(format!("query failed: {}", e)))?;
        rows.next()
            .await
            .map_err(|e| LocalError::Query(format!("row fetch failed: {}", e)))
    }

    /// Execute a query and collect all rows.
    pub(crate) async fn query_all(
        &self,
        sql: &str,
        params: impl libsql::params::Params,
    ) -> LocalResult<Vec<libsql::Row>> {
        let conn = self.connect()?;
        let mut rows = conn
            .query(sql, params)
            .await
            .map_err(|e| LocalError::Query(format!("query failed: {}", e)))?;
        let mut result = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| LocalError::Query(format!("row fetch failed: {}", e)))?
        {
            result.push(row);
        }
        Ok(result)
    }

    /// Run a closure inside a transaction.
    pub(crate) async fn transaction<F, T>(&self, f: F) -> LocalResult<T>
    where
        F: FnOnce(libsql::Transaction) -> futures::future::BoxFuture<'_, LocalResult<T>> + Send,
        T: Send,
    {
        let conn = self.connect()?;
        let tx = conn
            .transaction()
            .await
            .map_err(|e| LocalError::Database(format!("tx begin failed: {}", e)))?;
        match f(tx).await {
            Ok(val) => Ok(val),
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}

/// Helpers for extracting values from libsql rows.
pub(crate) trait RowExt {
    fn get_text(&self, idx: i32) -> LocalResult<String>;
    fn get_opt_text(&self, idx: i32) -> LocalResult<Option<String>>;
    fn get_i64(&self, idx: i32) -> LocalResult<i64>;
    fn get_opt_i64(&self, idx: i32) -> LocalResult<Option<i64>>;
    fn get_f64(&self, idx: i32) -> LocalResult<f64>;
    fn get_opt_f64(&self, idx: i32) -> LocalResult<Option<f64>>;
    fn get_bool(&self, idx: i32) -> LocalResult<bool>;
    fn get_opt_bool(&self, idx: i32) -> LocalResult<Option<bool>>;
}

impl RowExt for libsql::Row {
    fn get_text(&self, idx: i32) -> LocalResult<String> {
        self.get::<Option<String>>(idx)
            .map_err(|e| LocalError::Query(format!("row text col {} failed: {}", idx, e)))?
            .ok_or_else(|| LocalError::Query(format!("NULL in non-null text col {}", idx)))
    }

    fn get_opt_text(&self, idx: i32) -> LocalResult<Option<String>> {
        self.get::<Option<String>>(idx)
            .map_err(|e| LocalError::Query(format!("row opt text col {} failed: {}", idx, e)))
    }

    fn get_i64(&self, idx: i32) -> LocalResult<i64> {
        self.get::<i64>(idx)
            .map_err(|e| LocalError::Query(format!("row i64 col {} failed: {}", idx, e)))
    }

    fn get_opt_i64(&self, idx: i32) -> LocalResult<Option<i64>> {
        self.get::<Option<i64>>(idx)
            .map_err(|e| LocalError::Query(format!("row opt i64 col {} failed: {}", idx, e)))
    }

    fn get_f64(&self, idx: i32) -> LocalResult<f64> {
        self.get::<f64>(idx)
            .map_err(|e| LocalError::Query(format!("row f64 col {} failed: {}", idx, e)))
    }

    fn get_opt_f64(&self, idx: i32) -> LocalResult<Option<f64>> {
        self.get::<Option<f64>>(idx)
            .map_err(|e| LocalError::Query(format!("row opt f64 col {} failed: {}", idx, e)))
    }

    fn get_bool(&self, idx: i32) -> LocalResult<bool> {
        self.get::<Option<i64>>(idx)
            .map_err(|e| LocalError::Query(format!("row bool col {} failed: {}", idx, e)))
            .map(|v| v.unwrap_or(0) != 0)
    }

    fn get_opt_bool(&self, idx: i32) -> LocalResult<Option<bool>> {
        self.get::<Option<i64>>(idx)
            .map_err(|e| LocalError::Query(format!("row opt bool col {} failed: {}", idx, e)))
            .map(|v| v.map(|n| n != 0))
    }
}
