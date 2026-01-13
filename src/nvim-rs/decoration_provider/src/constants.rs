//! Constants for decoration provider system
//!
//! These constants match the definitions in `decoration_defs.h` and
//! `decoration_provider.c`.

use std::ffi::c_int;

// =============================================================================
// LuaRef Constants
// =============================================================================

/// Value indicating no Lua reference is set.
/// Matches LUA_NOREF from Lua header.
pub const LUA_NOREF: c_int = -2;

/// Value indicating a nil Lua reference.
/// Matches LUA_REFNIL from Lua header.
pub const LUA_REFNIL: c_int = -1;

// =============================================================================
// Provider State Constants
// =============================================================================

/// Provider is active and will be invoked.
pub const DECOR_PROVIDER_ACTIVE: c_int = 1;

/// Provider is disabled for current window only.
pub const DECOR_PROVIDER_WIN_DISABLED: c_int = 2;

/// Provider is disabled for current redraw cycle.
pub const DECOR_PROVIDER_REDRAW_DISABLED: c_int = 3;

/// Provider is fully disabled.
pub const DECOR_PROVIDER_DISABLED: c_int = 4;

// =============================================================================
// Callback Error Limits
// =============================================================================

/// Maximum errors before provider is auto-disabled.
pub const CB_MAX_ERROR: u8 = 5;

// =============================================================================
// Virtual Text Position Constants
// =============================================================================

/// Virtual text at end of line.
pub const VPOS_END_OF_LINE: c_int = 0;

/// Virtual text at end of line, right-aligned.
pub const VPOS_END_OF_LINE_RIGHT_ALIGN: c_int = 1;

/// Virtual text inline (inserted into text).
pub const VPOS_INLINE: c_int = 2;

/// Virtual text overlaid on existing text.
pub const VPOS_OVERLAY: c_int = 3;

/// Virtual text right-aligned in window.
pub const VPOS_RIGHT_ALIGN: c_int = 4;

/// Virtual text at specific window column.
pub const VPOS_WIN_COL: c_int = 5;

// =============================================================================
// Highlight Mode Constants
// =============================================================================

/// Highlight mode unknown/unset.
pub const HL_MODE_UNKNOWN: c_int = 0;

/// Highlight mode: replace existing.
pub const HL_MODE_REPLACE: c_int = 1;

/// Highlight mode: combine with existing.
pub const HL_MODE_COMBINE: c_int = 2;

/// Highlight mode: blend with existing.
pub const HL_MODE_BLEND: c_int = 3;

// =============================================================================
// Sign/Highlight Flags
// =============================================================================

/// Decoration is a sign.
pub const KSH_IS_SIGN: u16 = 1;

/// Highlight extends to end of line.
pub const KSH_HL_EOL: u16 = 2;

/// UI is watching this decoration.
pub const KSH_UI_WATCHED: u16 = 4;

/// UI is watching overlay.
pub const KSH_UI_WATCHED_OVERLAY: u16 = 8;

/// Spell checking enabled.
pub const KSH_SPELL_ON: u16 = 16;

/// Spell checking disabled.
pub const KSH_SPELL_OFF: u16 = 32;

/// Conceal enabled.
pub const KSH_CONCEAL: u16 = 64;

/// Conceal lines (fold-like).
pub const KSH_CONCEAL_LINES: u16 = 128;

// =============================================================================
// Virtual Text Flags
// =============================================================================

/// Virtual text is actually virtual lines.
pub const KVT_IS_LINES: u8 = 1;

/// Virtual text is hidden.
pub const KVT_HIDE: u8 = 2;

/// Virtual lines are above the line.
pub const KVT_LINES_ABOVE: u8 = 4;

/// Repeat linebreak for virtual text.
pub const KVT_REPEAT_LINEBREAK: u8 = 8;

// =============================================================================
// Virtual Line Flags
// =============================================================================

/// Virtual line starts at left column (ignoring signs/numbers).
pub const KVL_LEFTCOL: c_int = 1;

/// Virtual line can scroll horizontally with 'nowrap'.
pub const KVL_SCROLL: c_int = 2;

// =============================================================================
// Decoration Priority
// =============================================================================

/// Base priority for decorations.
pub const DECOR_PRIORITY_BASE: u16 = 0x1000;

