# Local Backend + Homebrew Distribution — Master Plan

> **Status:** Approved for implementation
> **Date:** 2026-09-09
> **Context:** vertebrae repo — embedded libSQL local backend replacing Docker-based Sacrum+Postgres for local development.

## Context

Vertebrae (`vtb` CLI, `vtb-daemon`, Tauri GUI) requires an external Sacrum server (Elixir/Phoenix + PostgreSQL) reached via GraphQL + Phoenix WebSocket channels. Goal: make the app fully self-contained — an embedded local backend (single-file libSQL/SQLite DB) that implements the existing core service traits plus a Rust port of Sacrum's orchestration, selected by default with zero configuration.

### Locked Decisions

- **Dual mode**: embedded local backend is default; remote Sacrum only when explicitly configured.
- **Engine**: `libsql` crate in embedded local-file mode (async-native, keeps future Turso path open).
- **Brew**: own tap `manudiv16/homebrew-vertebrae` — formula with `vtb`/`vtb-daemon`/`vtb-gate` + cask with signed/notarized `Vertebrae.app`.
- **No data import** from existing Sacrum instances.
- **Local mode**: no Elixir/Sacrum, no Phoenix channels, no PostgreSQL, no Docker. All state and orchestration run inside Rust binaries over single libSQL file.

### Issue Dependency Graph

```
#1 (crate skeleton)
  ├─▶ #2 (TaskService) ─┐
  ├─▶ #3 (Workflow+Step)┼──▶ #6 (backend selection) ──▶ #7 (daemon local) ──▶ #9 (homebrew)
  ├─▶ #4 (Artifact+Exec)┘                               └─▶ #8 (GUI local) ────┘
  └─▶ #5 (orchestrator+route)
```

- **#1 → #2, #3, #4, #5**: All need the crate skeleton and DB wrapper.
- **#2, #3, #4, #5 → #6**: Backend selection needs the service implementations.
- **#6 → #7, #8**: Daemon and GUI need the backend selection wiring.
- **#9**: Independent but depends on release artifacts from the repo.

---

## Issue #1: Crate Skeleton, DB Wrapper, Schema

### Goal
Add workspace member `crates/local-backend` with libsql DB wrapper, path resolution, embedded migrations, and initial schema.

### Files to Create/Modify

1. **Root `Cargo.toml`**: Add to `members` and `default-members`.
2. **`crates/local-backend/Cargo.toml`**: Package `vertebrae-local-backend`.
3. **`crates/local-backend/src/lib.rs`**: Public API — `LocalBackend::open()`, `LocalBackend::open_default()`, `services()`, `subscribe()`.
4. **`crates/local-backend/src/db.rs`**: Thin `Db` wrapper around `libsql::Builder::new_local(path)`. PRAGMAs: `journal_mode=WAL`, `busy_timeout=5000`, `foreign_keys=ON`.
5. **`crates/local-backend/src/paths.rs`**: `default_db_path()` = `vertebrae_installer::paths::data_dir()/vertebrae.db`. Env override `VTB_DB_PATH`.
6. **`crates/local-backend/src/migrations.rs`**: Embedded `migrations/0001_init.sql` applied transactionally at open.
7. **`crates/local-backend/migrations/0001_init.sql`**: Full schema (see below).
8. **`crates/local-backend/src/error.rs`**: `LocalError` enum with `From<LocalError> for ServiceError`.

### Schema (`0001_init.sql`)

