//! Ex commands integration for syntax highlighting.
//!
//! This module handles:
//! - Syntax subcommand dispatch
//! - Argument parsing for syntax commands
//! - Pattern option parsing (contained, containedin, etc.)
//! - Command completion support

use std::ffi::{c_char, c_int};

use crate::types::{SynBlockHandle, WinHandle};

// =============================================================================
// FFI declarations for command operations
// =============================================================================

extern "C" {
    // Subcommand execution
    fn nvim_syn_get_cmdlinep() -> *mut *mut c_char;

    // Current window synblock
    fn nvim_get_curwin_synblock() -> SynBlockHandle;
    fn nvim_get_curwin() -> WinHandle;

    // Syntax command state
    fn nvim_syn_get_include_link() -> c_int;
    fn nvim_syn_get_include_default() -> c_int;
    fn nvim_syn_get_include_none() -> c_int;

    // Option parsing helpers
    fn nvim_syn_get_conceal_setting(block: SynBlockHandle) -> c_int;
    fn nvim_syn_get_ic_setting(block: SynBlockHandle) -> c_int;

    // Running inc_tag for :syntax include
    fn nvim_syn_get_running_inc_tag() -> c_int;
    fn nvim_syn_set_running_inc_tag(tag: c_int);
}

// =============================================================================
// Syntax subcommand enumeration
// =============================================================================

/// Syntax subcommands for `:syntax` command family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxSubcommand {
    /// `:syntax case` - Set case sensitivity
    Case,
    /// `:syntax clear` - Clear syntax items
    Clear,
    /// `:syntax cluster` - Define a cluster
    Cluster,
    /// `:syntax conceal` - Set concealing
    Conceal,
    /// `:syntax enable` / `:syntax on` - Enable syntax highlighting
    Enable,
    /// `:syntax foldlevel` - Set fold level mode
    FoldLevel,
    /// `:syntax include` - Include another syntax file
    Include,
    /// `:syntax iskeyword` - Set iskeyword for syntax
    IsKeyword,
    /// `:syntax keyword` - Define a keyword
    Keyword,
    /// `:syntax list` - List syntax items
    List,
    /// `:syntax manual` - Set manual mode
    Manual,
    /// `:syntax match` - Define a match pattern
    Match,
    /// `:syntax off` - Disable syntax highlighting
    Off,
    /// `:syntax region` - Define a region
    Region,
    /// `:syntax reset` - Reset to default colors
    Reset,
    /// `:syntax spell` - Set spell checking mode
    Spell,
    /// `:syntax sync` - Set synchronization
    Sync,
}

impl SyntaxSubcommand {
    /// Parse a subcommand name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "case" => Some(Self::Case),
            "clear" => Some(Self::Clear),
            "cluster" => Some(Self::Cluster),
            "conceal" => Some(Self::Conceal),
            "enable" | "on" => Some(Self::Enable),
            "foldlevel" => Some(Self::FoldLevel),
            "include" => Some(Self::Include),
            "iskeyword" => Some(Self::IsKeyword),
            "keyword" => Some(Self::Keyword),
            "list" | "" => Some(Self::List),
            "manual" => Some(Self::Manual),
            "match" => Some(Self::Match),
            "off" => Some(Self::Off),
            "region" => Some(Self::Region),
            "reset" => Some(Self::Reset),
            "spell" => Some(Self::Spell),
            "sync" => Some(Self::Sync),
            _ => None,
        }
    }

    /// Get the canonical name of this subcommand.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Case => "case",
            Self::Clear => "clear",
            Self::Cluster => "cluster",
            Self::Conceal => "conceal",
            Self::Enable => "enable",
            Self::FoldLevel => "foldlevel",
            Self::Include => "include",
            Self::IsKeyword => "iskeyword",
            Self::Keyword => "keyword",
            Self::List => "list",
            Self::Manual => "manual",
            Self::Match => "match",
            Self::Off => "off",
            Self::Region => "region",
            Self::Reset => "reset",
            Self::Spell => "spell",
            Self::Sync => "sync",
        }
    }

    /// Get all available subcommand names.
    #[must_use]
    pub fn all_names() -> &'static [&'static str] {
        &[
            "case",
            "clear",
            "cluster",
            "conceal",
            "enable",
            "foldlevel",
            "include",
            "iskeyword",
            "keyword",
            "list",
            "manual",
            "match",
            "off",
            "on",
            "region",
            "reset",
            "spell",
            "sync",
        ]
    }
}

// =============================================================================
// Case sensitivity mode
// =============================================================================

/// Case sensitivity mode for syntax matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaseMode {
    /// Use case-sensitive matching.
    #[default]
    Match,
    /// Use case-insensitive matching.
    Ignore,
}

