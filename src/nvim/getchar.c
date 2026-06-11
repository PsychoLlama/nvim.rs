// getchar.c: Code related to getting a character from the user or a script
// file, manipulations with redo buffer and stuff buffer.

#include <assert.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vim.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/vars.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_getln.h"
#include "nvim/ex_getln_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/input.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/mapping.h"
#include "nvim/mapping_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/mouse.h"
#include "nvim/move.h"
#include "nvim/normal.h"
#include "nvim/normal_defs.h"
#include "nvim/ops.h"
#include "nvim/option_vars.h"
#include "nvim/os/fileio.h"
#include "nvim/os/fileio_defs.h"
#include "nvim/os/input.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/plines.h"
#include "nvim/pos_defs.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

// Rust implementation in nvim-event crate
extern int rs_multiqueue_empty(MultiQueue *mq);
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define multiqueue_empty(mq) rs_multiqueue_empty(mq)
#define loop_get_events(l) rs_loop_get_events(l)

// Buffer FFI functions (buffers owned by Rust)
extern int rs_read_readbuffers(int advance);
extern void rs_start_stuff(void);

// Typebuf lifecycle (owned by Rust: rs_init/alloc/free/save/close_typebuf)
extern void rs_init_typebuf(void);
extern void rs_alloc_typebuf(void);
extern void rs_free_typebuf(void);
extern void rs_save_typebuf(void);
extern void rs_close_typebuf(void);

// Recording/gotchars operations (full functions in Rust)
extern void rs_gotchars(const uint8_t *chars, size_t len);
extern void rs_add_byte_to_showcmd(uint8_t byte);

// no_reduce_keys is now owned by Rust; getchar_common calls these
extern void rs_inc_no_reduce_keys(void);
extern void rs_dec_no_reduce_keys(void);

/// Index in scriptin (non-static: accessed by Rust via extern)
int curscript = -1;
/// Streams to read script from (non-static: accessed by Rust via extern)
FileDescriptor scriptin[NSCRIPT] = { 0 };

int typeahead_char = 0;  ///< typeahead char that's not flushed (non-static: accessed by Rust)

int KeyNoremap = 0;  ///< remapping flags (non-static: accessed by Rust)

/// Variables used by vgetorpeek() and flush_buffers()
///
/// typebuf.tb_buf[] contains all characters that are not consumed yet.
/// typebuf.tb_buf[typebuf.tb_off] is the first valid character.
/// typebuf.tb_buf[typebuf.tb_off + typebuf.tb_len - 1] is the last valid char.
/// typebuf.tb_buf[typebuf.tb_off + typebuf.tb_len] must be NUL.
/// The head of the buffer may contain the result of mappings, abbreviations
/// and @a commands.  The length of this part is typebuf.tb_maplen.
/// typebuf.tb_silent is the part where <silent> applies.
/// After the head are characters that come from the terminal.
/// typebuf.tb_no_abbr_cnt is the number of characters in typebuf.tb_buf that
/// should not be considered for abbreviations.
/// Some parts of typebuf.tb_buf may not be mapped. These parts are remembered
/// in typebuf.tb_noremap[], which is the same length as typebuf.tb_buf and
/// contains RM_NONE for the characters that are not to be remapped.
/// typebuf.tb_noremap[typebuf.tb_off] is the first valid flag.
enum {
  RM_YES    = 0,  ///< tb_noremap: remap
  RM_NONE   = 1,  ///< tb_noremap: don't remap
  RM_SCRIPT = 2,  ///< tb_noremap: remap local script mappings
  RM_ABBR   = 4,  ///< tb_noremap: don't remap, do abbrev.
};

// last_recorded_len moved to Rust (macro_recording.rs); export as #[no_mangle]
extern size_t last_recorded_len;

enum {
  KEYLEN_PART_KEY = -1,  ///< keylen value for incomplete key-code
  KEYLEN_PART_MAP = -2,  ///< keylen value for incomplete mapping
};

#include "getchar.c.generated.h"


// Rust replacements: rs_init_typebuf, rs_alloc_typebuf, rs_free_typebuf,
// rs_save_typebuf, rs_close_typebuf (in nvim-getchar crate)

