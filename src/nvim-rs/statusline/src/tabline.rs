//! Tabline and window bar rendering for statusline
//!
//! This module provides tabline and window bar rendering utilities,
//! including tab label generation, close button handling, and click tracking.

use std::ffi::c_int;
use std::fmt::Write;

use crate::click::ClickType;

/// Tab information for rendering.
#[derive(Debug, Clone)]
pub struct TabInfo {
    /// Tab number (1-based)
    pub tabnr: c_int,
    /// Whether this is the current tab
    pub is_current: bool,
    /// Number of windows in this tab
    pub window_count: c_int,
    /// Whether any buffer in this tab is modified
    pub is_modified: bool,
    /// Display name for the tab
    pub name: String,
}

impl TabInfo {
    /// Create a new tab info.
    pub const fn new(tabnr: c_int, is_current: bool) -> Self {
        Self {
            tabnr,
            is_current,
            window_count: 1,
            is_modified: false,
            name: String::new(),
        }
    }

    /// Set the window count and modified status.
    #[must_use]
    pub const fn with_windows(mut self, count: c_int, modified: bool) -> Self {
        self.window_count = count;
        self.is_modified = modified;
        self
    }

    /// Set the display name.
    #[must_use]
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
}

/// Tabline rendering context.
#[derive(Debug, Clone)]
pub struct TablineContext {
    /// Total available columns
    pub columns: c_int,
    /// List of tabs
    pub tabs: Vec<TabInfo>,
    /// Width per tab (calculated)
    pub tab_width: c_int,
    /// Whether to use separator characters (for low-color terminals)
    pub use_sep_chars: bool,
}

impl TablineContext {
    /// Create a new tabline context.
    pub const fn new(columns: c_int) -> Self {
        Self {
            columns,
            tabs: Vec::new(),
            tab_width: 6,
            use_sep_chars: false,
        }
    }

    /// Add a tab to the context.
    pub fn add_tab(&mut self, tab: TabInfo) {
        self.tabs.push(tab);
    }

    /// Calculate tab width based on available columns and tab count.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn calculate_tab_width(&mut self) {
        let tabcount = self.tabs.len() as c_int;
        if tabcount > 0 {
            // Same formula as rs_tabwidth_calc
            let width = (self.columns - 1 + tabcount / 2) / tabcount;
            self.tab_width = width.max(6);
        }
    }

    /// Get the number of tabs.
    pub const fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Check if there are multiple tabs (for showing close button).
    pub const fn has_multiple_tabs(&self) -> bool {
        self.tabs.len() > 1
    }
}

/// Window bar context.
#[derive(Debug, Clone)]
pub struct WinbarContext {
    /// Window width
    pub width: c_int,
    /// Window bar format string (if custom)
    pub format: Option<String>,
    /// Whether to use global window bar
    pub use_global: bool,
}

impl WinbarContext {
    /// Create a new window bar context.
    pub const fn new(width: c_int) -> Self {
        Self {
            width,
            format: None,
            use_global: false,
        }
    }

    /// Set the format string.
    #[must_use]
    pub fn with_format(mut self, format: String) -> Self {
        self.format = Some(format);
        self
    }
}

/// Generate a shortened tab label from a buffer name.
///
/// This mimics the behavior of `shorten_dir()` in C - shortening
/// directory components to their first character.
pub fn shorten_tab_label(path: &str, max_width: usize) -> String {
    if path.is_empty() {
        return "[No Name]".to_string();
    }

    // If it fits, return as-is
    if path.len() <= max_width {
        return path.to_string();
    }

    // Split into directory and filename
    let path_bytes = path.as_bytes();
    let last_sep = path_bytes.iter().rposition(|&b| b == b'/' || b == b'\\');

    match last_sep {
        Some(sep_idx) => {
            let filename = &path[sep_idx + 1..];
            let dir_part = &path[..sep_idx];

            // If filename alone is too long, truncate from the end
            if filename.len() >= max_width {
                return filename[..max_width].to_string();
            }

            // Shorten directory components
            let remaining = max_width - filename.len() - 1; // -1 for separator
            let shortened_dir = shorten_path_components(dir_part, remaining);

            if shortened_dir.is_empty() {
                filename.to_string()
            } else {
                format!("{shortened_dir}/{filename}")
            }
        }
        None => {
            // No directory, just truncate filename
            path[..max_width.min(path.len())].to_string()
        }
    }
}

