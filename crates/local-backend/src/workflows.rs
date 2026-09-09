//! Local WorkflowService implementation against libSQL.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use async_trait::async_trait;
use vertebrae_core::error::{ServiceError, ServiceResult};
use vertebrae_core::models::{Workflow, WorkflowTransition};
use vertebrae_core::service::TaskService;
use vertebrae_core::workflow_service::{
    AssignResult, CreateWorkflowOptions, UpdateWorkflowOptions, WorkflowInfo, WorkflowService,
    WorkflowSummary, WorkflowTasksBundle,
};

use crate::db::Db;
use crate::ChangeEvent;

pub(crate) struct LocalWorkflowService {
    db: Db,
    #[allow(dead_code)]
    change_tx: broadcast::Sender<ChangeEvent>,
}

impl LocalWorkflowService {
    pub(crate) fn new(db: Db, change_tx: broadcast::Sender<ChangeEvent>) -> Self {
        Self { db, change_tx }
    }
}

#[async_trait]
impl WorkflowService for LocalWorkflowService {
    async fn create_workflow(&self, _options: CreateWorkflowOptions) -> ServiceResult<String> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn get_workflow(&self, _id: &str) -> ServiceResult<Workflow> {
        Err(ServiceError::WorkflowNotFound {
            workflow_id: _id.into(),
        })
    }

    async fn get_workflow_with_tasks(
        &self,
        tasks: &dyn TaskService,
        id: &str,
    ) -> ServiceResult<WorkflowTasksBundle> {
        let workflow = self.get_workflow(id).await?;
        let workflow_id = workflow.id.clone().unwrap_or_default();
        let filter = vertebrae_core::models::TaskFilter::new().with_workflow_id(workflow_id);
        let tasks = tasks.list_tasks(&filter).await?;
        Ok(WorkflowTasksBundle { workflow, tasks })
    }

    async fn resolve_short_id(&self, _prefix: &str) -> ServiceResult<String> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn list_workflows(&self) -> ServiceResult<Vec<WorkflowSummary>> {
        Ok(Vec::new())
    }

    async fn list_workflows_full(&self) -> ServiceResult<Vec<Workflow>> {
        Ok(Vec::new())
    }

    async fn update_workflow(&self, _id: &str, _options: UpdateWorkflowOptions) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn delete_workflow(&self, _id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn workflow_exists(&self, _id: &str) -> ServiceResult<bool> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn assign_workflow(
        &self,
        _task_id: &str,
        _workflow_id: &str,
    ) -> ServiceResult<AssignResult> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn unassign_workflow(&self, _task_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn get_workflow_info(
        &self,
        _workflow_id: &str,
        _current_step_id: Option<&str>,
    ) -> ServiceResult<WorkflowInfo> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn create_workflow_transition(
        &self,
        _from_workflow_id: &str,
        _to_workflow_id: &str,
        _label: &str,
        _target_step_id: Option<&str>,
    ) -> ServiceResult<WorkflowTransition> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn list_workflow_transitions(
        &self,
        _from_workflow_id: Option<&str>,
    ) -> ServiceResult<Vec<WorkflowTransition>> {
        Ok(Vec::new())
    }

    async fn list_workflow_transitions_with_names(
        &self,
        _from_workflow_id: Option<&str>,
    ) -> ServiceResult<(Vec<WorkflowTransition>, HashMap<String, String>)> {
        Ok((Vec::new(), HashMap::new()))
    }

    async fn get_transitions_from_workflow(
        &self,
        _workflow_id: &str,
    ) -> ServiceResult<Vec<WorkflowTransition>> {
        Ok(Vec::new())
    }

    async fn get_transitions_to_workflow(
        &self,
        _workflow_id: &str,
    ) -> ServiceResult<Vec<WorkflowTransition>> {
        Ok(Vec::new())
    }

    async fn delete_workflow_transition(
        &self,
        _from_workflow_id: &str,
        _to_workflow_id: &str,
    ) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }

    async fn workflow_transition_exists(
        &self,
        _from_workflow_id: &str,
        _to_workflow_id: &str,
    ) -> ServiceResult<bool> {
        Err(ServiceError::ValidationFailed {
            message: "LocalWorkflowService: not yet implemented".into(),
        })
    }
}
