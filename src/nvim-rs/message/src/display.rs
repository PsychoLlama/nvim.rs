//! Message display functions
//!
//! Provides Rust implementations for message display operations including
//! ext_messages UI protocol handling, scrolling coordination, and
//! display state management.

use std::ffi::{c_char, c_int, c_void};

use nvim_api::{Array, NvimString, Object, ObjectData};
use nvim_memory::{xcalloc, xfree, xrealloc};

use crate::history::{HlMessage, HlMessageChunk};

// ============================================================================
// Object type constants (kObjectType* enum values from api/private/defs.h)
// ============================================================================
const K_OBJECT_TYPE_NIL: c_int = 0;
const K_OBJECT_TYPE_INTEGER: c_int = 2;
const K_OBJECT_TYPE_STRING: c_int = 4;
const K_OBJECT_TYPE_ARRAY: c_int = 5;

// ============================================================================
// C Function Declarations
// ============================================================================

/// Message kind for ext_messages UI protocol (owned by Rust, also accessed from C).
#[no_mangle]
pub static mut msg_ext_kind: *const c_char = std::ptr::null();

/// Verbose message kind (saved/restored across verbose_enter/leave pairs).
#[no_mangle]
pub static mut verbose_kind: *const c_char = std::ptr::null();

/// Pre-verbose message kind (saved before entering verbose mode).
#[no_mangle]
pub static mut pre_verbose_kind: *const c_char = std::ptr::null();

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// MsgID for the current ext_messages batch (replaces C static msg_ext_id).
/// Initialized to INTEGER_OBJ(0) = { type=kObjectTypeInteger, data.integer=0 }.
#[no_mangle]
pub static mut msg_ext_id: Object = Object {
    obj_type: K_OBJECT_TYPE_INTEGER,
    data: ObjectData { integer: 0 },
};

/// Pointer to the chunks array for the current batch (replaces C static msg_ext_chunks).
#[no_mangle]
pub static mut msg_ext_chunks: *mut Array = std::ptr::null_mut();

/// Growing array for the current text chunk (replaces C static msg_ext_last_chunk).
/// Initialized to GA_INIT(sizeof(char), 40) = { 0, 0, 1, 40, NULL }.
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut msg_ext_last_chunk: GArray = GArray {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 1, // sizeof(char)
    ga_growsize: 40,
    ga_data: std::ptr::null_mut(),
};

/// Attribute for the current chunk, -1 means no active chunk (replaces C static msg_ext_last_attr).
#[no_mangle]
pub static mut msg_ext_last_attr: i32 = -1;

/// Highlight ID for the current chunk (replaces C static msg_ext_last_hl_id).
#[no_mangle]
pub static mut msg_ext_last_hl_id: c_int = 0;

/// Whether current message was added to history (replaces C static msg_ext_history).
#[no_mangle]
pub static mut msg_ext_history: bool = false;

// ============================================================================
// GArray type (matches C garray_T layout exactly)
// ============================================================================