/// Shorten path components to fit within max_len.
fn shorten_path_components(dir: &str, max_len: usize) -> String {
    if dir.is_empty() || max_len == 0 {
        return String::new();
    }

    // Split by separators
    let components: Vec<&str> = dir.split(['/', '\\']).filter(|s| !s.is_empty()).collect();

    if components.is_empty() {
        return String::new();
    }

    // Start with single-char components, expand if space allows
    let mut result = String::new();
    let sep = if dir.contains('\\') { '\\' } else { '/' };

    for (i, comp) in components.iter().enumerate() {
        if i > 0 {
            result.push(sep);
        }

        // Use first char of component
        if let Some(first_char) = comp.chars().next() {
            result.push(first_char);
        }

        if result.len() >= max_len {
            break;
        }
    }

    result
}

/// Format a tab label with window count and modified indicator.
///
/// Returns a string like "2+ filename" or just "filename".
pub fn format_tab_label(tab: &TabInfo, max_width: c_int) -> String {
    let mut prefix = String::new();

    // Add window count if more than 1
    if tab.window_count > 1 {
        let _ = write!(prefix, "{}", tab.window_count);
    }

    // Add modified indicator
    if tab.is_modified {
        prefix.push('+');
    }

    // Add space after prefix if not empty
    if !prefix.is_empty() {
        prefix.push(' ');
    }

    // Calculate remaining width for name
    #[allow(clippy::cast_sign_loss)]
    let name_width = (max_width as usize).saturating_sub(prefix.len() + 2); // +2 for padding

    let name = if name_width > 0 {
        shorten_tab_label(&tab.name, name_width)
    } else {
        String::new()
    };

    format!(" {prefix}{name} ")
}

/// Calculate click definition for a column position in the tabline.
///
/// Returns the click type and tab number for the given column.
pub const fn tabline_click_at(
    col: c_int,
    tab_positions: &[(c_int, c_int, c_int)],
) -> (ClickType, c_int) {
    // tab_positions is a slice of (start_col, end_col, tabnr)
    let mut i = 0;
    while i < tab_positions.len() {
        let (start, end, tabnr) = tab_positions[i];
        if col >= start && col < end {
            if tabnr < 0 {
                return (ClickType::TabClose, -tabnr);
            } else if tabnr > 0 {
                return (ClickType::TabSwitch, tabnr);
            }
            return (ClickType::Disabled, 0);
        }
        i += 1;
    }
    (ClickType::Disabled, 0)
}

/// Information about the close button position.
#[derive(Debug, Clone, Copy)]
pub struct CloseButtonInfo {
    /// Column position of the close button
    pub col: c_int,
    /// Whether the close button is visible
    pub visible: bool,
}

impl CloseButtonInfo {
    /// Create info for no close button.
    pub const fn hidden() -> Self {
        Self {
            col: -1,
            visible: false,
        }
    }

