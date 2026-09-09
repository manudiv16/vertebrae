-- 0001_init.sql: Initial schema mirroring vertebrae-core domain models.
-- Conventions:
--   ids = TEXT UUID v4
--   enums = TEXT snake_case
--   JSON fields = TEXT (serde_json)
--   timestamps = TEXT RFC3339 UTC
--   booleans = INTEGER 0/1

CREATE TABLE IF NOT EXISTS schema_migrations (
    version     INTEGER PRIMARY KEY,
    applied_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ─── Projects ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    slug        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    path        TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ─── Tasks ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS tasks (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id),
    short_id        TEXT NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT,
    level           TEXT NOT NULL,
    priority        TEXT NOT NULL,
    tags            TEXT,                    -- JSON array of strings
    parent_id       TEXT REFERENCES tasks(id),
    workflow_id     TEXT,
    current_step_id TEXT,
    worktree        TEXT,
    archived        INTEGER NOT NULL DEFAULT 0,
    started_at      TEXT,
    completed_at    TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_short_id ON tasks(short_id);
CREATE INDEX IF NOT EXISTS idx_tasks_archived ON tasks(archived);
CREATE INDEX IF NOT EXISTS idx_tasks_parent_id ON tasks(parent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_workflow_id ON tasks(workflow_id);

-- ─── Task Dependencies ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS task_dependencies (
    task_id       TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    depends_on_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, depends_on_id)
);

CREATE INDEX IF NOT EXISTS idx_task_dependencies_depends ON task_dependencies(depends_on_id);

-- ─── Sections ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS sections (
    id          TEXT PRIMARY KEY,
    task_id     TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    section_type TEXT NOT NULL,
    content     TEXT NOT NULL,
    position    INTEGER NOT NULL,
    done        INTEGER,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_sections_task_id ON sections(task_id);

-- ─── Code Refs ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS code_refs (
    id          TEXT PRIMARY KEY,
    task_id     TEXT REFERENCES tasks(id) ON DELETE CASCADE,
    section_id  TEXT REFERENCES sections(id) ON DELETE CASCADE,
    path        TEXT NOT NULL,
    line_start  INTEGER,
    line_end    INTEGER,
    name        TEXT,
    description TEXT
);

CREATE INDEX IF NOT EXISTS idx_code_refs_task_id ON code_refs(task_id);
CREATE INDEX IF NOT EXISTS idx_code_refs_section_id ON code_refs(section_id);

-- ─── Workflows ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS workflows (
    id                    TEXT PRIMARY KEY,
    project_id            TEXT NOT NULL REFERENCES projects(id),
    short_id              TEXT NOT NULL,
    name                  TEXT NOT NULL,
    initial_step_id       TEXT,
    on_done_workflow_id   TEXT REFERENCES workflows(id),
    on_reject_workflow_id TEXT REFERENCES workflows(id),
    created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_workflows_project_id ON workflows(project_id);
CREATE INDEX IF NOT EXISTS idx_workflows_short_id ON workflows(short_id);

-- ─── Steps ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS steps (
    id                    TEXT PRIMARY KEY,
    workflow_id           TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    short_id              TEXT NOT NULL,
    name                  TEXT NOT NULL,
    step_type             TEXT NOT NULL,
    goal                  TEXT,
    prompt                TEXT,
    output_schema         TEXT,
    route_config          TEXT,
    persistence_options   TEXT,
    agents                TEXT,          -- JSON array of strings
    skills                TEXT,          -- JSON array of strings
    agent_config          TEXT,
    created_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at            TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_steps_workflow_id ON steps(workflow_id);
CREATE INDEX IF NOT EXISTS idx_steps_short_id ON steps(short_id);

-- ─── Workflow Transitions (inter-workflow) ──────────────────────────────────
CREATE TABLE IF NOT EXISTS workflow_transitions (
    id              TEXT PRIMARY KEY,
    from_workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    to_workflow_id   TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    label            TEXT NOT NULL,
    target_step_id   TEXT REFERENCES steps(id),
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_wt_from ON workflow_transitions(from_workflow_id);
CREATE INDEX IF NOT EXISTS idx_wt_to ON workflow_transitions(to_workflow_id);

-- ─── Step Transitions (intra-workflow) ──────────────────────────────────────
CREATE TABLE IF NOT EXISTS step_transitions (
    id          TEXT PRIMARY KEY,
    from_step_id TEXT NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    to_step_id   TEXT NOT NULL REFERENCES steps(id) ON DELETE CASCADE,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_st_from ON step_transitions(from_step_id);
CREATE INDEX IF NOT EXISTS idx_st_to ON step_transitions(to_step_id);

-- ─── Task Runs ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS task_runs (
    id                TEXT PRIMARY KEY,
    task_id           TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    workflow_id       TEXT NOT NULL REFERENCES workflows(id),
    status            TEXT NOT NULL,
    outcome_kind      TEXT,
    max_concurrency   INTEGER,
    parent_task_run_id TEXT REFERENCES task_runs(id),
    started_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at      TEXT,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_task_runs_task_id ON task_runs(task_id);
CREATE INDEX IF NOT EXISTS idx_task_runs_status ON task_runs(status);

-- ─── Step Executions ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS step_executions (
    id                TEXT PRIMARY KEY,
    task_run_id       TEXT NOT NULL REFERENCES task_runs(id) ON DELETE CASCADE,
    task_id           TEXT NOT NULL REFERENCES tasks(id),
    step_id           TEXT NOT NULL REFERENCES steps(id),
    status            TEXT NOT NULL,
    prompt            TEXT,
    output            TEXT,
    transition_result TEXT,
    model             TEXT,
    input_tokens      INTEGER,
    output_tokens     INTEGER,
    cost              REAL,
    duration_ms       INTEGER,
    session_id        TEXT,
    handoff           TEXT,
    started_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    completed_at      TEXT,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_se_task_id ON step_executions(task_id);
CREATE INDEX IF NOT EXISTS idx_se_task_run_id ON step_executions(task_run_id);
CREATE INDEX IF NOT EXISTS idx_se_status ON step_executions(status);

-- ─── Session Logs ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS session_logs (
    id                TEXT PRIMARY KEY,
    step_execution_id TEXT NOT NULL REFERENCES step_executions(id) ON DELETE CASCADE,
    format            TEXT NOT NULL,
    content           TEXT NOT NULL,
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_sl_execution_id ON session_logs(step_execution_id);

-- ─── Artifacts ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS artifacts (
    id            TEXT PRIMARY KEY,
    project_id    TEXT NOT NULL REFERENCES projects(id),
    task_id       TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    logical_name  TEXT NOT NULL,
    body          TEXT NOT NULL,
    metadata      TEXT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(project_id, task_id, logical_name)
);

CREATE INDEX IF NOT EXISTS idx_artifacts_project_id ON artifacts(project_id);
CREATE INDEX IF NOT EXISTS idx_artifacts_task_id ON artifacts(task_id);

-- ─── Changes (event-sourcing table for polling) ─────────────────────────────
CREATE TABLE IF NOT EXISTS changes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    entity      TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    project_id  TEXT NOT NULL,
    event       TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_changes_project_id ON changes(project_id);