impl CaseMode {
    /// Parse from command argument.
    #[must_use]
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "match" => Some(Self::Match),
            "ignore" => Some(Self::Ignore),
            _ => None,
        }
    }

    /// Get the argument string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Ignore => "ignore",
        }
    }
}

// =============================================================================
// Spell check mode
// =============================================================================

/// Spell checking mode for syntax items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellMode {
    /// Default: spell check if no @Spell cluster.
    #[default]
    Default,
    /// Spell check top-level text only.
    TopLevel,
    /// No spell checking in syntax items.
    NoTopLevel,
}

/// Spell mode constants.
pub mod spell_mode {
    pub const DEFAULT: i32 = 0;
    pub const TOP: i32 = 1;
    pub const NOTOP: i32 = 2;
}

impl SpellMode {
    /// Parse from command argument.
    #[must_use]
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "toplevel" => Some(Self::TopLevel),
            "notoplevel" => Some(Self::NoTopLevel),
            "default" => Some(Self::Default),
            _ => None,
        }
    }

    /// Convert from raw C value.
    #[must_use]
    pub const fn from_raw(val: i32) -> Self {
        match val {
            spell_mode::TOP => Self::TopLevel,
            spell_mode::NOTOP => Self::NoTopLevel,
            _ => Self::Default,
        }
    }

    /// Convert to raw C value.
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::Default => spell_mode::DEFAULT,
            Self::TopLevel => spell_mode::TOP,
            Self::NoTopLevel => spell_mode::NOTOP,
        }
    }

    /// Get the argument string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::TopLevel => "toplevel",
            Self::NoTopLevel => "notoplevel",
        }
    }
}

// =============================================================================
// Pattern options
// =============================================================================

/// Options for syntax patterns (keyword, match, region).
#[derive(Debug, Clone, Default)]
pub struct PatternOptions {
    /// Pattern is contained within another.
    pub contained: bool,
    /// Display the pattern without highlighting.
    pub transparent: bool,
    /// Pattern starts at beginning of line.
    pub oneline: bool,
    /// Keep end position even when contained ends earlier.
    pub keepend: bool,
    /// Pattern can extend across lines.
    pub extend: bool,
    /// Exclude pattern from 'include'.
    pub excludenl: bool,
    /// Skip leading whitespace.
    pub skipwhite: bool,
    /// Skip blank lines.
    pub skipnl: bool,
    /// Skip empty lines.
    pub skipempty: bool,
    /// Conceal the matched text.
    pub conceal: bool,
    /// Conceal start and end of region.
    pub concealends: bool,
    /// This is a fold item.
    pub fold: bool,
    /// Display character for concealed text.
    pub cchar: Option<char>,
    /// Highlight group for contained items.
    pub contains: Vec<String>,
    /// Groups this pattern can be contained in.
    pub containedin: Vec<String>,
    /// Groups to look for at start of match.
    pub nextgroup: Vec<String>,
}

impl PatternOptions {
    /// Create new default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any containment is specified.
    #[must_use]
    pub fn has_containment(&self) -> bool {
        !self.contains.is_empty() || !self.containedin.is_empty()
    }
}

// =============================================================================
// Sync mode
// =============================================================================

/// Synchronization mode for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Start from beginning of file.
    #[default]
    FromStart,
    /// Use C-style comments.
    CComment,
    /// Search for match pattern.
    Match,
    /// Use line count for syncing.
    Lines,
    /// Minimum lines to look back.
    MinLines,
    /// Maximum lines to look back.
    MaxLines,
    /// Number of lines for line continuations.
    LineBreaks,
}

impl SyncMode {
    /// Parse from command argument.
    #[must_use]
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "fromstart" => Some(Self::FromStart),
            "ccomment" => Some(Self::CComment),
            "match" => Some(Self::Match),
            "lines" => Some(Self::Lines),
            "minlines" => Some(Self::MinLines),
            "maxlines" => Some(Self::MaxLines),
            "linebreaks" => Some(Self::LineBreaks),
            _ => None,
        }
    }
}

// =============================================================================
// Command state accessors
// =============================================================================

/// Get the current command line pointer.
///
/// # Safety
/// Must be called during command execution.
#[must_use]
pub unsafe fn cmdlinep() -> *mut *mut c_char {
    nvim_syn_get_cmdlinep()
}

/// Get the synblock for the current window.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn curwin_synblock() -> SynBlockHandle {
    nvim_get_curwin_synblock()
}

/// Get the current window handle.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn curwin() -> WinHandle {
    nvim_get_curwin()
}

/// Get whether to include linked groups in completion.
#[must_use]
pub fn include_link() -> bool {
    unsafe { nvim_syn_get_include_link() != 0 }
}

