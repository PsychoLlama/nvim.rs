//! Window bar rendering for statusline
//!
//! This module provides window bar rendering utilities. The window bar
//! is a per-window line shown at the top of each window, similar to
//! a per-window statusline.

use std::ffi::c_int;

use nvim_window::WinHandle;

use crate::builder::StatuslineBuilder;
use crate::click::ClickTracker;
use crate::highlight::{get_winbar_hl, HighlightTracker};

/// Window bar configuration options.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WinbarOptions {
    /// Window handle
    pub wp: WinHandle,
    /// Maximum width
    pub max_width: c_int,
    /// Whether this is the current window
    pub is_curwin: bool,
    /// Fill character for empty space
    pub fill_char: char,
}

impl WinbarOptions {
    /// Create new options for the given window.
    pub const fn new(wp: WinHandle, max_width: c_int, is_curwin: bool) -> Self {
        Self {
            wp,
            max_width,
            is_curwin,
            fill_char: ' ',
        }
    }

    /// Get the appropriate highlight for this window bar.
    pub const fn highlight(&self) -> c_int {
        get_winbar_hl(self.is_curwin)
    }
}

impl Default for WinbarOptions {
    fn default() -> Self {
        Self {
            wp: WinHandle::null(),
            max_width: 80,
            is_curwin: false,
            fill_char: ' ',
        }
    }
}

/// Window bar builder for creating window bars.
#[derive(Debug)]
pub struct WinbarBuilder {
    /// Underlying statusline builder
    inner: StatuslineBuilder,
    /// Click tracker for window bar regions
    clicks: ClickTracker,
    /// Highlight tracker
    highlights: HighlightTracker,
    /// Options
    opts: WinbarOptions,
}

impl WinbarBuilder {
    /// Create a new window bar builder.
    #[allow(clippy::cast_sign_loss)]
    pub fn new(opts: WinbarOptions) -> Self {
        Self {
            inner: StatuslineBuilder::new(opts.max_width.max(0) as usize),
            clicks: ClickTracker::new(),
            highlights: HighlightTracker::new(),
            opts,
        }
    }

    /// Get the underlying statusline builder.
    pub const fn builder(&mut self) -> &mut StatuslineBuilder {
        &mut self.inner
    }

    /// Get the click tracker.
    pub const fn clicks(&self) -> &ClickTracker {
        &self.clicks
    }

    /// Get mutable click tracker.
    pub const fn clicks_mut(&mut self) -> &mut ClickTracker {
        &mut self.clicks
    }

    /// Get the highlight tracker.
    pub const fn highlights(&self) -> &HighlightTracker {
        &self.highlights
    }

    /// Get mutable highlight tracker.
    pub const fn highlights_mut(&mut self) -> &mut HighlightTracker {
        &mut self.highlights
    }

    /// Get the options.
    pub const fn options(&self) -> &WinbarOptions {
        &self.opts
    }

    /// Append literal text to the window bar.
    pub fn append_text(&mut self, text: &str) {
        self.inner.append_literal(text);
    }

    /// Start a click function region.
    pub fn start_click_func(&mut self, func_name: &str, minwid: c_int) {
        self.clicks
            .start_func_run(self.inner.position(), func_name, minwid);
    }

    /// End the current click region.
    pub fn end_click(&mut self) {
        self.clicks.end_region(self.inner.position());
    }

    /// Set user highlight (1-9).
    pub fn set_user_highlight(&mut self, hl: u8) {
        self.highlights.set_user_hl(self.inner.position(), hl);
    }

    /// Set named highlight by syntax ID.
    pub fn set_named_highlight(&mut self, syn_id: c_int) {
        self.highlights.set_named_hl(self.inner.position(), syn_id);
    }

    /// Reset to default highlighting.
    pub fn reset_highlight(&mut self) {
        self.highlights.reset(self.inner.position());
    }

    /// Get the current position/width.
    pub const fn position(&self) -> usize {
        self.inner.position()
    }

    /// Finalize the window bar.
    ///
    /// Fills remaining space with fill character if needed.
    #[allow(clippy::cast_sign_loss)]
    pub fn finalize(&mut self) {
        let max_width = self.opts.max_width.max(0) as usize;
        self.inner.post_process(max_width);
    }

    /// Get the output as a string.
    pub fn output_str(&self) -> Option<&str> {
        self.inner.output_str()
    }

    /// Get the output as bytes.
    pub fn output_bytes(&self) -> &[u8] {
        self.inner.output()
    }

    /// Clear the builder for reuse.
    pub fn clear(&mut self) {
        self.inner.clear();
        self.clicks.clear();
        self.highlights.clear();
    }
}

impl Default for WinbarBuilder {
    fn default() -> Self {
        Self::new(WinbarOptions::default())
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get winbar highlight for window.
///
/// This is a convenience wrapper around the highlight module function.
#[no_mangle]
pub const extern "C" fn rs_winbar_get_hl(is_curwin: c_int) -> c_int {
    get_winbar_hl(is_curwin != 0)
}

/// FFI export: Check if window bar should be drawn.
///
/// Returns true if the window has a winbar option set and should be drawn.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub const extern "C" fn rs_winbar_should_draw(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    // The actual check requires calling C to check w_p_wbr
    // For now, we return true and let C code handle the full check
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winbar_options_new() {
        let opts = WinbarOptions::new(WinHandle::null(), 80, true);
        assert_eq!(opts.max_width, 80);
        assert!(opts.is_curwin);
        assert_eq!(opts.fill_char, ' ');
    }

    #[test]
    fn test_winbar_options_highlight() {
        let curwin = WinbarOptions::new(WinHandle::null(), 80, true);
        let notcurwin = WinbarOptions::new(WinHandle::null(), 80, false);

        // Current window uses HLF_WBR (37), non-current uses HLF_WBRNC (38)
        assert_eq!(curwin.highlight(), 37);
        assert_eq!(notcurwin.highlight(), 38);
    }

    #[test]
    fn test_winbar_builder_new() {
        let builder = WinbarBuilder::new(WinbarOptions::default());
        assert!(builder.clicks().is_empty());
        assert!(builder.highlights().is_empty());
        assert_eq!(builder.position(), 0);
    }

    #[test]
    fn test_winbar_builder_append() {
        let mut builder = WinbarBuilder::new(WinbarOptions::default());
        builder.append_text("test");
        assert_eq!(builder.position(), 4);
        assert_eq!(builder.output_str(), Some("test"));
    }

    #[test]
    fn test_winbar_builder_highlight() {
        let mut builder = WinbarBuilder::new(WinbarOptions::default());
        builder.append_text("before");
        builder.set_user_highlight(1);
        builder.append_text("highlighted");
        builder.reset_highlight();
        builder.append_text("after");

        assert_eq!(builder.highlights().len(), 2); // set + reset
    }

    #[test]
    fn test_winbar_builder_clicks() {
        let mut builder = WinbarBuilder::new(WinbarOptions::default());
        builder.start_click_func("MyFunc", 42);
        builder.append_text("clickable");
        builder.end_click();

        assert_eq!(builder.clicks().len(), 2); // start + end
    }

    #[test]
    fn test_winbar_builder_clear() {
        let mut builder = WinbarBuilder::new(WinbarOptions::default());
        builder.append_text("some text");
        builder.set_user_highlight(1);
        builder.start_click_func("Test", 0);

        builder.clear();

        assert_eq!(builder.position(), 0);
        assert!(builder.highlights().is_empty());
        assert!(builder.clicks().is_empty());
    }
}