Tables mirroring `crates/core/src/models.rs`:
- `projects(id, slug UNIQUE, name, path, created_at, updated_at)`
- `tasks(id, project_id FK, short_id, title, description, level, priority, tags, parent_id FK, workflow_id, current_step_id, worktree, archived, started_at, completed_at, created_at, updated_at)`
- `task_dependencies(task_id FK, depends_on_id FK, PK)`
- `sections(id, task_id FK, section_type, content, position, done, created_at, updated_at)`
- `code_refs(id, task_id FK, section_id FK, path, line_start, line_end, name, description)`
- `workflows(id, project_id FK, short_id, name, initial_step_id, on_done_workflow_id FK, on_reject_workflow_id FK, created_at, updated_at)`
- `steps(id, workflow_id FK, short_id, name, step_type, goal, prompt, output_schema, route_config, persistence_options, agents, skills, agent_config, created_at, updated_at)`
- `workflow_transitions(id, from_workflow_id FK, to_workflow_id FK, label, target_step_id FK, created_at)`
- `step_transitions(id, from_step_id FK, to_step_id FK, created_at)`
- `task_runs(id, task_id FK, workflow_id FK, status, outcome_kind, max_concurrency, parent_task_run_id FK, started_at, completed_at, created_at)`
- `step_executions(id, task_run_id FK, task_id FK, step_id FK, status, prompt, output, transition_result, model, input_tokens, output_tokens, cost, duration_ms, session_id, handoff, started_at, completed_at, created_at)`
- `session_logs(id, step_execution_id FK, format, content, created_at)`
- `artifacts(id, project_id FK, task_id FK, logical_name, body, metadata, created_at, updated_at, UNIQUE(project_id, task_id, logical_name))`
- `changes(id AUTOINCREMENT, entity, entity_id, project_id, event, created_at)` — event-sourcing table for polling.

### Conventions
- IDs = TEXT UUID v4
- Enums = TEXT snake_case
- JSON fields = TEXT (serde_json)
- Timestamps = TEXT RFC3339 UTC
- Booleans = INTEGER 0/1

### Acceptance Criteria
- `cargo build -p vertebrae-local-backend` succeeds.
- `LocalBackend::open()` creates parent dirs, runs migrations, returns handle.
- `LocalBackend::open_default()` uses `data_dir()/vertebrae.db` or `VTB_DB_PATH`.
- Second `open()` on same path succeeds (no duplicate migrations).

---

## Issue #2: TaskService Implementation

### Goal
Implement all ~38 `TaskService` methods (`crates/core/src/service.rs`) against libSQL.

### Files
- **`crates/local-backend/src/tasks.rs`**: `LocalTaskService` implementing `TaskService` trait.

### Non-Obvious Semantics (from `crates/sacrum-client/src/queries/tasks.rs`)

1. **`short_id`**: First 6 hex chars of UUID. `resolve_short_id` = prefix lookup over `id`/`short_id`, error on zero or multiple matches.
2. **`find_path`**: BFS shortest path over `task_dependencies` in Rust.
3. **`add_dependency`**: Reject cycles (error if path exists from `depends_on_id` back to `task_id`).
4. **Ready queue (`list_ready`)**: Tasks with zero incomplete blockers (`completed_at IS NULL`) and not archived.
5. **Cascade delete**: `DeleteTask(cascade)` deletes children recursively.
6. **Archive = soft delete**: `archived` flag, not physical delete.
7. **Sections**: Singleton types (`goal`, `context`, `current_behavior`, `desired_behavior`) upsert; multi types ordered by `position`; `check_item` toggles `done`.
8. **`set_code_refs`/`sync_dependencies`**: Idempotent delete+insert in one transaction.
9. **Every mutation**: Writes `changes` row in same transaction + pushes to in-process broadcast.

### Key Trait Methods (~38)
- CRUD: `create_task`, `get_task`, `update_task`, `delete_task`, `task_exists`, `resolve_short_id`
- Listing: `list_tasks`, `list_tasks_without_lookups`, `list_tasks_with_lookups`, `list_ready`
- Relationships: `set_parent`, `remove_parent`, `add_dependency`, `remove_dependency`, `sync_dependencies`, `get_blockers`, `get_incomplete_blockers_with_details`, `find_path`, `get_parent`, `get_children`, `get_dependencies`, `get_dependents`
- Sections: `add_section`, `upsert_section`, `remove_sections`, `edit_section_by_ordinal`, `remove_section_by_ordinal`, `mark_checklist_item_done`, `toggle_checklist_item_done`
- Code refs: `add_code_ref`, `remove_code_refs`, `set_code_refs`, `append_ref`, `append_section_ref`
- Workflow: `assign_workflow`, `unassign_workflow`, `set_current_step`, `advance_to_step`
- Show: `get_task_show_bundle`, `get_task_title`, `get_task_titles`

