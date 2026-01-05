//! Statusline builder infrastructure
//!
//! This module provides the core building blocks for constructing statuslines,
//! window bars, and tablines. It handles group management, truncation, separation,
//! and width calculations.

use std::ffi::c_int;

use crate::click::ClickTracker;
use crate::format::StlFlag;
use crate::highlight::HighlightTracker;

/// Item type for statusline items.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// Normal text item
    Normal = 0,
    /// Empty/placeholder item
    Empty = 1,
    /// Group start marker
    Group = 2,
    /// Separator (fill with space)
    Separate = 3,
    /// Highlight change
    Highlight = 4,
    /// Sign highlight (statuscolumn)
    HighlightSign = 5,
    /// Fold highlight (statuscolumn)
    HighlightFold = 6,
    /// Tab page click region
    TabPage = 7,
    /// Click function region
    ClickFunc = 8,
    /// Truncation marker
    Trunc = 9,
}

/// A single statusline item during building.
#[derive(Debug, Clone)]
pub struct BuilderItem {
    /// Item type
    pub item_type: ItemType,
    /// Start position in output buffer (byte offset)
    pub start: usize,
    /// Minimum width (negative = left-aligned)
    pub minwid: c_int,
    /// Maximum width (0 = unlimited)
    pub maxwid: c_int,
    /// Associated command/function for click items
    pub cmd: Option<String>,
}

impl BuilderItem {
    /// Create a new builder item.
    pub const fn new(item_type: ItemType, start: usize) -> Self {
        Self {
            item_type,
            start,
            minwid: 0,
            maxwid: 0,
            cmd: None,
        }
    }

    /// Create a group start item.
    pub const fn group(start: usize, minwid: c_int, maxwid: c_int) -> Self {
        Self {
            item_type: ItemType::Group,
            start,
            minwid,
            maxwid,
            cmd: None,
        }
    }

    /// Create a separator item.
    pub const fn separator(start: usize) -> Self {
        Self::new(ItemType::Separate, start)
    }

    /// Create a truncation marker item.
    pub const fn truncation(start: usize) -> Self {
        Self::new(ItemType::Trunc, start)
    }

    /// Create a highlight item.
    pub const fn highlight(start: usize, userhl: c_int) -> Self {
        Self {
            item_type: ItemType::Highlight,
            start,
            minwid: userhl,
            maxwid: 0,
            cmd: None,
        }
    }

    /// Create a tab page click item.
    pub const fn tab_page(start: usize, tabnr: c_int) -> Self {
        Self {
            item_type: ItemType::TabPage,
            start,
            minwid: tabnr,
            maxwid: 0,
            cmd: None,
        }
    }

    /// Create a click function item.
    pub fn click_func(start: usize, func: &str, minwid: c_int) -> Self {
        Self {
            item_type: ItemType::ClickFunc,
            start,
            minwid,
            maxwid: 0,
            cmd: Some(func.to_string()),
        }
    }
}

/// Group tracking during statusline building.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GroupState {
    /// Index into items array where group starts
    start_item: usize,
    /// Start position in output
    start_pos: usize,
    /// Minimum width for the group
    minwid: c_int,
    /// Maximum width for the group
    maxwid: c_int,
}

/// Statusline builder state.
///
/// This tracks the state during statusline construction, managing
/// groups, items, truncation, and separation.
#[derive(Debug)]
#[allow(dead_code)]
pub struct StatuslineBuilder {
    /// Output buffer
    output: Vec<u8>,
    /// Maximum output width (reserved for future use)
    max_width: usize,
    /// List of items
    items: Vec<BuilderItem>,
    /// Group stack
    groups: Vec<GroupState>,
    /// Highlight tracker
    highlights: HighlightTracker,
    /// Click tracker
    clicks: ClickTracker,
    /// Separator locations (item indices)
    separators: Vec<usize>,
    /// Truncation point (item index, if set)
    truncation_point: Option<usize>,
    /// Fill character for separators
    fill_char: char,
    /// Previous char was a flag item
    prev_char_is_flag: bool,
    /// Previous char was an item
    prev_char_is_item: bool,
}

impl Default for StatuslineBuilder {
    fn default() -> Self {
        Self::new(256)
    }
}

impl StatuslineBuilder {
    /// Create a new statusline builder with the specified max width.
    pub fn new(max_width: usize) -> Self {
        Self {
            output: Vec::with_capacity(max_width * 2),
            max_width,
            items: Vec::with_capacity(32),
            groups: Vec::with_capacity(8),
            highlights: HighlightTracker::new(),
            clicks: ClickTracker::new(),
            separators: Vec::with_capacity(4),
            truncation_point: None,
            fill_char: ' ',
            prev_char_is_flag: true,
            prev_char_is_item: false,
        }
    }