/// Invalid decoration ID.
pub const DECOR_ID_INVALID: u32 = u32::MAX;

// =============================================================================
// FFI Exports - Constants
// =============================================================================

/// Get LUA_NOREF constant.
#[no_mangle]
pub extern "C" fn rs_decor_provider_lua_noref() -> c_int {
    LUA_NOREF
}

/// Get CB_MAX_ERROR constant.
#[no_mangle]
pub extern "C" fn rs_decor_provider_cb_max_error() -> u8 {
    CB_MAX_ERROR
}

/// Get DECOR_PROVIDER_ACTIVE constant.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_active() -> c_int {
    DECOR_PROVIDER_ACTIVE
}

/// Get DECOR_PROVIDER_WIN_DISABLED constant.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_win_disabled() -> c_int {
    DECOR_PROVIDER_WIN_DISABLED
}

/// Get DECOR_PROVIDER_REDRAW_DISABLED constant.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_redraw_disabled() -> c_int {
    DECOR_PROVIDER_REDRAW_DISABLED
}

/// Get DECOR_PROVIDER_DISABLED constant.
#[no_mangle]
pub extern "C" fn rs_decor_provider_state_disabled() -> c_int {
    DECOR_PROVIDER_DISABLED
}

/// Get DECOR_PRIORITY_BASE constant.
#[no_mangle]
pub extern "C" fn rs_decor_priority_base() -> u16 {
    DECOR_PRIORITY_BASE
}

/// Get DECOR_ID_INVALID constant.
#[no_mangle]
pub extern "C" fn rs_decor_id_invalid() -> u32 {
    DECOR_ID_INVALID
}

/// Check if provider state is active.
#[no_mangle]
pub extern "C" fn rs_decor_provider_is_active(state: c_int) -> bool {
    state == DECOR_PROVIDER_ACTIVE
}

/// Check if provider state is any kind of disabled.
#[no_mangle]
pub extern "C" fn rs_decor_provider_is_disabled(state: c_int) -> bool {
    state == DECOR_PROVIDER_DISABLED
}

/// Check if provider state allows window callbacks.
#[no_mangle]
pub extern "C" fn rs_decor_provider_allows_win(state: c_int) -> bool {
    state == DECOR_PROVIDER_ACTIVE
}

/// Check if provider state allows line callbacks.
#[no_mangle]
pub extern "C" fn rs_decor_provider_allows_line(state: c_int) -> bool {
    state == DECOR_PROVIDER_ACTIVE
}

/// Check if Lua reference is valid (not NOREF or REFNIL).
#[no_mangle]
pub extern "C" fn rs_lua_ref_is_valid(lua_ref: c_int) -> bool {
    lua_ref != LUA_NOREF && lua_ref != LUA_REFNIL
}

/// Check if sign/highlight flags indicate a sign.
#[no_mangle]
pub extern "C" fn rs_decor_sh_is_sign(flags: u16) -> bool {
    (flags & KSH_IS_SIGN) != 0
}

/// Check if sign/highlight flags indicate EOL highlight.
#[no_mangle]
pub extern "C" fn rs_decor_sh_hl_eol(flags: u16) -> bool {
    (flags & KSH_HL_EOL) != 0
}

/// Check if sign/highlight flags indicate UI watching.
#[no_mangle]
pub extern "C" fn rs_decor_sh_ui_watched(flags: u16) -> bool {
    (flags & KSH_UI_WATCHED) != 0
}

/// Check if virtual text flags indicate lines.
#[no_mangle]
pub extern "C" fn rs_decor_vt_is_lines(flags: u8) -> bool {
    (flags & KVT_IS_LINES) != 0
}

/// Check if virtual text flags indicate hidden.
#[no_mangle]
pub extern "C" fn rs_decor_vt_is_hidden(flags: u8) -> bool {
    (flags & KVT_HIDE) != 0
}