### Acceptance Criteria
- All CRUD round-trips pass with tempdir DB.
- Dependency cycle rejection works.
- Ready queue excludes tasks with incomplete blockers.
- `resolve_short_id` returns error on ambiguity.
- Cascade delete removes all children.

---

## Issue #3: WorkflowService + StepService Implementation

### Goal
Implement `WorkflowService` (~10 ops) and `StepService` (~9 ops) against libSQL.

### Files
- **`crates/local-backend/src/workflows.rs`**: `LocalWorkflowService`.
- **`crates/local-backend/src/steps.rs`**: `LocalStepService`.

### WorkflowService (from `queries/workflows.rs`)
- `create_workflow`, `get_workflow`, `get_workflow_with_tasks`, `resolve_short_id`
- `list_workflows`, `list_workflows_full`, `update_workflow`, `delete_workflow`, `workflow_exists`
- `assign_workflow`, `unassign_workflow`, `get_workflow_info`
- Workflow transitions: `create_workflow_transition`, `list_workflow_transitions`, `list_workflow_transitions_with_names`, `get_transitions_from_workflow`, `get_transitions_to_workflow`, `delete_workflow_transition`, `workflow_transition_exists`

### StepService (from `queries/steps.rs`)
- `create_step`, `create_step_with_id`, `get_step`, `resolve_short_id`, `step_exists`, `get_step_by_id`
- `list_steps_for_workflow`, `update_step`, `delete_step`, `get_initial_step`, `get_transitions`
- `get_finish_steps`, `list_all_steps`

