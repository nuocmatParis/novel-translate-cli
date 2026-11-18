-- novel-translate-cli — STEP 06 / COORDINATOR SQLITE — DESIGN DRAFT
-- Independent per-user-profile SQLite. Does not contain translation content.
PRAGMA foreign_keys=ON;
BEGIN IMMEDIATE;
CREATE TABLE coordinator_schema_migrations (
  version INTEGER PRIMARY KEY CHECK(version>=1),
  applied_at_ms INTEGER NOT NULL,
  migration_sha256 TEXT NOT NULL CHECK(length(migration_sha256)=64)
);
CREATE TABLE coordinator_state (
  singleton_id INTEGER PRIMARY KEY CHECK(singleton_id=1),
  last_permit_sequence INTEGER NOT NULL DEFAULT 0 CHECK(last_permit_sequence>=0)
);
CREATE TABLE queue_entries (
  queue_id TEXT PRIMARY KEY,
  workspace_ref TEXT NOT NULL,
  task_id TEXT NOT NULL,
  request_kind TEXT NOT NULL CHECK(request_kind IN ('start','resume')),
  idempotency_key TEXT NOT NULL UNIQUE,
  state TEXT NOT NULL CHECK(state IN ('queued','granted','cancelled','consumed')),
  requested_at_ms INTEGER NOT NULL,
  queue_order INTEGER NOT NULL UNIQUE
);
CREATE TABLE active_permit (
  singleton_id INTEGER PRIMARY KEY CHECK(singleton_id=1),
  permit_sequence INTEGER NOT NULL CHECK(permit_sequence>=1),
  queue_id TEXT NOT NULL REFERENCES queue_entries(queue_id),
  workspace_ref TEXT NOT NULL,
  task_id TEXT NOT NULL,
  granted_at_ms INTEGER NOT NULL
);
CREATE TABLE permit_history (
  permit_sequence INTEGER PRIMARY KEY CHECK(permit_sequence>=1),
  queue_id TEXT NOT NULL REFERENCES queue_entries(queue_id),
  workspace_ref TEXT NOT NULL,
  task_id TEXT NOT NULL,
  granted_at_ms INTEGER NOT NULL,
  released_at_ms INTEGER,
  release_reason TEXT
);
CREATE TABLE handoff_operations (
  handoff_id TEXT PRIMARY KEY,
  permit_sequence INTEGER NOT NULL REFERENCES permit_history(permit_sequence),
  state TEXT NOT NULL CHECK(state IN ('intent_saved','checkpoint_ack','released','needs_reconcile')),
  expected_task_checkpoint_id TEXT,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);
CREATE TABLE reconciliation_records (
  reconciliation_id TEXT PRIMARY KEY,
  workspace_ref TEXT NOT NULL,
  task_id TEXT NOT NULL,
  permit_sequence INTEGER,
  finding TEXT NOT NULL,
  resolution_state TEXT NOT NULL CHECK(resolution_state IN ('pending','resolved','needs_user')),
  created_at_ms INTEGER NOT NULL
);
CREATE INDEX idx_queue_active ON queue_entries(state,queue_order);
CREATE INDEX idx_handoff_permit ON handoff_operations(permit_sequence,state);
CREATE INDEX idx_reconcile_task ON reconciliation_records(workspace_ref,task_id,created_at_ms);
PRAGMA user_version=1;
COMMIT;