/// Growing array structure matching C garray_T.
#[repr(C)]
pub struct GArray {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

extern "C" {
    static Rows: c_int;

    // Display state accessors (getters in format.rs, only setters needed here)
    static mut msg_row: c_int;
    static mut msg_col: c_int;
    static mut cmdline_row: c_int;
    // (msg_ext_ui_flush and msg_ext_flush_showmode are now implemented in Rust)

    // UI capability check
    fn ui_has(ext: c_int) -> bool;

    // Display coordination
    fn msg_grid_validate();

    // grid_line_mirror and grid_line_flush_if_valid_row are implemented in Rust (grid crate)
    fn grid_line_mirror(width: c_int);
    fn grid_line_flush_if_valid_row();
    static mut cmdmsg_rl: bool;
    static mut msg_grid: crate::ScreenGrid;

    // Position and display state
    static mut sc_col: c_int;
    fn nvim_set_redraw_cmdline(val: bool);

    // Wait state — direct access to C global
    static mut did_wait_return: bool;

    // Overwrite state — direct access to C global
    static mut msg_ext_overwrite: bool;

    // Skip flush state — direct access to C global
    static mut msg_ext_skip_flush: bool;

    // Append state — direct access to C global
    static mut msg_ext_append: bool;

    // Clear EOS flag — direct access to C global
    static mut need_clr_eos: bool;
    // nvim_set_need_clr_eos kept for other crates
    static mut need_wait_return: bool;

    // For msg_ext_emit_chunk
    fn ga_take_string(ga: *mut GArray) -> NvimString;

    // For msg_ext_ui_flush
    fn ui_call_msg_show(
        kind: NvimString,
        content: Array,
        replace_last: bool,
        history: bool,
        append: bool,
        id: Object,
    );
    fn api_free_array(arr: Array);
    // For msg_ext_flush_showmode
    fn ui_call_msg_showmode(content: Array);
    // For cstr_as_string
    fn cstr_as_string(s: *const c_char) -> NvimString;

    // For msg_multihl
    static mut no_wait_return: c_int;
    fn copy_string(s: NvimString, arena: *mut c_void) -> NvimString;
    fn cstr_to_string(s: *const c_char) -> NvimString;
    fn syn_check_group(name: *const c_char, len: usize) -> c_int;
    fn msg_multiline(
        str_: NvimString,
        hl_id: c_int,
        check_int: bool,
        hist: bool,
        need_clear: *mut bool,
    );

    // For ex_messages
    fn emsg(s: *const c_char) -> bool;
    fn redirecting() -> c_int;
    fn ui_call_msg_history_show(entries: Array, prev_cmd: bool);
    fn syn_id2attr(hl_id: c_int) -> c_int;
    static mut msg_silent: c_int;
    #[link_name = "e_invarg"]
    static e_invarg: [c_char; 0];
    fn gettext(s: *const c_char) -> *const c_char;
    fn nvim_eap_get_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_get_addr_count(eap: *const c_void) -> c_int;
    fn nvim_eap_get_line2(eap: *const c_void) -> c_int;
    fn nvim_eap_get_skip(eap: *const c_void) -> c_int;
    fn msg_hist_clear(keep: c_int);
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

// ============================================================================
// Private helpers
// ============================================================================

/// Push an item onto a kvec Array (equivalent to kv_push for Array).
///
/// # Safety
/// Must be called with a valid Array.
unsafe fn array_push(arr: &mut Array, item: Object) {
    if arr.size == arr.capacity {
        let new_cap = if arr.capacity == 0 {
            8
        } else {
            arr.capacity * 2
        };
        arr.items = xrealloc(
            arr.items.cast::<c_void>(),
            new_cap * std::mem::size_of::<Object>(),
        )
        .cast::<Object>();
        arr.capacity = new_cap;
    }
    *arr.items.add(arr.size) = item;
    arr.size += 1;
}

/// Push an item onto an HlMessage (equivalent to kv_push for HlMessage).
///
/// # Safety
/// Must be called with a valid HlMessage.
unsafe fn hl_msg_push(msg: &mut HlMessage, chunk: HlMessageChunk) {
    if msg.size == msg.capacity {
        let new_cap = if msg.capacity == 0 {
            8
        } else {
            msg.capacity * 2
        };
        msg.items = xrealloc(
            msg.items.cast::<c_void>(),
            new_cap * std::mem::size_of::<HlMessageChunk>(),
        )
        .cast::<HlMessageChunk>();
        msg.capacity = new_cap;
    }
    *msg.items.add(msg.size) = chunk;
    msg.size += 1;
}

/// Emit the current text chunk into msg_ext_chunks.
///
/// If msg_ext_last_attr == -1, there is no active chunk and this is a no-op.
/// Equivalent to the C static function `msg_ext_emit_chunk()`.
/// Exported as `msg_ext_emit_chunk` so msg_puts_display (still in C) can call it.
///
/// # Safety
/// Accesses and modifies msg_ext_* globals.
#[no_mangle]
pub unsafe extern "C" fn msg_ext_emit_chunk() {
    if msg_ext_chunks.is_null() {
        let _ = msg_ext_init_chunks_impl();
    }
    if msg_ext_last_attr == -1 {
        return;
    }
    let mut chunk = Array {
        size: 0,
        capacity: 0,
        items: std::ptr::null_mut(),
    };
    array_push(
        &mut chunk,
        Object {
            obj_type: K_OBJECT_TYPE_INTEGER,
            data: ObjectData {
                integer: i64::from(msg_ext_last_attr),
            },
        },
    );
    msg_ext_last_attr = -1;
    let text = ga_take_string(std::ptr::addr_of_mut!(msg_ext_last_chunk));
    array_push(
        &mut chunk,
        Object {
            obj_type: K_OBJECT_TYPE_STRING,
            data: ObjectData { string: text },
        },
    );
    array_push(
        &mut chunk,
        Object {
            obj_type: K_OBJECT_TYPE_INTEGER,
            data: ObjectData {
                integer: i64::from(msg_ext_last_hl_id),
            },
        },
    );
    array_push(
        &mut *msg_ext_chunks,
        Object {
            obj_type: K_OBJECT_TYPE_ARRAY,
            data: ObjectData { array: chunk },
        },
    );
}

/// Clear "msg_ext_chunks" before flushing so that ui_flush() does not re-emit
/// the same message recursively.
/// Returns the old (to-be-freed) Array pointer.
/// Equivalent to the C static function `msg_ext_init_chunks()`.
///
/// # Safety
/// Modifies msg_ext_chunks and msg_col globals.
unsafe fn msg_ext_init_chunks_impl() -> *mut Array {
    let tofree = msg_ext_chunks;
    msg_ext_chunks = xcalloc(1, std::mem::size_of::<Array>()).cast::<Array>();
    msg_col = 0;
    tofree
}

// ============================================================================
// ext_messages Protocol Functions
// ============================================================================

/// Set the message kind for ext_messages UI protocol.
///
/// This sets the kind label for the next batch of messages
/// sent to external UIs.
///
/// # Arguments
/// * `msg_kind` - The message kind string (e.g., "emsg", "echo")
///
/// # Safety
/// - `msg_kind` must be a valid NUL-terminated C string or NULL
#[export_name = "msg_ext_set_kind"]
pub unsafe extern "C" fn rs_msg_ext_set_kind(msg_kind: *const c_char) {
    // Don't change the label of an existing batch:
    rs_msg_ext_ui_flush();
    msg_ext_kind = msg_kind;
}

/// Flush pending messages to ext_messages UI.
///
/// Emits any accumulated message chunks to external UIs using the `msg_show`
/// UI event. Equivalent to the C function `msg_ext_ui_flush()`.
///
/// # Safety
/// Accesses and modifies msg_ext_* globals; calls UI functions.
#[export_name = "msg_ext_ui_flush"]
pub unsafe extern "C" fn rs_msg_ext_ui_flush() {
    if !ui_has(K_UI_MESSAGES) {
        msg_ext_kind = std::ptr::null();
        return;
    } else if msg_ext_skip_flush {
        return;
    }

    msg_ext_emit_chunk();

    // Only proceed if we have content to send
    if msg_ext_chunks.is_null() || (*msg_ext_chunks).size == 0 {
        return;
    }

    let tofree_ptr = msg_ext_init_chunks_impl();
    let tofree = *tofree_ptr;

    ui_call_msg_show(
        cstr_as_string(msg_ext_kind),
        tofree,
        msg_ext_overwrite,
        msg_ext_history,
        msg_ext_append,
        msg_ext_id,
    );

    // Clear info after emitting message.
    if msg_ext_history {
        api_free_array(tofree);
    } else {
        // Add to history as temporary message for "g<".
        let mut msg = HlMessage {
            size: 0,
            capacity: 0,
            items: std::ptr::null_mut(),
        };
        for i in 0..tofree.size {
            let entry = &*tofree.items.add(i);
            // entry.data.array.items: [Integer(attr), String(text), Integer(hl_id)]
            let chunk_arr = entry.data.array.items;
            let text: NvimString = (*chunk_arr.add(1)).data.string;
            #[allow(clippy::cast_possible_truncation)]
            let hl_id = (*chunk_arr.add(2)).data.integer as c_int;
            hl_msg_push(&mut msg, HlMessageChunk::new(text.data, text.size, hl_id));
            xfree(chunk_arr.cast::<c_void>());
        }
        xfree(tofree.items.cast::<c_void>());
        crate::history::rs_msg_hist_add_multihl(
            Object {
                obj_type: K_OBJECT_TYPE_INTEGER,
                data: ObjectData { integer: 0 },
            },
            msg,
            true,
            std::ptr::null_mut(),
        );
    }

    xfree(tofree_ptr.cast::<c_void>());
    msg_ext_overwrite = false;
    msg_ext_history = false;
    msg_ext_append = false;
    msg_ext_kind = std::ptr::null();
    msg_ext_id = Object {
        obj_type: K_OBJECT_TYPE_INTEGER,
        data: ObjectData { integer: 0 },
    };
}

/// Flush showmode messages to ext_messages UI.
///
/// Uses the separate `msg_showmode` event which doesn't interrupt normal
/// message flow. Equivalent to the C function `msg_ext_flush_showmode()`.
///
/// # Safety
/// Accesses and modifies msg_ext_* globals; calls UI functions.
#[export_name = "msg_ext_flush_showmode"]
pub unsafe extern "C" fn rs_msg_ext_flush_showmode() {
    static mut CLEAR: bool = false;

    if ui_has(K_UI_MESSAGES) && (msg_ext_last_attr != -1 || CLEAR) {
        CLEAR = msg_ext_last_attr != -1;
        msg_ext_emit_chunk();
        let tofree_ptr = msg_ext_init_chunks_impl();
        let tofree = *tofree_ptr;
        ui_call_msg_showmode(tofree);
        api_free_array(tofree);
        xfree(tofree_ptr.cast::<c_void>());
    }
}

/// Check if UI has ext_messages capability.
///
/// # Returns
/// * Non-zero if ext_messages UI protocol is active
/// * Zero if using traditional terminal messages
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_has_messages() -> c_int {
    c_int::from(ui_has(K_UI_MESSAGES))
}

// ============================================================================
// Display Position Functions
// ============================================================================

// Note: rs_msg_row(), rs_msg_col(), rs_cmdline_row() and their setters
// are defined in format.rs to avoid duplication

/// Set the command line row.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_cmdline_row(val: c_int) {
    cmdline_row = val;
}

/// Reset message position to start of message area.
///
/// Sets msg_col to 0.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_reset_col() {
    msg_col = 0;
}