// save_typeahead / restore_typeahead moved to Rust (typebuf.rs) as #[export_name] fns
// openscript moved to Rust (typebuf.rs, Phase 2) as #[unsafe(export_name = "openscript")]

// closescript, close_all_scripts, open_scriptin moved to Rust (typebuf.rs, Phase 5)
// rs_closescript is the Rust implementation; close_all_scripts and open_scriptin
// are exported with #[export_name] from Rust.

// no_reduce_keys moved to Rust (typebuf.rs); getchar_common uses rs_inc/dec_no_reduce_keys()

// getchar_common, f_getchar, f_getcharstr, f_getcharmod moved to Rust (input.rs, Phase 3)
// f_getchar/f_getcharstr/f_getcharmod use #[unsafe(export_name)] for symbol replacement

// map_result_T, handle_mapping, vgetorpeek migrated to Rust (Phases 1+2)
int put_string_in_typebuf(int offset, int slen, uint8_t *string, int new_slen);
bool at_ins_compl_key(void);

// inchar moved to Rust (typebuf.rs, Phase 5) as #[no_mangle] pub unsafe extern "C" fn inchar
// map_execute_lua, paste_repeat migrated to Rust (Phase 3)

// nvim_get_typebuf_change_cnt/was_filled/maplen/len deleted: moved to Rust (typebuf.rs)
// nvim_get_curscript, nvim_get_keynoremap deleted: curscript, KeyNoremap now non-static
// nvim_get_rm_none, nvim_get_rm_script, nvim_get_maxmaplen deleted: Rust uses constants directly
// nvim_get_typebuf_buf/noremap/buflen/off, nvim_set_typebuf_* deleted: Rust uses typebuf_ptr() directly (Phase 4)
// nvim_get_typebuf_silent/no_abbr_cnt, nvim_init_typebuf deleted: Rust uses typebuf_ptr() directly (Phase 4)

// nvim_get/set/add_last_recorded_len deleted: moved to Rust (macro_recording.rs)
// nvim_set_keynoremap deleted: Rust uses static mut KeyNoremap directly
// nvim_get/set_no_mapping deleted: Rust uses static mut no_mapping directly
// nvim_get/set_allow_keys deleted: Rust uses static mut allow_keys directly
// nvim_get/set_mapped_ctrl_c deleted: Rust uses static mut mapped_ctrl_c directly
// nvim_get/set_keytyped deleted: no Rust callers (Phase 4)
// nvim_get/set_keystuffed deleted: Rust uses static mut KeyStuffed directly
// nvim_get_vgetc_busy deleted: Rust uses static vgetc_busy directly
// nvim_inc/dec_vgetc_busy deleted: no callers (Phase 4)
// nvim_get_ex_normal_busy deleted: Rust uses static ex_normal_busy directly
// nvim_get_maptick, nvim_inc_maptick deleted: no callers (Phase 4)
// nvim_get/set_mod_mask deleted: Rust uses static mut mod_mask directly
// nvim_get/set_cmd_silent deleted: Rust uses static mut cmd_silent directly
// nvim_get/set_mouse_grid/row/col deleted: Rust uses static mut mouse_* directly
int nvim_char_avail(void) { return char_avail() ? 1 : 0; }
// nvim_set_reg_executing deleted: Rust uses static mut reg_executing directly
// nvim_get/set_pending_end_reg_executing deleted: Rust uses static mut directly
// nvim_mb_byte2len_check deleted: no Rust callers
void nvim_state_no_longer_safe(void) { state_no_longer_safe("rs_ins_typebuf()"); }
// nvim_get_key_stuffed deleted: duplicate of nvim_get_keystuffed
// nvim_get/set_typeahead_char deleted: typeahead_char now non-static
// nvim_get/set_old_keystuffed deleted: moved to Rust (input.rs)

// nvim_set_visual_from_cursor deleted: moved to Rust (stuff.rs)
// nvim_map_execute_lua_discard, nvim_paste_repeat_discard deleted:
// Rust calls map_execute_lua/paste_repeat directly