    /// Create info for a visible close button.
    pub const fn at(col: c_int) -> Self {
        Self { col, visible: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_info_new() {
        let tab = TabInfo::new(1, true);
        assert_eq!(tab.tabnr, 1);
        assert!(tab.is_current);
        assert_eq!(tab.window_count, 1);
        assert!(!tab.is_modified);
    }

    #[test]
    fn test_tab_info_with_windows() {
        let tab = TabInfo::new(1, false).with_windows(3, true);
        assert_eq!(tab.window_count, 3);
        assert!(tab.is_modified);
    }

    #[test]
    fn test_tab_info_with_name() {
        let tab = TabInfo::new(1, true).with_name("test.rs".to_string());
        assert_eq!(tab.name, "test.rs");
    }

    #[test]
    fn test_tabline_context_new() {
        let ctx = TablineContext::new(80);
        assert_eq!(ctx.columns, 80);
        assert!(ctx.tabs.is_empty());
    }

    #[test]
    fn test_tabline_context_calculate_tab_width() {
        let mut ctx = TablineContext::new(80);
        ctx.add_tab(TabInfo::new(1, true));
        ctx.add_tab(TabInfo::new(2, false));
        ctx.add_tab(TabInfo::new(3, false));
        ctx.add_tab(TabInfo::new(4, false));
        ctx.add_tab(TabInfo::new(5, false));

        ctx.calculate_tab_width();
        // (80 - 1 + 2) / 5 = 16, max(16, 6) = 16
        assert_eq!(ctx.tab_width, 16);
    }

    #[test]
    fn test_tabline_context_has_multiple_tabs() {
        let mut ctx = TablineContext::new(80);
        assert!(!ctx.has_multiple_tabs());

        ctx.add_tab(TabInfo::new(1, true));
        assert!(!ctx.has_multiple_tabs());

        ctx.add_tab(TabInfo::new(2, false));
        assert!(ctx.has_multiple_tabs());
    }

    #[test]
    fn test_shorten_tab_label_short() {
        let result = shorten_tab_label("test.rs", 20);
        assert_eq!(result, "test.rs");
    }

    #[test]
    fn test_shorten_tab_label_empty() {
        let result = shorten_tab_label("", 20);
        assert_eq!(result, "[No Name]");
    }

    #[test]
    fn test_shorten_tab_label_long_filename() {
        let result = shorten_tab_label("very_long_filename_that_needs_truncation.rs", 15);
        assert!(result.len() <= 15);
    }

    #[test]
    fn test_shorten_tab_label_with_path() {
        let result = shorten_tab_label("/home/user/project/src/main.rs", 20);
        // Should shorten directory components
        assert!(result.contains("main.rs"));
        assert!(result.len() <= 20);
    }

    #[test]
    fn test_format_tab_label_simple() {
        let tab = TabInfo::new(1, true).with_name("test.rs".to_string());
        let label = format_tab_label(&tab, 20);
        assert!(label.contains("test.rs"));
        assert!(label.starts_with(' '));
        assert!(label.ends_with(' '));
    }

    #[test]
    fn test_format_tab_label_with_windows() {
        let tab = TabInfo::new(1, false)
            .with_windows(3, false)
            .with_name("test.rs".to_string());
        let label = format_tab_label(&tab, 20);
        assert!(label.contains('3'));
        assert!(label.contains("test.rs"));
    }

    #[test]
    fn test_format_tab_label_modified() {
        let tab = TabInfo::new(1, false)
            .with_windows(1, true)
            .with_name("test.rs".to_string());
        let label = format_tab_label(&tab, 20);
        assert!(label.contains('+'));
    }

    #[test]
    fn test_tabline_click_at() {
        let positions = [(0, 10, 1), (10, 20, 2), (20, 21, -1)];

        let (click_type, tabnr) = tabline_click_at(5, &positions);
        assert_eq!(click_type, ClickType::TabSwitch);
        assert_eq!(tabnr, 1);

        let (click_type, tabnr) = tabline_click_at(15, &positions);
        assert_eq!(click_type, ClickType::TabSwitch);
        assert_eq!(tabnr, 2);

        let (click_type, tabnr) = tabline_click_at(20, &positions);
        assert_eq!(click_type, ClickType::TabClose);
        assert_eq!(tabnr, 1);
    }

    #[test]
    fn test_close_button_info() {
        let hidden = CloseButtonInfo::hidden();
        assert!(!hidden.visible);
        assert_eq!(hidden.col, -1);

        let visible = CloseButtonInfo::at(79);
        assert!(visible.visible);
        assert_eq!(visible.col, 79);
    }

    #[test]
    fn test_winbar_context() {
        let ctx = WinbarContext::new(80);
        assert_eq!(ctx.width, 80);
        assert!(ctx.format.is_none());

        let ctx = ctx.with_format("%f".to_string());
        assert_eq!(ctx.format, Some("%f".to_string()));
    }
}