/// Move message position to a new line.
///
/// Sets msg_col to 0 and increments msg_row.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_newline() {
    msg_col = 0;
    let row = msg_row;
    msg_row = row + 1;
}

// ============================================================================
// Display State Functions
// ============================================================================

/// Check if wait_return was called.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_did_wait_return() -> c_int {
    c_int::from(did_wait_return)
}

/// Set the did_wait_return flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_did_wait_return(val: c_int) {
    did_wait_return = val != 0;
}

/// Check if ext_messages should overwrite previous message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_overwrite() -> c_int {
    c_int::from(msg_ext_overwrite)
}

/// Set the ext_messages overwrite flag.
///
/// When true, the next message will overwrite the previous one
/// in external UIs.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_ext_overwrite(val: c_int) {
    msg_ext_overwrite = val != 0;
}

/// Check if ext_messages flush is skipped.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_skip_flush() -> c_int {
    c_int::from(msg_ext_skip_flush)
}

/// Set the ext_messages skip flush flag.
///
/// When true, message chunks are accumulated but not flushed.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_ext_skip_flush(val: c_int) {
    msg_ext_skip_flush = val != 0;
}

/// Check if clear to end of screen is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_clr_eos() -> c_int {
    c_int::from(need_clr_eos)
}

/// Set the need_clr_eos flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_clr_eos(val: c_int) {
    need_clr_eos = (val) != 0;
}

