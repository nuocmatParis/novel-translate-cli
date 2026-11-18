-- novel-translate-cli — STEP 06 / SERIES SQLITE — DDL MẪU CHO CÁC BẢNG LÕI
-- Status: DESIGN DRAFT, not final production migration. 2026-11-05.
-- Independent SQLite for each series; apply only to a NEW Rust workspace.
-- Foreign keys must be enabled on EVERY connection before transactions.
PRAGMA foreign_keys = ON;

BEGIN IMMEDIATE;

CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY CHECK (version >= 1),
  applied_at_ms INTEGER NOT NULL,
  migration_sha256 TEXT NOT NULL CHECK (length(migration_sha256) = 64)
);

CREATE TABLE series (
  series_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
  default_scope TEXT NOT NULL DEFAULT 'all_text'
    CHECK (default_scope IN ('all_text','story_only')),
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE volumes (
  volume_id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES series(series_id),
  volume_order INTEGER NOT NULL CHECK (volume_order >= 1),
  display_title TEXT NOT NULL CHECK (length(trim(display_title)) > 0),
  content_scope TEXT NOT NULL CHECK (content_scope IN ('all_text','story_only')),
  created_at_ms INTEGER NOT NULL,
  UNIQUE(series_id, volume_order)
);

CREATE TABLE immutable_objects (
  object_id TEXT PRIMARY KEY,
  sha256 TEXT NOT NULL CHECK (length(sha256)=64),
  relative_path TEXT NOT NULL CHECK (length(trim(relative_path))>0),
  byte_length INTEGER NOT NULL CHECK (byte_length >= 0),
  content_type TEXT,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(sha256, byte_length)
);

CREATE TABLE source_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  epub_object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  epub_sha256 TEXT NOT NULL CHECK(length(epub_sha256)=64),
  parser_version TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(volume_id, epub_sha256, parser_version)
);

CREATE TABLE source_resources (
  resource_id TEXT PRIMARY KEY,
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  href TEXT NOT NULL,
  media_type TEXT NOT NULL,
  object_id TEXT REFERENCES immutable_objects(object_id),
  spine_order INTEGER CHECK (spine_order >= 0),
  UNIQUE(snapshot_id,href)
);

CREATE TABLE source_blocks (
  block_id TEXT PRIMARY KEY CHECK(length(block_id)=64),
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  resource_id TEXT NOT NULL REFERENCES source_resources(resource_id),
  node_locator TEXT NOT NULL,
  source_text_sha256 TEXT NOT NULL CHECK(length(source_text_sha256)=64),
  source_text_object_id TEXT REFERENCES immutable_objects(object_id),
  char_length INTEGER NOT NULL CHECK(char_length >= 0),
  UNIQUE(snapshot_id,resource_id,node_locator)
);

CREATE TABLE mapping_versions (
  mapping_id TEXT PRIMARY KEY,
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  version_no INTEGER NOT NULL CHECK(version_no >= 1),
  base_mapping_id TEXT REFERENCES mapping_versions(mapping_id),
  created_at_ms INTEGER NOT NULL,
  UNIQUE(snapshot_id,version_no)
);

CREATE TABLE chapters (
  chapter_id TEXT PRIMARY KEY,
  mapping_id TEXT NOT NULL REFERENCES mapping_versions(mapping_id),
  chapter_order INTEGER NOT NULL CHECK(chapter_order >= 1),
  display_title TEXT NOT NULL,
  UNIQUE(mapping_id,chapter_order)
);

CREATE TABLE chapter_source_ranges (
  chapter_range_id TEXT PRIMARY KEY,
  chapter_id TEXT NOT NULL REFERENCES chapters(chapter_id),
  block_id TEXT NOT NULL REFERENCES source_blocks(block_id),
  start_offset INTEGER NOT NULL CHECK(start_offset >= 0),
  end_offset INTEGER NOT NULL CHECK(end_offset > start_offset),
  order_index INTEGER NOT NULL CHECK(order_index >= 1),
  UNIQUE(chapter_id,order_index)
);

