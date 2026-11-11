use crate::state::AppState;
use crate::state::logs::LogLevel;
use crate::state::workflow::{ModelRole, StepState, WorkflowStage};
use std::time::{Duration, Instant};

pub struct SimulationEngine {
    last_tick: Instant,
    tick_interval: Duration,
    accumulated: Duration,
}

impl Default for SimulationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationEngine {
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
            tick_interval: Duration::from_millis(1200),
            accumulated: Duration::ZERO,
        }
    }

    pub fn update(&mut self, state: &mut AppState) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick);
        self.last_tick = now;

        if !state.task.is_running || state.task.is_paused {
            return;
        }

        self.accumulated += elapsed;
        if self.accumulated >= self.tick_interval {
            self.accumulated = Duration::ZERO;
            self.step_simulation(state);
        }
    }

    fn step_simulation(&self, state: &mut AppState) {
        // Increment tokens and costs
        state.task.session_tokens += 1250;
        state.task.session_cost_usd += 0.008;

        // Check budget cap
        if !state.task.budget_unlimited {
            if let Some(limit) = state.task.budget_limit_usd {
                if state.task.session_cost_usd >= limit {
                    state.task.is_paused = true;
                    state.task.current_operation =
                        format!("Budget limit of ${:.2} reached. Paused simulation.", limit);
                    state.add_log(
                        LogLevel::Warn,
                        "Budget Limit Reached",
                        "Simulation paused due to cost cap enforcement",
                    );
                    return;
                }
            }
        }

        // Advance workflow state machine
        match state.task.current_stage {
            WorkflowStage::PreparingContext => {
                state.task.stage_states[0] = StepState::Completed;
                state.task.stage_states[1] = StepState::Running;
                state.task.current_stage = WorkflowStage::TranslatingBatch;
                state.task.active_model.model_name = "claude-3-5-sonnet".to_string();
                state.task.active_model.role = ModelRole::Translation;
                state.task.current_operation = format!(
                    "Translating batch {}/{} (blocks #{:03}-#{:03})",
                    state.task.current_batch,
                    state.task.total_batches,
                    state.task.current_batch * 15,
                    (state.task.current_batch + 1) * 15
                );
                state.add_log(
                    LogLevel::Info,
                    "Translation Batch Started",
                    &format!("Translating batch {}/5", state.task.current_batch),
                );
            }
            WorkflowStage::TranslatingBatch => {
                state.task.stage_states[1] = StepState::Completed;
                state.task.stage_states[2] = StepState::Running;
                state.task.current_stage = WorkflowStage::ValidatingStructure;
                state.task.current_operation =
                    "Validating XHTML tags, Ruby annotations & sentence boundaries".to_string();
                state.add_log(
                    LogLevel::Info,
                    "Batch Translation Done",
                    "Received raw translation batch; validating XHTML structure",
                );
            }
            WorkflowStage::ValidatingStructure => {
                state.task.stage_states[2] = StepState::Completed;
                state.task.stage_states[3] = StepState::Running;
                state.task.current_stage = WorkflowStage::ReviewingBatch;
                state.task.active_model.model_name = "gpt-4o".to_string();
                state.task.active_model.role = ModelRole::Review;
                state.task.current_operation =
                    "Evaluating batch quality against Rubric S0-S3 (Terminology & Addressing)"
                        .to_string();
                state.add_log(
                    LogLevel::Info,
                    "Structure Validated",
                    "XHTML tags intact; dispatched to Review model (gpt-4o)",
                );
            }
            WorkflowStage::ReviewingBatch => {
                state.task.stage_states[3] = StepState::Completed;
                state.task.stage_states[4] = StepState::Running;
                state.task.current_stage = WorkflowStage::FixingIssues;
                state.task.current_operation =
                    "Applying M1 scoped corrections for detected nuance flags".to_string();
                state.add_log(
                    LogLevel::Info,
                    "Review Completed",
                    "Assessed batch; applying minor M1 corrections",
                );
            }
            WorkflowStage::FixingIssues => {
                state.task.stage_states[4] = StepState::Completed;

                // Advance batch or chapter
                if state.task.current_batch < state.task.total_batches {
                    state.task.current_batch += 1;
                    state.task.chapter_progress_pct = ((state.task.current_batch as f32
                        / state.task.total_batches as f32)
                        * 100.0) as u16;
                    state.task.stage_states[1] = StepState::Running;
                    state.task.stage_states[2] = StepState::Pending;
                    state.task.stage_states[3] = StepState::Pending;
                    state.task.stage_states[4] = StepState::Pending;
                    state.task.current_stage = WorkflowStage::TranslatingBatch;
                    state.task.active_model.model_name = "claude-3-5-sonnet".to_string();
                    state.task.active_model.role = ModelRole::Translation;
                    state.task.current_operation = format!(
                        "Moving to next batch {}/{}",
                        state.task.current_batch, state.task.total_batches
                    );
                } else {
                    state.task.stage_states[5] = StepState::Running;
                    state.task.current_stage = WorkflowStage::FinalChapterReview;
                    state.task.active_model.model_name = "gpt-4o".to_string();
                    state.task.active_model.role = ModelRole::Review;
                    state.task.current_operation =
                        "Performing full chapter consistency & continuity check".to_string();
                    state.add_log(
                        LogLevel::Info,
                        "All Batches Done",
                        "Commencing final chapter-level review",
                    );
                }
            }
            WorkflowStage::FinalChapterReview => {
                state.task.stage_states[5] = StepState::Completed;
                state.task.is_running = false;
                state.task.chapter_progress_pct = 100;
                state.task.volume_progress_pct = 72;
                state.task.current_operation =
                    "Chapter 04 completed successfully! Ready for Chapter 05.".to_string();
                state.add_log(
                    LogLevel::Info,
                    "Chapter 04 Completed",
                    "Final chapter review passed; chapter marked completed",
                );
            }
        }
    }
}