// ============================================================================
// Display Coordination Functions
// ============================================================================

/// Check if message display overlaps with command/ruler.
///
/// If the written message runs into the shown command or ruler,
/// sets need_wait_return and schedules a redraw.
///
/// # Safety
/// Calls C accessor functions that read and modify global state.
#[export_name = "msg_check"]
pub unsafe extern "C" fn rs_msg_check() {
    if ui_has(K_UI_MESSAGES) {
        return;
    }
    if msg_row == Rows - 1 && msg_col >= sc_col {
        need_wait_return = true;
        nvim_set_redraw_cmdline(true);
    }
}

/// Validate the message grid for output.
///
/// Ensures the message grid is properly allocated and sized
/// for message display.
///
/// # Safety
/// Calls C function that may allocate memory.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_validate() {
    msg_grid_validate();
}

/// Flush pending line content to display.
///
/// For right-to-left command lines, mirrors the line first.
/// Then flushes the grid line if the row is valid.
///
/// Equivalent to the C function `msg_line_flush()`.
///
/// # Safety
/// Calls grid functions that modify display state.
#[export_name = "msg_line_flush"]
pub unsafe extern "C" fn rs_msg_line_flush() {
    if cmdmsg_rl {
        grid_line_mirror(msg_grid.cols);
    }
    grid_line_flush_if_valid_row();
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Begin an ext_messages batch with the given kind.
///
/// Sets the message kind and enables skip_flush to accumulate
/// chunks before sending.
///
/// # Arguments
/// * `kind` - Message kind string
///
/// # Safety
/// - `kind` must be a valid NUL-terminated C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_begin(kind: *const c_char) {
    rs_msg_ext_set_kind(kind);
    msg_ext_skip_flush = true;
}