CREATE TABLE content_classifications (
  classification_id TEXT PRIMARY KEY,
  mapping_id TEXT NOT NULL REFERENCES mapping_versions(mapping_id),
  block_id TEXT NOT NULL REFERENCES source_blocks(block_id),
  start_offset INTEGER NOT NULL CHECK(start_offset >= 0),
  end_offset INTEGER NOT NULL CHECK(end_offset > start_offset),
  classification TEXT NOT NULL CHECK (classification IN ('translate','keep_original','image','unresolved')),
  evidence_text TEXT
);

CREATE TABLE generations (
  generation_id TEXT PRIMARY KEY,
  parent_generation_id TEXT REFERENCES generations(generation_id),
  created_at_ms INTEGER NOT NULL,
  checksum_sha256 TEXT NOT NULL CHECK(length(checksum_sha256)=64),
  change_manifest_object_id TEXT REFERENCES immutable_objects(object_id)
);

CREATE TABLE current_generation (
  singleton_id INTEGER PRIMARY KEY CHECK(singleton_id=1),
  generation_id TEXT NOT NULL REFERENCES generations(generation_id)
);

CREATE TABLE tasks (
  task_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  execution_status TEXT NOT NULL CHECK(execution_status IN
     ('queued','running','pausing','paused','waiting_user','operational_error','completed','cancelled')),
  run_mode TEXT NOT NULL CHECK(run_mode IN ('selected','to_end')),
  scope_snapshot_json TEXT NOT NULL,
  config_snapshot_json TEXT NOT NULL,
  next_action_json TEXT,
  current_step TEXT,
  model_call_count INTEGER NOT NULL DEFAULT 0 CHECK(model_call_count >= 0),
  optional_call_cap INTEGER CHECK(optional_call_cap > 0),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE task_checkpoints (
  checkpoint_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  generation_id TEXT REFERENCES generations(generation_id),
  step_code TEXT NOT NULL,
  checkpoint_payload_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE translation_slots (
  slot_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  block_id TEXT NOT NULL REFERENCES source_blocks(block_id),
  start_offset INTEGER NOT NULL CHECK(start_offset >= 0),
  end_offset INTEGER NOT NULL CHECK(end_offset > start_offset),
  UNIQUE(snapshot_id,block_id,start_offset,end_offset)
);

CREATE TABLE translation_revisions (
  revision_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  base_revision_id TEXT REFERENCES translation_revisions(revision_id),
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  mapping_id TEXT NOT NULL REFERENCES mapping_versions(mapping_id),
  content_object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  author_kind TEXT NOT NULL CHECK(author_kind IN ('ai','user')),
  created_by_task_id TEXT REFERENCES tasks(task_id),
  created_generation_id TEXT REFERENCES generations(generation_id),
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE translation_revision_ranges (
  revision_range_id TEXT PRIMARY KEY,
  revision_id TEXT NOT NULL REFERENCES translation_revisions(revision_id),
  slot_id TEXT NOT NULL REFERENCES translation_slots(slot_id),
  order_index INTEGER NOT NULL CHECK(order_index >= 1),
  UNIQUE(revision_id,order_index),
  UNIQUE(revision_id,slot_id)
);

CREATE TABLE current_translations (
  slot_id TEXT PRIMARY KEY REFERENCES translation_slots(slot_id),
  revision_id TEXT REFERENCES translation_revisions(revision_id),
  quality_status TEXT NOT NULL CHECK(quality_status IN
    ('not_translated','pending_review','needs_repair','needs_verification','passed','user_exception')),
  validity_status TEXT NOT NULL CHECK(validity_status IN ('current','recheck_required')),
  generation_id TEXT NOT NULL REFERENCES generations(generation_id)
);

CREATE TABLE review_runs (
  review_run_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  mapping_id TEXT NOT NULL REFERENCES mapping_versions(mapping_id),
  review_scope TEXT NOT NULL CHECK(review_scope IN ('unit','chapter')),
  conclusion TEXT NOT NULL CHECK(conclusion IN ('pending','passed','needs_repair','needs_verification','user_exception')),
  started_at_ms INTEGER NOT NULL,
  committed_generation_id TEXT REFERENCES generations(generation_id)
);

CREATE TABLE review_targets (
  review_run_id TEXT NOT NULL REFERENCES review_runs(review_run_id),
  revision_id TEXT NOT NULL REFERENCES translation_revisions(revision_id),
  PRIMARY KEY(review_run_id,revision_id)
);

CREATE TABLE review_coverage (
  coverage_id TEXT PRIMARY KEY,
  review_run_id TEXT NOT NULL REFERENCES review_runs(review_run_id),
  direction TEXT NOT NULL CHECK(direction IN ('source_to_translation','translation_to_source','scene_join')),
  block_id TEXT REFERENCES source_blocks(block_id),
  start_offset INTEGER CHECK(start_offset >= 0),
  end_offset INTEGER,
  coverage_status TEXT NOT NULL CHECK(coverage_status IN ('checked','unresolved')),
  CHECK (start_offset IS NULL OR (end_offset IS NOT NULL AND end_offset > start_offset))
);

CREATE TABLE review_issues (
  issue_id TEXT PRIMARY KEY,
  review_run_id TEXT NOT NULL REFERENCES review_runs(review_run_id),
  severity TEXT NOT NULL CHECK(severity IN ('S0','S1','S2','S3')),
  repair_scope TEXT CHECK(repair_scope IN ('M1','M2','M3','M4')),
  issue_type TEXT NOT NULL,
  explanation TEXT NOT NULL,
  evidence_status TEXT NOT NULL CHECK(evidence_status IN ('candidate','verified','disputed','insufficient')),
  status TEXT NOT NULL CHECK(status IN ('open','resolved','needs_verification','excepted'))
);

CREATE TABLE review_evidence (
  evidence_id TEXT PRIMARY KEY,
  issue_id TEXT NOT NULL REFERENCES review_issues(issue_id),
  source_block_id TEXT REFERENCES source_blocks(block_id),
  source_quote TEXT,
  translation_revision_id TEXT REFERENCES translation_revisions(revision_id),
  target_quote TEXT,
  quote_verified INTEGER NOT NULL DEFAULT 0 CHECK(quote_verified IN (0,1))
);

CREATE TABLE skill_files (
  skill_file_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK(kind IN ('common','series')),
  relative_path TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK(enabled IN (0,1)),
  UNIQUE(kind,relative_path)
);

CREATE TABLE skill_versions (
  skill_version_id TEXT PRIMARY KEY,
  skill_file_id TEXT NOT NULL REFERENCES skill_files(skill_file_id),
  version_no INTEGER NOT NULL CHECK(version_no>=1),
  content_sha256 TEXT NOT NULL CHECK(length(content_sha256)=64),
  created_at_ms INTEGER NOT NULL,
  UNIQUE(skill_file_id,version_no)
);

CREATE TABLE rule_decisions (
  decision_id TEXT PRIMARY KEY,
  decision_type TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE rule_decision_versions (
  decision_version_id TEXT PRIMARY KEY,
  decision_id TEXT NOT NULL REFERENCES rule_decisions(decision_id),
  version_no INTEGER NOT NULL CHECK(version_no>=1),
  scope_json TEXT NOT NULL,
  value_json TEXT NOT NULL,
  evidence_json TEXT NOT NULL,
  approved_by TEXT NOT NULL,
  approved_at_ms INTEGER NOT NULL,
  active_generation_id TEXT REFERENCES generations(generation_id),
  UNIQUE(decision_id,version_no)
);

CREATE TABLE skill_publish_operations (
  publish_operation_id TEXT PRIMARY KEY,
  skill_file_id TEXT NOT NULL REFERENCES skill_files(skill_file_id),
  base_sha256 TEXT NOT NULL CHECK(length(base_sha256)=64),
  target_sha256 TEXT NOT NULL CHECK(length(target_sha256)=64),
  status TEXT NOT NULL CHECK(status IN ('intent_saved','file_replaced','verified','activated','conflict')),
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE dependency_edges (
  edge_id TEXT PRIMARY KEY,
  upstream_kind TEXT NOT NULL,
  upstream_id TEXT NOT NULL,
  upstream_version TEXT NOT NULL,
  downstream_kind TEXT NOT NULL,
  downstream_id TEXT NOT NULL,
  downstream_version TEXT NOT NULL,
  scope_json TEXT,
  created_generation_id TEXT REFERENCES generations(generation_id),
  UNIQUE(upstream_kind,upstream_id,upstream_version,downstream_kind,downstream_id,downstream_version)
);

CREATE TABLE dependency_invalidations (
  invalidation_id TEXT PRIMARY KEY,
  edge_id TEXT REFERENCES dependency_edges(edge_id),
  affected_kind TEXT NOT NULL,
  affected_id TEXT NOT NULL,
  cause TEXT NOT NULL,
  generation_id TEXT NOT NULL REFERENCES generations(generation_id),
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE model_requests (
  request_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  role TEXT NOT NULL CHECK(role IN ('translation','review','research')),
  requested_model TEXT NOT NULL,
  request_contract_version TEXT NOT NULL,
  input_dependency_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE model_attempts (
  attempt_id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL REFERENCES model_requests(request_id),
  attempt_no INTEGER NOT NULL CHECK(attempt_no >= 1),
  provider TEXT NOT NULL,
  actual_model TEXT,
  provider_request_id TEXT,
  deadline_ms INTEGER,
  status TEXT NOT NULL CHECK(status IN ('prepared','sent','uncertain','completed','failed','timed_out','cancelled')),
  sent_at_ms INTEGER,
  completed_at_ms INTEGER,
  finish_reason TEXT,
  result_object_id TEXT REFERENCES immutable_objects(object_id),
  UNIQUE(request_id,attempt_no)
);

CREATE TABLE model_attempt_usage (
  usage_id TEXT PRIMARY KEY,
  attempt_id TEXT NOT NULL REFERENCES model_attempts(attempt_id),
  metric_kind TEXT NOT NULL,
  metric_value INTEGER NOT NULL CHECK(metric_value >= 0),
  reported_by TEXT NOT NULL CHECK(reported_by IN ('provider','estimated','reconciled')),
  UNIQUE(attempt_id,metric_kind,reported_by)
);

CREATE TABLE model_price_versions (
  price_version_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  model_id TEXT NOT NULL,
  currency TEXT NOT NULL,
  unit_kind TEXT NOT NULL,
  price_decimal TEXT NOT NULL,
  effective_at_ms INTEGER NOT NULL,
  source TEXT NOT NULL
);

CREATE TABLE cost_reservations (
  reservation_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  request_id TEXT NOT NULL REFERENCES model_requests(request_id),
  currency TEXT NOT NULL,
  amount_decimal TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active','settled','released','unknown_pending')),
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE cost_ledger (
  ledger_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  attempt_id TEXT REFERENCES model_attempts(attempt_id),
  entry_kind TEXT NOT NULL CHECK(entry_kind IN ('model','research_tool','adjustment')),
  amount_decimal TEXT,
  currency TEXT NOT NULL,
  price_version_id TEXT REFERENCES model_price_versions(price_version_id),
  determination TEXT NOT NULL CHECK(determination IN ('provider_reported','estimated','unknown','reconciled')),
  created_at_ms INTEGER NOT NULL,
  CHECK (determination = 'unknown' OR amount_decimal IS NOT NULL)
);

CREATE TABLE prepared_export_bundles (
  bundle_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  manifest_object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  manifest_sha256 TEXT NOT NULL CHECK(length(manifest_sha256)=64),
  manifest_schema_version INTEGER NOT NULL CHECK(manifest_schema_version>=1),
  base_generation_id TEXT NOT NULL REFERENCES generations(generation_id),
  created_at_ms INTEGER NOT NULL
);

CREATE TABLE prepared_export_items (
  bundle_item_id TEXT PRIMARY KEY,
  bundle_id TEXT NOT NULL REFERENCES prepared_export_bundles(bundle_id),
  item_kind TEXT NOT NULL,
  item_id TEXT NOT NULL,
  item_version TEXT NOT NULL,
  item_sha256 TEXT,
  UNIQUE(bundle_id,item_kind,item_id,item_version)
);

CREATE TABLE export_versions (
  export_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  version_no INTEGER NOT NULL CHECK(version_no>=1),
  bundle_id TEXT NOT NULL REFERENCES prepared_export_bundles(bundle_id),
  status TEXT NOT NULL CHECK(status IN ('reserved','building','validated','published','failed')),
  epub_object_id TEXT REFERENCES immutable_objects(object_id),
  final_relative_path TEXT,
  validation_report_object_id TEXT REFERENCES immutable_objects(object_id),
  created_at_ms INTEGER NOT NULL,
  published_at_ms INTEGER,
  UNIQUE(volume_id,version_no)
);

-- Additional workflow/task and content evidence records.
CREATE TABLE task_scopes (
  scope_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  chapter_id TEXT REFERENCES chapters(chapter_id),
  mapping_id TEXT NOT NULL REFERENCES mapping_versions(mapping_id),
  source_scope_json TEXT NOT NULL,
  order_index INTEGER NOT NULL CHECK(order_index>=0),
  UNIQUE(task_id,order_index)
);
CREATE TABLE task_steps (
  step_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  step_kind TEXT NOT NULL,
  step_status TEXT NOT NULL CHECK(step_status IN ('pending','running','waiting','completed','failed','skipped')),
  step_order INTEGER NOT NULL CHECK(step_order>=1),
  input_versions_json TEXT NOT NULL,
  output_reference_json TEXT,
  UNIQUE(task_id,step_order)
);
CREATE TABLE task_wait_conditions (
  wait_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  wait_kind TEXT NOT NULL,
  affected_scope_json TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active','resolved','cancelled')),
  created_at_ms INTEGER NOT NULL,
  resolved_at_ms INTEGER
);
CREATE TABLE task_state_events (
  event_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL REFERENCES tasks(task_id),
  from_status TEXT,
  to_status TEXT NOT NULL,
  reason_code TEXT NOT NULL,
  generation_id TEXT REFERENCES generations(generation_id),
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE task_budgets (
  task_id TEXT PRIMARY KEY REFERENCES tasks(task_id),
  mode TEXT NOT NULL CHECK(mode IN ('limited','unlimited')),
  currency TEXT NOT NULL,
  cost_limit_decimal TEXT,
  inherited_from TEXT NOT NULL CHECK(inherited_from IN ('task','series','tool')),
  configured_at_ms INTEGER NOT NULL,
  CHECK ((mode='limited' AND cost_limit_decimal IS NOT NULL) OR mode='unlimited')
);
CREATE TABLE model_connections (
  connection_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  connection_name TEXT NOT NULL UNIQUE,
  credential_reference TEXT,
  endpoint_reference TEXT,
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE model_capability_versions (
  capability_version_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  model_id TEXT NOT NULL,
  capability_json TEXT NOT NULL,
  source_reference TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE translation_candidates (
  candidate_id TEXT PRIMARY KEY,
  task_id TEXT REFERENCES tasks(task_id),
  request_id TEXT REFERENCES model_requests(request_id),
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  candidate_content_object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  candidate_status TEXT NOT NULL CHECK(candidate_status IN ('waiting_approval','structural_invalid','stale','ready_to_publish','discarded')),
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE review_source_findings (
  finding_id TEXT PRIMARY KEY,
  review_run_id TEXT NOT NULL REFERENCES review_runs(review_run_id),
  source_block_id TEXT NOT NULL REFERENCES source_blocks(block_id),
  finding_kind TEXT NOT NULL,
  source_quote TEXT NOT NULL,
  assertion_json TEXT NOT NULL,
  verification_status TEXT NOT NULL CHECK(verification_status IN ('working','verified','disputed'))
);
CREATE TABLE repair_plans (
  repair_plan_id TEXT PRIMARY KEY,
  review_run_id TEXT NOT NULL REFERENCES review_runs(review_run_id),
  base_generation_id TEXT NOT NULL REFERENCES generations(generation_id),
  proposed_scope TEXT NOT NULL CHECK(proposed_scope IN ('M1','M2','M3','M4')),
  affected_scope_json TEXT NOT NULL,
  m4_verification_reference TEXT,
  status TEXT NOT NULL CHECK(status IN ('proposed','verified','rejected','running','completed','user_wait')),
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE repair_rounds (
  repair_round_id TEXT PRIMARY KEY,
  repair_plan_id TEXT NOT NULL REFERENCES repair_plans(repair_plan_id),
  round_no INTEGER NOT NULL CHECK(round_no >= 1),
  base_revision_id TEXT REFERENCES translation_revisions(revision_id),
  result_revision_id TEXT REFERENCES translation_revisions(revision_id),
  followup_review_run_id TEXT REFERENCES review_runs(review_run_id),
  outcome TEXT NOT NULL CHECK(outcome IN ('pending','not_counted','failed_quality','passed')),
  UNIQUE(repair_plan_id,round_no)
);
CREATE TABLE user_exceptions (
  exception_id TEXT PRIMARY KEY,
  issue_id TEXT NOT NULL REFERENCES review_issues(issue_id),
  revision_id TEXT NOT NULL REFERENCES translation_revisions(revision_id),
  approved_by TEXT NOT NULL,
  reason TEXT NOT NULL,
  affected_scope_json TEXT NOT NULL,
  evidence_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE review_chapter_sets (
  review_run_id TEXT PRIMARY KEY REFERENCES review_runs(review_run_id),
  chapter_id TEXT NOT NULL REFERENCES chapters(chapter_id),
  review_target_manifest_object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  reviewer_conclusion_json TEXT NOT NULL
);
CREATE TABLE rule_candidates (
  candidate_id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES series(series_id),
  candidate_kind TEXT NOT NULL,
  source_scope_json TEXT NOT NULL,
  source_evidence_json TEXT NOT NULL,
  proposal_json TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('pending','approved','rejected','conflict')),
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE approval_batches (
  batch_id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES series(series_id),
  status TEXT NOT NULL CHECK(status IN ('selected','waiting_skill_sync','committed','cancelled','conflict')),
  selected_versions_sha256 TEXT NOT NULL CHECK(length(selected_versions_sha256)=64),
  approver TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  committed_generation_id TEXT REFERENCES generations(generation_id)
);
CREATE TABLE approval_batch_items (
  batch_id TEXT NOT NULL REFERENCES approval_batches(batch_id),
  candidate_id TEXT NOT NULL REFERENCES rule_candidates(candidate_id),
  decision_version_id TEXT REFERENCES rule_decision_versions(decision_version_id),
  chosen_value_json TEXT NOT NULL,
  PRIMARY KEY(batch_id,candidate_id)
);
CREATE TABLE characters (
  character_id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES series(series_id),
  display_name TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE character_aliases (
  alias_id TEXT PRIMARY KEY,
  character_id TEXT NOT NULL REFERENCES characters(character_id),
  alias_original TEXT NOT NULL,
  approved_rule_version_id TEXT REFERENCES rule_decision_versions(decision_version_id),
  scope_json TEXT NOT NULL
);
CREATE TABLE addressing_rule_versions (
  addressing_version_id TEXT PRIMARY KEY,
  decision_version_id TEXT NOT NULL REFERENCES rule_decision_versions(decision_version_id),
  speaker_character_id TEXT NOT NULL REFERENCES characters(character_id),
  listener_character_id TEXT REFERENCES characters(character_id),
  self_reference TEXT NOT NULL,
  listener_reference TEXT NOT NULL,
  scene_scope_json TEXT NOT NULL,
  story_time_scope_json TEXT NOT NULL,
  reader_disclosure_scope_json TEXT NOT NULL
);
CREATE TABLE context_events (
  event_id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES series(series_id),
  entity_ref TEXT,
  source_evidence_json TEXT NOT NULL,
  story_time_json TEXT NOT NULL,
  reader_disclosure_json TEXT NOT NULL,
  verification_status TEXT NOT NULL CHECK(verification_status IN ('working','verified','disputed','unknown')),
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE term_occurrences (
  occurrence_id TEXT PRIMARY KEY,
  series_id TEXT NOT NULL REFERENCES series(series_id),
  original_form TEXT NOT NULL,
  referent_id TEXT,
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  block_id TEXT NOT NULL REFERENCES source_blocks(block_id),
  start_offset INTEGER NOT NULL CHECK(start_offset>=0),
  end_offset INTEGER NOT NULL CHECK(end_offset>start_offset)
);
CREATE TABLE research_requests (
  research_request_id TEXT PRIMARY KEY,
  task_id TEXT REFERENCES tasks(task_id),
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  question TEXT NOT NULL,
  position_scope_json TEXT NOT NULL,
  spoiler_boundary_json TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('queued','running','completed','waiting','failed')),
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE research_runs (
  research_run_id TEXT PRIMARY KEY,
  research_request_id TEXT NOT NULL REFERENCES research_requests(research_request_id),
  queries_used INTEGER NOT NULL CHECK(queries_used>=0),
  pages_read INTEGER NOT NULL CHECK(pages_read>=0),
  started_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER
);
CREATE TABLE research_sources (
  source_id TEXT PRIMARY KEY,
  research_run_id TEXT NOT NULL REFERENCES research_runs(research_run_id),
  source_type TEXT NOT NULL CHECK(source_type IN ('internal','web')),
  source_locator TEXT NOT NULL,
  fetched_at_ms INTEGER NOT NULL,
  excerpt_object_id TEXT REFERENCES immutable_objects(object_id)
);
CREATE TABLE research_evidence (
  evidence_id TEXT PRIMARY KEY,
  research_run_id TEXT NOT NULL REFERENCES research_runs(research_run_id),
  source_id TEXT NOT NULL REFERENCES research_sources(source_id),
  excerpt_sha256 TEXT NOT NULL CHECK(length(excerpt_sha256)=64),
  excerpt_text TEXT NOT NULL,
  entity_ref TEXT,
  story_time_json TEXT NOT NULL,
  reader_disclosure_json TEXT NOT NULL,
  verification_status TEXT NOT NULL CHECK(verification_status IN ('verified','unverified','disputed','unknown_spoiler')),
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE research_cache_entries (
  cache_id TEXT PRIMARY KEY,
  research_key_sha256 TEXT NOT NULL CHECK(length(research_key_sha256)=64),
  dependency_fingerprint TEXT NOT NULL,
  evidence_manifest_object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  expires_at_ms INTEGER,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(research_key_sha256,dependency_fingerprint)
);
CREATE TABLE image_assets (
  image_asset_id TEXT PRIMARY KEY,
  snapshot_id TEXT NOT NULL REFERENCES source_snapshots(snapshot_id),
  source_href TEXT NOT NULL,
  source_object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  UNIQUE(snapshot_id,source_href)
);
CREATE TABLE image_asset_versions (
  image_version_id TEXT PRIMARY KEY,
  image_asset_id TEXT NOT NULL REFERENCES image_assets(image_asset_id),
  selected_kind TEXT NOT NULL CHECK(selected_kind IN ('source','edit')),
  object_id TEXT NOT NULL REFERENCES immutable_objects(object_id),
  content_validation_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL
);
CREATE TABLE volume_metadata_versions (
  metadata_version_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  version_no INTEGER NOT NULL CHECK(version_no>=1),
  metadata_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  UNIQUE(volume_id,version_no)
);
CREATE TABLE generation_changes (
  change_id TEXT PRIMARY KEY,
  generation_id TEXT NOT NULL REFERENCES generations(generation_id),
  entity_kind TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  change_kind TEXT NOT NULL,
  checksum_sha256 TEXT CHECK(checksum_sha256 IS NULL OR length(checksum_sha256)=64)
);
CREATE TABLE markdown_sync_operations (
  operation_id TEXT PRIMARY KEY,
  volume_id TEXT NOT NULL REFERENCES volumes(volume_id),
  base_revision_id TEXT REFERENCES translation_revisions(revision_id),
  base_file_sha256 TEXT CHECK(base_file_sha256 IS NULL OR length(base_file_sha256)=64),
  target_file_sha256 TEXT CHECK(target_file_sha256 IS NULL OR length(target_file_sha256)=64),
  state TEXT NOT NULL CHECK(state IN ('pending','synced','conflict','failed','needs_remap')),
  changed_units_json TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);
CREATE INDEX idx_task_steps_task ON task_steps(task_id,step_order);
CREATE INDEX idx_task_wait ON task_wait_conditions(task_id,status);
CREATE INDEX idx_repair_rounds_plan ON repair_rounds(repair_plan_id,round_no);
CREATE INDEX idx_rule_version_decision ON rule_decision_versions(decision_id,version_no);
CREATE INDEX idx_approval_batch_state ON approval_batches(series_id,status);
CREATE INDEX idx_character_aliases ON character_aliases(character_id);
CREATE INDEX idx_addressing_pair ON addressing_rule_versions(speaker_character_id,listener_character_id);
CREATE INDEX idx_term_occurrences ON term_occurrences(series_id,original_form,snapshot_id);
CREATE INDEX idx_research_run_request ON research_runs(research_request_id);
CREATE INDEX idx_research_evidence_source ON research_evidence(source_id);
CREATE INDEX idx_image_asset_versions ON image_asset_versions(image_asset_id,created_at_ms);
CREATE INDEX idx_generation_changes_gen ON generation_changes(generation_id);

CREATE INDEX idx_volumes_series ON volumes(series_id,volume_order);
CREATE INDEX idx_snapshots_volume ON source_snapshots(volume_id,created_at_ms);
CREATE INDEX idx_source_blocks_snapshot ON source_blocks(snapshot_id,resource_id);
CREATE INDEX idx_chapters_mapping ON chapters(mapping_id,chapter_order);
CREATE INDEX idx_chapter_ranges ON chapter_source_ranges(chapter_id,order_index);
CREATE INDEX idx_classification_mapping ON content_classifications(mapping_id,block_id);
CREATE INDEX idx_tasks_volume_status ON tasks(volume_id,execution_status);
CREATE INDEX idx_checkpoints_task ON task_checkpoints(task_id,created_at_ms);
CREATE INDEX idx_revision_volume ON translation_revisions(volume_id,created_at_ms);
CREATE INDEX idx_revision_ranges_slot ON translation_revision_ranges(slot_id);
CREATE INDEX idx_reviews_volume ON review_runs(volume_id,started_at_ms);
CREATE INDEX idx_review_targets_revision ON review_targets(revision_id);
CREATE INDEX idx_issues_review ON review_issues(review_run_id,severity);
CREATE INDEX idx_dependencies_upstream ON dependency_edges(upstream_kind,upstream_id,upstream_version);
CREATE INDEX idx_dependencies_downstream ON dependency_edges(downstream_kind,downstream_id,downstream_version);
CREATE INDEX idx_requests_task ON model_requests(task_id,created_at_ms);
CREATE INDEX idx_attempts_request ON model_attempts(request_id,attempt_no);
CREATE INDEX idx_cost_task ON cost_ledger(task_id,created_at_ms);
CREATE INDEX idx_exports_volume ON export_versions(volume_id,version_no);

PRAGMA user_version=1;
COMMIT;