/// Get whether to include default groups in completion.
#[must_use]
pub fn include_default() -> bool {
    unsafe { nvim_syn_get_include_default() != 0 }
}

/// Get whether to include "None" in completion.
#[must_use]
pub fn include_none() -> bool {
    unsafe { nvim_syn_get_include_none() != 0 }
}

// =============================================================================
// Include tag management
// =============================================================================

/// Get the current running include tag.
///
/// This is used by `:syntax include` to track nested includes.
#[must_use]
pub fn running_inc_tag() -> i32 {
    unsafe { nvim_syn_get_running_inc_tag() }
}

/// Set the running include tag.
///
/// # Safety
/// Must be called from the main thread.
pub unsafe fn set_running_inc_tag(tag: i32) {
    nvim_syn_set_running_inc_tag(tag);
}

// =============================================================================
// Synblock settings accessors
// =============================================================================

/// Get the conceal setting for current window's synblock.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn synblock_conceal_setting(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    nvim_syn_get_conceal_setting(block) != 0
}

/// Get the case-insensitive setting for current window's synblock.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn synblock_ic_setting(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    nvim_syn_get_ic_setting(block) != 0
}

// =============================================================================
// Command expansion helpers
// =============================================================================

/// What to expand in syntax commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandWhat {
    /// Expand `:syn` sub-commands.
    Subcmd,
    /// Expand `:syn case` arguments.
    Case,
    /// Expand `:syn spell` arguments.
    Spell,
    /// Expand `:syn sync` arguments.
    Sync,
    /// Expand `:syn list @cluster` arguments.
    Cluster,
}

