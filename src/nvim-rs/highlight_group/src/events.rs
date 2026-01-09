//! Highlight-related event handling.
//!
//! This module provides types and utilities for handling events related to
//! highlight groups, including:
//! - ColorScheme and ColorSchemePre autocommands
//! - Highlight change notifications
//! - UI update triggers
//!
//! The actual autocommand execution is handled by C code, but this module
//! provides the Rust-side types and logic for event processing.

use std::ffi::c_int;

/// Events that can be triggered by highlight operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightEvent {
    /// Triggered before a colorscheme is loaded (ColorSchemePre)
    ColorSchemePre,
    /// Triggered after a colorscheme is loaded (ColorScheme)
    ColorScheme,
    /// Triggered when a highlight group is modified
    HighlightChanged,
    /// Triggered when the Normal group is modified
    NormalChanged,
    /// Triggered when UI colors need to be updated
    UiColorsChanged,
}

/// Information about a colorscheme event.
#[derive(Debug, Clone)]
pub struct ColorSchemeEvent<'a> {
    /// Name of the colorscheme being loaded
    pub name: &'a str,
    /// Whether this is the initial load (during startup)
    pub is_init: bool,
    /// Whether the colorscheme was found and loaded successfully
    pub success: bool,
}

impl<'a> ColorSchemeEvent<'a> {
    /// Create a new colorscheme event.
    pub fn new(name: &'a str, is_init: bool) -> Self {
        ColorSchemeEvent {
            name,
            is_init,
            success: false,
        }
    }
}

/// Information about a highlight group change.
#[derive(Debug, Clone)]
pub struct HighlightChangeEvent {
    /// ID of the changed highlight group (1-based)
    pub group_id: c_int,
    /// Whether this was a link change
    pub is_link: bool,
    /// Whether this was a clear operation
    pub is_clear: bool,
    /// Whether this is a default setting
    pub is_default: bool,
}

impl HighlightChangeEvent {
    /// Create a new highlight change event.
    pub fn new(group_id: c_int) -> Self {
        HighlightChangeEvent {
            group_id,
            is_link: false,
            is_clear: false,
            is_default: false,
        }
    }

    /// Set this as a link change.
    #[inline]
    pub fn with_link(mut self) -> Self {
        self.is_link = true;
        self
    }

    /// Set this as a clear operation.
    #[inline]
    pub fn with_clear(mut self) -> Self {
        self.is_clear = true;
        self
    }

    /// Set this as a default setting.
    #[inline]
    pub fn with_default(mut self) -> Self {
        self.is_default = true;
        self
    }
}

/// Actions to take after a highlight change.
#[derive(Debug, Clone, Copy, Default)]
pub struct PostChangeActions {
    /// Redraw all windows
    pub redraw_all: bool,
    /// Update UI default colors
    pub update_ui_colors: bool,
    /// Refresh all highlight attributes
    pub refresh_attrs: bool,
    /// Update mode info (for cursor styles)
    pub update_mode_info: bool,
}

impl PostChangeActions {
    /// No actions needed.
    pub const NONE: PostChangeActions = PostChangeActions {
        redraw_all: false,
        update_ui_colors: false,
        refresh_attrs: false,
        update_mode_info: false,
    };

    /// All actions needed.
    pub const ALL: PostChangeActions = PostChangeActions {
        redraw_all: true,
        update_ui_colors: true,
        refresh_attrs: true,
        update_mode_info: true,
    };

    /// Actions for Normal group changes.
    pub const NORMAL: PostChangeActions = PostChangeActions {
        redraw_all: true,
        update_ui_colors: true,
        refresh_attrs: true,
        update_mode_info: false,
    };

    /// Actions for cursor-related group changes.
    pub const CURSOR: PostChangeActions = PostChangeActions {
        redraw_all: true,
        update_ui_colors: false,
        refresh_attrs: false,
        update_mode_info: true,
    };

    /// Combine two sets of actions (logical OR).
    #[inline]
    pub fn merge(self, other: PostChangeActions) -> PostChangeActions {
        PostChangeActions {
            redraw_all: self.redraw_all || other.redraw_all,
            update_ui_colors: self.update_ui_colors || other.update_ui_colors,
            refresh_attrs: self.refresh_attrs || other.refresh_attrs,
            update_mode_info: self.update_mode_info || other.update_mode_info,
        }
    }

    /// Check if any action is needed.
    #[inline]
    pub fn any_needed(&self) -> bool {
        self.redraw_all || self.update_ui_colors || self.refresh_attrs || self.update_mode_info
    }
}

/// Determine post-change actions based on the group name.
///
/// Some highlight groups have special handling requirements:
/// - Normal: updates UI colors and refreshes all attributes
/// - Cursor groups: updates mode info
/// - Most groups: just redraw
pub fn actions_for_group(name: &str) -> PostChangeActions {
    if name.eq_ignore_ascii_case("Normal") {
        PostChangeActions::NORMAL
    } else if name.eq_ignore_ascii_case("Cursor")
        || name.eq_ignore_ascii_case("lCursor")
        || name.eq_ignore_ascii_case("CursorIM")
        || name.eq_ignore_ascii_case("TermCursor")
        || name.eq_ignore_ascii_case("TermCursorNC")
    {
        PostChangeActions::CURSOR
    } else {
        PostChangeActions {
            redraw_all: true,
            update_ui_colors: false,
            refresh_attrs: false,
            update_mode_info: false,
        }
    }
}