/// End an ext_messages batch and flush.
///
/// Clears skip_flush and flushes accumulated chunks to the UI.
///
/// # Safety
/// Calls C functions that may emit UI events.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_end() {
    msg_ext_skip_flush = false;
    rs_msg_ext_ui_flush();
}

/// Reset display state for new message sequence.
///
/// Clears position and state flags for a fresh start.
///
/// # Safety
/// Calls C mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_display_reset() {
    msg_col = 0;
    did_wait_return = false;
    need_clr_eos = false;
}

/// Check if displaying to external UI.
///
/// Returns true if messages go to ext_messages UI rather than
/// the internal terminal grid.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_using_ext_messages() -> c_int {
    c_int::from(ui_has(K_UI_MESSAGES))
}

// ============================================================================
// Phase 12: msg_multihl + format_progress_message migrated to Rust
// ============================================================================

/// Layout-compatible representation of C `MessageData` struct.
/// Size: 64 bytes (i64 + NvimString + NvimString + KVec).
#[repr(C)]
pub struct MessageData {
    pub percent: i64,       // offset 0
    pub title: NvimString,  // offset 8 (16 bytes)
    pub status: NvimString, // offset 24 (16 bytes)
    _data: [u8; 24],        // offset 40: opaque Dict field (24 bytes)
}

/// `msg_id_next` — counter for auto-generated message IDs (was C static).
#[no_mangle]
pub static mut msg_id_next: i64 = 1;