impl ExpandWhat {
    /// Get expansion options for a mode.
    #[must_use]
    pub fn options(&self) -> &'static [&'static str] {
        match self {
            Self::Subcmd => SyntaxSubcommand::all_names(),
            Self::Case => &["match", "ignore"],
            Self::Spell => &["toplevel", "notoplevel", "default"],
            Self::Sync => &[
                "ccomment",
                "clear",
                "fromstart",
                "linebreaks=",
                "lines=",
                "linecont",
                "match",
                "maxlines=",
                "minlines=",
                "region",
            ],
            Self::Cluster => &[], // Clusters are dynamically expanded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_subcommand() {
        assert_eq!(
            SyntaxSubcommand::from_name("case"),
            Some(SyntaxSubcommand::Case)
        );
        assert_eq!(
            SyntaxSubcommand::from_name("clear"),
            Some(SyntaxSubcommand::Clear)
        );
        assert_eq!(
            SyntaxSubcommand::from_name("on"),
            Some(SyntaxSubcommand::Enable)
        );
        assert_eq!(
            SyntaxSubcommand::from_name("enable"),
            Some(SyntaxSubcommand::Enable)
        );
        assert_eq!(
            SyntaxSubcommand::from_name(""),
            Some(SyntaxSubcommand::List)
        );
        assert_eq!(SyntaxSubcommand::from_name("invalid"), None);

        assert_eq!(SyntaxSubcommand::Case.name(), "case");
        assert_eq!(SyntaxSubcommand::Enable.name(), "enable");
    }

    #[test]
    fn test_case_mode() {
        assert_eq!(CaseMode::from_arg("match"), Some(CaseMode::Match));
        assert_eq!(CaseMode::from_arg("ignore"), Some(CaseMode::Ignore));
        assert_eq!(CaseMode::from_arg("invalid"), None);

        assert_eq!(CaseMode::Match.as_str(), "match");
        assert_eq!(CaseMode::Ignore.as_str(), "ignore");
    }

    #[test]
    fn test_spell_mode() {
        assert_eq!(SpellMode::from_arg("toplevel"), Some(SpellMode::TopLevel));
        assert_eq!(
            SpellMode::from_arg("notoplevel"),
            Some(SpellMode::NoTopLevel)
        );
        assert_eq!(SpellMode::from_arg("default"), Some(SpellMode::Default));
        assert_eq!(SpellMode::from_arg("invalid"), None);

        assert_eq!(SpellMode::from_raw(spell_mode::DEFAULT), SpellMode::Default);
        assert_eq!(SpellMode::from_raw(spell_mode::TOP), SpellMode::TopLevel);
        assert_eq!(
            SpellMode::from_raw(spell_mode::NOTOP),
            SpellMode::NoTopLevel
        );

        assert_eq!(SpellMode::Default.to_raw(), spell_mode::DEFAULT);
        assert_eq!(SpellMode::TopLevel.to_raw(), spell_mode::TOP);
        assert_eq!(SpellMode::NoTopLevel.to_raw(), spell_mode::NOTOP);
    }

    #[test]
    fn test_pattern_options() {
        let opts = PatternOptions::new();
        assert!(!opts.contained);
        assert!(!opts.transparent);
        assert!(!opts.has_containment());

        let opts = PatternOptions {
            contained: true,
            contains: vec!["Comment".to_string()],
            ..Default::default()
        };
        assert!(opts.contained);
        assert!(opts.has_containment());
    }

    #[test]
    fn test_sync_mode() {
        assert_eq!(SyncMode::from_arg("fromstart"), Some(SyncMode::FromStart));
        assert_eq!(SyncMode::from_arg("ccomment"), Some(SyncMode::CComment));
        assert_eq!(SyncMode::from_arg("match"), Some(SyncMode::Match));
        assert_eq!(SyncMode::from_arg("invalid"), None);
    }

    #[test]
    fn test_expand_what() {
        assert!(!ExpandWhat::Subcmd.options().is_empty());
        assert!(ExpandWhat::Subcmd.options().contains(&"case"));
        assert!(ExpandWhat::Subcmd.options().contains(&"clear"));

        assert_eq!(ExpandWhat::Case.options(), &["match", "ignore"]);
        assert_eq!(
            ExpandWhat::Spell.options(),
            &["toplevel", "notoplevel", "default"]
        );
    }

    #[test]
    fn test_all_subcommand_names() {
        let names = SyntaxSubcommand::all_names();
        // Should have at least the basic commands
        assert!(names.len() >= 15);
        assert!(names.contains(&"case"));
        assert!(names.contains(&"clear"));
        assert!(names.contains(&"keyword"));
        assert!(names.contains(&"match"));
        assert!(names.contains(&"region"));
        // "on" is an alias for "enable"
        assert!(names.contains(&"on"));
        assert!(names.contains(&"enable"));
    }

    #[test]
    fn test_subcommand_roundtrip() {
        // All subcommands should roundtrip through name()
        let cmds = [
            SyntaxSubcommand::Case,
            SyntaxSubcommand::Clear,
            SyntaxSubcommand::Cluster,
            SyntaxSubcommand::Conceal,
            SyntaxSubcommand::Enable,
            SyntaxSubcommand::FoldLevel,
            SyntaxSubcommand::Include,
            SyntaxSubcommand::IsKeyword,
            SyntaxSubcommand::Keyword,
            SyntaxSubcommand::List,
            SyntaxSubcommand::Manual,
            SyntaxSubcommand::Match,
            SyntaxSubcommand::Off,
            SyntaxSubcommand::Region,
            SyntaxSubcommand::Reset,
            SyntaxSubcommand::Spell,
            SyntaxSubcommand::Sync,
        ];

        for cmd in cmds {
            let name = cmd.name();
            let parsed = SyntaxSubcommand::from_name(name);
            assert_eq!(parsed, Some(cmd), "Failed roundtrip for {:?}", cmd);
        }
    }

    #[test]
    fn test_pattern_options_default_values() {
        let opts = PatternOptions::default();
        assert!(!opts.contained);
        assert!(!opts.transparent);
        assert!(!opts.oneline);
        assert!(!opts.keepend);
        assert!(!opts.extend);
        assert!(!opts.excludenl);
        assert!(!opts.skipwhite);
        assert!(!opts.skipnl);
        assert!(!opts.skipempty);
        assert!(!opts.conceal);
        assert!(!opts.concealends);
        assert!(!opts.fold);
        assert!(opts.cchar.is_none());
        assert!(opts.contains.is_empty());
        assert!(opts.containedin.is_empty());
        assert!(opts.nextgroup.is_empty());
    }

    #[test]
    fn test_sync_mode_all_variants() {
        // Test all sync mode variants can be parsed
        assert_eq!(SyncMode::from_arg("fromstart"), Some(SyncMode::FromStart));
        assert_eq!(SyncMode::from_arg("ccomment"), Some(SyncMode::CComment));
        assert_eq!(SyncMode::from_arg("match"), Some(SyncMode::Match));
        assert_eq!(SyncMode::from_arg("lines"), Some(SyncMode::Lines));
        assert_eq!(SyncMode::from_arg("minlines"), Some(SyncMode::MinLines));
        assert_eq!(SyncMode::from_arg("maxlines"), Some(SyncMode::MaxLines));
        assert_eq!(SyncMode::from_arg("linebreaks"), Some(SyncMode::LineBreaks));
    }

    #[test]
    fn test_case_mode_default() {
        // Default should be Match (case-sensitive)
        let default = CaseMode::default();
        assert_eq!(default, CaseMode::Match);
    }

    #[test]
    fn test_spell_mode_default() {
        let default = SpellMode::default();
        assert_eq!(default, SpellMode::Default);
    }

    #[test]
    fn test_sync_mode_default() {
        let default = SyncMode::default();
        assert_eq!(default, SyncMode::FromStart);
    }
}