    /// Set the fill character.
    pub const fn set_fill_char(&mut self, ch: char) {
        self.fill_char = ch;
    }

    /// Get current output position.
    pub const fn position(&self) -> usize {
        self.output.len()
    }

    /// Get current output as bytes.
    pub fn output(&self) -> &[u8] {
        &self.output
    }

    /// Get current output as string (if valid UTF-8).
    pub fn output_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.output).ok()
    }

    /// Append literal text to output.
    pub fn append_literal(&mut self, text: &str) {
        self.output.extend_from_slice(text.as_bytes());
        self.prev_char_is_flag = false;
        self.prev_char_is_item = false;
    }

    /// Append a single byte to output.
    pub fn append_byte(&mut self, byte: u8) {
        self.output.push(byte);
        self.prev_char_is_flag = false;
        self.prev_char_is_item = false;
    }

    /// Start a group with the given width constraints.
    pub fn start_group(&mut self, minwid: c_int, maxwid: c_int) {
        let start_pos = self.output.len();
        let start_item = self.items.len();

        self.items
            .push(BuilderItem::group(start_pos, minwid, maxwid));
        self.groups.push(GroupState {
            start_item,
            start_pos,
            minwid,
            maxwid,
        });
    }

    /// End the current group.
    ///
    /// Returns true if a group was ended, false if not in a group.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn end_group(&mut self) -> bool {
        let Some(group) = self.groups.pop() else {
            return false;
        };

        let group_len = self.calculate_group_width(group.start_pos);

        // Handle group truncation if needed
        if group.maxwid > 0 && group_len > group.maxwid as usize {
            self.truncate_group(&group, group_len);
        }
        // Handle group padding if needed
        else if group.minwid.abs() > group_len as c_int {
            self.pad_group(&group, group_len);
        }

        true
    }

    /// Calculate width of group content (simplified - counts bytes for ASCII).
    const fn calculate_group_width(&self, start_pos: usize) -> usize {
        // Simplified: count bytes. In real implementation, use vim_strsize.
        self.output.len() - start_pos
    }

    /// Truncate a group that exceeds its maximum width.
    #[allow(clippy::cast_sign_loss)]
    fn truncate_group(&mut self, group: &GroupState, current_len: usize) {
        let max_len = group.maxwid as usize;
        if current_len <= max_len || max_len == 0 {
            return;
        }

        let start = group.start_pos;
        let excess = current_len - max_len + 1; // +1 for '<' marker

        // Remove excess bytes from the start of the group
        if excess < self.output.len() - start {
            // Insert '<' marker
            self.output[start] = b'<';
            // Remove bytes after marker
            self.output.drain(start + 1..start + excess);

            // Adjust item positions
            for item in &mut self.items {
                if item.start > start {
                    item.start = item.start.saturating_sub(excess - 1);
                    item.start = item.start.max(start);
                }
            }
        }
    }

    /// Pad a group that's shorter than its minimum width.
    fn pad_group(&mut self, group: &GroupState, current_len: usize) {
        let min_len = group.minwid.unsigned_abs() as usize;
        if current_len >= min_len {
            return;
        }

        let padding = min_len - current_len;
        let fill = self.fill_char.to_string();

        if group.minwid < 0 {
            // Left-aligned: pad on the right
            for _ in 0..padding {
                self.output.extend_from_slice(fill.as_bytes());
            }
        } else {
            // Right-aligned: pad on the left
            let start = group.start_pos;
            let pad_bytes: Vec<u8> = fill.repeat(padding).into_bytes();

            // Insert padding at group start
            self.output.splice(start..start, pad_bytes.iter().copied());

            // Adjust item positions
            for item in &mut self.items {
                if item.start >= start {
                    item.start += padding;
                }
            }
        }
    }

    /// Add a separator marker at current position.
    pub fn add_separator(&mut self) {
        // Separators are ignored inside groups
        if !self.groups.is_empty() {
            return;
        }

        let pos = self.output.len();
        self.items.push(BuilderItem::separator(pos));
        self.separators.push(self.items.len() - 1);
    }

    /// Add a truncation marker at current position.
    pub fn add_truncation_marker(&mut self) {
        let pos = self.output.len();
        self.items.push(BuilderItem::truncation(pos));
        self.truncation_point = Some(self.items.len() - 1);
    }

    /// Set highlight at current position.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn set_highlight(&mut self, userhl: c_int) {
        let pos = self.output.len();
        self.highlights.set_user_hl(pos, userhl as u8);
        self.items.push(BuilderItem::highlight(pos, userhl));
    }

    /// Set named highlight by syntax ID.
    pub fn set_named_highlight(&mut self, syn_id: c_int) {
        let pos = self.output.len();
        self.highlights.set_named_hl(pos, syn_id);
        self.items.push(BuilderItem::highlight(pos, -syn_id));
    }

    /// Reset highlight to default.
    pub fn reset_highlight(&mut self) {
        let pos = self.output.len();
        self.highlights.reset(pos);
        self.items.push(BuilderItem::highlight(pos, 0));
    }

    /// Add a tab page click region.
    pub fn add_tab_page(&mut self, tabnr: c_int) {
        let pos = self.output.len();
        self.items.push(BuilderItem::tab_page(pos, tabnr));

        match tabnr.cmp(&0) {
            std::cmp::Ordering::Greater => self.clicks.start_tab_switch(pos, tabnr),
            std::cmp::Ordering::Less => self.clicks.start_tab_close(pos, -tabnr),
            std::cmp::Ordering::Equal => self.clicks.end_region(pos),
        }
    }

    /// Add a click function region.
    pub fn add_click_func(&mut self, func: &str, minwid: c_int) {
        let pos = self.output.len();
        self.items.push(BuilderItem::click_func(pos, func, minwid));
        self.clicks.start_func_run(pos, func, minwid);
    }

    /// Add a normal item.
    pub fn add_item(&mut self, flag: StlFlag, content: &str, is_flag: bool) {
        let pos = self.output.len();
        self.output.extend_from_slice(content.as_bytes());

        // Determine item type based on flag
        let item_type = match flag {
            StlFlag::SignCol => ItemType::HighlightSign,
            StlFlag::FoldCol => ItemType::HighlightFold,
            _ => ItemType::Normal,
        };

        self.items.push(BuilderItem {
            item_type,
            start: pos,
            minwid: 0,
            maxwid: 0,
            cmd: None,
        });

        self.prev_char_is_flag = is_flag;
        self.prev_char_is_item = true;
    }

    /// Check if we're currently inside a group.
    pub const fn in_group(&self) -> bool {
        !self.groups.is_empty()
    }

    /// Get current group depth.
    pub const fn group_depth(&self) -> usize {
        self.groups.len()
    }

    /// Get the number of items.
    pub const fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Get all items.
    pub fn items(&self) -> &[BuilderItem] {
        &self.items
    }

    /// Get highlight tracker.
    pub const fn highlights(&self) -> &HighlightTracker {
        &self.highlights
    }

    /// Get click tracker.
    pub const fn clicks(&self) -> &ClickTracker {
        &self.clicks
    }

    /// Finalize the statusline and apply separators.
    ///
    /// This fills the space between separators to achieve the target width.
    pub fn finalize(&mut self, target_width: usize) {
        if self.separators.is_empty() || target_width == 0 {
            return;
        }

        let current_width = self.output.len();
        if current_width >= target_width {
            return;
        }

        let fill_space = target_width - current_width;
        let sep_count = self.separators.len();
        let fill_per_sep = fill_space / sep_count;
        let extra = fill_space % sep_count;

        // Fill separators from right to left to avoid position shifts
        for (i, &sep_idx) in self.separators.iter().enumerate().rev() {
            let sep_item = &self.items[sep_idx];
            let sep_pos = sep_item.start;
            let fill_amount = fill_per_sep + usize::from(i < extra);

            if fill_amount > 0 {
                let fill = self.fill_char.to_string().repeat(fill_amount);

                // Insert fill at separator position
                self.output.splice(sep_pos..sep_pos, fill.bytes());

                // Adjust positions of items after this separator
                for item in &mut self.items {
                    if item.start > sep_pos {
                        item.start += fill_amount;
                    }
                }
            }
        }
    }

    /// Clear the builder for reuse.
    pub fn clear(&mut self) {
        self.output.clear();
        self.items.clear();
        self.groups.clear();
        self.highlights.clear();
        self.clicks.clear();
        self.separators.clear();
        self.truncation_point = None;
        self.prev_char_is_flag = true;
        self.prev_char_is_item = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = StatuslineBuilder::new(80);
        assert_eq!(builder.position(), 0);
        assert!(builder.items().is_empty());
        assert!(!builder.in_group());
    }

    #[test]
    fn test_append_literal() {
        let mut builder = StatuslineBuilder::new(80);
        builder.append_literal("hello");
        assert_eq!(builder.output_str(), Some("hello"));
        assert_eq!(builder.position(), 5);
    }

    #[test]
    fn test_append_byte() {
        let mut builder = StatuslineBuilder::new(80);
        builder.append_byte(b'X');
        builder.append_byte(b'Y');
        assert_eq!(builder.output_str(), Some("XY"));
    }

    #[test]
    fn test_group_basic() {
        let mut builder = StatuslineBuilder::new(80);

        builder.start_group(0, 0);
        assert!(builder.in_group());
        assert_eq!(builder.group_depth(), 1);

        builder.append_literal("test");

        assert!(builder.end_group());
        assert!(!builder.in_group());
        assert_eq!(builder.output_str(), Some("test"));
    }

    #[test]
    fn test_group_nested() {
        let mut builder = StatuslineBuilder::new(80);

        builder.start_group(0, 0);
        builder.append_literal("outer");
        builder.start_group(0, 0);
        builder.append_literal("inner");
        builder.end_group();
        builder.append_literal("!");
        builder.end_group();

        assert_eq!(builder.output_str(), Some("outerinner!"));
    }

    #[test]
    fn test_group_padding_right() {
        let mut builder = StatuslineBuilder::new(80);

        builder.start_group(-10, 0); // Left-aligned, min 10
        builder.append_literal("hi");
        builder.end_group();

        assert_eq!(builder.output_str(), Some("hi        ")); // "hi" + 8 spaces
    }

    #[test]
    fn test_group_padding_left() {
        let mut builder = StatuslineBuilder::new(80);

        builder.start_group(10, 0); // Right-aligned, min 10
        builder.append_literal("hi");
        builder.end_group();

        assert_eq!(builder.output_str(), Some("        hi")); // 8 spaces + "hi"
    }

    #[test]
    fn test_separator() {
        let mut builder = StatuslineBuilder::new(80);

        builder.append_literal("left");
        builder.add_separator();
        builder.append_literal("right");

        // Before finalize
        assert_eq!(builder.output_str(), Some("leftright"));

        // After finalize with target width
        builder.finalize(20);
        let output = builder.output_str().unwrap();
        assert!(output.starts_with("left"));
        assert!(output.ends_with("right"));
        assert_eq!(output.len(), 20);
    }

    #[test]
    fn test_highlight() {
        let mut builder = StatuslineBuilder::new(80);

        builder.append_literal("normal");
        builder.set_highlight(1);
        builder.append_literal("highlighted");
        builder.reset_highlight();
        builder.append_literal("normal again");

        assert_eq!(builder.highlights().len(), 2); // set + reset
    }

    #[test]
    fn test_click_region() {
        let mut builder = StatuslineBuilder::new(80);

        builder.append_literal("[Tab 1]");
        builder.add_tab_page(1);
        builder.append_literal("[Tab 2]");
        builder.add_tab_page(2);
        builder.append_literal("[Close]");
        builder.add_tab_page(-1); // Close

        assert!(builder.clicks().len() >= 2);
    }

    #[test]
    fn test_truncation_marker() {
        let mut builder = StatuslineBuilder::new(80);

        builder.append_literal("before");
        builder.add_truncation_marker();
        builder.append_literal("after");

        assert!(builder.truncation_point.is_some());
    }

    #[test]
    fn test_clear() {
        let mut builder = StatuslineBuilder::new(80);

        builder.append_literal("test");
        builder.start_group(5, 10);
        builder.set_highlight(1);

        builder.clear();

        assert_eq!(builder.position(), 0);
        assert!(builder.items().is_empty());
        assert!(!builder.in_group());
        assert!(builder.highlights().is_empty());
    }

    #[test]
    fn test_item_types() {
        assert_eq!(ItemType::Normal as c_int, 0);
        assert_eq!(ItemType::Empty as c_int, 1);
        assert_eq!(ItemType::Group as c_int, 2);
        assert_eq!(ItemType::Separate as c_int, 3);
    }

    #[test]
    fn test_builder_item_constructors() {
        let sep = BuilderItem::separator(10);
        assert_eq!(sep.item_type, ItemType::Separate);
        assert_eq!(sep.start, 10);

        let trunc = BuilderItem::truncation(20);
        assert_eq!(trunc.item_type, ItemType::Trunc);
        assert_eq!(trunc.start, 20);

        let hl = BuilderItem::highlight(30, 5);
        assert_eq!(hl.item_type, ItemType::Highlight);
        assert_eq!(hl.minwid, 5);

        let tab = BuilderItem::tab_page(40, 3);
        assert_eq!(tab.item_type, ItemType::TabPage);
        assert_eq!(tab.minwid, 3);

        let click = BuilderItem::click_func(50, "MyFunc", 42);
        assert_eq!(click.item_type, ItemType::ClickFunc);
        assert_eq!(click.cmd, Some("MyFunc".to_string()));
        assert_eq!(click.minwid, 42);
    }
}