/// Check if virtual text flags indicate lines above.
#[no_mangle]
pub extern "C" fn rs_decor_vt_lines_above(flags: u8) -> bool {
    (flags & KVT_LINES_ABOVE) != 0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_ref_constants() {
        assert_eq!(LUA_NOREF, -2);
        assert_eq!(LUA_REFNIL, -1);
        assert!(!rs_lua_ref_is_valid(LUA_NOREF));
        assert!(!rs_lua_ref_is_valid(LUA_REFNIL));
        assert!(rs_lua_ref_is_valid(0));
        assert!(rs_lua_ref_is_valid(1));
        assert!(rs_lua_ref_is_valid(100));
    }

    #[test]
    fn test_provider_state_constants() {
        assert_eq!(DECOR_PROVIDER_ACTIVE, 1);
        assert_eq!(DECOR_PROVIDER_WIN_DISABLED, 2);
        assert_eq!(DECOR_PROVIDER_REDRAW_DISABLED, 3);
        assert_eq!(DECOR_PROVIDER_DISABLED, 4);
    }

    #[test]
    fn test_provider_state_checks() {
        assert!(rs_decor_provider_is_active(DECOR_PROVIDER_ACTIVE));
        assert!(!rs_decor_provider_is_active(DECOR_PROVIDER_DISABLED));

        assert!(rs_decor_provider_is_disabled(DECOR_PROVIDER_DISABLED));
        assert!(!rs_decor_provider_is_disabled(DECOR_PROVIDER_ACTIVE));

        assert!(rs_decor_provider_allows_win(DECOR_PROVIDER_ACTIVE));
        assert!(!rs_decor_provider_allows_win(DECOR_PROVIDER_WIN_DISABLED));
    }

    #[test]
    fn test_virt_text_pos_constants() {
        assert_eq!(VPOS_END_OF_LINE, 0);
        assert_eq!(VPOS_END_OF_LINE_RIGHT_ALIGN, 1);
        assert_eq!(VPOS_INLINE, 2);
        assert_eq!(VPOS_OVERLAY, 3);
        assert_eq!(VPOS_RIGHT_ALIGN, 4);
        assert_eq!(VPOS_WIN_COL, 5);
    }

    #[test]
    fn test_hl_mode_constants() {
        assert_eq!(HL_MODE_UNKNOWN, 0);
        assert_eq!(HL_MODE_REPLACE, 1);
        assert_eq!(HL_MODE_COMBINE, 2);
        assert_eq!(HL_MODE_BLEND, 3);
    }

    #[test]
    fn test_sign_highlight_flags() {
        assert!(rs_decor_sh_is_sign(KSH_IS_SIGN));
        assert!(!rs_decor_sh_is_sign(0));

        assert!(rs_decor_sh_hl_eol(KSH_HL_EOL));
        assert!(!rs_decor_sh_hl_eol(KSH_IS_SIGN));

        assert!(rs_decor_sh_ui_watched(KSH_UI_WATCHED));
        assert!(!rs_decor_sh_ui_watched(KSH_IS_SIGN));

        // Test combined flags
        let combined = KSH_IS_SIGN | KSH_HL_EOL | KSH_UI_WATCHED;
        assert!(rs_decor_sh_is_sign(combined));
        assert!(rs_decor_sh_hl_eol(combined));
        assert!(rs_decor_sh_ui_watched(combined));
    }

    #[test]
    fn test_virt_text_flags() {
        assert!(rs_decor_vt_is_lines(KVT_IS_LINES));
        assert!(!rs_decor_vt_is_lines(0));

        assert!(rs_decor_vt_is_hidden(KVT_HIDE));
        assert!(!rs_decor_vt_is_hidden(KVT_IS_LINES));

        assert!(rs_decor_vt_lines_above(KVT_LINES_ABOVE));
        assert!(!rs_decor_vt_lines_above(KVT_HIDE));

        // Test combined flags
        let combined = KVT_IS_LINES | KVT_HIDE | KVT_LINES_ABOVE;
        assert!(rs_decor_vt_is_lines(combined));
        assert!(rs_decor_vt_is_hidden(combined));
        assert!(rs_decor_vt_lines_above(combined));
    }

    #[test]
    fn test_priority_constants() {
        assert_eq!(DECOR_PRIORITY_BASE, 0x1000);
        assert_eq!(DECOR_ID_INVALID, u32::MAX);
    }

    #[test]
    fn test_cb_max_error() {
        assert_eq!(CB_MAX_ERROR, 5);
        assert_eq!(rs_decor_provider_cb_max_error(), 5);
    }
}