/// Compare a C string pointer with a Rust string literal, safely.
///
/// Returns true if `ptr` is non-null and the C string equals `s`.
unsafe fn cstr_eq(ptr: *const c_char, s: &str) -> bool {
    if ptr.is_null() {
        return false;
    }
    let cstr = std::ffi::CStr::from_ptr(ptr);
    cstr.to_bytes() == s.as_bytes()
}

/// Format a progress message with title and percent prefix.
///
/// Equivalent to C static `format_progress_message()`.
///
/// # Safety
/// `msg_data` must be a valid pointer to `MessageData`.
unsafe fn format_progress_message_impl(
    hl_msg: crate::history::HlMessage,
    msg_data: *const MessageData,
) -> crate::history::HlMessage {
    use crate::history::{HlMessage, HlMessageChunk};
    let mut updated: HlMessage = HlMessage {
        size: 0,
        capacity: 0,
        items: std::ptr::null_mut(),
    };

    // Add title prefix if present
    if (*msg_data).title.size != 0 {
        let status = (*msg_data).status.data;
        #[allow(clippy::cast_possible_truncation)]
        let hl_id: c_int = if status.is_null() {
            0
        } else if cstr_eq(status, "success") {
            syn_check_group(c"OkMsg".as_ptr(), 5)
        } else if cstr_eq(status, "failed") {
            syn_check_group(c"ErrorMsg".as_ptr(), 8)
        } else if cstr_eq(status, "running") {
            syn_check_group(c"MoreMsg".as_ptr(), 7)
        } else if cstr_eq(status, "cancel") {
            syn_check_group(c"WarningMsg".as_ptr(), 10)
        } else {
            0
        };
        let title_copy = copy_string((*msg_data).title, std::ptr::null_mut());
        crate::display::hl_msg_push_impl(
            &mut updated,
            HlMessageChunk::new(title_copy.data, title_copy.size, hl_id),
        );
        let colon = cstr_to_string(c": ".as_ptr());
        crate::display::hl_msg_push_impl(
            &mut updated,
            HlMessageChunk::new(colon.data, colon.size, 0),
        );
    }

    // Add percent prefix if present
    if (*msg_data).percent > 0 {
        #[allow(clippy::cast_possible_truncation)]
        let pct = (*msg_data).percent as i32;
        let pct_str = format!("{pct:3}% ");
        let pct_nvim = cstr_to_string(std::ffi::CString::new(pct_str).unwrap_or_default().as_ptr());
        #[allow(clippy::cast_possible_truncation)]
        let hl_id = syn_check_group(c"WarningMsg".as_ptr(), 10);
        crate::display::hl_msg_push_impl(
            &mut updated,
            HlMessageChunk::new(pct_nvim.data, pct_nvim.size, hl_id),
        );
    }

    if updated.size != 0 {
        // Copy all chunks from hl_msg into updated
        for i in 0..hl_msg.size {
            let chunk = &*hl_msg.items.add(i);
            let text_copy = copy_string(
                NvimString {
                    data: chunk.text_data,
                    size: chunk.text_size,
                },
                std::ptr::null_mut(),
            );
            crate::display::hl_msg_push_impl(
                &mut updated,
                HlMessageChunk::new(text_copy.data, text_copy.size, chunk.hl_id),
            );
        }
        updated
    } else {
        hl_msg
    }
}

/// Helper to push an HlMessageChunk onto an HlMessage (exposed for format_progress_message_impl).
///
/// # Safety
/// `msg` must point to a valid HlMessage.
pub(crate) unsafe fn hl_msg_push_impl(
    msg: &mut crate::history::HlMessage,
    chunk: crate::history::HlMessageChunk,
) {
    hl_msg_push(msg, chunk);
}