### Validation Rules (Step create/update)
- `route_config` only on `route` steps.
- `finish` steps have no prompt/agent_config/output_schema/outgoing transitions.
- `stop` steps have exactly one outgoing transition.
- `persistence_options` requires `output_schema`.
- `route_config` validated by route V1 validator (Issue #5).
- New route authoring rejects `output_schema`.

### Transition Semantics
- `sync_*_transitions` = idempotent replace in one transaction.
- `move_to_step` validates target is in step's outgoing transitions.
- `advance_to_step` does NOT validate (documented asymmetry).

### Acceptance Criteria
- Workflow CRUD with step creation works.
- Transition sync is idempotent.
- Validation rules reject invalid step configs.
- `get_initial_step` returns correct step.

---

## Issue #4: ArtifactService + ExecutionService Implementation

### Goal
Implement `ArtifactService` and `ExecutionService` records against libSQL.

### Files
- **`crates/local-backend/src/artifacts.rs`**: `LocalArtifactService`.
- **`crates/local-backend/src/executions.rs`**: `LocalExecutionService`.

### ArtifactService
- CRUD: `create_artifact`, `list_artifacts`, `list_task_artifacts`, `get_artifact`, `get_artifact_by_logical_name`, `update_artifact`, `delete_artifact`.
- Upsert by logical name (UNIQUE constraint on `project_id, task_id, logical_name`).
- Pagination with `limit`/`offset` (only paginated surface, matching today).

### ExecutionService
- `list_executions_for_task`, `get_execution`, `list_logs_for_execution`
- `get_latest_execution_for_task`, `update_execution`, `add_log`
- Task-run queries: `active_run`, `task_runs`, `task_run`, `task_run_trace`
- Orchestration entry: `run_step`, `orchestrate_task`, `stop_orchestrator`
- `update_execution_status`, `run_workflow`, `stop_run`

### Semantics
- Orchestration-entry mutations create `task_runs`/`step_executions` rows with status `entered`.
- They do NOT dispatch anything; daemon picks work up by polling.
- `stop_run`/`cancel_step_execution` set status `cancelling`/`stopped`.
- `TaskRunControls` computed from active run presence.
- `pipeline_summary` computed from active executions.
- `step_visit_count` = count of prior executions for task's current step.

### Acceptance Criteria
- Artifact upsert by logical name works (same name updates).
- Task-run queries return correct data.
- Execution status updates work.
- Session logs are created and retrieved.

---

## Issue #5: Orchestrator + Route V1 Evaluator

### Goal
Port Sacrum's `gen_statem` orchestrator to Rust — workflow state machine + deterministic route evaluator.

### Files
- **`crates/local-backend/src/route.rs`**: Route V1 evaluator + validator.
- **`crates/local-backend/src/orchestrator.rs`**: `Orchestrator` with `claim_pending`, `on_execution_finished`, `tick`.

### Route V1 Evaluator (`src/route.rs`)
Spec from `docs/vtb-guide/steps.md` "Deterministic route configuration" (~lines 340–430):

**Envelope**: `version: 1`, `match_policy: "exactly_one"`, `rules[]` with `id`/`when`/`transition`/optional `handoff`, optional `default`.

**Closed refs and operators**:
- `previous_output.route.result` (`eq`/`neq`/`in`)
- `task.level` (`eq`/`neq`/`in`)
- `task.tags` (`contains`/`contains_any`/`contains_all`)
- `execution.step_visit_count` (`eq`/`neq`/`lt`/`lte`/`gt`/`gte`/`in`)

**Condition composition**: non-empty `all`, `any`, `not` nodes.

**Transitions**:
- `intra_workflow` (target = persisted outgoing step)
- `inter_workflow` (target = persisted outgoing workflow transition → destination initial step)

**`exactly_one` semantics**:
- Exactly one rule must match.
- Use `default` only when zero rules match.
- Zero matches and no default → error.
- >1 match → error.
- Errors carry full `route_config` path of offending node.

**Handoff interpolation**: `{{ previous_output.route.result }}` resolved against closed interpolation context; copied into next execution's `handoff`.

**`validate_route_config(value)`**: Rejects unknown refs/operators/nodes/transition types with path-qualified diagnostics.

### Orchestrator (`src/orchestrator.rs`)

1. **`claim_pending(db, limit)`**: Atomic claim — `UPDATE step_executions SET status='in_progress' WHERE status='entered' ORDER BY created_at LIMIT $limit RETURNING id`. `limit` = `VTB_LOCAL_MAX_CONCURRENCY` (default 4).

2. **`on_execution_finished(db, execution_id)`**: Load workflow graph; branch on step_type:
   - `execute` → follow single outgoing transition.
   - `evaluate` → match `transition_result` against outgoing target step `name`.
   - `route` → run route V1 evaluator; advance or chain.
   - `wait_children` → leave waiting until all child tasks have `completed_at`.
   - `human_input` → leave waiting; resumed by CLI/GUI commands.
   - `stop` → end TaskRun, `outcome_kind='run_boundary'`, task stays incomplete.
   - `finish` → set task `completed_at=now`, terminal, no dispatch.
   - Chaining: after terminal/`on_done_workflow_id` → assign next workflow.

3. **Artifact persistence**: For completions with `persistence_options={"artifact":{"logical_name":...}}`, upsert artifact by logical name.

4. **`tick(db)`**: Claim pending + re-check `waiting` runs whose barrier may now be satisfied.

### Acceptance Criteria
- Route evaluator handles documented example config (every operator, `all`/`any`/`not`, `exactly_one`, default-only, zero-match-error, multi-match-error).
- Validation errors have path-qualified diagnostics.
- Orchestrator workflow `execute → route → finish` completes task and chains.
- `wait_children` blocks until child `completed_at`.
- `stop` sets `run_boundary` and keeps task incomplete.
- Artifact upsert by logical name works.

---

## Issue #6: Backend Selection + Service Construction Wiring

### Goal
Add `[backend] mode = "local" | "remote"` to config; wire at all 5 construction sites.

### Files to Modify

1. **`crates/sacrum-client/src/config.rs`**: Add `[backend]` section to `VertebraeConfigFile`.
   - Resolution order: `VTB_BACKEND` env → `[backend].mode` → default.
   - Default = `local` when `[sacrum].url` is absent, else `remote`.

2. **5 Construction Sites** (replace `vertebrae_sacrum_client::from_sacrum(client)` with mode match):
   - `crates/cli/src/main.rs:66-75`
   - `crates/daemon/src/actors/daemon_supervisor.rs:525`
   - `crates/daemon/src/actors/project_supervisor.rs:1084`
   - `crates/gui/src-tauri/src/lib.rs:304`
   - `crates/gui/src-tauri/src/commands/project.rs:461`

3. **`docs/SACRUM_CONFIG.md`**: Update with new `[backend]` section.

### CLI Zero-Config Path
- `run_with_args` resolves mode before `SacrumConfig::load()`.
- In local mode with absent/unwritable config: do not error — create DB via `LocalBackend::open_default()` and proceed.
- Project rows auto-provisioned: when configured `[projects.<slug>] id/path` has no `projects` row, upsert on first service use.

### Daemon-Fleet + Enrollment
- `vtb daemon-*` subcommands and `vtb-daemon enroll` return `ServiceError("requires remote Sacrum backend")` in local mode.

### Acceptance Criteria
- `VTB_BACKEND=local vtb list` works without config file.
- `VTB_BACKEND=remote VTB_URL=... VTB_TOKEN=... vtb list` works against remote.
- Default mode is local when no sacrum URL configured.
- All 5 construction sites respect the mode.

---

## Issue #7: Daemon Local Mode + Single-Instance Lock

### Goal
Add poller actor to daemon for local mode; add flock to prevent double-run.

### Files to Modify

1. **`crates/daemon/src/actors/daemon_supervisor.rs`**:
   - In local mode: do NOT connect Phoenix socket.
   - Spawn poller actor every 1s: `Orchestrator::tick` → `claim_pending` → spawn `StepExecutor` per claimed execution.
   - Cancellation: `StepExecutor` checks execution status each second; `cancelling` triggers existing cancel path.
   - After each completion: call `Orchestrator::on_execution_finished` to drive transitions.

2. **`crates/daemon/src/main.rs`**:
   - Acquire exclusive flock on `<data_dir>/daemon.lock` at startup (non-blocking).
   - If held: exit with `another vtb-daemon instance is running`.
   - Prevents double-run when both `brew services` and GUI launchd agent are enabled.

### Semantics
- `SessionLogEventSink` and completion reporting go through `ExecutionService` trait → local impl, no code change.
- Poller re-checks `waiting` runs on every tick.

### Acceptance Criteria
- `vtb-daemon` in local mode polls and executes steps.
- Second daemon instance exits with clear error message.
- Cancellation via `stop_run` works (sets `cancelling`, daemon observes).

---

## Issue #8: GUI Local Mode

### Goal
Delete Docker-based local backend module; add local mode with `LocalChangeNotifier`.

### Files to Delete
- `crates/gui/src-tauri/src/local_backend/` (`compose.rs`, `provisioning.rs`, `state.rs`, `manifest.rs`, `command.rs`, `assets/compose.yaml`)
- Tauri commands in `crates/gui/src-tauri/src/commands/backend.rs`: `setup_local_backend`, `adopt_local_backend`, `check_local_backend_update`, `apply_approved_local_backend_update`
- React invocations of above commands.
- `local_backend::ensure_for_startup()` call in `lib.rs`.

### Files to Modify

1. **`crates/gui/src-tauri/src/websocket_client.rs`**:
   - In local mode: do NOT open Phoenix socket.
   - Add `LocalChangeNotifier`:
     - Subscribes to `LocalBackend::subscribe()` for in-process change events.
     - Polls `changes` table every 2s using `max(id)` watermark, refetches affected entities.
     - Emits SAME Tauri event names/payloads the frontend already listens for.
   - While viewing active execution: poll `session_logs`/`step_executions` at 1s for live trace.

2. **React first-run flow**:
   - Replace "GUI-managed local backend (Docker)" with "Local (embedded)" — default, no fields.
   - "Remote backend" (URL + token) unchanged.
   - Remove Docker status/adoption panels.

### Acceptance Criteria
- GUI builds without Docker local backend code.
- First-run shows "Local (embedded)" as default option.
- Task created by CLI appears in GUI within ~2s (poll path).
- Existing GUI-managed Docker stacks still work as remote backends (their loopback URL+token in config.toml).

---

## Issue #9: Homebrew Tap + CI

### Goal
Create Homebrew tap repository and wire into release CI.

### New Repository: `manudiv16/homebrew-vertebrae`

#### `Formula/vertebrae.rb`
- Binary formula consuming immutable `vX.Y.Z` release assets.
- Naming per `scripts/build-release-binaries.sh`: `vtb-$VERSION-$BUILD-$TARGET`.
- `on_macos`/`on_linux` + `arm`/`intel` blocks for `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu`.
- Installs `vtb`, `vtb-daemon`, `vtb-gate` into `bin`.
- `service do run [opt_bin/"vtb-daemon"] ... log_path var/"log/vertebrae-daemon.log" end`.
- `caveats`: start daemon with `brew services start vertebrae`; do not enable both brew service and GUI launchd agent.

#### `Casks/vertebrae.rb`
- `url` = `Vertebrae.app.tar.gz` from `vX.Y.Z` tag (codesigned + notarized).
- `app "Vertebrae.app"`.

### CI Integration

#### `.github/workflows/release.yml` publish job
On `channel=release` only, after `scripts/publish-release-assets.sh`:
- Run new `scripts/update-homebrew-tap.sh`:
  - Computes per-target sha256.
  - Renders formula + cask from templates in `scripts/templates/`.
  - Pushes to `manudiv16/homebrew-vertebrae` using `HOMEBREW_TAP_TOKEN` secret.

### Acceptance Criteria
- `brew tap manudiv16/vertebrae && brew install vertebrae` → `vtb --version` runs.
- `brew services start vertebrae` runs lock-respecting daemon.
- `brew install --cask vertebrae` → GUI opens into embedded first-run, creates `vertebrae.db`.

---

## Critical Reference Files

1. `crates/core/src/services.rs` and `crates/core/src/service.rs` — trait boundary (~93 methods across 5 traits).
2. `crates/sacrum-client/src/config.rs` — owns `config.toml` parsing; `[backend]` section goes here.
3. `crates/cli/src/main.rs:66-75` — canonical construction site; replicated at 4 other sites.
4. `docs/vtb-guide/steps.md:340-430` — complete route V1 DSL; authoritative spec for `src/route.rs`.
5. `.github/workflows/release.yml` + `scripts/publish-release-assets.sh` — release asset naming/signing.

## Assumptions & Contingencies

- **Engine swap fallback**: If `libsql` fails to build for any CI target (highest risk: `aarch64-unknown-linux-gnu` cross), replace `src/db.rs` internals with `rusqlite` (`bundled` feature) wrapped in `tokio::task::spawn_blocking` for writes.
- **`evaluate` transition matching**: If `WorkflowTransition` model has no distinct label field, match `transition_result` against target step `name`; single-outgoing-no-match advances, multi-outgoing-no-match fails run.
- **Route semantics**: Implement exactly documented V1 node shapes; anything outside is validation error with full path.
- **Existing Docker stacks**: Treated as plain remote backends after GUI module deletion; no migration, no orphaned-volume cleanup.
- **Tap repository**: `manudiv16/homebrew-vertebrae` must be created and `HOMEBREW_TAP_TOKEN` secret added before Issue #9 CI step can succeed.
