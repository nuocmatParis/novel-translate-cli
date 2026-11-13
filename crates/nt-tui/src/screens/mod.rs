//! Screen-specific, truthful empty states for the initial shell.

mod empty_state;
mod help;
mod workspace_form;

pub(crate) use empty_state::empty_state;
pub(crate) use help::help_summary;
pub(crate) use workspace_form::{WorkspaceForm, WorkspaceFormSnapshot};