/// Print message chunks, each with their own highlight ID.
///
/// Equivalent to the C `msg_multihl()` function.
///
/// # Safety
/// Accesses and modifies global message state.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
#[export_name = "msg_multihl"]
pub unsafe extern "C" fn rs_msg_multihl(
    id: Object,
    hl_msg: crate::history::HlMessage,
    kind: *const c_char,
    history: bool,
    err: bool,
    msg_data: *mut MessageData,
    needs_msg_clear: *mut bool,
) -> Object {
    no_wait_return += 1;
    crate::output_core::rs_msg_start();
    crate::output_core::rs_msg_clr_eos();
    let mut need_clear = false;
    let mut hl_msg_updated = false;
    msg_ext_history = history;
    if !kind.is_null() {
        rs_msg_ext_set_kind(kind);
    }
    crate::misc::is_multihl = true;
    msg_ext_skip_flush = true;

    // Assign a new ID if not given, or generate one if the given one is invalid.
    let mut id = id;
    if id.obj_type == K_OBJECT_TYPE_NIL {
        id = Object {
            obj_type: K_OBJECT_TYPE_INTEGER,
            data: ObjectData {
                integer: msg_id_next,
            },
        };
        msg_id_next += 1;
    } else if id.obj_type == K_OBJECT_TYPE_INTEGER {
        if id.data.integer <= 0 {
            id = Object {
                obj_type: K_OBJECT_TYPE_INTEGER,
                data: ObjectData {
                    integer: msg_id_next,
                },
            };
            msg_id_next += 1;
        } else if msg_id_next < id.data.integer {
            msg_id_next = id.data.integer + 1;
        }
    }
    msg_ext_id = id;

    // Progress messages get title/percent prefix
    let mut hl_msg = hl_msg;
    if !kind.is_null() && cstr_eq(kind, "progress") && !msg_data.is_null() {
        let formatted = format_progress_message_impl(hl_msg, msg_data);
        if formatted.items != hl_msg.items {
            if !needs_msg_clear.is_null() {
                *needs_msg_clear = true;
            }
            hl_msg_updated = true;
            hl_msg = formatted;
        }
    }

    // Print each chunk
    for i in 0..hl_msg.size {
        let chunk = &*hl_msg.items.add(i);
        let text = NvimString {
            data: chunk.text_data,
            size: chunk.text_size,
        };
        if err {
            let _ = crate::error::rs_emsg_multiline(chunk.text_data, kind, chunk.hl_id, 1);
        } else {
            msg_multiline(text, chunk.hl_id, true, false, &raw mut need_clear);
        }
    }

    if history && hl_msg.size > 0 {
        crate::history::rs_msg_hist_add_multihl(id, hl_msg, false, msg_data.cast::<c_void>());
    }

    msg_ext_skip_flush = false;
    crate::misc::is_multihl = false;
    no_wait_return -= 1;
    let _ = crate::output_core::rs_msg_end();

    if hl_msg_updated && !(history && hl_msg.size > 0) {
        crate::history::rs_hl_msg_free(hl_msg);
    }

    id
}