/// State for tracking batch highlight changes.
///
/// When multiple highlight groups are changed in sequence (e.g., during
/// colorscheme loading), we can batch the post-change actions to avoid
/// redundant redraws.
#[derive(Debug, Clone, Default)]
pub struct BatchChangeState {
    /// Accumulated actions from all changes
    pub pending_actions: PostChangeActions,
    /// Number of changes in this batch
    pub change_count: usize,
    /// Whether we're in update mode (suppress individual actions)
    pub in_batch: bool,
}

impl BatchChangeState {
    /// Create a new batch state.
    pub fn new() -> Self {
        BatchChangeState::default()
    }

    /// Start a batch update.
    pub fn begin_batch(&mut self) {
        self.in_batch = true;
    }

    /// Record a change and its required actions.
    pub fn record_change(&mut self, actions: PostChangeActions) {
        self.pending_actions = self.pending_actions.merge(actions);
        self.change_count += 1;
    }

    /// End the batch and return the accumulated actions.
    pub fn end_batch(&mut self) -> PostChangeActions {
        let actions = self.pending_actions;
        self.pending_actions = PostChangeActions::NONE;
        self.change_count = 0;
        self.in_batch = false;
        actions
    }

    /// Check if we're currently in a batch.
    #[inline]
    pub fn is_batching(&self) -> bool {
        self.in_batch
    }
}

/// Result of processing a highlight command.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Actions to take after the command
    pub actions: PostChangeActions,
    /// Error message if failed
    pub error: Option<String>,
}

impl CommandResult {
    /// Successful result with no actions.
    pub fn ok() -> Self {
        CommandResult {
            success: true,
            actions: PostChangeActions::NONE,
            error: None,
        }
    }

    /// Successful result with actions.
    pub fn ok_with_actions(actions: PostChangeActions) -> Self {
        CommandResult {
            success: true,
            actions,
            error: None,
        }
    }

    /// Failed result with error message.
    pub fn error(msg: impl Into<String>) -> Self {
        CommandResult {
            success: false,
            actions: PostChangeActions::NONE,
            error: Some(msg.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_event_types() {
        assert_eq!(HighlightEvent::ColorScheme, HighlightEvent::ColorScheme);
        assert_ne!(HighlightEvent::ColorScheme, HighlightEvent::NormalChanged);
    }

    #[test]
    fn test_colorscheme_event() {
        let event = ColorSchemeEvent::new("desert", false);
        assert_eq!(event.name, "desert");
        assert!(!event.is_init);
        assert!(!event.success);
    }

    #[test]
    fn test_highlight_change_event() {
        let event = HighlightChangeEvent::new(1).with_link().with_default();
        assert_eq!(event.group_id, 1);
        assert!(event.is_link);
        assert!(event.is_default);
        assert!(!event.is_clear);
    }

    #[test]
    fn test_post_change_actions() {
        assert!(!PostChangeActions::NONE.any_needed());
        assert!(PostChangeActions::ALL.any_needed());

        let merged = PostChangeActions::NORMAL.merge(PostChangeActions::CURSOR);
        assert!(merged.redraw_all);
        assert!(merged.update_ui_colors);
        assert!(merged.update_mode_info);
    }

    #[test]
    fn test_actions_for_group() {
        let normal = actions_for_group("Normal");
        assert!(normal.update_ui_colors);
        assert!(normal.refresh_attrs);
        assert!(!normal.update_mode_info);

        let cursor = actions_for_group("Cursor");
        assert!(cursor.update_mode_info);
        assert!(!cursor.update_ui_colors);

        let other = actions_for_group("StatusLine");
        assert!(other.redraw_all);
        assert!(!other.update_ui_colors);
        assert!(!other.update_mode_info);
    }

    #[test]
    fn test_batch_change_state() {
        let mut batch = BatchChangeState::new();
        assert!(!batch.is_batching());

        batch.begin_batch();
        assert!(batch.is_batching());

        batch.record_change(PostChangeActions::NORMAL);
        batch.record_change(PostChangeActions::CURSOR);
        assert_eq!(batch.change_count, 2);

        let actions = batch.end_batch();
        assert!(actions.update_ui_colors);
        assert!(actions.update_mode_info);
        assert!(!batch.is_batching());
    }

    #[test]
    fn test_command_result() {
        let ok = CommandResult::ok();
        assert!(ok.success);
        assert!(ok.error.is_none());

        let with_actions = CommandResult::ok_with_actions(PostChangeActions::ALL);
        assert!(with_actions.success);
        assert!(with_actions.actions.redraw_all);

        let err = CommandResult::error("E123: Something went wrong");
        assert!(!err.success);
        assert!(err.error.is_some());
    }
}
