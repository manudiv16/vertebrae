//! Embedded libSQL local backend for Vertebrae.
//!
//! Provides a self-contained backend implementing the core service traits
//! (`TaskService`, `WorkflowService`, `StepService`, `ExecutionService`,
//! `ArtifactService`) over a single libSQL file.

pub mod error;
pub mod paths;

mod db;
mod migrations;

use std::path::Path;
use std::sync::Arc;

use tokio::sync::broadcast;
use vertebrae_core::error::ServiceError;
use vertebrae_core::services::VertebraeServices;

use crate::db::Db;
use crate::error::{LocalError, LocalResult};

/// Event emitted when a local-backend entity is mutated.
///
/// Mirrors the Phoenix event names used by the remote backend so the GUI
/// event mapping is 1:1.
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    /// Entity type (e.g. "task", "workflow", "step", "step_execution", ...)
    pub entity: String,
    /// Entity ID
    pub entity_id: String,
    /// Project ID
    pub project_id: String,
    /// Event name matching Phoenix event names
    /// (e.g. "task_created", "task_updated", "workflow_created", ...)
    pub event: String,
}

/// The embedded local backend.
///
/// Holds the database connection, service implementations, and a broadcast
/// channel for in-process change notifications.
pub struct LocalBackend {
    db: Db,
    services: VertebraeServices,
    change_tx: broadcast::Sender<ChangeEvent>,
}

impl LocalBackend {
    /// Open a local backend at the given path.
    ///
    /// Creates parent directories if needed and runs migrations.
    pub async fn open(path: &Path) -> LocalResult<Self> {
        let db = Db::open(path).await?;
        migrations::run_migrations(&db).await?;

        // Broadcast channel with a small buffer; slow receivers get skipped.
        let (change_tx, _) = broadcast::channel(256);

        // Build service implementations (stubs — will be filled in Steps 2–5).
        let services = build_services(&db, change_tx.clone());

        Ok(Self {
            db,
            services,
            change_tx,
        })
    }

    /// Open a local backend using the default database path.
    ///
    /// Resolution: `VTB_DB_PATH` env → `data_dir()/vertebrae.db`.
    pub async fn open_default() -> LocalResult<Self> {
        let path = paths::default_db_path()?;
        Self::open(&path).await
    }

    /// Get the unified services container.
    pub fn services(&self) -> VertebraeServices {
        self.services.clone()
    }

    /// Subscribe to in-process change notifications.
    ///
    /// The returned receiver will receive a `ChangeEvent` for every mutation.
    /// If the channel is full (slow consumer), events are silently dropped.
    pub fn subscribe(&self) -> broadcast::Receiver<ChangeEvent> {
        self.change_tx.subscribe()
    }

    /// Emit a change event to all subscribers.
    pub(crate) fn emit_change(&self, event: ChangeEvent) {
        // Ignore send errors — no active subscribers is fine.
        let _ = self.change_tx.send(event);
    }

    /// Access the database for internal use.
    pub(crate) fn db(&self) -> &Db {
        &self.db
    }
}

/// Build the services container with local implementations.
fn build_services(db: &Db, change_tx: broadcast::Sender<ChangeEvent>) -> VertebraeServices {
    // Stub implementations — will be replaced as each service module is added.
    let task_svc = Arc::new(crate::tasks::LocalTaskService::new(db.clone(), change_tx.clone()));
    let workflow_svc = Arc::new(crate::workflows::LocalWorkflowService::new(
        db.clone(),
        change_tx.clone(),
    ));
    let step_svc = Arc::new(crate::steps::LocalStepService::new(
        db.clone(),
        change_tx.clone(),
    ));
    let execution_svc = Arc::new(crate::executions::LocalExecutionService::new(
        db.clone(),
        change_tx.clone(),
    ));
    let artifact_svc = Arc::new(crate::artifacts::LocalArtifactService::new(
        db.clone(),
        change_tx.clone(),
    ));

    VertebraeServices::from_services(task_svc, workflow_svc, execution_svc, step_svc, artifact_svc)
}

// Service modules — will be populated in Steps 2–5.
mod tasks;
mod workflows;
mod steps;
mod executions;
mod artifacts;

// Orchestrator and route evaluator — Step 5.
pub mod orchestrator;
pub mod route;