/// Implementation of the :messages command.
///
/// Displays previous messages. When count is given, show only last <count> messages.
/// With "clear" argument, clears the message history.
///
/// # Safety
/// Accesses global message history state.
#[allow(clippy::too_many_lines)]
#[export_name = "ex_messages"]
pub unsafe extern "C" fn rs_ex_messages(eap: *const c_void) {
    let arg = nvim_eap_get_arg(eap);
    // Check for "clear" argument
    if strcmp(arg, c"clear".as_ptr()) == 0 {
        let keep = if nvim_eap_get_addr_count(eap) != 0 {
            nvim_eap_get_line2(eap)
        } else {
            0
        };
        msg_hist_clear(keep);
        return;
    }

    // Non-empty argument (other than "clear") is invalid
    if *arg != 0 {
        emsg(gettext(e_invarg.as_ptr()));
        return;
    }

    let mut entries = Array {
        size: 0,
        capacity: 0,
        items: std::ptr::null_mut(),
    };

    let p_start: *mut crate::history::MessageHistoryEntry = if nvim_eap_get_skip(eap) != 0 {
        crate::history::msg_hist_temp
    } else {
        crate::history::msg_hist_first
    };

    let mut skip = if nvim_eap_get_addr_count(eap) != 0 {
        crate::history::msg_hist_len - nvim_eap_get_line2(eap)
    } else {
        0
    };

    let mut p = p_start;
    while !p.is_null() {
        let entry = &*p;
        // Skip temporary entries or entries covered by count (skip-- > 0 in C)
        let do_skip = {
            let s = skip;
            skip -= 1;
            s > 0
        };
        if (entry.temp && nvim_eap_get_skip(eap) == 0) || do_skip {
            p = entry.next;
            continue;
        }

        if ui_has(K_UI_MESSAGES) && msg_silent == 0 {
            // Build [kind, content, append] entry for ext_messages UI
            let mut ui_entry = Array {
                size: 0,
                capacity: 0,
                items: std::ptr::null_mut(),
            };
            array_push(
                &mut ui_entry,
                Object {
                    obj_type: K_OBJECT_TYPE_STRING,
                    data: ObjectData {
                        string: cstr_as_string(entry.kind),
                    },
                },
            );
            // Build content array: [[attr, text, hl_id], ...]
            let mut content = Array {
                size: 0,
                capacity: 0,
                items: std::ptr::null_mut(),
            };
            for i in 0..entry.msg.size {
                let chunk = &*entry.msg.items.add(i);
                let attr = if chunk.hl_id != 0 {
                    syn_id2attr(chunk.hl_id)
                } else {
                    0
                };
                let mut content_entry = Array {
                    size: 0,
                    capacity: 0,
                    items: std::ptr::null_mut(),
                };
                array_push(
                    &mut content_entry,
                    Object {
                        obj_type: K_OBJECT_TYPE_INTEGER,
                        data: ObjectData {
                            integer: i64::from(attr),
                        },
                    },
                );
                let chunk_text = NvimString {
                    data: chunk.text_data,
                    size: chunk.text_size,
                };
                array_push(
                    &mut content_entry,
                    Object {
                        obj_type: K_OBJECT_TYPE_STRING,
                        data: ObjectData {
                            string: copy_string(chunk_text, std::ptr::null_mut()),
                        },
                    },
                );
                array_push(
                    &mut content_entry,
                    Object {
                        obj_type: K_OBJECT_TYPE_INTEGER,
                        data: ObjectData {
                            integer: i64::from(chunk.hl_id),
                        },
                    },
                );
                array_push(
                    &mut content,
                    Object {
                        obj_type: K_OBJECT_TYPE_ARRAY,
                        data: ObjectData {
                            array: content_entry,
                        },
                    },
                );
            }
            array_push(
                &mut ui_entry,
                Object {
                    obj_type: K_OBJECT_TYPE_ARRAY,
                    data: ObjectData { array: content },
                },
            );
            array_push(
                &mut ui_entry,
                Object {
                    obj_type: 1, // K_OBJECT_TYPE_BOOLEAN
                    data: ObjectData {
                        boolean: entry.append,
                    },
                },
            );
            array_push(
                &mut entries,
                Object {
                    obj_type: K_OBJECT_TYPE_ARRAY,
                    data: ObjectData { array: ui_entry },
                },
            );
        }

        if redirecting() != 0 || !ui_has(K_UI_MESSAGES) {
            msg_silent += c_int::from(ui_has(K_UI_MESSAGES));
            let mut needs_clear = false;
            rs_msg_multihl(
                Object {
                    obj_type: K_OBJECT_TYPE_INTEGER,
                    data: ObjectData { integer: 0 },
                },
                entry.msg,
                entry.kind,
                false,
                false,
                std::ptr::null_mut(),
                &raw mut needs_clear,
            );
            msg_silent -= c_int::from(ui_has(K_UI_MESSAGES));
        }

        p = entry.next;
    }

    if entries.size > 0 {
        ui_call_msg_history_show(entries, nvim_eap_get_skip(eap) != 0);
        api_free_array(entries);
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
