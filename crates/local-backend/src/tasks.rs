//! Local TaskService implementation against libSQL.
//!
//! Implements all ~38 TaskService methods.

use std::sync::Arc;
use tokio::sync::broadcast;
use async_trait::async_trait;
use vertebrae_core::error::{ServiceError, ServiceResult};
use vertebrae_core::models::{
    BlockerNode, CodeRef, Section, SectionType, Task, TaskFilter, TaskShowBundle,
};
use vertebrae_core::service::{
    CreateTaskOptions, MutationCallback, MutationEvent, TaskService, UpdateTaskOptions,
};

use crate::db::Db;
use crate::ChangeEvent;

pub(crate) struct LocalTaskService {
    db: Db,
    #[allow(dead_code)]
    change_tx: broadcast::Sender<ChangeEvent>,
    mutation_callback: Option<MutationCallback>,
}

impl LocalTaskService {
    pub(crate) fn new(db: Db, change_tx: broadcast::Sender<ChangeEvent>) -> Self {
        Self {
            db,
            change_tx,
            mutation_callback: None,
        }
    }
}

#[async_trait]
impl TaskService for LocalTaskService {
    async fn create_task(&self, _options: CreateTaskOptions) -> ServiceResult<String> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn get_task(&self, _id: &str) -> ServiceResult<Task> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn resolve_short_id(&self, _prefix: &str) -> ServiceResult<String> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn update_task(&self, _id: &str, _options: UpdateTaskOptions) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn set_current_step(&self, _task_id: &str, _step_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn advance_to_step(&self, _task_id: &str, _step_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn delete_task(&self, _id: &str, _cascade: bool) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn task_exists(&self, _id: &str) -> ServiceResult<bool> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn list_tasks(&self, _filter: &TaskFilter) -> ServiceResult<Vec<Task>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn list_ready(&self) -> ServiceResult<Vec<Task>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn set_parent(&self, _child_id: &str, _parent_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn remove_parent(&self, _child_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn add_dependency(&self, _task_id: &str, _depends_on_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn remove_dependency(&self, _task_id: &str, _depends_on_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn get_blockers(&self, _id: &str) -> ServiceResult<Vec<BlockerNode>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn get_incomplete_blockers_with_details(&self, _id: &str) -> ServiceResult<Vec<Task>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn find_path(&self, _from_id: &str, _to_id: &str) -> ServiceResult<Option<Vec<String>>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn get_parent(&self, _task_id: &str) -> ServiceResult<Option<String>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn get_children(&self, _task_id: &str) -> ServiceResult<Vec<String>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn get_dependencies(&self, _task_id: &str) -> ServiceResult<Vec<String>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn get_dependents(&self, _task_id: &str) -> ServiceResult<Vec<String>> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn add_section(&self, _id: &str, _section: Section) -> ServiceResult<Section> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn remove_sections(
        &self,
        _id: &str,
        _section_type: SectionType,
        _indices: Option<Vec<usize>>,
    ) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn edit_section_by_ordinal(
        &self,
        _id: &str,
        _section_type: SectionType,
        _ordinal: u32,
        _new_content: &str,
    ) -> ServiceResult<Section> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn remove_section_by_ordinal(
        &self,
        _id: &str,
        _section_type: SectionType,
        _ordinal: u32,
    ) -> ServiceResult<Section> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn mark_checklist_item_done(
        &self,
        _id: &str,
        _section_order: u32,
    ) -> ServiceResult<Section> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn toggle_checklist_item_done(
        &self,
        _id: &str,
        _section_order: u32,
    ) -> ServiceResult<Section> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn add_code_ref(&self, _id: &str, _code_ref: CodeRef) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn remove_code_refs(
        &self,
        _id: &str,
        _indices: Option<Vec<usize>>,
    ) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn append_ref(&self, _id: &str, _code_ref: &CodeRef) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn append_section_ref(
        &self,
        _id: &str,
        _section_index: usize,
        _code_ref: &CodeRef,
    ) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn assign_workflow(&self, _task_id: &str, _workflow_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    async fn unassign_workflow(&self, _task_id: &str) -> ServiceResult<()> {
        Err(ServiceError::ValidationFailed {
            message: "LocalTaskService: not yet implemented".into(),
        })
    }

    fn set_mutation_callback(&mut self, _cb: MutationCallback) {
        self.mutation_callback = Some(_cb);
    }
}
