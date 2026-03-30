#include <assert.h>
#include <float.h>
#include <inttypes.h>
#include <limits.h>
#include <math.h>
#include <signal.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <uv.h>

#include "auto/config.h"
#include "klib/kvec.h"
#include "mpack/mpack_core.h"
#include "mpack/object.h"
#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/api/vim.h"
#include "nvim/ascii_defs.h"
#include "nvim/assert_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/channel.h"
#include "nvim/channel_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/context.h"
#include "nvim/cursor.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/buffer.h"
#include "nvim/eval/decode.h"
#include "nvim/eval/encode.h"
#include "nvim/eval/executor.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/eval/window.h"
#include "nvim/event/defs.h"
#include "nvim/event/loop.h"
#include "nvim/event/multiqueue.h"
#include "nvim/event/proc.h"
#include "nvim/event/time.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/ex_eval.h"
#include "nvim/ex_getln.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/getchar_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/grid_defs.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/indent.h"
#include "nvim/indent_c.h"
#include "nvim/input.h"
#include "nvim/insexpand.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/main.h"
#include "nvim/mark.h"
#include "nvim/mark_defs.h"
#include "nvim/math.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/menu.h"
#include "nvim/menu_defs.h"
#include "nvim/message.h"
#include "nvim/move.h"
#include "nvim/msgpack_rpc/channel.h"
#include "nvim/msgpack_rpc/channel_defs.h"
#include "nvim/msgpack_rpc/packer.h"
#include "nvim/msgpack_rpc/packer_defs.h"
#include "nvim/msgpack_rpc/server.h"
#include "nvim/normal.h"
#include "nvim/normal_defs.h"
#include "nvim/ops.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/optionstr.h"
#include "nvim/os/dl.h"
#include "nvim/os/fs.h"
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/pty_proc.h"
#include "nvim/os/shell.h"
#include "nvim/os/stdpaths_defs.h"
#include "nvim/os/time.h"
#include "nvim/path.h"
#include "nvim/plines.h"
#include "nvim/popupmenu.h"
#include "nvim/pos_defs.h"
#include "nvim/profile.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/register.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/search.h"
#include "nvim/sha256.h"
#include "nvim/spell.h"
#include "nvim/spellsuggest.h"
#include "nvim/state.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/tag.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_compositor.h"
#include "nvim/version.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// Rust implementation in nvim-event crate
extern int rs_ins_compl_active(void);
extern int rs_proc_get_pid(Proc *proc);
extern MultiQueue *rs_loop_get_events(Loop *loop);
#define proc_get_pid(p) rs_proc_get_pid(p)
#define loop_get_events(l) rs_loop_get_events(l)

/// Describe data to return from find_some_match()
typedef enum {
  kSomeMatch,  ///< Data for match().
  kSomeMatchEnd,  ///< Data for matchend().
  kSomeMatchList,  ///< Data for matchlist().
  kSomeMatchStr,  ///< Data for matchstr().
  kSomeMatchStrPos,  ///< Data for matchstrpos().
} SomeMatchType;

#include "eval/funcs.c.generated.h"

// Rust FFI declarations (memline crate)
extern int rs_ml_find_line_or_offset(buf_T *buf, linenr_T lnum, int *offp, bool no_ff);

// Rust FFI declarations (tag module)
extern void rs_get_tagstack(void *wp, void *retdict);
extern int rs_set_tagstack(void *wp, const void *d, int action);

extern bool rs_op_pending(void);

// Rust typval blob functions (migrated from typval.c, Phase 1)
extern int tv_blob_set_range(blob_T *dest, varnumber_T n1, varnumber_T n2, typval_T *src);

// Rust math VimL function declarations (direct dispatch via #[export_name])
extern void f_abs(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_sin(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_cos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_tan(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_asin(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_acos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_atan(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_atan2(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_sinh(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_cosh(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_tanh(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_exp(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_log(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_log10(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_sqrt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_pow(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_fmod(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_ceil(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_floor(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_round(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_trunc(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_float2nr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_isnan(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_isinf(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust bitwise VimL function declarations (direct dispatch via #[export_name])
extern void f_and(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_or(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_xor(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_invert(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust type VimL function declarations (direct dispatch via #[export_name])
extern void f_type(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);


#ifdef _MSC_VER
// This prevents MSVC from replacing the functions with intrinsics,
// and causing errors when trying to get their addresses in funcs.generated.h
# pragma function(ceil)
# pragma function(floor)
#endif

// Rust list/container VimL function declarations (exported from nvim-eval crate via #[export_name])
extern void f_remove(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reverse(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_extend(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_extendnew(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_add(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_insert(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_count(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust window VimL function declarations (exported from nvim-window crate via #[export_name])
extern void f_getwinpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getwinposx(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getwinposy(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_wincol(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_winline(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_winheight(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_winwidth(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_winbufnr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getcmdwintype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_win_screenpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_tabpagenr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_tabpagewinnr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_win_getid(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_win_id2tabwin(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_win_id2win(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_win_findbuf(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_winnr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_gettabinfo(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getwininfo(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_winsaveview(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_winrestview(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust misc VimL function declarations (exported from nvim-eval crate via #[export_name])
extern void f_foreground(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getfontname(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_windowsversion(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getpid(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_localtime(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_screencol(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_screenrow(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_eventhandler(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_did_filetype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_changenr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_interrupt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_pumvisible(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reg_executing(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reg_recording(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reg_recorded(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_charcol(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_col(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getcharpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getcurpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getcursorcharpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 3 VimL function declarations (exported from nvim-eval crate via #[export_name])
extern void f_char2nr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_nr2char(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_str2float(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_escape(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_shellescape(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_fnameescape(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_hostname(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_empty(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_copy(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_deepcopy(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_len(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_ctxsize(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_ctxpop(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_max(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_min(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setcharpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setcursorcharpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_cursor(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_searchpair(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_match(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchend(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchstr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchstrpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 4 VimL function declarations (exported from nvim-eval crate via #[export_name])
extern void f_execute(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_flatten(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_flattennew(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_funcref(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_function(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_hlID(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_hlexists(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_input(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_inputdialog(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_json_encode(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_libcall(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_libcallnr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_py3eval(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_perleval(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_rubyeval(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_search(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_searchpairpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_swapfilelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_swapinfo(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 3 VimL function declarations (system.rs - exported via #[export_name])
extern void f_environ(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_stdpath(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 2 VimL function declarations (display.rs - exported via #[export_name])
extern void f_screenattr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_screenchar(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_screenchars(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_screenstring(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 10 VimL function declarations (misc.rs - exported via #[export_name])
extern void f_eval(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_exists(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_has(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_json_decode(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_printf(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_sha256(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 9 VimL function declarations (misc.rs - exported via #[export_name])
extern void f_index(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_indexof(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_range(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_repeat(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reduce(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 8 VimL function declarations (misc.rs - exported via #[export_name])
extern void f_spellbadword(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_spellsuggest(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_submatch(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_substitute(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_synID(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_synIDattr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_synconcealed(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_synstack(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 7 VimL function declarations (timer.rs - exported via #[export_name])
extern void f_timer_info(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_timer_pause(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_timer_start(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_timer_stop(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 6 VimL function declarations (misc.rs - exported via #[export_name])
extern void f_ctxget(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_ctxpush(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_ctxset(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getcharsearch(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setcharsearch(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getreg(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getregtype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getreginfo(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_state(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_searchdecl(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_searchpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 5 VimL function declarations (simple.rs - exported via #[export_name])
extern void f_api_info(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_byte2line(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_line2byte(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_gettext(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_garbagecollect(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_debugbreak(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getenv(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setenv(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_pum_getpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_wordcount(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_soundfold(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_wildmenumode(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_timer_stopall(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_synIDtrans(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_keytrans(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_luaeval(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_shiftwidth(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_mode(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_visualmode(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_nextnonblank(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_prevnonblank(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_menu_get(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 1 (plan 40f0fb72) VimL function declarations (simple.rs)
extern void f_feedkeys(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_tagfiles(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_taglist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_serverstop(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 2 (plan 40f0fb72) VimL function declarations (misc.rs)
extern void f_id(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setfperm(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reltimefloat(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reltimestr(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 3 (plan 40f0fb72) VimL function declarations (misc.rs)
extern void f_rand(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_srand(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_reltime(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 4 (plan 40f0fb72) VimL function declarations (misc.rs)
extern void f_chanclose(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_serverstart(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_confirm(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 5 (plan 40f0fb72) VimL function declarations (misc.rs)
extern void f_strftime(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_strptime(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust Phase 6 (plan 40f0fb72) VimL function declarations (misc.rs)
extern void f_dictwatcheradd(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_dictwatcherdel(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust cmdline VimL function declarations (cmdline.rs in nvim-eval crate)
extern void f_getcmdcomplpat(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getcmdcompltype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setcmdline(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setcmdpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_wildtrigger(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

PRAGMA_DIAG_PUSH_IGNORE_MISSING_PROTOTYPES
PRAGMA_DIAG_PUSH_IGNORE_IMPLICIT_FALLTHROUGH
#include "funcs.generated.h"

// Rust FFI declarations (window wrappers removed)
extern tabpage_T *rs_find_tabpage(int n);
extern int rs_eval_expr_valid_arg(const typval_T *tv);
extern size_t rs_string2float(const char *text, float_T *ret_value);
extern int rs_buf_byteidx_to_charidx(buf_T *buf, linenr_T lnum, int byteidx);
extern int rs_buf_charidx_to_byteidx(buf_T *buf, linenr_T lnum, int charidx);
extern int rs_get_callback_depth(void);
extern bool rs_callback_from_typval(Callback *callback, const typval_T *arg);
extern char *rs_partial_name(partial_T *pt);
extern int rs_get_copyID(void);

PRAGMA_DIAG_POP
PRAGMA_DIAG_POP

static const char *e_invalwindow = N_("E957: Invalid window number");
static const char e_string_list_or_blob_required[]
  = N_("E1098: String, List or Blob required");
static const char e_missing_function_argument[]
  = N_("E1132: Missing function argument");

/// Dummy va_list for passing to vim_snprintf
///
/// Used because:
/// - passing a NULL pointer doesn't work when va_list isn't a pointer
/// - locally in the function results in a "used before set" warning
/// - using va_start() to initialize it gives "function with fixed args" error
static va_list dummy_ap;

/// Function given to ExpandGeneric() to obtain the list of internal
/// or user defined function names.
char *get_function_name(expand_T *xp, int idx)
{
  static int intidx = -1;

  if (idx == 0) {
    intidx = -1;
  }
  if (intidx < 0) {
    char *name = get_user_func_name(xp, idx);
    if (name != NULL) {
      if (*name != NUL && *name != '<'
          && strncmp("g:", xp->xp_pattern, 2) == 0) {
        return cat_prefix_varname('g', name);
      }
      return name;
    }
  }

  const char *const key = functions[++intidx].name;
  if (!key) {
    return NULL;
  }
  const size_t key_len = strlen(key);
  memcpy(IObuff, key, key_len);
  IObuff[key_len] = '(';
  if (functions[intidx].max_argc == 0) {
    IObuff[key_len + 1] = ')';
    IObuff[key_len + 2] = NUL;
  } else {
    IObuff[key_len + 1] = NUL;
  }
  return IObuff;
}

/// Function given to ExpandGeneric() to obtain the list of internal or
/// user defined variable or function names.
char *get_expr_name(expand_T *xp, int idx)
{
  static int intidx = -1;

  if (idx == 0) {
    intidx = -1;
  }
  if (intidx < 0) {
    char *name = get_function_name(xp, idx);
    if (name != NULL) {
      return name;
    }
  }
  return get_user_var_name(xp, ++intidx);
}

/// Find internal function in hash functions
///
/// @param[in]  name  Name of the function.
///
/// @return  pointer to the function definition or NULL if not found.
const EvalFuncDef *find_internal_func(const char *const name)
  FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE FUNC_ATTR_NONNULL_ALL
{
  size_t len = strlen(name);
  int index = find_internal_func_hash(name, len);
  return index >= 0 ? &functions[index] : NULL;
}

/// Check the argument count to use for internal function "fdef".
/// @return  -1 for failure, 0 if no method base accepted, 1 if method base is
/// first argument, 2 if method base is second argument, etc.
int check_internal_func(const EvalFuncDef *const fdef, const int argcount)
  FUNC_ATTR_NONNULL_ALL
{
  int res;

  if (argcount < fdef->min_argc) {
    res = FCERR_TOOFEW;
  } else if (argcount > fdef->max_argc) {
    res = FCERR_TOOMANY;
  } else {
    return fdef->base_arg;
  }

  const char *const name = fdef->name;
  if (res == FCERR_TOOMANY) {
    semsg(_(e_toomanyarg), name);
  } else {
    semsg(_(e_toofewarg), name);
  }
  return -1;
}

int call_internal_func(const char *const fname, const int argcount, typval_T *const argvars,
                       typval_T *const rettv)
  FUNC_ATTR_NONNULL_ALL
{
  const EvalFuncDef *const fdef = find_internal_func(fname);
  if (fdef == NULL) {
    return FCERR_UNKNOWN;
  } else if (argcount < fdef->min_argc) {
    return FCERR_TOOFEW;
  } else if (argcount > fdef->max_argc) {
    return FCERR_TOOMANY;
  }
  argvars[argcount].v_type = VAR_UNKNOWN;
  fdef->func(argvars, rettv, fdef->data);
  return FCERR_NONE;
}

/// Invoke a method for base->method().
int call_internal_method(const char *const fname, const int argcount, typval_T *const argvars,
                         typval_T *const rettv, typval_T *const basetv)
  FUNC_ATTR_NONNULL_ALL
{
  const EvalFuncDef *const fdef = find_internal_func(fname);
  if (fdef == NULL) {
    return FCERR_UNKNOWN;
  } else if (fdef->base_arg == BASE_NONE) {
    return FCERR_NOTMETHOD;
  } else if (argcount + 1 < fdef->min_argc) {
    return FCERR_TOOFEW;
  } else if (argcount + 1 > fdef->max_argc) {
    return FCERR_TOOMANY;
  }

  typval_T argv[MAX_FUNC_ARGS + 1];
  const ptrdiff_t base_index = fdef->base_arg == BASE_LAST ? argcount : fdef->base_arg - 1;
  if (argcount < base_index) {
    return FCERR_TOOFEW;
  }
  memcpy(argv, argvars, (size_t)base_index * sizeof(typval_T));
  argv[base_index] = *basetv;
  memcpy(argv + base_index + 1, argvars + base_index,
         (size_t)(argcount - base_index) * sizeof(typval_T));
  argv[argcount + 1].v_type = VAR_UNKNOWN;

  fdef->func(argv, rettv, fdef->data);
  return FCERR_NONE;
}

/// @return  true for a non-zero Number and a non-empty String.
static bool non_zero_arg(typval_T *argvars)
{
  return ((argvars[0].v_type == VAR_NUMBER
           && argvars[0].vval.v_number != 0)
          || (argvars[0].v_type == VAR_BOOL
              && argvars[0].vval.v_bool == kBoolVarTrue)
          || (argvars[0].v_type == VAR_STRING
              && argvars[0].vval.v_string != NULL
              && *argvars[0].vval.v_string != NUL));
}

static void api_wrapper(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (rs_check_secure()) {
    return;
  }

  MsgpackRpcRequestHandler handler = *fptr.api_handler;

  MAXSIZE_TEMP_ARRAY(args, MAX_FUNC_ARGS);
  Arena arena = ARENA_EMPTY;

  for (typval_T *tv = argvars; tv->v_type != VAR_UNKNOWN; tv++) {
    ADD_C(args, vim_to_object(tv, &arena, false));
  }

  Error err = ERROR_INIT;
  Object result = handler.fn(VIML_INTERNAL_CALL, args, &arena, &err);

  if (ERROR_SET(&err)) {
    semsg_multiline("emsg", e_api_error, err.msg);
    goto end;
  }

  object_to_vim_take_luaref(&result, rettv, true, &err);

end:
  if (handler.ret_alloc) {
    api_free_object(result);
  }
  arena_mem_free(arena_finish(&arena));
  api_clear_error(&err);
}


/// Get buffer by number or pattern.
buf_T *tv_get_buf(typval_T *tv, int curtab_only)
{
  if (tv->v_type == VAR_NUMBER) {
    return buflist_findnr((int)tv->vval.v_number);
  }
  if (tv->v_type != VAR_STRING) {
    return NULL;
  }

  char *name = tv->vval.v_string;

  if (name == NULL || *name == NUL) {
    return curbuf;
  }
  if (name[0] == '$' && name[1] == NUL) {
    return lastbuf;
  }

  // Ignore 'magic' and 'cpoptions' here to make scripts portable
  int save_magic = p_magic;
  p_magic = true;
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;

  buf_T *buf = buflist_findnr(buflist_findpat(name, name + strlen(name),
                                              true, false, curtab_only));

  p_magic = save_magic;
  p_cpo = save_cpo;

  // If not found, try expanding the name, like done for bufexists().
  if (buf == NULL) {
    buf = find_buffer(tv);
  }

  return buf;
}

/// Like tv_get_buf() but give an error message if the type is wrong.
buf_T *tv_get_buf_from_arg(typval_T *const tv) FUNC_ATTR_NONNULL_ALL
{
  if (!tv_check_str_or_nr(tv)) {
    return NULL;
  }
  emsg_off++;
  buf_T *const buf = tv_get_buf(tv, false);
  emsg_off--;
  return buf;
}

/// Get the buffer from "arg" and give an error and return NULL if it is not
/// valid.
buf_T *get_buf_arg(typval_T *arg)
{
  emsg_off++;
  buf_T *buf = tv_get_buf(arg, false);
  emsg_off--;
  if (buf == NULL) {
    semsg(_("E158: Invalid buffer name: %s"), tv_get_string(arg));
  }
  return buf;
}


/// "call(func, arglist [, dict])" function
static void f_call(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (tv_check_for_list_arg(argvars, 1) == FAIL) {
    return;
  }
  if (argvars[1].vval.v_list == NULL) {
    return;
  }

  bool owned = false;
  char *func;
  partial_T *partial = NULL;
  if (argvars[0].v_type == VAR_FUNC) {
    func = argvars[0].vval.v_string;
  } else if (argvars[0].v_type == VAR_PARTIAL) {
    partial = argvars[0].vval.v_partial;
    func = rs_partial_name(partial);
  } else if (nlua_is_table_from_lua(&argvars[0])) {
    // TODO(tjdevries): UnifiedCallback
    func = nlua_register_table_as_callable(&argvars[0]);
    owned = true;
  } else {
    func = (char *)tv_get_string(&argvars[0]);
  }

  if (func == NULL || *func == NUL) {
    return;         // type error, empty name or null function
  }
  char *tofree = NULL;
  if (argvars[0].v_type == VAR_STRING) {
    char *p = func;
    tofree = trans_function_name(&p, false, TFN_INT|TFN_QUIET, NULL, NULL);
    if (tofree == NULL) {
      emsg_funcname(e_unknown_function_str, func);
      return;
    }
    func = tofree;
  }

  dict_T *selfdict = NULL;
  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (tv_check_for_dict_arg(argvars, 2) == FAIL) {
      goto done;
    }
    selfdict = argvars[2].vval.v_dict;
  }

  func_call(func, &argvars[1], partial, selfdict, rettv);

done:
  if (owned) {
    func_unref(func);
  }
  xfree(tofree);
}

/// "chanclose(id[, stream])" function

/// "chansend(id, data)" function
static void f_chansend(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;

  if (rs_check_secure()) {
    return;
  }

  if (argvars[0].v_type != VAR_NUMBER || argvars[1].v_type == VAR_UNKNOWN) {
    // First argument is the channel id and second is the data to write
    emsg(_(e_invarg));
    return;
  }

  ptrdiff_t input_len = 0;
  char *input = NULL;
  uint64_t id = (uint64_t)argvars[0].vval.v_number;
#ifdef UNIX
  bool crlf = false;
#else
  Channel *chan = find_channel(id);
  bool crlf = (chan != NULL && chan->term) ? true : false;
#endif

  if (argvars[1].v_type == VAR_BLOB) {
    const blob_T *const b = argvars[1].vval.v_blob;
    input_len = tv_blob_len(b);
    if (input_len > 0) {
      input = xmemdup(b->bv_ga.ga_data, (size_t)input_len);
    }
  } else {
    input = save_tv_as_string(&argvars[1], &input_len, false, crlf);
  }

  if (!input) {
    // Either the error has been handled by save_tv_as_string(),
    // or there is no input to send.
    return;
  }
  const char *error = NULL;
  rettv->vval.v_number = (varnumber_T)channel_send(id, input, (size_t)input_len, true, &error);
  if (error) {
    emsg(error);
  }
}

/// Get the current cursor column and store it in 'rettv'.
///
/// @return  the character index of the column if 'charcol' is true,
///          otherwise the byte index of the column.


win_T *get_optional_window(typval_T *argvars, int idx)
{
  if (argvars[idx].v_type == VAR_UNKNOWN) {
    return curwin;
  }

  win_T *win = find_win_by_nr_or_id(&argvars[idx]);
  if (win == NULL) {
    emsg(_(e_invalwindow));
    return NULL;
  }
  return win;
}

/// "confirm(message, buttons[, default [, type]])" function


/// Set the cursor position.
/// If "charcol" is true, then use the column number as a character offset.
/// Otherwise use the column number as a byte offset.


/// "cursor(lnum, col)" function, or
/// "cursor(list)"
///
/// Moves the cursor to the specified line and column.
///
/// @return  0 when the position could be set, -1 otherwise.
/// "debugbreak()" function

/// dictwatcheradd(dict, key, funcref) function


/// "eval()" function


typedef struct {
  const list_T *const l;
  const listitem_T *li;
} GetListLineCookie;

static char *get_list_line(int c, void *cookie, int indent, bool do_concat)
{
  GetListLineCookie *const p = (GetListLineCookie *)cookie;

  const listitem_T *const item = p->li;
  if (item == NULL) {
    return NULL;
  }
  char buf[NUMBUFLEN];
  const char *const s = tv_get_string_buf_chk(TV_LIST_ITEM_TV(item), buf);
  p->li = TV_LIST_ITEM_NEXT(p->l, item);
  return s == NULL ? NULL : xstrdup(s);
}

void execute_common(typval_T *argvars, typval_T *rettv, int arg_off)
{
  const int save_msg_silent = msg_silent;
  const int save_emsg_silent = emsg_silent;
  const bool save_emsg_noredir = emsg_noredir;
  const bool save_redir_off = redir_off;
  garray_T *const save_capture_ga = capture_ga;
  const int save_msg_col = msg_col;
  bool echo_output = false;

  if (rs_check_secure()) {
    return;
  }

  if (argvars[arg_off + 1].v_type != VAR_UNKNOWN) {
    char buf[NUMBUFLEN];
    const char *const s = tv_get_string_buf_chk(&argvars[arg_off + 1], buf);

    if (s == NULL) {
      return;
    }
    if (*s == NUL) {
      echo_output = true;
    }
    if (strncmp(s, "silent", 6) == 0) {
      msg_silent++;
    }
    if (strcmp(s, "silent!") == 0) {
      emsg_silent = true;
      emsg_noredir = true;
    }
  } else {
    msg_silent++;
  }

  garray_T capture_local;
  ga_init(&capture_local, (int)sizeof(char), 80);
  capture_ga = &capture_local;
  redir_off = false;
  if (!echo_output) {
    msg_col = 0;  // prevent leading spaces
  }

  if (argvars[arg_off].v_type != VAR_LIST) {
    do_cmdline_cmd(tv_get_string(&argvars[arg_off]));
  } else if (argvars[arg_off].vval.v_list != NULL) {
    list_T *const list = argvars[arg_off].vval.v_list;
    tv_list_ref(list);
    GetListLineCookie cookie = {
      .l = list,
      .li = tv_list_first(list),
    };
    do_cmdline(NULL, get_list_line, (void *)&cookie,
               DOCMD_NOWAIT|DOCMD_VERBOSE|DOCMD_REPEAT|DOCMD_KEYTYPED);
    tv_list_unref(list);
  }
  msg_silent = save_msg_silent;
  emsg_silent = save_emsg_silent;
  emsg_noredir = save_emsg_noredir;
  redir_off = save_redir_off;
  // "silent reg" or "silent echo x" leaves msg_col somewhere in the line.
  if (echo_output) {
    // When not working silently: put it in column zero.  A following
    // "echon" will overwrite the message, unavoidably.
    msg_col = 0;
  } else {
    // When working silently: Put it back where it was, since nothing
    // should have been written.
    msg_col = save_msg_col;
  }

  ga_append(capture_ga, NUL);
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = capture_ga->ga_data;

  capture_ga = save_capture_ga;
}


/// "expand()" function
static void f_expand(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  int options = WILD_SILENT|WILD_USE_NL|WILD_LIST_NOTFOUND;
  bool error = false;
#ifdef BACKSLASH_IN_FILENAME
  char *p_csl_save = p_csl;

  // avoid using 'completeslash' here
  p_csl = empty_string_option;
#endif

  rettv->v_type = VAR_STRING;
  if (argvars[1].v_type != VAR_UNKNOWN
      && argvars[2].v_type != VAR_UNKNOWN
      && tv_get_number_chk(&argvars[2], &error)
      && !error) {
    tv_list_set_ret(rettv, NULL);
  }

  const char *s = tv_get_string(&argvars[0]);
  if (*s == '%' || *s == '#' || *s == '<') {
    if (p_verbose == 0) {
      emsg_off++;
    }
    size_t len;
    const char *errormsg = NULL;
    char *result = eval_vars((char *)s, s, &len, NULL, &errormsg, NULL, false);
    if (p_verbose == 0) {
      emsg_off--;
    } else if (errormsg != NULL) {
      emsg(errormsg);
    }
    if (rettv->v_type == VAR_LIST) {
      tv_list_alloc_ret(rettv, (result != NULL));
      if (result != NULL) {
        tv_list_append_string(rettv->vval.v_list, result, -1);
      }
      XFREE_CLEAR(result);
    } else {
      rettv->vval.v_string = result;
    }
  } else {
    // When the optional second argument is non-zero, don't remove matches
    // for 'wildignore' and don't put matches for 'suffixes' at the end.
    if (argvars[1].v_type != VAR_UNKNOWN
        && tv_get_number_chk(&argvars[1], &error)) {
      options |= WILD_KEEP_ALL;
    }
    if (!error) {
      expand_T xpc;
      ExpandInit(&xpc);
      xpc.xp_context = EXPAND_FILES;
      if (p_wic) {
        options += WILD_ICASE;
      }
      if (rettv->v_type == VAR_STRING) {
        rettv->vval.v_string = ExpandOne(&xpc, (char *)s, NULL, options, WILD_ALL);
      } else {
        ExpandOne(&xpc, (char *)s, NULL, options, WILD_ALL_KEEP);
        tv_list_alloc_ret(rettv, xpc.xp_numfiles);
        for (int i = 0; i < xpc.xp_numfiles; i++) {
          tv_list_append_string(rettv->vval.v_list, xpc.xp_files[i], -1);
        }
        ExpandCleanup(&xpc);
      }
    } else {
      rettv->vval.v_string = NULL;
    }
  }
#ifdef BACKSLASH_IN_FILENAME
  p_csl = p_csl_save;
#endif
}

/// "menu_get(path [, modes])" function

/// "expandcmd()" function
/// Expand all the special characters in a command string.
static void f_expandcmd(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const char *errormsg = NULL;
  bool emsgoff = true;

  if (argvars[1].v_type == VAR_DICT
      && tv_dict_get_bool(argvars[1].vval.v_dict, "errmsg", kBoolVarFalse)) {
    emsgoff = false;
  }

  rettv->v_type = VAR_STRING;
  char *cmdstr = xstrdup(tv_get_string(&argvars[0]));

  exarg_T eap = {
    .cmd = cmdstr,
    .arg = cmdstr,
    .usefilter = false,
    .nextcmd = NULL,
    .cmdidx = CMD_USER,
  };
  eap.argt |= EX_NOSPC;

  if (emsgoff) {
    emsg_off++;
  }
  if (expand_filename(&eap, &cmdstr, &errormsg) == FAIL) {
    if (!emsgoff && errormsg != NULL && *errormsg != NUL) {
      emsg(errormsg);
    }
  }
  if (emsgoff) {
    emsg_off--;
  }

  rettv->vval.v_string = cmdstr;
}

/// "flatten()" and "flattennew()" functions


/// "flatten(list[, {maxdepth}])" function
/// "feedkeys()" function

/// "function()" function
/// "funcref()" function


/// "garbagecollect()" function

/// "get()" function
static void f_get(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  typval_T *tv = NULL;
  bool what_is_dict = false;

  if (argvars[0].v_type == VAR_BLOB) {
    bool error = false;
    int idx = (int)tv_get_number_chk(&argvars[1], &error);

    if (!error) {
      rettv->v_type = VAR_NUMBER;
      if (idx < 0) {
        idx = tv_blob_len(argvars[0].vval.v_blob) + idx;
      }
      if (idx < 0 || idx >= tv_blob_len(argvars[0].vval.v_blob)) {
        rettv->vval.v_number = -1;
      } else {
        rettv->vval.v_number = tv_blob_get(argvars[0].vval.v_blob, idx);
        tv = rettv;
      }
    }
  } else if (argvars[0].v_type == VAR_LIST) {
    list_T *l = argvars[0].vval.v_list;
    if (l != NULL) {
      bool error = false;

      listitem_T *li = tv_list_find(l, (int)tv_get_number_chk(&argvars[1], &error));
      if (!error && li != NULL) {
        tv = TV_LIST_ITEM_TV(li);
      }
    }
  } else if (argvars[0].v_type == VAR_DICT) {
    dict_T *d = argvars[0].vval.v_dict;
    if (d != NULL) {
      dictitem_T *di = tv_dict_find(d, tv_get_string(&argvars[1]), -1);
      if (di != NULL) {
        tv = &di->di_tv;
      }
    }
  } else if (tv_is_func(argvars[0])) {
    partial_T *pt;
    partial_T fref_pt;

    if (argvars[0].v_type == VAR_PARTIAL) {
      pt = argvars[0].vval.v_partial;
    } else {
      CLEAR_FIELD(fref_pt);
      fref_pt.pt_name = argvars[0].vval.v_string;
      pt = &fref_pt;
    }

    if (pt != NULL) {
      const char *const what = tv_get_string(&argvars[1]);

      if (strcmp(what, "func") == 0 || strcmp(what, "name") == 0) {
        const char *name = rs_partial_name(pt);
        rettv->v_type = (*what == 'f' ? VAR_FUNC : VAR_STRING);
        assert(name != NULL);
        if (rettv->v_type == VAR_FUNC) {
          func_ref((char *)name);
        }
        if (*what == 'n' && pt->pt_name == NULL && pt->pt_func != NULL) {
          // use <SNR> instead of the byte code
          name = printable_func_name(pt->pt_func);
        }
        rettv->vval.v_string = xstrdup(name);
      } else if (strcmp(what, "dict") == 0) {
        what_is_dict = true;
        if (pt->pt_dict != NULL) {
          tv_dict_set_ret(rettv, pt->pt_dict);
        }
      } else if (strcmp(what, "args") == 0) {
        rettv->v_type = VAR_LIST;
        tv_list_alloc_ret(rettv, pt->pt_argc);
        for (int i = 0; i < pt->pt_argc; i++) {
          tv_list_append_tv(rettv->vval.v_list, &pt->pt_argv[i]);
        }
      } else if (strcmp(what, "arity") == 0) {
        int required = 0;
        int optional = 0;
        bool varargs = false;
        const char *name = rs_partial_name(pt);

        get_func_arity(name, &required, &optional, &varargs);

        rettv->v_type = VAR_DICT;
        tv_dict_alloc_ret(rettv);
        dict_T *dict = rettv->vval.v_dict;

        // Take into account the arguments of the partial, if any.
        // Note that it is possible to supply more arguments than the function
        // accepts.
        if (pt->pt_argc >= required + optional) {
          required = optional = 0;
        } else if (pt->pt_argc > required) {
          optional -= pt->pt_argc - required;
          required = 0;
        } else {
          required -= pt->pt_argc;
        }

        tv_dict_add_nr(dict, S_LEN("required"), required);
        tv_dict_add_nr(dict, S_LEN("optional"), optional);
        tv_dict_add_bool(dict, S_LEN("varargs"), varargs);
      } else {
        semsg(_(e_invarg2), what);
      }

      // When {what} == "dict" and pt->pt_dict == NULL, evaluate the
      // third argument
      if (!what_is_dict) {
        return;
      }
    }
  } else {
    semsg(_(e_listdictblobarg), "get()");
  }

  if (tv == NULL) {
    if (argvars[2].v_type != VAR_UNKNOWN) {
      tv_copy(&argvars[2], rettv);
    }
  } else {
    tv_copy(tv, rettv);
  }
}

/// "getchangelist()" function
static void f_getchangelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_list_alloc_ret(rettv, 2);

  const buf_T *buf;
  if (argvars[0].v_type == VAR_UNKNOWN) {
    buf = curbuf;
  } else {
    vim_ignored = (int)tv_get_number(&argvars[0]);  // issue errmsg if type error
    emsg_off++;
    buf = tv_get_buf(&argvars[0], false);
    emsg_off--;
  }
  if (buf == NULL) {
    return;
  }

  list_T *const l = tv_list_alloc(buf->b_changelistlen);
  tv_list_append_list(rettv->vval.v_list, l);
  // The current window change list index tracks only the position for the
  // current buffer. For other buffers use the stored index for the current
  // window, or, if that's not available, the change list length.
  int changelistindex;
  if (buf == curwin->w_buffer) {
    changelistindex = curwin->w_changelistidx;
  } else {
    changelistindex = buf->b_changelistlen;

    for (size_t i = 0; i < kv_size(buf->b_wininfo); i++) {
      WinInfo *wip = kv_A(buf->b_wininfo, i);
      if (wip->wi_win == curwin) {
        changelistindex = wip->wi_changelistidx;
        break;
      }
    }
  }
  tv_list_append_number(rettv->vval.v_list, (varnumber_T)changelistindex);

  for (int i = 0; i < buf->b_changelistlen; i++) {
    if (buf->b_changelist[i].mark.lnum == 0) {
      continue;
    }
    dict_T *const d = tv_dict_alloc();
    tv_list_append_dict(l, d);
    tv_dict_add_nr(d, S_LEN("lnum"), buf->b_changelist[i].mark.lnum);
    tv_dict_add_nr(d, S_LEN("col"), buf->b_changelist[i].mark.col);
    tv_dict_add_nr(d, S_LEN("coladd"), buf->b_changelist[i].mark.coladd);
  }
}


/// "getjumplist()" function
static void f_getjumplist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);
  win_T *const wp = find_tabwin(&argvars[0], &argvars[1]);
  if (wp == NULL) {
    return;
  }

  cleanup_jumplist(wp, true);

  list_T *const l = tv_list_alloc(wp->w_jumplistlen);
  tv_list_append_list(rettv->vval.v_list, l);
  tv_list_append_number(rettv->vval.v_list, wp->w_jumplistidx);

  for (int i = 0; i < wp->w_jumplistlen; i++) {
    if (wp->w_jumplist[i].fmark.mark.lnum == 0) {
      continue;
    }
    dict_T *const d = tv_dict_alloc();
    tv_list_append_dict(l, d);
    tv_dict_add_nr(d, S_LEN("lnum"), wp->w_jumplist[i].fmark.mark.lnum);
    tv_dict_add_nr(d, S_LEN("col"), wp->w_jumplist[i].fmark.mark.col);
    tv_dict_add_nr(d, S_LEN("coladd"), wp->w_jumplist[i].fmark.mark.coladd);
    tv_dict_add_nr(d, S_LEN("bufnr"), wp->w_jumplist[i].fmark.fnum);
    if (wp->w_jumplist[i].fname != NULL) {
      tv_dict_add_str(d, S_LEN("filename"), wp->w_jumplist[i].fname);
    }
  }
}

/// "getmarklist()" function
static void f_getmarklist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (argvars[0].v_type == VAR_UNKNOWN) {
    get_global_marks(rettv->vval.v_list);
    return;
  }

  buf_T *buf = tv_get_buf(&argvars[0], false);
  if (buf == NULL) {
    return;
  }

  get_buf_local_marks(buf, rettv->vval.v_list);
}

/// Convert from block_def to string
static char *block_def2str(struct block_def *bd)
{
  size_t size = (size_t)bd->startspaces + (size_t)bd->endspaces + (size_t)bd->textlen;
  char *ret = xmalloc(size + 1);
  char *p = ret;
  memset(p, ' ', (size_t)bd->startspaces);
  p += bd->startspaces;
  memmove(p, bd->textstart, (size_t)bd->textlen);
  p += bd->textlen;
  memset(p, ' ', (size_t)bd->endspaces);
  *(p + bd->endspaces) = NUL;
  return ret;
}

static int getregionpos(typval_T *argvars, typval_T *rettv, pos_T *p1, pos_T *p2,
                        bool *const inclusive, MotionType *region_type, oparg_T *oap)
  FUNC_ATTR_NONNULL_ALL
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (tv_check_for_list_arg(argvars, 0) == FAIL
      || tv_check_for_list_arg(argvars, 1) == FAIL
      || tv_check_for_opt_dict_arg(argvars, 2) == FAIL) {
    return FAIL;
  }

  int fnum1 = -1;
  int fnum2 = -1;
  if (list2fpos(&argvars[0], p1, &fnum1, NULL, false) != OK
      || list2fpos(&argvars[1], p2, &fnum2, NULL, false) != OK
      || fnum1 != fnum2) {
    return FAIL;
  }

  bool is_select_exclusive;
  char *type;
  char default_type[] = "v";
  if (argvars[2].v_type == VAR_DICT) {
    is_select_exclusive = tv_dict_get_bool(argvars[2].vval.v_dict, "exclusive",
                                           *p_sel == 'e');
    type = tv_dict_get_string(argvars[2].vval.v_dict, "type", false);
    if (type == NULL) {
      type = default_type;
    }
  } else {
    is_select_exclusive = *p_sel == 'e';
    type = default_type;
  }

  int block_width = 0;
  if (type[0] == 'v' && type[1] == NUL) {
    *region_type = kMTCharWise;
  } else if (type[0] == 'V' && type[1] == NUL) {
    *region_type = kMTLineWise;
  } else if (type[0] == Ctrl_V) {
    char *p = type + 1;
    if (*p != NUL && ((block_width = getdigits_int(&p, false, 0)) <= 0 || *p != NUL)) {
      semsg(_(e_invargNval), "type", type);
      return FAIL;
    }
    *region_type = kMTBlockWise;
  } else {
    semsg(_(e_invargNval), "type", type);
    return FAIL;
  }

  buf_T *findbuf = fnum1 != 0 ? buflist_findnr(fnum1) : curbuf;
  if (findbuf == NULL || findbuf->b_ml.ml_mfp == NULL) {
    emsg(_(e_buffer_is_not_loaded));
    return FAIL;
  }

  if (p1->lnum < 1 || p1->lnum > findbuf->b_ml.ml_line_count) {
    semsg(_(e_invalid_line_number_nr), p1->lnum);
    return FAIL;
  }
  if (p1->col == MAXCOL) {
    p1->col = ml_get_buf_len(findbuf, p1->lnum) + 1;
  } else if (p1->col < 1 || p1->col > ml_get_buf_len(findbuf, p1->lnum) + 1) {
    semsg(_(e_invalid_column_number_nr), p1->col);
    return FAIL;
  }

  if (p2->lnum < 1 || p2->lnum > findbuf->b_ml.ml_line_count) {
    semsg(_(e_invalid_line_number_nr), p2->lnum);
    return FAIL;
  }
  if (p2->col == MAXCOL) {
    p2->col = ml_get_buf_len(findbuf, p2->lnum) + 1;
  } else if (p2->col < 1 || p2->col > ml_get_buf_len(findbuf, p2->lnum) + 1) {
    semsg(_(e_invalid_column_number_nr), p2->col);
    return FAIL;
  }

  curbuf = findbuf;
  curwin->w_buffer = curbuf;
  virtual_op = virtual_active(curwin);

  // NOTE: Adjustment is needed.
  p1->col--;
  p2->col--;

  if (!lt(*p1, *p2)) {
    // swap position
    pos_T p = *p1;
    *p1 = *p2;
    *p2 = p;
  }

  if (*region_type == kMTCharWise) {
    // Handle 'selection' == "exclusive".
    if (is_select_exclusive && !equalpos(*p1, *p2)) {
      // When backing up to previous line, inclusive becomes false.
      *inclusive = !unadjust_for_sel_inner(p2);
    }
    // If p2 is on NUL (end of line), inclusive becomes false.
    if (*inclusive && !virtual_op && *ml_get_pos(p2) == NUL) {
      *inclusive = false;
    }
  } else if (*region_type == kMTBlockWise) {
    colnr_T sc1, ec1, sc2, ec2;
    getvvcol(curwin, p1, &sc1, NULL, &ec1);
    getvvcol(curwin, p2, &sc2, NULL, &ec2);
    oap->motion_type = kMTBlockWise;
    oap->inclusive = true;
    oap->op_type = OP_NOP;
    oap->start = *p1;
    oap->end = *p2;
    oap->start_vcol = MIN(sc1, sc2);
    if (block_width > 0) {
      oap->end_vcol = oap->start_vcol + block_width - 1;
    } else if (is_select_exclusive && ec1 < sc2 && 0 < sc2 && ec2 > ec1) {
      oap->end_vcol = sc2 - 1;
    } else {
      oap->end_vcol = MAX(ec1, ec2);
    }
  }

  // Include the trailing byte of a multi-byte char.
  int l = utfc_ptr2len(ml_get_pos(p2));
  if (l > 1) {
    p2->col += l - 1;
  }

  return OK;
}

/// "getregion()" function
static void f_getregion(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  buf_T *const save_curbuf = curbuf;
  const TriState save_virtual = virtual_op;

  pos_T p1, p2;
  bool inclusive = true;
  MotionType region_type = kMTUnknown;
  oparg_T oa;

  if (getregionpos(argvars, rettv, &p1, &p2, &inclusive, &region_type, &oa) == FAIL) {
    return;
  }

  for (linenr_T lnum = p1.lnum; lnum <= p2.lnum; lnum++) {
    char *akt = NULL;

    if (region_type == kMTLineWise) {
      akt = xstrdup(ml_get(lnum));
    } else if (region_type == kMTBlockWise) {
      struct block_def bd;
      block_prep(&oa, &bd, lnum, false);
      akt = block_def2str(&bd);
    } else if (p1.lnum < lnum && lnum < p2.lnum) {
      akt = xstrdup(ml_get(lnum));
    } else {
      struct block_def bd;
      charwise_block_prep(p1, p2, &bd, lnum, inclusive);
      akt = block_def2str(&bd);
    }

    assert(akt != NULL);
    tv_list_append_allocated_string(rettv->vval.v_list, akt);
  }

  // getregionpos() may change curbuf and virtual_op
  curbuf = save_curbuf;
  curwin->w_buffer = curbuf;
  virtual_op = save_virtual;
}

static void add_regionpos_range(typval_T *rettv, pos_T p1, pos_T p2)
{
  list_T *l1 = tv_list_alloc(2);
  tv_list_append_list(rettv->vval.v_list, l1);

  list_T *l2 = tv_list_alloc(4);
  tv_list_append_list(l1, l2);

  list_T *l3 = tv_list_alloc(4);
  tv_list_append_list(l1, l3);

  tv_list_append_number(l2, curbuf->b_fnum);
  tv_list_append_number(l2, p1.lnum);
  tv_list_append_number(l2, p1.col);
  tv_list_append_number(l2, p1.coladd);

  tv_list_append_number(l3, curbuf->b_fnum);
  tv_list_append_number(l3, p2.lnum);
  tv_list_append_number(l3, p2.col);
  tv_list_append_number(l3, p2.coladd);
}

/// "getregionpos()" function
static void f_getregionpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  buf_T *const save_curbuf = curbuf;
  const TriState save_virtual = virtual_op;

  pos_T p1, p2;
  bool inclusive = true;
  MotionType region_type = kMTUnknown;
  bool allow_eol = false;
  oparg_T oa;

  if (getregionpos(argvars, rettv, &p1, &p2, &inclusive, &region_type, &oa) == FAIL) {
    return;
  }

  if (argvars[2].v_type == VAR_DICT) {
    allow_eol = tv_dict_get_bool(argvars[2].vval.v_dict, "eol", false);
  }

  for (linenr_T lnum = p1.lnum; lnum <= p2.lnum; lnum++) {
    pos_T ret_p1, ret_p2;
    char *line = ml_get(lnum);
    colnr_T line_len = ml_get_len(lnum);

    if (region_type == kMTLineWise) {
      ret_p1.col = 1;
      ret_p1.coladd = 0;
      ret_p2.col = MAXCOL;
      ret_p2.coladd = 0;
    } else {
      struct block_def bd;

      if (region_type == kMTBlockWise) {
        block_prep(&oa, &bd, lnum, false);
      } else {
        charwise_block_prep(p1, p2, &bd, lnum, inclusive);
      }

      if (bd.is_oneChar) {  // selection entirely inside one char
        if (region_type == kMTBlockWise) {
          ret_p1.col = (colnr_T)(mb_prevptr(line, bd.textstart) - line) + 1;
          ret_p1.coladd = bd.start_char_vcols - (bd.start_vcol - oa.start_vcol);
        } else {
          ret_p1.col = p1.col + 1;
          ret_p1.coladd = p1.coladd;
        }
      } else if (region_type == kMTBlockWise && oa.start_vcol > bd.start_vcol) {
        // blockwise selection entirely beyond end of line
        ret_p1.col = MAXCOL;
        ret_p1.coladd = oa.start_vcol - bd.start_vcol;
        bd.is_oneChar = true;
      } else if (bd.startspaces > 0) {
        ret_p1.col = (colnr_T)(mb_prevptr(line, bd.textstart) - line) + 1;
        ret_p1.coladd = bd.start_char_vcols - bd.startspaces;
      } else {
        ret_p1.col = bd.textcol + 1;
        ret_p1.coladd = 0;
      }

      if (bd.is_oneChar) {  // selection entirely inside one char
        ret_p2.col = ret_p1.col;
        ret_p2.coladd = ret_p1.coladd + bd.startspaces + bd.endspaces;
      } else if (bd.endspaces > 0) {
        ret_p2.col = bd.textcol + bd.textlen + 1;
        ret_p2.coladd = bd.endspaces;
      } else {
        ret_p2.col = bd.textcol + bd.textlen;
        ret_p2.coladd = 0;
      }
    }

    if (!allow_eol && ret_p1.col > line_len) {
      ret_p1.col = 0;
      ret_p1.coladd = 0;
    } else if (ret_p1.col > line_len + 1) {
      ret_p1.col = line_len + 1;
    }

    if (!allow_eol && ret_p2.col > line_len) {
      ret_p2.col = ret_p1.col == 0 ? 0 : line_len;
      ret_p2.coladd = 0;
    } else if (ret_p2.col > line_len + 1) {
      ret_p2.col = line_len + 1;
    }

    ret_p1.lnum = lnum;
    ret_p2.lnum = lnum;
    add_regionpos_range(rettv, ret_p1, ret_p2);
  }

  // getregionpos() may change curbuf and virtual_op
  curbuf = save_curbuf;
  curwin->w_buffer = curbuf;
  virtual_op = save_virtual;
}


/// "gettagstack()" function
static void f_gettagstack(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  win_T *wp = curwin;                  // default is current window

  tv_dict_alloc_ret(rettv);

  if (argvars[0].v_type != VAR_UNKNOWN) {
    wp = find_win_by_nr_or_id(&argvars[0]);
    if (wp == NULL) {
      return;
    }
  }

  rs_get_tagstack(wp, rettv->vval.v_dict);
}

/// Dummy timer callback. Used by f_wait().
static void dummy_timer_due_cb(TimeWatcher *tw, void *data)
{
}

/// Dummy timer close callback. Used by f_wait().
static void dummy_timer_close_cb(TimeWatcher *tw, void *data) { xfree(tw); }

/// "wait(timeout, condition[, interval])" function
static void f_wait(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = -1;

  if (argvars[0].v_type != VAR_NUMBER) {
    semsg(_(e_invargval), "1");
    return;
  }
  if ((argvars[2].v_type != VAR_NUMBER && argvars[2].v_type != VAR_UNKNOWN)
      || (argvars[2].v_type == VAR_NUMBER && argvars[2].vval.v_number <= 0)) {
    semsg(_(e_invargval), "3");
    return;
  }

  int timeout = (int)argvars[0].vval.v_number;
  typval_T expr = argvars[1];
  int interval = argvars[2].v_type == VAR_NUMBER
                 ? (int)argvars[2].vval.v_number
                 : 200;  // Default.
  TimeWatcher *tw = xmalloc(sizeof(TimeWatcher));

  // Start dummy timer.
  time_watcher_init(&main_loop, tw, NULL);
  tw->events = loop_get_events(&main_loop);
  tw->blockable = true;
  time_watcher_start(tw, dummy_timer_due_cb, (uint64_t)interval, (uint64_t)interval);

  typval_T argv = TV_INITIAL_VALUE;
  typval_T exprval = TV_INITIAL_VALUE;
  bool error = false;
  const int called_emsg_before = called_emsg;

  // Flush screen updates before blocking.
  ui_flush();

  LOOP_PROCESS_EVENTS_UNTIL(&main_loop, loop_get_events(&main_loop), timeout,
                            eval_expr_typval(&expr, false, &argv, 0, &exprval) != OK
                            || tv_get_number_chk(&exprval, &error)
                            || called_emsg > called_emsg_before || error || got_int);

  if (called_emsg > called_emsg_before || error) {
    rettv->vval.v_number = -3;
  } else if (got_int) {
    got_int = false;
    vgetc();
    rettv->vval.v_number = -2;
  } else if (tv_get_number_chk(&exprval, &error)) {
    rettv->vval.v_number = 0;
  }

  // Stop dummy timer
  time_watcher_stop(tw);
  time_watcher_close(tw, dummy_timer_close_cb);
}


/// "has()" function

static bool has_wsl(void)
{
  static TriState has_wsl = kNone;
  if (has_wsl == kNone) {
    Error err = ERROR_INIT;
    Object o = NLUA_EXEC_STATIC("return vim.uv.os_uname()['release']:lower()"
                                ":match('microsoft')",
                                (Array)ARRAY_DICT_INIT, kRetNilBool, NULL, &err);
    assert(!ERROR_SET(&err));
    has_wsl = LUARET_TRUTHY(o) ? kTrue : kFalse;
  }
  return has_wsl == kTrue;
}

/// Public wrapper for has_wsl() used by Rust has() implementation.
int nvim_eval_has_wsl(void)
{
  return has_wsl() ? 1 : 0;
}

/// "highlightID(name)" function
/// "index()" function

static bool inputsecret_flag = false;

/// "input()" function
///     Also handles inputsecret() when inputsecret is set.
/// "inputlist()" function
static void f_inputlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (argvars[0].v_type != VAR_LIST) {
    semsg(_(e_listarg), "inputlist()");
    return;
  }

  msg_ext_set_kind("confirm");
  msg_start();
  msg_row = Rows - 1;   // for when 'cmdheight' > 1
  lines_left = Rows;    // avoid more prompt
  msg_scroll = true;
  msg_clr_eos();

  list_T *l = argvars[0].vval.v_list;
  TV_LIST_ITER_CONST(l, li, {
    msg_puts(tv_get_string(TV_LIST_ITEM_TV(li)));
    if (!ui_has(kUIMessages) || TV_LIST_ITEM_NEXT(l, li) != NULL) {
      msg_putchar('\n');
    }
  });

  // Ask for choice.
  bool mouse_used = false;
  int selected = prompt_for_input(NULL, 0, false, &mouse_used);
  if (mouse_used) {
    selected = tv_list_len(l) - (cmdline_row - mouse_row);
  }

  rettv->vval.v_number = selected;
}

static garray_T ga_userinput = { 0, 0, sizeof(tasave_T), 4, NULL };

/// "inputrestore()" function
static void f_inputrestore(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (!GA_EMPTY(&ga_userinput)) {
    ga_userinput.ga_len--;
    restore_typeahead((tasave_T *)(ga_userinput.ga_data)
                      + ga_userinput.ga_len);
    // default return is zero == OK
  } else if (p_verbose > 1) {
    verb_msg(_("called inputrestore() more often than inputsave()"));
    rettv->vval.v_number = 1;  // Failed
  }
}

/// "inputsave()" function
static void f_inputsave(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  // Add an entry to the stack of typeahead storage.
  tasave_T *p = GA_APPEND_VIA_PTR(tasave_T, &ga_userinput);
  save_typeahead(p);
}

/// "inputsecret()" function
static void f_inputsecret(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  cmdline_star++;
  inputsecret_flag = true;
  f_input(argvars, rettv, fptr);
  cmdline_star--;
  inputsecret_flag = false;
}

/// "islocked()" function
static void f_islocked(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  lval_T lv;

  rettv->vval.v_number = -1;
  const char *const end = get_lval((char *)tv_get_string(&argvars[0]),
                                   NULL,
                                   &lv, false, false,
                                   GLV_NO_AUTOLOAD|GLV_READ_ONLY,
                                   FNE_CHECK_START);
  if (end != NULL && lv.ll_name != NULL) {
    if (*end != NUL) {
      semsg(_(lv.ll_name_len == 0 ? e_invarg2 : e_trailing_arg), end);
    } else {
      if (lv.ll_tv == NULL) {
        dictitem_T *di = find_var(lv.ll_name, lv.ll_name_len, NULL, true);
        if (di != NULL) {
          // Consider a variable locked when:
          // 1. the variable itself is locked
          // 2. the value of the variable is locked.
          // 3. the List or Dict value is locked.
          rettv->vval.v_number = ((di->di_flags & DI_FLAGS_LOCK)
                                  || tv_islocked(&di->di_tv));
        }
      } else if (lv.ll_range) {
        emsg(_("E786: Range not allowed"));
      } else if (lv.ll_newkey != NULL) {
        semsg(_(e_dictkey), lv.ll_newkey);
      } else if (lv.ll_list != NULL) {
        // List item.
        rettv->vval.v_number = tv_islocked(TV_LIST_ITEM_TV(lv.ll_li));
      } else {
        // Dictionary item.
        rettv->vval.v_number = tv_islocked(&lv.ll_di->di_tv);
      }
    }
  }

  clear_lval(&lv);
}

/// "id()" function

/// "jobpid(id)" function
static void f_jobpid(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;

  if (rs_check_secure()) {
    return;
  }

  if (argvars[0].v_type != VAR_NUMBER) {
    emsg(_(e_invarg));
    return;
  }

  Channel *data = find_job((uint64_t)argvars[0].vval.v_number, true);
  if (!data) {
    return;
  }

  Proc *proc = &data->stream.proc;
  rettv->vval.v_number = proc_get_pid(proc);
}

/// "jobresize(job, width, height)" function
static void f_jobresize(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;

  if (rs_check_secure()) {
    return;
  }

  if (argvars[0].v_type != VAR_NUMBER || argvars[1].v_type != VAR_NUMBER
      || argvars[2].v_type != VAR_NUMBER) {
    // job id, width, height
    emsg(_(e_invarg));
    return;
  }

  Channel *data = find_job((uint64_t)argvars[0].vval.v_number, true);
  if (!data) {
    return;
  }

  if (data->stream.proc.type != kProcTypePty) {
    emsg(_(e_channotpty));
    return;
  }

  pty_proc_resize(&data->stream.pty, (uint16_t)argvars[1].vval.v_number,
                  (uint16_t)argvars[2].vval.v_number);
  rettv->vval.v_number = 1;
}

static const char *pty_ignored_env_vars[] = {
#ifndef MSWIN
  "COLUMNS",
  "LINES",
  "TERMCAP",
  "COLORFGBG",
  "COLORTERM",
#endif
  // Nvim-owned env vars. #6764
  "VIM",
  "VIMRUNTIME",
  NULL
};

/// According to comments in src/win/process.c of libuv, Windows has a few
/// "essential" environment variables.
static const char *required_env_vars[] = {
#ifdef MSWIN
  "HOMEDRIVE",
  "HOMEPATH",
  "LOGONSERVER",
  "PATH",
  "SYSTEMDRIVE",
  "SYSTEMROOT",
  "TEMP",
  "USERDOMAIN",
  "USERNAME",
  "USERPROFILE",
  "WINDIR",
#endif
  NULL
};

dict_T *create_environment(const dictitem_T *job_env, const bool clear_env, const bool pty,
                           const char * const pty_term_name)
{
  dict_T *env = tv_dict_alloc();

  if (!clear_env) {
    typval_T temp_env = TV_INITIAL_VALUE;
    f_environ(NULL, &temp_env, (EvalFuncData){ .null = NULL });
    tv_dict_extend(env, temp_env.vval.v_dict, "force");
    tv_dict_free(temp_env.vval.v_dict);

    if (pty) {
      // These env vars shouldn't propagate to the child process. #6764
      // Remove them here, then the user may decide to explicitly set them below.
      for (size_t i = 0;
           i < ARRAY_SIZE(pty_ignored_env_vars) && pty_ignored_env_vars[i];
           i++) {
        dictitem_T *dv = tv_dict_find(env, pty_ignored_env_vars[i], -1);
        if (dv) {
          tv_dict_item_remove(env, dv);
        }
      }
#ifndef MSWIN
      // Set COLORTERM to "truecolor" if termguicolors is set
      if (p_tgc) {
        tv_dict_add_str(env, S_LEN("COLORTERM"), "truecolor");
      }
#endif
    }
  }

  // For a pty, we need a sane $TERM set.  We can't rely on nvim's environment,
  // because the child process is going to be communicating with nvim, not the
  // parent terminal.  Set a sane default, but let the user override it in the
  // job's environment if they want.
  if (pty) {
    dictitem_T *dv = tv_dict_find(env, S_LEN("TERM"));
    if (dv) {
      tv_dict_item_remove(env, dv);
    }
    tv_dict_add_str(env, S_LEN("TERM"), pty_term_name);
  }

  // Set $NVIM (in the child process) to v:servername. #3118
  char *nvim_addr = get_vim_var_str(VV_SEND_SERVER);
  if (nvim_addr[0] != NUL) {
    dictitem_T *dv = tv_dict_find(env, S_LEN("NVIM"));
    if (dv) {
      tv_dict_item_remove(env, dv);
    }
    tv_dict_add_str(env, S_LEN("NVIM"), nvim_addr);
  }

  if (job_env) {
#ifdef MSWIN
    TV_DICT_ITER(job_env->di_tv.vval.v_dict, var, {
      // Always use upper-case keys for Windows so we detect duplicate keys
      char *const key = strcase_save(var->di_key, true);
      size_t len = strlen(key);
      dictitem_T *dv = tv_dict_find(env, key, len);
      if (dv) {
        tv_dict_item_remove(env, dv);
      }
      tv_dict_add_str(env, key, len, tv_get_string(&var->di_tv));
      xfree(key);
    });
#else
    tv_dict_extend(env, job_env->di_tv.vval.v_dict, "force");
#endif
  }

  if (pty) {
    // Now that the custom environment is configured, we need to ensure certain
    // environment variables are present.
    for (size_t i = 0;
         i < ARRAY_SIZE(required_env_vars) && required_env_vars[i];
         i++) {
      size_t len = strlen(required_env_vars[i]);
      dictitem_T *dv = tv_dict_find(env, required_env_vars[i], (ptrdiff_t)len);
      if (!dv) {
        char *env_var = os_getenv(required_env_vars[i]);
        if (env_var) {
          tv_dict_add_allocated_str(env, required_env_vars[i], len, env_var);
        }
      }
    }
  }

  return env;
}

/// "jobstart()" function
void f_jobstart(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;

  if (rs_check_secure()) {
    return;
  }

  const char *cmd;
  bool executable = true;
  char **argv = tv_to_argv(&argvars[0], &cmd, &executable);
  if (!argv) {
    rettv->vval.v_number = executable ? 0 : -1;
    return;  // Did error message in tv_to_argv.
  }

  if (argvars[1].v_type != VAR_DICT && argvars[1].v_type != VAR_UNKNOWN) {
    // Wrong argument types
    semsg(_(e_invarg2), "expected dictionary");
    shell_free_argv(argv);
    return;
  }

  dict_T *job_opts = NULL;
  bool detach = false;
  bool rpc = false;
  bool pty = false;
  bool term = false;
  bool clear_env = false;
  bool overlapped = false;
  ChannelStdinMode stdin_mode = kChannelStdinPipe;
  CallbackReader on_stdout = CALLBACK_READER_INIT;
  CallbackReader on_stderr = CALLBACK_READER_INIT;
  Callback on_exit = CALLBACK_NONE;
  char *cwd = NULL;
  dictitem_T *job_env = NULL;
  if (argvars[1].v_type == VAR_DICT) {
    job_opts = argvars[1].vval.v_dict;

    detach = tv_dict_get_number(job_opts, "detach") != 0;
    rpc = tv_dict_get_number(job_opts, "rpc") != 0;
    term = tv_dict_get_number(job_opts, "term") != 0;
    pty = term || tv_dict_get_number(job_opts, "pty") != 0;
    clear_env = tv_dict_get_number(job_opts, "clear_env") != 0;
    overlapped = tv_dict_get_number(job_opts, "overlapped") != 0;

    char *s = tv_dict_get_string(job_opts, "stdin", false);
    if (s) {
      if (!strncmp(s, "null", NUMBUFLEN)) {
        stdin_mode = kChannelStdinNull;
      } else if (!strncmp(s, "pipe", NUMBUFLEN)) {
        // Nothing to do, default value
      } else {
        semsg(_(e_invargNval), "stdin", s);
      }
    }

    dictitem_T *const job_term = tv_dict_find(job_opts, S_LEN("term"));
    if (job_term && VAR_BOOL != job_term->di_tv.v_type) {
      // Restrict "term" field to boolean, in case we want to allow buffer numbers in the future.
      semsg(_(e_invarg2), "'term' must be Boolean");
      shell_free_argv(argv);
      return;
    }

    if (pty && rpc) {
      semsg(_(e_invarg2), "job cannot have both 'pty' and 'rpc' options set");
      shell_free_argv(argv);
      return;
    }

#ifdef MSWIN
    if (pty && overlapped) {
      semsg(_(e_invarg2),
            "job cannot have both 'pty' and 'overlapped' options set");
      shell_free_argv(argv);
      return;
    }
#endif

    char *new_cwd = tv_dict_get_string(job_opts, "cwd", false);
    if (new_cwd && *new_cwd != NUL) {
      cwd = new_cwd;
      // The new cwd must be a directory.
      if (!os_isdir(cwd)) {
        semsg(_(e_invarg2), "expected valid directory");
        shell_free_argv(argv);
        return;
      }
    }

    job_env = tv_dict_find(job_opts, S_LEN("env"));
    if (job_env && job_env->di_tv.v_type != VAR_DICT) {
      semsg(_(e_invarg2), "env");
      shell_free_argv(argv);
      return;
    }

    if (!common_job_callbacks(job_opts, &on_stdout, &on_stderr, &on_exit)) {
      shell_free_argv(argv);
      return;
    }
  }

  uint16_t width = (uint16_t)tv_dict_get_number(job_opts, "width");
  uint16_t height = (uint16_t)tv_dict_get_number(job_opts, "height");
  char *term_name = NULL;

  if (term) {
    if (text_locked()) {
      text_locked_msg();
      shell_free_argv(argv);
      return;
    }
    if (curbuf->b_changed) {
      emsg(_("jobstart(...,{term=true}) requires unmodified buffer"));
      shell_free_argv(argv);
      return;
    }
    assert(!rpc);
    term_name = "xterm-256color";
    cwd = cwd ? cwd : ".";
    overlapped = false;
    detach = false;
    stdin_mode = kChannelStdinPipe;
    width = width ? width : (uint16_t)MAX(0, curwin->w_view_width - win_col_off(curwin));
    height = height ? height : (uint16_t)curwin->w_view_height;
  }

  if (pty) {
    // Deprecated TERM field is from before `env` option existed.
    term_name = term_name ? term_name : tv_dict_get_string(job_opts, "TERM", false);
    term_name = term_name ? term_name : "ansi";
  }

  dict_T *env = create_environment(job_env, clear_env, pty, term_name);
  Channel *chan = channel_job_start(argv, NULL, on_stdout, on_stderr, on_exit, pty,
                                    rpc, overlapped, detach, stdin_mode, cwd,
                                    width, height, env, &rettv->vval.v_number);
  if (!chan) {
    return;
  } else if (!term) {
    channel_create_event(chan, NULL);
  } else {
    if (rettv->vval.v_number <= 0) {
      return;
    }

    int pid = chan->stream.pty.proc.pid;

    // "./…" => "/home/foo/…"
    vim_FullName(cwd, NameBuff, sizeof(NameBuff), false);
    // "/home/foo/…" => "~/…"
    size_t len = home_replace(NULL, NameBuff, IObuff, sizeof(IObuff), true);
    // Trim slash.
    if (len != 1 && (IObuff[len - 1] == '\\' || IObuff[len - 1] == '/')) {
      IObuff[len - 1] = NUL;
    }

    if (len == 1 && IObuff[0] == '/') {
      // Avoid ambiguity in the URI when CWD is root directory.
      IObuff[1] = '.';
      IObuff[2] = NUL;
    }

    // Terminal URI: "term://$CWD//$PID:$CMD"
    snprintf(NameBuff, sizeof(NameBuff), "term://%s//%d:%s", IObuff, pid, cmd);
    // Buffer has no terminal associated yet; unset 'swapfile' to ensure no swapfile is created.
    curbuf->b_p_swf = false;

    apply_autocmds(EVENT_BUFFILEPRE, NULL, NULL, false, curbuf);
    setfname(curbuf, NameBuff, NULL, true);
    apply_autocmds(EVENT_BUFFILEPOST, NULL, NULL, false, curbuf);

    Error err = ERROR_INIT;
    // Set (deprecated) buffer-local vars (prefer 'channel' buffer-local option).
    dict_set_var(curbuf->b_vars, cstr_as_string("terminal_job_id"),
                 INTEGER_OBJ((Integer)chan->id), false, false, NULL, &err);
    api_clear_error(&err);
    dict_set_var(curbuf->b_vars, cstr_as_string("terminal_job_pid"),
                 INTEGER_OBJ(pid), false, false, NULL, &err);
    api_clear_error(&err);

    channel_incref(chan);
    channel_terminal_open(curbuf, chan);
    channel_create_event(chan, NULL);
    channel_decref(chan);
  }
}

/// "jobstop()" function
void f_jobstop(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;

  if (rs_check_secure()) {
    return;
  }

  if (argvars[0].v_type != VAR_NUMBER) {
    // Only argument is the job id
    emsg(_(e_invarg));
    return;
  }

  Channel *data = find_job((uint64_t)argvars[0].vval.v_number, false);
  if (!data) {
    return;
  }

  const char *error = NULL;
  if (data->is_rpc) {
    // Ignore return code, but show error later.
    channel_close(data->id, kChannelPartRpc, &error);
  }
  proc_stop(&data->stream.proc);
  rettv->vval.v_number = 1;
  if (error) {
    emsg(error);
  }
}

/// "jobwait(ids[, timeout])" function
static void f_jobwait(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;

  if (rs_check_secure()) {
    return;
  }
  if (argvars[0].v_type != VAR_LIST || (argvars[1].v_type != VAR_NUMBER
                                        && argvars[1].v_type != VAR_UNKNOWN)) {
    emsg(_(e_invarg));
    return;
  }

  list_T *args = argvars[0].vval.v_list;
  Channel **jobs = xcalloc((size_t)tv_list_len(args), sizeof(*jobs));
  MultiQueue *waiting_jobs = multiqueue_new(loop_on_put, &main_loop);

  // Validate, prepare jobs for waiting.
  int i = 0;
  TV_LIST_ITER_CONST(args, arg, {
    Channel *chan = NULL;
    if (TV_LIST_ITEM_TV(arg)->v_type != VAR_NUMBER
        || !(chan = find_channel((uint64_t)TV_LIST_ITEM_TV(arg)->vval.v_number))
        || chan->streamtype != kChannelStreamProc) {
      jobs[i] = NULL;  // Invalid job.
    } else if (proc_is_stopped(&chan->stream.proc)) {
      // Job is stopped but not fully destroyed.
      // Ensure all callbacks on its event queue are executed. #15402
      proc_wait(&chan->stream.proc, -1, NULL);
      jobs[i] = NULL;  // Invalid job.
    } else {
      jobs[i] = chan;
      channel_incref(chan);
      if (chan->stream.proc.status < 0) {
        // Flush any events in the job's queue before temporarily replacing it.
        multiqueue_process_events(chan->events);
        multiqueue_replace_parent(chan->events, waiting_jobs);
      }
    }
    i++;
  });

  int remaining = -1;
  uint64_t before = 0;
  if (argvars[1].v_type == VAR_NUMBER && argvars[1].vval.v_number >= 0) {
    remaining = (int)argvars[1].vval.v_number;
    before = os_hrtime();
  }

  // Only mark the UI as busy when jobwait() blocks
  const bool busy = remaining != 0;
  if (busy) {
    ui_busy_start();
    ui_flush();
  }

  for (i = 0; i < tv_list_len(args); i++) {
    if (remaining == 0) {
      break;  // Timeout.
    }
    if (jobs[i] == NULL) {
      continue;  // Invalid job, will assign status=-3 below.
    }
    int status = proc_wait(&jobs[i]->stream.proc, remaining,
                           waiting_jobs);
    if (status < 0) {
      break;  // Interrupted (CTRL-C) or timeout, skip remaining jobs.
    }
    if (remaining > 0) {
      uint64_t now = os_hrtime();
      remaining = MIN(0, remaining - (int)((now - before) / 1000000));
      before = now;
    }
  }

  list_T *const rv = tv_list_alloc(tv_list_len(args));

  // For each job:
  //  * Restore its parent queue if the job is still alive.
  //  * Append its status to the output list, or:
  //       -3 for "invalid job id"
  //       -2 for "interrupted" (user hit CTRL-C)
  //       -1 for jobs that were skipped or timed out
  for (i = 0; i < tv_list_len(args); i++) {
    if (jobs[i] == NULL) {
      tv_list_append_number(rv, -3);
      continue;
    }
    multiqueue_process_events(jobs[i]->events);
    multiqueue_replace_parent(jobs[i]->events, loop_get_events(&main_loop));

    tv_list_append_number(rv, jobs[i]->stream.proc.status);
    channel_decref(jobs[i]);
  }

  multiqueue_free(waiting_jobs);
  xfree(jobs);
  if (busy) {
    ui_busy_stop();
  }
  tv_list_ref(rv);
  rettv->v_type = VAR_LIST;
  rettv->vval.v_list = rv;
}

/// json_decode() function

/// json_encode() function
/// "keytrans()" function


/// "libcall()" function
/// "line(string, [winid])" function
static void f_line(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  linenr_T lnum = 0;
  pos_T *fp = NULL;
  int fnum;

  if (argvars[1].v_type != VAR_UNKNOWN) {
    // use window specified in the second argument
    int id = (int)tv_get_number(&argvars[1]);
    tabpage_T *tp;
    win_T *wp = win_id2wp_tp(id, &tp);
    if (wp != NULL && tp != NULL) {
      switchwin_T switchwin;
      if (switch_win_noblock(&switchwin, wp, tp, true) == OK) {
        // With 'splitkeep' != cursor and in diff mode, prevent that the
        // window scrolls and keep the topline.
        if (*p_spk != 'c' || (curwin->w_p_diff && switchwin.sw_curwin->w_p_diff)) {
          skip_update_topline = true;
        }
        check_cursor(curwin);
        fp = var2fpos(&argvars[0], true, &fnum, false);
      }
      skip_update_topline = false;
      restore_win_noblock(&switchwin, true);
    }
  } else {
    // use current window
    fp = var2fpos(&argvars[0], true, &fnum, false);
  }

  if (fp != NULL) {
    lnum = fp->lnum;
  }
  rettv->vval.v_number = lnum;
}


/// Return all the matches in string "str" for pattern "rmp".
/// The matches are returned in the List "mlist".
/// If "submatches" is true, then submatch information is also returned.
/// "matchbuf" is true when called for matchbufline().
static void get_matches_in_str(const char *str, regmatch_T *rmp, list_T *mlist, int idx,
                               bool submatches, bool matchbuf)
{
  size_t len = strlen(str);
  int match = 0;
  colnr_T startidx = 0;

  while (true) {
    match = vim_regexec_nl(rmp, str, startidx);
    if (!match) {
      break;
    }

    dict_T *d = tv_dict_alloc();
    tv_list_append_dict(mlist, d);

    if (matchbuf) {
      tv_dict_add_nr(d, S_LEN("lnum"), idx);
    } else {
      tv_dict_add_nr(d, S_LEN("idx"), idx);
    }

    tv_dict_add_nr(d, S_LEN("byteidx"),
                   (colnr_T)(rmp->startp[0] - str));

    tv_dict_add_str_len(d, S_LEN("text"), rmp->startp[0],
                        (int)(rmp->endp[0] - rmp->startp[0]));

    if (submatches) {
      list_T *sml = tv_list_alloc(NSUBEXP - 1);

      tv_dict_add_list(d, S_LEN("submatches"), sml);

      // return a list with the submatches
      for (int i = 1; i < NSUBEXP; i++) {
        if (rmp->endp[i] == NULL) {
          tv_list_append_string(sml, "", 0);
        } else {
          tv_list_append_string(sml, rmp->startp[i], rmp->endp[i] - rmp->startp[i]);
        }
      }
    }
    startidx = (colnr_T)(rmp->endp[0] - str);
    if (startidx >= (colnr_T)len || str + startidx <= rmp->startp[0]) {
      break;
    }
  }
}

/// "matchbufline()" function
static void f_matchbufline(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->vval.v_number = -1;
  tv_list_alloc_ret(rettv, kListLenUnknown);
  list_T *retlist = rettv->vval.v_list;

  if (tv_check_for_buffer_arg(argvars, 0) == FAIL
      || tv_check_for_string_arg(argvars, 1) == FAIL
      || tv_check_for_lnum_arg(argvars, 2) == FAIL
      || tv_check_for_lnum_arg(argvars, 3) == FAIL
      || tv_check_for_opt_dict_arg(argvars, 4) == FAIL) {
    return;
  }

  const int prev_did_emsg = did_emsg;
  buf_T *buf = tv_get_buf(&argvars[0], false);
  if (buf == NULL) {
    if (did_emsg == prev_did_emsg) {
      semsg(_(e_invalid_buffer_name_str), tv_get_string(&argvars[0]));
    }
    return;
  }
  if (buf->b_ml.ml_mfp == NULL) {
    emsg(_(e_buffer_is_not_loaded));
    return;
  }

  char patbuf[NUMBUFLEN];
  const char *pat = tv_get_string_buf(&argvars[1], patbuf);

  const int did_emsg_before = did_emsg;
  linenr_T slnum = tv_get_lnum_buf(&argvars[2], buf);
  if (did_emsg > did_emsg_before) {
    return;
  }
  if (slnum < 1) {
    semsg(_(e_invargval), "lnum");
    return;
  }

  linenr_T elnum = tv_get_lnum_buf(&argvars[3], buf);
  if (did_emsg > did_emsg_before) {
    return;
  }
  if (elnum < 1 || elnum < slnum) {
    semsg(_(e_invargval), "end_lnum");
    return;
  }

  if (elnum > buf->b_ml.ml_line_count) {
    elnum = buf->b_ml.ml_line_count;
  }

  bool submatches = false;
  if (argvars[4].v_type != VAR_UNKNOWN) {
    dict_T *d = argvars[4].vval.v_dict;
    if (d != NULL) {
      dictitem_T *di = tv_dict_find(d, S_LEN("submatches"));
      if (di != NULL) {
        if (di->di_tv.v_type != VAR_BOOL) {
          semsg(_(e_invargval), "submatches");
          return;
        }
        submatches = tv_get_bool(&di->di_tv);
      }
    }
  }

  // Make 'cpoptions' empty, the 'l' flag should not be used here.
  char *const save_cpo = p_cpo;
  p_cpo = empty_string_option;

  regmatch_T regmatch;
  regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
  if (regmatch.regprog == NULL) {
    goto theend;
  }
  regmatch.rm_ic = p_ic;

  while (slnum <= elnum) {
    const char *str = ml_get_buf(buf, slnum);
    get_matches_in_str(str, &regmatch, retlist, slnum, submatches, true);
    slnum++;
  }

  vim_regfree(regmatch.regprog);

theend:
  p_cpo = save_cpo;
}

/// "matchstrlist()" function
static void f_matchstrlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->vval.v_number = -1;
  tv_list_alloc_ret(rettv, kListLenUnknown);
  list_T *retlist = rettv->vval.v_list;

  if (tv_check_for_list_arg(argvars, 0) == FAIL
      || tv_check_for_string_arg(argvars, 1) == FAIL
      || tv_check_for_opt_dict_arg(argvars, 2) == FAIL) {
    return;
  }

  list_T *l = NULL;
  if ((l = argvars[0].vval.v_list) == NULL) {
    return;
  }

  char patbuf[NUMBUFLEN];
  const char *pat = tv_get_string_buf_chk(&argvars[1], patbuf);
  if (pat == NULL) {
    return;
  }

  // Make 'cpoptions' empty, the 'l' flag should not be used here.
  char *const save_cpo = p_cpo;
  p_cpo = empty_string_option;

  regmatch_T regmatch;
  regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
  if (regmatch.regprog == NULL) {
    goto theend;
  }
  regmatch.rm_ic = p_ic;

  bool submatches = false;
  if (argvars[2].v_type != VAR_UNKNOWN) {
    dict_T *d = argvars[2].vval.v_dict;
    if (d != NULL) {
      dictitem_T *di = tv_dict_find(d, S_LEN("submatches"));
      if (di != NULL) {
        if (di->di_tv.v_type != VAR_BOOL) {
          semsg(_(e_invargval), "submatches");
          goto cleanup;
        }
        submatches = tv_get_bool(&di->di_tv);
      }
    }
  }

  int idx = 0;
  TV_LIST_ITER_CONST(l, li, {
    const typval_T *const li_tv = TV_LIST_ITEM_TV(li);
    if (li_tv->v_type == VAR_STRING && li_tv->vval.v_string != NULL) {
      const char *str = li_tv->vval.v_string;
      get_matches_in_str(str, &regmatch, retlist, idx, submatches, false);
    }
    idx++;
  });

cleanup:
  vim_regfree(regmatch.regprog);

theend:
  p_cpo = save_cpo;
}

/// Get maximal/minimal number value in a list or dictionary
///
/// @param[in]  tv  List or dictionary to work with. If it contains something
///                 that is not an integer number (or cannot be coerced to
///                 it) error is given.
/// @param[out]  rettv  Location where result will be saved. Only assigns
///                     vval.v_number, type is not touched. Returns zero for
///                     empty lists/dictionaries.
/// @param[in]  domax  Determines whether maximal or minimal value is desired.


/// "mode()" function


/// "msgpackdump()" function
static void f_msgpackdump(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
  FUNC_ATTR_NONNULL_ALL
{
  if (argvars[0].v_type != VAR_LIST) {
    semsg(_(e_listarg), "msgpackdump()");
    return;
  }
  list_T *const list = argvars[0].vval.v_list;
  PackerBuffer packer = packer_string_buffer();
  const char *const msg = _("msgpackdump() argument, index %i");
  // Assume that translation will not take more then 4 times more space
  char msgbuf[sizeof("msgpackdump() argument, index ") * 4 + NUMBUFLEN];
  int idx = 0;
  TV_LIST_ITER(list, li, {
    vim_snprintf(msgbuf, sizeof(msgbuf), msg, idx);
    idx++;
    if (encode_vim_to_msgpack(&packer, TV_LIST_ITEM_TV(li), msgbuf) == FAIL) {
      break;
    }
  });
  String data = packer_take_string(&packer);
  if (argvars[1].v_type != VAR_UNKNOWN && strequal(tv_get_string(&argvars[1]), "B")) {
    blob_T *b = tv_blob_alloc_ret(rettv);
    b->bv_ga.ga_data = data.data;
    b->bv_ga.ga_len = (int)data.size;
    b->bv_ga.ga_maxlen = (int)(packer.endptr - packer.startptr);
  } else {
    encode_list_write(tv_list_alloc_ret(rettv, kListLenMayKnow), data.data, data.size);
    api_free_string(data);
  }
}

static void emsg_mpack_error(int status)
{
  switch (status) {
  case MPACK_ERROR:
    semsg(_(e_invarg2), "Failed to parse msgpack string");
    break;

  case MPACK_EOF:
    semsg(_(e_invarg2), "Incomplete msgpack string");
    break;

  case MPACK_NOMEM:
    semsg(_(e_invarg2), "object was too deep to unpack");
    break;

  default:
    break;
  }
}

static void msgpackparse_unpack_list(const list_T *const list, list_T *const ret_list)
  FUNC_ATTR_NONNULL_ARG(2)
{
  if (tv_list_len(list) == 0) {
    return;
  }
  if (TV_LIST_ITEM_TV(tv_list_first(list))->v_type != VAR_STRING) {
    semsg(_(e_invarg2), "List item is not a string");
    return;
  }
  ListReaderState lrstate = encode_init_lrstate(list);
  char *buf = alloc_block();
  size_t buf_size = 0;

  typval_T cur_item = { .v_type = VAR_UNKNOWN };
  mpack_parser_t parser;
  mpack_parser_init(&parser, 0);
  parser.data.p = &cur_item;

  int status = MPACK_OK;
  while (true) {
    size_t read_bytes;
    const int rlret = encode_read_from_list(&lrstate, buf + buf_size, ARENA_BLOCK_SIZE - buf_size,
                                            &read_bytes);
    if (rlret == FAIL) {
      semsg(_(e_invarg2), "List item is not a string");
      goto end;
    }
    buf_size += read_bytes;

    const char *ptr = buf;
    while (buf_size) {
      status = mpack_parse_typval(&parser, &ptr, &buf_size);
      if (status == MPACK_OK) {
        tv_list_append_owned_tv(ret_list, cur_item);
        cur_item.v_type = VAR_UNKNOWN;
      } else {
        break;
      }
    }

    if (rlret == OK) {
      break;
    }

    if (status == MPACK_EOF) {
      // move remaining data to front of buffer
      if (buf_size && ptr > buf) {
        memmove(buf, ptr, buf_size);
      }
    } else if (status != MPACK_OK) {
      break;
    }
  }

  if (status != MPACK_OK) {
    typval_parser_error_free(&parser);
    emsg_mpack_error(status);
  }

end:
  free_block(buf);
}

static void msgpackparse_unpack_blob(const blob_T *const blob, list_T *const ret_list)
  FUNC_ATTR_NONNULL_ARG(2)
{
  const int len = tv_blob_len(blob);
  if (len == 0) {
    return;
  }

  const char *data = blob->bv_ga.ga_data;
  size_t remaining = (size_t)len;
  while (remaining) {
    typval_T tv;
    int status = unpack_typval(&data, &remaining, &tv);
    if (status != MPACK_OK) {
      emsg_mpack_error(status);
      return;
    }

    tv_list_append_owned_tv(ret_list, tv);
  }
}

/// "msgpackparse" function
static void f_msgpackparse(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
  FUNC_ATTR_NONNULL_ALL
{
  if (argvars[0].v_type != VAR_LIST && argvars[0].v_type != VAR_BLOB) {
    semsg(_(e_listblobarg), "msgpackparse()");
    return;
  }
  list_T *const ret_list = tv_list_alloc_ret(rettv, kListLenMayKnow);
  if (argvars[0].v_type == VAR_LIST) {
    msgpackparse_unpack_list(argvars[0].vval.v_list, ret_list);
  } else {
    msgpackparse_unpack_blob(argvars[0].vval.v_blob, ret_list);
  }
}

/// "nextnonblank()" function


/// "printf()" function

/// "prompt_getprompt({buffer})" function
static void f_prompt_getprompt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
  FUNC_ATTR_NONNULL_ALL
{
  // return an empty string by default, e.g. it's not a prompt buffer
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  buf_T *const buf = tv_get_buf_from_arg(&argvars[0]);
  if (buf == NULL) {
    return;
  }

  if (!bt_prompt(buf)) {
    return;
  }

  rettv->vval.v_string = xstrdup(buf_prompt_text(buf));
}

/// "prompt_getinput({buffer})" function
static void f_prompt_getinput(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
  FUNC_ATTR_NONNULL_ALL
{
  // return an empty string by default, e.g. it's not a prompt buffer
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  buf_T *const buf = tv_get_buf_from_arg(&argvars[0]);
  if (buf == NULL) {
    return;
  }

  if (!bt_prompt(buf)) {
    return;
  }

  rettv->vval.v_string = prompt_get_input(buf);
}


/// "py3eval()" and "pyxeval()" functions (always python3)


/// "perleval()" function
/// "range()" function

/// "getreginfo()" function


/// "reltimestr()" function


#define SP_NOMOVE       0x01        ///< don't move cursor
#define SP_REPEAT       0x02        ///< repeat to find outer pair
#define SP_RETCOUNT     0x04        ///< return matchcount
#define SP_SETPCMARK    0x08        ///< set previous context mark
#define SP_START        0x10        ///< accept match at start position
#define SP_SUBPAT       0x20        ///< return nr of matching sub-pattern
#define SP_END          0x40        ///< leave cursor at end of match
#define SP_COLUMN       0x80        ///< start at cursor column

/// Get flags for a search function.
/// Possibly sets "p_ws".
///
/// @return  BACKWARD, FORWARD or zero (for an error).
static int get_search_arg(typval_T *varp, int *flagsp)
{
  int dir = FORWARD;

  if (varp->v_type == VAR_UNKNOWN) {
    return FORWARD;
  }

  char nbuf[NUMBUFLEN];
  const char *flags = tv_get_string_buf_chk(varp, nbuf);
  if (flags == NULL) {
    return 0;  // Type error; errmsg already given.
  }
  int mask;
  while (*flags != NUL) {
    switch (*flags) {
    case 'b':
      dir = BACKWARD; break;
    case 'w':
      p_ws = true; break;
    case 'W':
      p_ws = false; break;
    default:
      mask = 0;
      if (flagsp != NULL) {
        switch (*flags) {
        case 'c':
          mask = SP_START; break;
        case 'e':
          mask = SP_END; break;
        case 'm':
          mask = SP_RETCOUNT; break;
        case 'n':
          mask = SP_NOMOVE; break;
        case 'p':
          mask = SP_SUBPAT; break;
        case 'r':
          mask = SP_REPEAT; break;
        case 's':
          mask = SP_SETPCMARK; break;
        case 'z':
          mask = SP_COLUMN; break;
        }
      }
      if (mask == 0) {
        semsg(_(e_invarg2), flags);
        dir = 0;
      } else {
        *flagsp |= mask;
      }
    }
    if (dir == 0) {
      break;
    }
    flags++;
  }
  return dir;
}

/// Shared by search() and searchpos() functions.
static int search_cmn(typval_T *argvars, pos_T *match_pos, int *flagsp)
{
  bool save_p_ws = p_ws;
  int retval = 0;               // default: FAIL
  linenr_T lnum_stop = 0;
  int64_t time_limit = 0;
  int options = SEARCH_KEEP;
  bool use_skip = false;

  const char *const pat = tv_get_string(&argvars[0]);
  int dir = get_search_arg(&argvars[1], flagsp);  // May set p_ws.
  if (dir == 0) {
    goto theend;
  }
  int flags = *flagsp;
  if (flags & SP_START) {
    options |= SEARCH_START;
  }
  if (flags & SP_END) {
    options |= SEARCH_END;
  }
  if (flags & SP_COLUMN) {
    options |= SEARCH_COL;
  }

  // Optional arguments: line number to stop searching, timeout and skip.
  if (argvars[1].v_type != VAR_UNKNOWN && argvars[2].v_type != VAR_UNKNOWN) {
    lnum_stop = (linenr_T)tv_get_number_chk(&argvars[2], NULL);
    if (lnum_stop < 0) {
      goto theend;
    }
    if (argvars[3].v_type != VAR_UNKNOWN) {
      time_limit = tv_get_number_chk(&argvars[3], NULL);
      if (time_limit < 0) {
        goto theend;
      }
      use_skip = rs_eval_expr_valid_arg(&argvars[4]) != 0;
    }
  }

  // Set the time limit, if there is one.
  proftime_T tm = profile_setlimit(time_limit);

  // This function does not accept SP_REPEAT and SP_RETCOUNT flags.
  // Check to make sure only those flags are set.
  // Also, Only the SP_NOMOVE or the SP_SETPCMARK flag can be set. Both
  // flags cannot be set. Check for that condition also.
  if (((flags & (SP_REPEAT | SP_RETCOUNT)) != 0)
      || ((flags & SP_NOMOVE) && (flags & SP_SETPCMARK))) {
    semsg(_(e_invarg2), tv_get_string(&argvars[1]));
    goto theend;
  }

  pos_T save_cursor;
  pos_T pos = save_cursor = curwin->w_cursor;
  pos_T firstpos = { 0 };
  searchit_arg_T sia = {
    .sa_stop_lnum = lnum_stop,
    .sa_tm = &tm,
  };

  const size_t patlen = strlen(pat);
  int subpatnum;

  // Repeat until {skip} returns false.
  while (true) {
    subpatnum = searchit(curwin, curbuf, &pos, NULL, dir, (char *)pat, patlen, 1,
                         options, RE_SEARCH, &sia);
    // finding the first match again means there is no match where {skip}
    // evaluates to zero.
    if (firstpos.lnum != 0 && equalpos(pos, firstpos)) {
      subpatnum = FAIL;
    }

    if (subpatnum == FAIL || !use_skip) {
      // didn't find it or no skip argument
      break;
    }
    if (firstpos.lnum == 0) {
      firstpos = pos;
    }

    // If the skip expression matches, ignore this match.
    {
      const pos_T save_pos = curwin->w_cursor;

      curwin->w_cursor = pos;
      bool err = false;
      const bool do_skip = eval_expr_to_bool(&argvars[4], &err);
      curwin->w_cursor = save_pos;
      if (err) {
        // Evaluating {skip} caused an error, break here.
        subpatnum = FAIL;
        break;
      }
      if (!do_skip) {
        break;
      }
    }

    // clear the start flag to avoid getting stuck here
    options &= ~SEARCH_START;
  }

  if (subpatnum != FAIL) {
    if (flags & SP_SUBPAT) {
      retval = subpatnum;
    } else {
      retval = pos.lnum;
    }
    if (flags & SP_SETPCMARK) {
      setpcmark();
    }
    curwin->w_cursor = pos;
    if (match_pos != NULL) {
      // Store the match cursor position
      match_pos->lnum = pos.lnum;
      match_pos->col = pos.col + 1;
    }
    // "/$" will put the cursor after the end of the line, may need to
    // correct that here
    check_cursor(curwin);
  }

  // If 'n' flag is used: restore cursor position.
  if (flags & SP_NOMOVE) {
    curwin->w_cursor = save_cursor;
  } else {
    curwin->w_set_curswant = true;
  }
theend:
  p_ws = save_p_ws;

  return retval;
}

/// "rpcnotify()" function
static void f_rpcnotify(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;

  if (rs_check_secure()) {
    return;
  }

  if (argvars[0].v_type != VAR_NUMBER || argvars[0].vval.v_number < 0) {
    semsg(_(e_invarg2), "Channel id must be a positive integer");
    return;
  }

  if (argvars[1].v_type != VAR_STRING) {
    semsg(_(e_invarg2), "Event type must be a string");
    return;
  }

  MAXSIZE_TEMP_ARRAY(args, MAX_FUNC_ARGS);
  Arena arena = ARENA_EMPTY;

  for (typval_T *tv = argvars + 2; tv->v_type != VAR_UNKNOWN; tv++) {
    ADD_C(args, vim_to_object(tv, &arena, true));
  }

  bool ok = rpc_send_event((uint64_t)argvars[0].vval.v_number,
                           tv_get_string(&argvars[1]), args);

  arena_mem_free(arena_finish(&arena));

  if (!ok) {
    semsg(_(e_invarg2), "Channel doesn't exist");
    return;
  }
  rettv->vval.v_number = 1;
}

/// "rpcrequest()" function
static void f_rpcrequest(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_NUMBER;
  rettv->vval.v_number = 0;
  const int l_provider_call_nesting = provider_call_nesting;

  if (rs_check_secure()) {
    return;
  }

  if (argvars[0].v_type != VAR_NUMBER || argvars[0].vval.v_number <= 0) {
    semsg(_(e_invarg2), "Channel id must be a positive integer");
    return;
  }

  if (argvars[1].v_type != VAR_STRING) {
    semsg(_(e_invarg2), "Method name must be a string");
    return;
  }

  MAXSIZE_TEMP_ARRAY(args, MAX_FUNC_ARGS);
  Arena arena = ARENA_EMPTY;

  for (typval_T *tv = argvars + 2; tv->v_type != VAR_UNKNOWN; tv++) {
    ADD_C(args, vim_to_object(tv, &arena, true));
  }

  sctx_T save_current_sctx;
  char *save_autocmd_fname, *save_autocmd_match;
  bool save_autocmd_fname_full;
  int save_autocmd_bufnr;
  funccal_entry_T funccal_entry;

  if (l_provider_call_nesting) {
    // If this is called from a provider function, restore the scope
    // information of the caller.
    save_current_sctx = current_sctx;
    save_autocmd_fname = autocmd_fname;
    save_autocmd_match = autocmd_match;
    save_autocmd_fname_full = autocmd_fname_full;
    save_autocmd_bufnr = autocmd_bufnr;
    save_funccal(&funccal_entry);

    current_sctx = provider_caller_scope.script_ctx;
    ga_grow(&exestack, 1);
    ((estack_T *)exestack.ga_data)[exestack.ga_len++] = provider_caller_scope.es_entry;
    autocmd_fname = provider_caller_scope.autocmd_fname;
    autocmd_match = provider_caller_scope.autocmd_match;
    autocmd_fname_full = provider_caller_scope.autocmd_fname_full;
    autocmd_bufnr = provider_caller_scope.autocmd_bufnr;
    set_current_funccal((funccall_T *)(provider_caller_scope.funccalp));
  }

  Error err = ERROR_INIT;

  uint64_t chan_id = (uint64_t)argvars[0].vval.v_number;
  const char *method = tv_get_string(&argvars[1]);

  ArenaMem res_mem = NULL;
  Object result = rpc_send_call(chan_id, method, args, &res_mem, &err);
  arena_mem_free(arena_finish(&arena));

  if (l_provider_call_nesting) {
    current_sctx = save_current_sctx;
    exestack.ga_len--;
    autocmd_fname = save_autocmd_fname;
    autocmd_match = save_autocmd_match;
    autocmd_fname_full = save_autocmd_fname_full;
    autocmd_bufnr = save_autocmd_bufnr;
    restore_funccal();
  }

  if (ERROR_SET(&err)) {
    const char *name = NULL;
    Channel *chan = find_channel(chan_id);
    if (chan) {
      name = get_client_info(chan, "name");
    }
    if (name) {
      semsg_multiline("rpc_error", "Invoking '%s' on channel %" PRIu64 " (%s):\n%s",
                      method, chan_id, name, err.msg);
    } else {
      semsg_multiline("rpc_error", "Invoking '%s' on channel %" PRIu64 ":\n%s",
                      method, chan_id, err.msg);
    }

    goto end;
  }

  object_to_vim(result, rettv, &err);

end:
  arena_mem_free(res_mem);
  api_clear_error(&err);
}


/// Used by searchpair() and searchpairpos()
static int searchpair_cmn(typval_T *argvars, pos_T *match_pos)
{
  bool save_p_ws = p_ws;
  int flags = 0;
  int retval = 0;  // default: FAIL
  linenr_T lnum_stop = 0;
  int64_t time_limit = 0;

  // Get the three pattern arguments: start, middle, end. Will result in an
  // error if not a valid argument.
  char nbuf1[NUMBUFLEN];
  char nbuf2[NUMBUFLEN];
  const char *spat = tv_get_string_chk(&argvars[0]);
  const char *mpat = tv_get_string_buf_chk(&argvars[1], nbuf1);
  const char *epat = tv_get_string_buf_chk(&argvars[2], nbuf2);
  if (spat == NULL || mpat == NULL || epat == NULL) {
    goto theend;  // Type error.
  }

  // Handle the optional fourth argument: flags.
  int dir = get_search_arg(&argvars[3], &flags);   // may set p_ws.
  if (dir == 0) {
    goto theend;
  }

  // Don't accept SP_END or SP_SUBPAT.
  // Only one of the SP_NOMOVE or SP_SETPCMARK flags can be set.
  if ((flags & (SP_END | SP_SUBPAT)) != 0
      || ((flags & SP_NOMOVE) && (flags & SP_SETPCMARK))) {
    semsg(_(e_invarg2), tv_get_string(&argvars[3]));
    goto theend;
  }

  // Using 'r' implies 'W', otherwise it doesn't work.
  if (flags & SP_REPEAT) {
    p_ws = false;
  }

  // Optional fifth argument: skip expression.
  const typval_T *skip;
  if (argvars[3].v_type == VAR_UNKNOWN
      || argvars[4].v_type == VAR_UNKNOWN) {
    skip = NULL;
  } else {
    // Type is checked later.
    skip = &argvars[4];

    if (argvars[5].v_type != VAR_UNKNOWN) {
      lnum_stop = (linenr_T)tv_get_number_chk(&argvars[5], NULL);
      if (lnum_stop < 0) {
        semsg(_(e_invarg2), tv_get_string(&argvars[5]));
        goto theend;
      }
      if (argvars[6].v_type != VAR_UNKNOWN) {
        time_limit = tv_get_number_chk(&argvars[6], NULL);
        if (time_limit < 0) {
          semsg(_(e_invarg2), tv_get_string(&argvars[6]));
          goto theend;
        }
      }
    }
  }

  retval = do_searchpair(spat, mpat, epat, dir, skip,
                         flags, match_pos, lnum_stop, time_limit);

theend:
  p_ws = save_p_ws;

  return retval;
}

/// "searchpair()" function
/// Search for a start/middle/end thing.
/// Used by searchpair(), see its documentation for the details.
///
/// @param spat  start pattern
/// @param mpat  middle pattern
/// @param epat  end pattern
/// @param dir  BACKWARD or FORWARD
/// @param skip  skip expression
/// @param flags  SP_SETPCMARK and other SP_ values
/// @param lnum_stop  stop at this line if not zero
/// @param time_limit  stop after this many msec
///
/// @returns  0 or -1 for no match,
int do_searchpair(const char *spat, const char *mpat, const char *epat, int dir,
                  const typval_T *skip, int flags, pos_T *match_pos, linenr_T lnum_stop,
                  int64_t time_limit)
  FUNC_ATTR_NONNULL_ARG(1, 2, 3)
{
  int retval = 0;
  int nest = 1;
  bool use_skip = false;
  int options = SEARCH_KEEP;

  // Make 'cpoptions' empty, the 'l' flag should not be used here.
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;

  // Set the time limit, if there is one.
  proftime_T tm = profile_setlimit(time_limit);

  // Make two search patterns: start/end (pat2, for in nested pairs) and
  // start/middle/end (pat3, for the top pair).
  const size_t spatlen = strlen(spat);
  const size_t epatlen = strlen(epat);
  const size_t pat2size = spatlen + epatlen + 17;
  char *pat2 = xmalloc(pat2size);
  const size_t pat3size = spatlen + strlen(mpat) + epatlen + 25;
  char *pat3 = xmalloc(pat3size);
  int pat2len = snprintf(pat2, pat2size, "\\m\\(%s\\m\\)\\|\\(%s\\m\\)", spat, epat);
  int pat3len;
  if (*mpat == NUL) {
    STRCPY(pat3, pat2);
    pat3len = pat2len;
  } else {
    pat3len = snprintf(pat3, pat3size,
                       "\\m\\(%s\\m\\)\\|\\(%s\\m\\)\\|\\(%s\\m\\)", spat, epat, mpat);
  }
  if (flags & SP_START) {
    options |= SEARCH_START;
  }

  if (skip != NULL) {
    use_skip = rs_eval_expr_valid_arg(skip) != 0;
  }

  pos_T save_cursor = curwin->w_cursor;
  pos_T pos = curwin->w_cursor;
  pos_T firstpos;
  clearpos(&firstpos);
  pos_T foundpos;
  clearpos(&foundpos);
  char *pat = pat3;
  assert(pat3len >= 0);
  size_t patlen = (size_t)pat3len;
  while (true) {
    searchit_arg_T sia = {
      .sa_stop_lnum = lnum_stop,
      .sa_tm = &tm,
    };

    int n = searchit(curwin, curbuf, &pos, NULL, dir, pat, patlen, 1,
                     options, RE_SEARCH, &sia);
    if (n == FAIL || (firstpos.lnum != 0 && equalpos(pos, firstpos))) {
      // didn't find it or found the first match again: FAIL
      break;
    }

    if (firstpos.lnum == 0) {
      firstpos = pos;
    }
    if (equalpos(pos, foundpos)) {
      // Found the same position again.  Can happen with a pattern that
      // has "\zs" at the end and searching backwards.  Advance one
      // character and try again.
      if (dir == BACKWARD) {
        decl(&pos);
      } else {
        incl(&pos);
      }
    }
    foundpos = pos;

    // clear the start flag to avoid getting stuck here
    options &= ~SEARCH_START;

    // If the skip pattern matches, ignore this match.
    if (use_skip) {
      pos_T save_pos = curwin->w_cursor;
      curwin->w_cursor = pos;
      bool err = false;
      const bool r = eval_expr_to_bool(skip, &err);
      curwin->w_cursor = save_pos;
      if (err) {
        // Evaluating {skip} caused an error, break here.
        curwin->w_cursor = save_cursor;
        retval = -1;
        break;
      }
      if (r) {
        continue;
      }
    }

    if ((dir == BACKWARD && n == 3) || (dir == FORWARD && n == 2)) {
      // Found end when searching backwards or start when searching
      // forward: nested pair.
      nest++;
      pat = pat2;               // nested, don't search for middle
    } else {
      // Found end when searching forward or start when searching
      // backward: end of (nested) pair; or found middle in outer pair.
      if (--nest == 1) {
        pat = pat3;             // outer level, search for middle
      }
    }

    if (nest == 0) {
      // Found the match: return matchcount or line number.
      if (flags & SP_RETCOUNT) {
        retval++;
      } else {
        retval = pos.lnum;
      }
      if (flags & SP_SETPCMARK) {
        setpcmark();
      }
      curwin->w_cursor = pos;
      if (!(flags & SP_REPEAT)) {
        break;
      }
      nest = 1;             // search for next unmatched
    }
  }

  if (match_pos != NULL) {
    // Store the match cursor position
    match_pos->lnum = curwin->w_cursor.lnum;
    match_pos->col = curwin->w_cursor.col + 1;
  }

  // If 'n' flag is used or search failed: restore cursor position.
  if ((flags & SP_NOMOVE) || retval == 0) {
    curwin->w_cursor = save_cursor;
  }

  xfree(pat2);
  xfree(pat3);
  if (p_cpo == empty_string_option) {
    p_cpo = save_cpo;
  } else {
    // Darn, evaluating the {skip} expression changed the value.
    // If it's still empty it was changed and restored, need to restore in
    // the complicated way.
    if (*p_cpo == NUL) {
      set_option_value_give_err(kOptCpoptions, CSTR_AS_OPTVAL(save_cpo), 0);
    }
    free_string_option(save_cpo);
  }

  return retval;
}

/// "searchpos()" function

/// "serverlist()" function
static void f_serverlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  size_t n;
  char **addrs = server_address_list(&n);

  Arena arena = ARENA_EMPTY;
  // Passed to vim._core.server.serverlist() to avoid duplicates
  Array addrs_arr = arena_array(&arena, n);

  // Copy addrs into a linked list.
  list_T *const l = tv_list_alloc_ret(rettv, (ptrdiff_t)n);
  for (size_t i = 0; i < n; i++) {
    tv_list_append_allocated_string(l, addrs[i]);
    ADD_C(addrs_arr, CSTR_AS_OBJ(addrs[i]));
  }

  if (!(argvars[0].v_type == VAR_DICT && tv_dict_get_bool(argvars[0].vval.v_dict, "peer", false))) {
    goto cleanup;
  }

  MAXSIZE_TEMP_ARRAY(args, 1);
  ADD_C(args, ARRAY_OBJ(addrs_arr));

  Error err = ERROR_INIT;
  Object rv = NLUA_EXEC_STATIC("return require('vim._core.server').serverlist(...)",
                               args, kRetObject,
                               &arena, &err);

  if (ERROR_SET(&err)) {
    ELOG("vim._core.serverlist failed: %s", err.msg);
    goto cleanup;
  }

  for (size_t i = 0; i < rv.data.array.size; i++) {
    char *curr_server = rv.data.array.items[i].data.string.data;
    tv_list_append_string(l, curr_server, -1);
  }

cleanup:
  xfree(addrs);
  arena_mem_free(arena_finish(&arena));
}


/// "serverstop()" function

/// Set the cursor or mark position.
/// If "charpos" is true, then use the column number as a character offset.
/// Otherwise use the column number as a byte offset.


/// "setfperm({fname}, {mode})" function

/// "setpos()" function
/// Translate a register type string to the yank type and block length
static int get_yank_type(char **const pp, MotionType *const yank_type, int *const block_len)
  FUNC_ATTR_NONNULL_ALL
{
  char *stropt = *pp;
  switch (*stropt) {
  case 'v':
  case 'c':  // character-wise selection
    *yank_type = kMTCharWise;
    break;
  case 'V':
  case 'l':  // line-wise selection
    *yank_type = kMTLineWise;
    break;
  case 'b':
  case Ctrl_V:  // block-wise selection
    *yank_type = kMTBlockWise;
    if (ascii_isdigit(stropt[1])) {
      stropt++;
      *block_len = getdigits_int(&stropt, false, 0) - 1;
      stropt--;
    }
    break;
  default:
    return FAIL;
  }
  *pp = stropt;
  return OK;
}

/// "setreg()" function
static void f_setreg(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  bool append = false;

  int block_len = -1;
  MotionType yank_type = kMTUnknown;

  rettv->vval.v_number = 1;  // FAIL is default.

  const char *const strregname = tv_get_string_chk(argvars);
  if (strregname == NULL) {
    return;  // Type error; errmsg already given.
  }
  char regname = *strregname;
  if (regname == 0 || regname == '@') {
    regname = '"';
  }

  const typval_T *regcontents = NULL;
  char pointreg = 0;
  if (argvars[1].v_type == VAR_DICT) {
    dict_T *const d = argvars[1].vval.v_dict;

    if (tv_dict_len(d) == 0) {
      // Empty dict, clear the register (like setreg(0, []))
      char *lstval[2] = { NULL, NULL };
      write_reg_contents_lst(regname, lstval, false, kMTUnknown, -1);
      return;
    }

    dictitem_T *const di = tv_dict_find(d, "regcontents", -1);
    if (di != NULL) {
      regcontents = &di->di_tv;
    }

    const char *stropt = tv_dict_get_string(d, "regtype", false);
    if (stropt != NULL) {
      const int ret = get_yank_type((char **)&stropt, &yank_type, &block_len);

      if (ret == FAIL || *(++stropt) != NUL) {
        semsg(_(e_invargval), "value");
        return;
      }
    }

    if (regname == '"') {
      stropt = tv_dict_get_string(d, "points_to", false);
      if (stropt != NULL) {
        pointreg = *stropt;
        regname = pointreg;
      }
    } else if (tv_dict_get_number(d, "isunnamed")) {
      pointreg = regname;
    }
  } else {
    regcontents = &argvars[1];
  }

  bool set_unnamed = false;
  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (yank_type != kMTUnknown) {
      semsg(_(e_toomanyarg), "setreg");
      return;
    }

    const char *stropt = tv_get_string_chk(&argvars[2]);
    if (stropt == NULL) {
      return;  // Type error.
    }
    for (; *stropt != NUL; stropt++) {
      switch (*stropt) {
      case 'a':
      case 'A':    // append
        append = true;
        break;
      case 'u':
      case '"':    // unnamed register
        set_unnamed = true;
        break;
      default:
        get_yank_type((char **)&stropt, &yank_type, &block_len);
      }
    }
  }

  if (regcontents != NULL && regcontents->v_type == VAR_LIST) {
    list_T *const ll = regcontents->vval.v_list;
    // If the list is NULL handle like an empty list.
    const int len = tv_list_len(ll);

    // First half: use for pointers to result lines; second half: use for
    // pointers to allocated copies.
    char **lstval = xmalloc(sizeof(char *) * (((size_t)len + 1) * 2));
    const char **curval = (const char **)lstval;
    char **allocval = lstval + len + 2;
    char **curallocval = allocval;

    TV_LIST_ITER_CONST(ll, li, {
      char buf[NUMBUFLEN];
      *curval = tv_get_string_buf_chk(TV_LIST_ITEM_TV(li), buf);
      if (*curval == NULL) {
        goto free_lstval;
      }
      if (*curval == buf) {
        // Need to make a copy,
        // next tv_get_string_buf_chk() will overwrite the string.
        *curallocval = xstrdup(*curval);
        *curval = *curallocval;
        curallocval++;
      }
      curval++;
    });
    *curval++ = NULL;

    write_reg_contents_lst(regname, lstval, append, yank_type, (colnr_T)block_len);

free_lstval:
    while (curallocval > allocval) {
      xfree(*--curallocval);
    }
    xfree(lstval);
  } else if (regcontents != NULL) {
    const char *const strval = tv_get_string_chk(regcontents);
    if (strval == NULL) {
      return;
    }
    write_reg_contents_ex(regname, strval, (ssize_t)strlen(strval),
                          append, yank_type, (colnr_T)block_len);
  }
  if (pointreg != 0) {
    get_yank_register(pointreg, YREG_YANK);
  }
  rettv->vval.v_number = 0;

  if (set_unnamed) {
    // Discard the result. We already handle the error case.
    op_reg_set_previous(regname);
  }
}

/// "settagstack()" function
static void f_settagstack(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  static const char *e_invact2 = N_("E962: Invalid action: '%s'");
  char action = 'r';

  rettv->vval.v_number = -1;

  // first argument: window number or id
  win_T *wp = find_win_by_nr_or_id(&argvars[0]);
  if (wp == NULL) {
    return;
  }

  // second argument: dict with items to set in the tag stack
  if (tv_check_for_dict_arg(argvars, 1) == FAIL) {
    return;
  }
  dict_T *d = argvars[1].vval.v_dict;
  if (d == NULL) {
    return;
  }

  // third argument: action - 'a' for append and 'r' for replace.
  // default is to replace the stack.
  if (argvars[2].v_type == VAR_UNKNOWN) {
    // action = 'r';
  } else if (tv_check_for_string_arg(argvars, 2) == FAIL) {
    return;
  } else {
    const char *actstr;
    actstr = tv_get_string_chk(&argvars[2]);
    if (actstr == NULL) {
      return;
    }
    if ((*actstr == 'r' || *actstr == 'a' || *actstr == 't')
        && actstr[1] == NUL) {
      action = *actstr;
    } else {
      semsg(_(e_invact2), actstr);
      return;
    }
  }

  if (rs_set_tagstack(wp, d, action) == OK) {
    rettv->vval.v_number = 0;
  }
}

/// "sha256({expr})" function

/// "shellescape({string})" function
/// shiftwidth() function

/// "sockconnect()" function
static void f_sockconnect(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (argvars[0].v_type != VAR_STRING || argvars[1].v_type != VAR_STRING) {
    emsg(_(e_invarg));
    return;
  }
  if (argvars[2].v_type != VAR_DICT && argvars[2].v_type != VAR_UNKNOWN) {
    // Wrong argument types
    semsg(_(e_invarg2), "expected dictionary");
    return;
  }

  const char *mode = tv_get_string(&argvars[0]);
  const char *address = tv_get_string(&argvars[1]);

  bool tcp;
  if (strcmp(mode, "tcp") == 0) {
    tcp = true;
  } else if (strcmp(mode, "pipe") == 0) {
    tcp = false;
  } else {
    semsg(_(e_invarg2), "invalid mode");
    return;
  }

  bool rpc = false;
  CallbackReader on_data = CALLBACK_READER_INIT;
  if (argvars[2].v_type == VAR_DICT) {
    dict_T *opts = argvars[2].vval.v_dict;
    rpc = tv_dict_get_number(opts, "rpc") != 0;

    if (!tv_dict_get_callback(opts, S_LEN("on_data"), &on_data.cb)) {
      return;
    }
    on_data.buffered = tv_dict_get_number(opts, "data_buffered");
    if (on_data.buffered && on_data.cb.type == kCallbackNone) {
      on_data.self = opts;
    }
  }

  const char *error = NULL;
  uint64_t id = channel_connect(tcp, address, rpc, on_data, 50, &error);

  if (error) {
    semsg(_("connection failed: %s"), error);
  }

  rettv->vval.v_number = (varnumber_T)id;
  rettv->v_type = VAR_NUMBER;
}

/// "stdioopen()" function
static void f_stdioopen(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  if (argvars[0].v_type != VAR_DICT) {
    emsg(_(e_invarg));
    return;
  }

  CallbackReader on_stdin = CALLBACK_READER_INIT;
  dict_T *opts = argvars[0].vval.v_dict;
  bool rpc = tv_dict_get_number(opts, "rpc") != 0;

  if (!tv_dict_get_callback(opts, S_LEN("on_stdin"), &on_stdin.cb)) {
    return;
  }
  if (!tv_dict_get_callback(opts, S_LEN("on_print"), &on_print)) {
    return;
  }

  on_stdin.buffered = tv_dict_get_number(opts, "stdin_buffered");
  if (on_stdin.buffered && on_stdin.cb.type == kCallbackNone) {
    on_stdin.self = opts;
  }

  const char *error;
  uint64_t id = channel_from_stdio(rpc, on_stdin, &error);
  if (!id) {
    semsg(e_stdiochan2, error);
  }

  rettv->vval.v_number = (varnumber_T)id;
  rettv->v_type = VAR_NUMBER;
}

/// "reltimefloat()" function

/// "soundfold({word})" function

/// "spellbadword()" function

static void f_split(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  colnr_T col = 0;
  bool keepempty = false;
  bool typeerr = false;

  // Make 'cpoptions' empty, the 'l' flag should not be used here.
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;

  const char *str = tv_get_string(&argvars[0]);
  const char *pat = NULL;
  char patbuf[NUMBUFLEN];
  if (argvars[1].v_type != VAR_UNKNOWN) {
    pat = tv_get_string_buf_chk(&argvars[1], patbuf);
    if (pat == NULL) {
      typeerr = true;
    }
    if (argvars[2].v_type != VAR_UNKNOWN) {
      keepempty = (bool)tv_get_bool_chk(&argvars[2], &typeerr);
    }
  }
  if (pat == NULL || *pat == NUL) {
    pat = "[\\x01- ]\\+";
  }

  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (typeerr) {
    goto theend;
  }

  regmatch_T regmatch = {
    .regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING),
    .startp = { NULL },
    .endp = { NULL },
    .rm_ic = false,
  };
  if (regmatch.regprog != NULL) {
    while (*str != NUL || keepempty) {
      bool match;
      if (*str == NUL) {
        match = false;  // Empty item at the end.
      } else {
        match = vim_regexec_nl(&regmatch, str, col);
      }
      const char *end;
      if (match) {
        end = regmatch.startp[0];
      } else {
        end = str + strlen(str);
      }
      if (keepempty || end > str || (tv_list_len(rettv->vval.v_list) > 0
                                     && *str != NUL
                                     && match
                                     && end < regmatch.endp[0])) {
        tv_list_append_string(rettv->vval.v_list, str, end - str);
      }
      if (!match) {
        break;
      }
      // Advance to just after the match.
      if (regmatch.endp[0] > str) {
        col = 0;
      } else {
        // Don't get stuck at the same match.
        col = utfc_ptr2len(regmatch.endp[0]);
      }
      str = regmatch.endp[0];
    }

    vim_regfree(regmatch.regprog);
  }

theend:
  p_cpo = save_cpo;
}


/// "str2float()" function

/// "strftime({format}[, {time}])" function


/// "submatch()" function

/// "swapfilelist()" function
/// "swapname(expr)" function
static void f_swapname(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rettv->v_type = VAR_STRING;
  buf_T *buf = tv_get_buf(&argvars[0], false);
  if (buf == NULL
      || buf->b_ml.ml_mfp == NULL
      || buf->b_ml.ml_mfp->mf_fname == NULL) {
    rettv->vval.v_string = NULL;
  } else {
    rettv->vval.v_string = xstrdup(buf->b_ml.ml_mfp->mf_fname);
  }
}


/// "tabpagebuflist()" function
static void f_tabpagebuflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  win_T *wp = NULL;

  if (argvars[0].v_type == VAR_UNKNOWN) {
    wp = firstwin;
  } else {
    tabpage_T *const tp = rs_find_tabpage((int)tv_get_number(&argvars[0]));
    if (tp != NULL) {
      wp = (tp == curtab) ? firstwin : tp->tp_firstwin;
    }
  }
  if (wp != NULL) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    while (wp != NULL) {
      tv_list_append_number(rettv->vval.v_list, wp->w_buffer->b_fnum);
      wp = wp->w_next;
    }
  }
}


/// "timer_info([timer])" function


/// "virtcol({expr}, [, {list} [, {winid}]])" function
static void f_virtcol(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  colnr_T vcol_start = 0;
  colnr_T vcol_end = 0;
  switchwin_T switchwin;
  bool winchanged = false;

  if (argvars[1].v_type != VAR_UNKNOWN && argvars[2].v_type != VAR_UNKNOWN) {
    // use the window specified in the third argument
    tabpage_T *tp;
    win_T *wp = win_id2wp_tp((int)tv_get_number(&argvars[2]), &tp);
    if (wp == NULL || tp == NULL) {
      goto theend;
    }

    if (switch_win_noblock(&switchwin, wp, tp, true) != OK) {
      goto theend;
    }

    check_cursor(curwin);
    winchanged = true;
  }

  int fnum = curbuf->b_fnum;
  pos_T *fp = var2fpos(&argvars[0], false, &fnum, false);
  if (fp != NULL && fp->lnum <= curbuf->b_ml.ml_line_count
      && fnum == curbuf->b_fnum) {
    // Limit the column to a valid value, getvvcol() doesn't check.
    if (fp->col < 0) {
      fp->col = 0;
    } else {
      const colnr_T len = ml_get_len(fp->lnum);
      if (fp->col > len) {
        fp->col = len;
      }
    }
    getvvcol(curwin, fp, &vcol_start, NULL, &vcol_end);
    vcol_start++;
    vcol_end++;
  }

theend:
  if (argvars[1].v_type != VAR_UNKNOWN && tv_get_bool(&argvars[1])) {
    tv_list_alloc_ret(rettv, 2);
    tv_list_append_number(rettv->vval.v_list, vcol_start);
    tv_list_append_number(rettv->vval.v_list, vcol_end);
  } else {
    rettv->vval.v_number = vcol_end;
  }

  if (winchanged) {
    restore_win_noblock(&switchwin, true);
  }
}


int nvim_curbuf_get_did_filetype(void) { return curbuf->b_did_filetype; }
int nvim_curbuf_get_u_seq_cur(void) { return (int)curbuf->b_u_seq_cur; }
int nvim_get_reg_recorded(void) { return reg_recorded; }
// nvim_eval_ui_current_col: inlined into Rust (misc.rs) — direct ui_current_col() call
// nvim_eval_ui_current_row: inlined into Rust (misc.rs) — direct ui_current_row() call
// nvim_eval_pum_visible: inlined into Rust (misc.rs) — direct pum_visible() call
// nvim_eval_os_get_pid: inlined into Rust (misc.rs) — direct os_get_pid() call
void nvim_eval_get_col(typval_T *argvars, typval_T *rettv, bool charcol)
{
  if (tv_check_for_string_or_list_arg(argvars, 0) == FAIL
      || tv_check_for_opt_number_arg(argvars, 1) == FAIL) {
    return;
  }

  switchwin_T switchwin;
  bool winchanged = false;

  if (argvars[1].v_type != VAR_UNKNOWN) {
    // use the window specified in the second argument
    tabpage_T *tp;
    win_T *wp = win_id2wp_tp((int)tv_get_number(&argvars[1]), &tp);
    if (wp == NULL || tp == NULL) {
      return;
    }

    if (switch_win_noblock(&switchwin, wp, tp, true) != OK) {
      return;
    }

    check_cursor(curwin);
    winchanged = true;
  }

  colnr_T col = 0;
  int fnum = curbuf->b_fnum;
  pos_T *fp = var2fpos(&argvars[0], false, &fnum, charcol);
  if (fp != NULL && fnum == curbuf->b_fnum) {
    if (fp->col == MAXCOL) {
      // '> can be MAXCOL, get the length of the line then
      if (fp->lnum <= curbuf->b_ml.ml_line_count) {
        col = ml_get_len(fp->lnum) + 1;
      } else {
        col = MAXCOL;
      }
    } else {
      col = fp->col + 1;
      // col(".") when the cursor is on the NUL at the end of the line
      // because of "coladd" can be seen as an extra column.
      if (virtual_active(curwin) && fp == &curwin->w_cursor) {
        char *p = get_cursor_pos_ptr();
        if (curwin->w_cursor.coladd >=
            (colnr_T)win_chartabsize(curwin, p,
                                     curwin->w_virtcol - curwin->w_cursor.coladd)) {
          int l;
          if (*p != NUL && p[(l = utfc_ptr2len(p))] == NUL) {
            col += l;
          }
        }
      }
    }
  }
  rettv->vval.v_number = col;

  if (winchanged) {
    restore_win_noblock(&switchwin, true);
  }
}
void nvim_eval_getpos_both(typval_T *argvars, typval_T *rettv, bool getcurpos, bool charcol)
{
  pos_T *fp = NULL;
  pos_T pos;
  win_T *wp = curwin;
  int fnum = -1;

  if (getcurpos) {
    if (argvars[0].v_type != VAR_UNKNOWN) {
      wp = find_win_by_nr_or_id(&argvars[0]);
      if (wp != NULL) {
        fp = &wp->w_cursor;
      }
    } else {
      fp = &curwin->w_cursor;
    }
    if (fp != NULL && charcol) {
      pos = *fp;
      pos.col = rs_buf_byteidx_to_charidx(wp->w_buffer, pos.lnum, pos.col);
      fp = &pos;
    }
  } else {
    fp = var2fpos(&argvars[0], true, &fnum, charcol);
  }

  list_T *const l = tv_list_alloc_ret(rettv, 4 + getcurpos);
  tv_list_append_number(l, (fnum != -1) ? (varnumber_T)fnum : (varnumber_T)0);
  tv_list_append_number(l, ((fp != NULL) ? (varnumber_T)fp->lnum : (varnumber_T)0));
  tv_list_append_number(l, ((fp != NULL)
                            ? (varnumber_T)(fp->col == MAXCOL ? MAXCOL : fp->col + 1)
                            : (varnumber_T)0));
  tv_list_append_number(l, (fp != NULL) ? (varnumber_T)fp->coladd : (varnumber_T)0);
  if (getcurpos) {
    const bool save_set_curswant = curwin->w_set_curswant;
    const colnr_T save_curswant = curwin->w_curswant;
    const colnr_T save_virtcol = curwin->w_virtcol;

    if (wp == curwin) {
      update_curswant();
    }
    tv_list_append_number(l, (wp == NULL) ? 0 : ((wp->w_curswant == MAXCOL)
                                                 ? (varnumber_T)MAXCOL
                                                 : (varnumber_T)wp->w_curswant + 1));

    // Do not change "curswant", as it is unexpected that a get
    // function has a side effect.
    if (wp == curwin && save_set_curswant) {
      curwin->w_set_curswant = save_set_curswant;
      curwin->w_curswant = save_curswant;
      curwin->w_virtcol = save_virtcol;
      curwin->w_valid &= ~VALID_VIRTCOL;
    }
  }
}
const char *nvim_eval_get_windows_version(void) { return windowsVersion; }


// Delegation wrappers for static helpers
void nvim_eval_find_some_match(typval_T *argvars, typval_T *rettv, int kind)
{
  const SomeMatchType type = (SomeMatchType)kind;

  char *str = NULL;
  int64_t len = 0;
  char *expr = NULL;
  regmatch_T regmatch;
  int64_t start = 0;
  int64_t nth = 1;
  colnr_T startcol = 0;
  bool match = false;
  list_T *l = NULL;
  int idx = 0;
  char *tofree = NULL;

  // Make 'cpoptions' empty, the 'l' flag should not be used here.
  char *save_cpo = p_cpo;
  p_cpo = empty_string_option;

  rettv->vval.v_number = -1;
  switch (type) {
  // matchlist(): return empty list when there are no matches.
  case kSomeMatchList:
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    break;
  // matchstrpos(): return ["", -1, -1, -1]
  case kSomeMatchStrPos:
    tv_list_alloc_ret(rettv, 4);
    tv_list_append_string(rettv->vval.v_list, "", 0);
    tv_list_append_number(rettv->vval.v_list, -1);
    tv_list_append_number(rettv->vval.v_list, -1);
    tv_list_append_number(rettv->vval.v_list, -1);
    break;
  case kSomeMatchStr:
    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = NULL;
    break;
  case kSomeMatch:
  case kSomeMatchEnd:
    // Do nothing: zero is default.
    break;
  }

  listitem_T *li = NULL;
  if (argvars[0].v_type == VAR_LIST) {
    if ((l = argvars[0].vval.v_list) == NULL) {
      goto theend;
    }
    li = tv_list_first(l);
  } else {
    expr = str = (char *)tv_get_string(&argvars[0]);
    len = (int64_t)strlen(str);
  }

  char patbuf[NUMBUFLEN];
  const char *const pat = tv_get_string_buf_chk(&argvars[1], patbuf);
  if (pat == NULL) {
    goto theend;
  }

  if (argvars[2].v_type != VAR_UNKNOWN) {
    bool error = false;

    start = tv_get_number_chk(&argvars[2], &error);
    if (error) {
      goto theend;
    }
    if (l != NULL) {
      idx = tv_list_uidx(l, (int)start);
      if (idx == -1) {
        goto theend;
      }
      li = tv_list_find(l, idx);
    } else {
      if (start < 0) {
        start = 0;
      }
      if (start > len) {
        goto theend;
      }
      // When "count" argument is there ignore matches before "start",
      // otherwise skip part of the string.  Differs when pattern is "^"
      // or "\<".
      if (argvars[3].v_type != VAR_UNKNOWN) {
        startcol = (colnr_T)start;
      } else {
        str += start;
        len -= start;
      }
    }

    if (argvars[3].v_type != VAR_UNKNOWN) {
      nth = tv_get_number_chk(&argvars[3], &error);
    }
    if (error) {
      goto theend;
    }
  }

  regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
  if (regmatch.regprog != NULL) {
    regmatch.rm_ic = p_ic;

    while (true) {
      if (l != NULL) {
        if (li == NULL) {
          match = false;
          break;
        }
        xfree(tofree);
        tofree = expr = str = encode_tv2echo(TV_LIST_ITEM_TV(li), NULL);
        if (str == NULL) {
          break;
        }
      }

      match = vim_regexec_nl(&regmatch, str, startcol);

      if (match && --nth <= 0) {
        break;
      }
      if (l == NULL && !match) {
        break;
      }

      // Advance to just after the match.
      if (l != NULL) {
        li = TV_LIST_ITEM_NEXT(l, li);
        idx++;
      } else {
        startcol = (colnr_T)(regmatch.startp[0]
                             + utfc_ptr2len(regmatch.startp[0]) - str);
        if (startcol > (colnr_T)len || str + startcol <= regmatch.startp[0]) {
          match = false;
          break;
        }
      }
    }

    if (match) {
      switch (type) {
      case kSomeMatchStrPos: {
        list_T *const ret_l = rettv->vval.v_list;
        listitem_T *li1 = tv_list_first(ret_l);
        listitem_T *li2 = TV_LIST_ITEM_NEXT(ret_l, li1);
        listitem_T *li3 = TV_LIST_ITEM_NEXT(ret_l, li2);
        listitem_T *li4 = TV_LIST_ITEM_NEXT(ret_l, li3);
        xfree(TV_LIST_ITEM_TV(li1)->vval.v_string);

        const size_t rd = (size_t)(regmatch.endp[0] - regmatch.startp[0]);
        TV_LIST_ITEM_TV(li1)->vval.v_string = xmemdupz(regmatch.startp[0], rd);
        TV_LIST_ITEM_TV(li3)->vval.v_number = (varnumber_T)(regmatch.startp[0] - expr);
        TV_LIST_ITEM_TV(li4)->vval.v_number = (varnumber_T)(regmatch.endp[0] - expr);
        if (l != NULL) {
          TV_LIST_ITEM_TV(li2)->vval.v_number = (varnumber_T)idx;
        }
        break;
      }
      case kSomeMatchList:
        // Return list with matched string and submatches.
        for (int i = 0; i < NSUBEXP; i++) {
          if (regmatch.endp[i] == NULL) {
            tv_list_append_string(rettv->vval.v_list, NULL, 0);
          } else {
            tv_list_append_string(rettv->vval.v_list, regmatch.startp[i],
                                  (regmatch.endp[i] - regmatch.startp[i]));
          }
        }
        break;
      case kSomeMatchStr:
        // Return matched string.
        if (l != NULL) {
          tv_copy(TV_LIST_ITEM_TV(li), rettv);
        } else {
          rettv->vval.v_string = xmemdupz(regmatch.startp[0],
                                          (size_t)(regmatch.endp[0] -
                                                   regmatch.startp[0]));
        }
        break;
      case kSomeMatch:
      case kSomeMatchEnd:
        if (l != NULL) {
          rettv->vval.v_number = idx;
        } else {
          if (type == kSomeMatch) {
            rettv->vval.v_number = (varnumber_T)(regmatch.startp[0] - str);
          } else {
            rettv->vval.v_number = (varnumber_T)(regmatch.endp[0] - str);
          }
          rettv->vval.v_number += (varnumber_T)(str - expr);
        }
        break;
      }
    }
    vim_regfree(regmatch.regprog);
  }

theend:
  if (type == kSomeMatchStrPos && l == NULL && rettv->vval.v_list != NULL) {
    // matchstrpos() without a list: drop the second item
    list_T *const ret_l = rettv->vval.v_list;
    tv_list_item_remove(ret_l, TV_LIST_ITEM_NEXT(ret_l, tv_list_first(ret_l)));
  }

  xfree(tofree);
  p_cpo = save_cpo;
}
void nvim_eval_max_min(typval_T *argvars, typval_T *rettv, bool domax)
{
  bool error = false;

  rettv->vval.v_number = 0;
  varnumber_T n = (domax ? VARNUMBER_MIN : VARNUMBER_MAX);
  if (argvars->v_type == VAR_LIST) {
    if (tv_list_len(argvars->vval.v_list) == 0) {
      return;
    }
    TV_LIST_ITER_CONST(argvars->vval.v_list, li, {
      const varnumber_T i = tv_get_number_chk(TV_LIST_ITEM_TV(li), &error);
      if (error) {
        return;  // type error; errmsg already given
      }
      if (domax ? i > n : i < n) {
        n = i;
      }
    });
  } else if (argvars->v_type == VAR_DICT) {
    if (tv_dict_len(argvars->vval.v_dict) == 0) {
      return;
    }
    TV_DICT_ITER(argvars->vval.v_dict, di, {
      const varnumber_T i = tv_get_number_chk(&di->di_tv, &error);
      if (error) {
        return;  // type error; errmsg already given
      }
      if (domax ? i > n : i < n) {
        n = i;
      }
    });
  } else {
    semsg(_(e_listdictarg), domax ? "max()" : "min()");
    return;
  }

  rettv->vval.v_number = n;
}
int nvim_eval_searchpair_cmn(typval_T *argvars) { return (int)searchpair_cmn(argvars, NULL); }
void nvim_eval_set_position(typval_T *argvars, typval_T *rettv, bool charpos)
{
  colnr_T curswant = -1;

  rettv->vval.v_number = -1;
  const char *const name = tv_get_string_chk(argvars);
  if (name == NULL) {
    return;
  }

  pos_T pos;
  int fnum;
  if (list2fpos(&argvars[1], &pos, &fnum, &curswant, charpos) != OK) {
    return;
  }

  if (pos.col != MAXCOL && --pos.col < 0) {
    pos.col = 0;
  }
  if (name[0] == '.' && name[1] == NUL) {
    // set cursor; "fnum" is ignored
    curwin->w_cursor = pos;
    if (curswant >= 0) {
      curwin->w_curswant = curswant - 1;
      curwin->w_set_curswant = false;
    }
    check_cursor(curwin);
    rettv->vval.v_number = 0;
  } else if (name[0] == '\'' && name[1] != NUL && name[2] == NUL) {
    // set mark
    if (setmark_pos((uint8_t)name[1], &pos, fnum, NULL) == OK) {
      rettv->vval.v_number = 0;
    }
  } else {
    emsg(_(e_invarg));
  }
}
void nvim_eval_set_cursorpos(typval_T *argvars, typval_T *rettv, bool charcol)
{
  linenr_T lnum;
  colnr_T col;
  colnr_T coladd = 0;
  bool set_curswant = true;

  rettv->vval.v_number = -1;
  if (argvars[0].v_type == VAR_LIST) {
    pos_T pos;
    colnr_T curswant = -1;

    if (list2fpos(argvars, &pos, NULL, &curswant, charcol) == FAIL) {
      emsg(_(e_invarg));
      return;
    }

    lnum = pos.lnum;
    col = pos.col;
    coladd = pos.coladd;
    if (curswant >= 0) {
      curwin->w_curswant = curswant - 1;
      set_curswant = false;
    }
  } else if ((argvars[0].v_type == VAR_NUMBER || argvars[0].v_type == VAR_STRING)
             && (argvars[1].v_type == VAR_NUMBER || argvars[1].v_type == VAR_STRING)) {
    lnum = tv_get_lnum(argvars);
    if (lnum < 0) {
      semsg(_(e_invarg2), tv_get_string(&argvars[0]));
    } else if (lnum == 0) {
      lnum = curwin->w_cursor.lnum;
    }
    col = (colnr_T)tv_get_number_chk(&argvars[1], NULL);
    if (charcol) {
      col = rs_buf_charidx_to_byteidx(curbuf, lnum, (int)col) + 1;
    }
    if (argvars[2].v_type != VAR_UNKNOWN) {
      coladd = (colnr_T)tv_get_number_chk(&argvars[2], NULL);
    }
  } else {
    emsg(_(e_invarg));
    return;
  }
  if (lnum < 0 || col < 0 || coladd < 0) {
    return;  // type error; errmsg already given
  }
  if (lnum > 0) {
    curwin->w_cursor.lnum = lnum;
  }
  if (col != MAXCOL && --col < 0) {
    col = 0;
  }
  curwin->w_cursor.col = col;
  curwin->w_cursor.coladd = coladd;

  // Make sure the cursor is in a valid position.
  check_cursor(curwin);
  // Correct cursor for multi-byte character.
  mb_adjust_cursor();

  curwin->w_set_curswant = set_curswant;
  rettv->vval.v_number = 0;
}

// Full-body wrappers for copy/deepcopy
// nvim_eval_copy: inlined into Rust (misc.rs) — direct var_item_copy() call
// nvim_eval_deepcopy: inlined into Rust (misc.rs) — direct var_item_copy() call

// nvim_eval_ctx_size: inlined into Rust (misc.rs) via nvim_eval_ctx_size_impl shim
// nvim_eval_ctxpop: inlined into Rust (misc.rs) via nvim_eval_ctxpop_impl shim
// nvim_eval_char2nr: inlined into Rust (misc.rs) — direct utf_ptr2char() call
// nvim_eval_nr2char: inlined into Rust (misc.rs) — utf_char2bytes delegation
// nvim_eval_str2float: inlined into Rust (misc.rs) — rs_string2float delegation
// nvim_eval_escape: inlined into Rust (misc.rs) — direct vim_strsave_escaped() call
// nvim_eval_shellescape: inlined into Rust (misc.rs) — direct vim_strsave_shellescape() call
// nvim_eval_fnameescape: inlined into Rust (misc.rs) — direct vim_strsave_fnameescape() call
// nvim_eval_hostname: inlined into Rust (misc.rs) — direct os_get_hostname() call
// nvim_eval_empty: inlined into Rust (misc.rs) — uses typval field accessor shims
// nvim_eval_len: inlined into Rust (misc.rs) — uses typval field accessor shims


// nvim_eval_execute: inlined into Rust (misc.rs)

void nvim_eval_flatten(typval_T *argvars, typval_T *rettv, bool make_copy)
{
  bool error = false;

  if (argvars[0].v_type != VAR_LIST) {
    semsg(_(e_listarg), "flatten()");
    return;
  }

  int maxdepth;
  if (argvars[1].v_type == VAR_UNKNOWN) {
    maxdepth = 999999;
  } else {
    maxdepth = (int)tv_get_number_chk(&argvars[1], &error);
    if (error) {
      return;
    }
    if (maxdepth < 0) {
      emsg(_("E900: maxdepth must be non-negative number"));
      return;
    }
  }

  list_T *list = argvars[0].vval.v_list;
  rettv->v_type = VAR_LIST;
  rettv->vval.v_list = list;
  if (list == NULL) {
    return;
  }

  if (make_copy) {
    list = tv_list_copy(NULL, list, false, rs_get_copyID());
    rettv->vval.v_list = list;
    if (list == NULL) {
      return;
    }
  } else {
    if (value_check_lock(tv_list_locked(list), N_("flatten() argument"), TV_TRANSLATE)) {
      return;
    }
    tv_list_ref(list);
  }

  tv_list_flatten(list, NULL, tv_list_len(list), maxdepth);
}

void nvim_eval_common_function(typval_T *argvars, typval_T *rettv, bool is_funcref)
{
  char *s;
  char *name;
  bool use_string = false;
  partial_T *arg_pt = NULL;
  char *trans_name = NULL;

  if (argvars[0].v_type == VAR_FUNC) {
    // function(MyFunc, [arg], dict)
    s = argvars[0].vval.v_string;
  } else if (argvars[0].v_type == VAR_PARTIAL
             && argvars[0].vval.v_partial != NULL) {
    // function(dict.MyFunc, [arg])
    arg_pt = argvars[0].vval.v_partial;
    s = rs_partial_name(arg_pt);
    // TODO(bfredl): do the entire nlua_is_table_from_lua dance
  } else {
    // function('MyFunc', [arg], dict)
    s = (char *)tv_get_string(&argvars[0]);
    use_string = true;
  }

  if ((use_string && vim_strchr(s, AUTOLOAD_CHAR) == NULL) || is_funcref) {
    name = s;
    trans_name = save_function_name(&name, false,
                                    TFN_INT | TFN_QUIET | TFN_NO_AUTOLOAD | TFN_NO_DEREF, NULL);
    if (*name != NUL) {
      s = NULL;
    }
  }
  if (s == NULL || *s == NUL || (use_string && ascii_isdigit(*s))
      || (is_funcref && trans_name == NULL)) {
    semsg(_(e_invarg2), (use_string ? tv_get_string(&argvars[0]) : s));
    // Don't check an autoload name for existence here.
  } else if (trans_name != NULL
             && (is_funcref
                 ? find_func(trans_name) == NULL
                 : !translated_function_exists(trans_name))) {
    semsg(_("E700: Unknown function: %s"), s);
  } else {
    int dict_idx = 0;
    int arg_idx = 0;
    list_T *list = NULL;
    if (strncmp(s, "s:", 2) == 0 || strncmp(s, "<SID>", 5) == 0) {
      // Expand s: and <SID> into <SNR>nr_, so that the function can
      // also be called from another script. Using trans_function_name()
      // would also work, but some plugins depend on the name being
      // printable text.
      name = get_scriptlocal_funcname(s);
    } else {
      name = xstrdup(s);
    }

    if (argvars[1].v_type != VAR_UNKNOWN) {
      if (argvars[2].v_type != VAR_UNKNOWN) {
        // function(name, [args], dict)
        arg_idx = 1;
        dict_idx = 2;
      } else if (argvars[1].v_type == VAR_DICT) {
        // function(name, dict)
        dict_idx = 1;
      } else {
        // function(name, [args])
        arg_idx = 1;
      }
      if (dict_idx > 0) {
        if (tv_check_for_dict_arg(argvars, dict_idx) == FAIL) {
          xfree(name);
          goto theend;
        }
        if (argvars[dict_idx].vval.v_dict == NULL) {
          dict_idx = 0;
        }
      }
      if (arg_idx > 0) {
        if (argvars[arg_idx].v_type != VAR_LIST) {
          emsg(_("E923: Second argument of function() must be "
                 "a list or a dict"));
          xfree(name);
          goto theend;
        }
        list = argvars[arg_idx].vval.v_list;
        if (tv_list_len(list) == 0) {
          arg_idx = 0;
        } else if (tv_list_len(list) > MAX_FUNC_ARGS) {
          emsg_funcname(e_toomanyarg, s);
          xfree(name);
          goto theend;
        }
      }
    }
    if (dict_idx > 0 || arg_idx > 0 || arg_pt != NULL || is_funcref) {
      partial_T *const pt = xcalloc(1, sizeof(*pt));

      // result is a VAR_PARTIAL
      if (arg_idx > 0 || (arg_pt != NULL && arg_pt->pt_argc > 0)) {
        const int arg_len = (arg_pt == NULL ? 0 : arg_pt->pt_argc);
        const int lv_len = tv_list_len(list);

        pt->pt_argc = arg_len + lv_len;
        pt->pt_argv = xmalloc(sizeof(pt->pt_argv[0]) * (size_t)pt->pt_argc);
        int i = 0;
        for (; i < arg_len; i++) {
          tv_copy(&arg_pt->pt_argv[i], &pt->pt_argv[i]);
        }
        if (lv_len > 0) {
          TV_LIST_ITER(list, li, {
            tv_copy(TV_LIST_ITEM_TV(li), &pt->pt_argv[i++]);
          });
        }
      }

      // For "function(dict.func, [], dict)" and "func" is a partial
      // use "dict". That is backwards compatible.
      if (dict_idx > 0) {
        // The dict is bound explicitly, pt_auto is false
        pt->pt_dict = argvars[dict_idx].vval.v_dict;
        (pt->pt_dict->dv_refcount)++;
      } else if (arg_pt != NULL) {
        // If the dict was bound automatically the result is also
        // bound automatically.
        pt->pt_dict = arg_pt->pt_dict;
        pt->pt_auto = arg_pt->pt_auto;
        if (pt->pt_dict != NULL) {
          (pt->pt_dict->dv_refcount)++;
        }
      }

      pt->pt_refcount = 1;
      if (arg_pt != NULL && arg_pt->pt_func != NULL) {
        pt->pt_func = arg_pt->pt_func;
        func_ptr_ref(pt->pt_func);
        xfree(name);
      } else if (is_funcref) {
        pt->pt_func = find_func(trans_name);
        func_ptr_ref(pt->pt_func);
        xfree(name);
      } else {
        pt->pt_name = name;
        func_ref(name);
      }

      rettv->v_type = VAR_PARTIAL;
      rettv->vval.v_partial = pt;
    } else {
      // result is a VAR_FUNC
      rettv->v_type = VAR_FUNC;
      rettv->vval.v_string = name;
      func_ref(name);
    }
  }
theend:
  xfree(trans_name);
}

// nvim_eval_hlID: inlined into Rust (misc.rs) — syn_name2id delegation
// nvim_eval_hlexists: inlined into Rust (misc.rs) — highlight_exists delegation

void nvim_eval_input(typval_T *argvars, typval_T *rettv, bool dialog)
{
  get_user_input(argvars, rettv, dialog, inputsecret_flag);
}

// nvim_eval_json_encode: inlined into Rust (misc.rs) — encode_tv2json delegation

void nvim_eval_libcall(typval_T *argvars, typval_T *rettv, bool retstr)
{
  int out_type = retstr ? VAR_STRING : VAR_NUMBER;

  rettv->v_type = (VarType)out_type;
  if (out_type != VAR_NUMBER) {
    rettv->vval.v_string = NULL;
  }

  if (rs_check_secure()) {
    return;
  }

  // The first two args (libname and funcname) must be strings
  if (argvars[0].v_type != VAR_STRING || argvars[1].v_type != VAR_STRING) {
    return;
  }

  const char *libname = argvars[0].vval.v_string;
  const char *funcname = argvars[1].vval.v_string;

  VarType in_type = argvars[2].v_type;

  // input variables
  char *str_in = (in_type == VAR_STRING) ? argvars[2].vval.v_string : NULL;
  int int_in = (int)argvars[2].vval.v_number;

  // output variables
  char **str_out = (out_type == VAR_STRING) ? &rettv->vval.v_string : NULL;
  int int_out = 0;

  bool success = os_libcall(libname, funcname,
                            str_in, int_in,
                            str_out, &int_out);

  if (!success) {
    semsg(_(e_libcall), funcname);
    return;
  }

  if (out_type == VAR_NUMBER) {
    rettv->vval.v_number = (varnumber_T)int_out;
  }
}

void nvim_eval_script_host_eval(const char *name, typval_T *argvars, typval_T *rettv)
{
  script_host_eval(name, argvars, rettv);
}

void nvim_eval_search(typval_T *argvars, typval_T *rettv)
{
  int flags = 0;
  rettv->vval.v_number = search_cmn(argvars, NULL, &flags);
}

void nvim_eval_searchpairpos(typval_T *argvars, typval_T *rettv)
{
  pos_T match_pos;
  int lnum = 0;
  int col = 0;

  tv_list_alloc_ret(rettv, 2);

  if (searchpair_cmn(argvars, &match_pos) > 0) {
    lnum = match_pos.lnum;
    col = match_pos.col;
  }

  tv_list_append_number(rettv->vval.v_list, (varnumber_T)lnum);
  tv_list_append_number(rettv->vval.v_list, (varnumber_T)col);
}

// nvim_eval_swapfilelist: inlined into Rust (misc.rs) — rs_recover_names delegation
// nvim_eval_swapinfo: inlined into Rust (misc.rs) — swapfile_dict delegation


void nvim_eval_api_info(typval_T *argvars, typval_T *rettv) { object_to_vim(api_metadata(), rettv, NULL); }

// nvim_eval_byte2line: inlined into Rust (simple.rs) — rs_ml_find_line_or_offset delegation
// nvim_eval_line2byte: inlined into Rust (simple.rs) — rs_ml_find_line_or_offset delegation

// nvim_eval_gettext: inlined into Rust (simple.rs) — gettext() delegation

// nvim_eval_garbagecollect: inlined into Rust (simple.rs)
// nvim_eval_debugbreak: inlined into Rust (simple.rs)
// nvim_eval_getenv: inlined into Rust (simple.rs)
// nvim_eval_setenv: inlined into Rust (simple.rs)

// nvim_eval_pum_getpos: inlined into Rust (simple.rs) — pum_set_event_info delegation
// nvim_eval_wordcount: inlined into Rust (simple.rs) — cursor_pos_info delegation

// nvim_eval_soundfold: inlined into Rust (simple.rs)
// nvim_eval_wildmenumode: inlined into Rust (simple.rs)
// nvim_eval_timer_stopall: inlined into Rust (simple.rs)
// nvim_eval_synIDtrans: inlined into Rust (simple.rs)

// nvim_eval_keytrans: inlined into Rust (simple.rs) — vim_strsave_escape_ks + str2special_save

// nvim_eval_luaeval: inlined into Rust (simple.rs) — nlua_typval_eval delegation

// nvim_eval_shiftwidth: inlined into Rust (simple.rs)
// nvim_eval_mode: inlined into Rust (simple.rs)
// nvim_eval_visualmode: still delegates to nvim_eval_visualmode (struct access needed)
// nvim_eval_nextnonblank: inlined into Rust (simple.rs)
// nvim_eval_prevnonblank: inlined into Rust (simple.rs)

// nvim_eval_menu_get: inlined into Rust (simple.rs) — menu_get delegation


static void screenchar_adjust_inner(ScreenGrid **grid, int *row, int *col)
{
  msg_scroll_flush();
  *grid = ui_comp_get_grid_at_coord(*row, *col);
  *row -= (*grid)->comp_row;
  *col -= (*grid)->comp_col;
}

void nvim_eval_screenattr(typval_T *argvars, typval_T *rettv)
{
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  int c;
  if (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols) {
    c = -1;
  } else {
    c = grid->attrs[grid->line_offset[row] + (size_t)col];
  }
  rettv->vval.v_number = c;
}

void nvim_eval_screenchar(typval_T *argvars, typval_T *rettv)
{
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  rettv->vval.v_number = (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols)
    ? -1 : schar_get_first_codepoint(grid_getchar(grid, row, col, NULL));
}

void nvim_eval_screenchars(typval_T *argvars, typval_T *rettv)
{
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  tv_list_alloc_ret(rettv, kListLenMayKnow);
  if (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols) {
    return;
  }
  char buf[MAX_SCHAR_SIZE + 1];
  schar_get(buf, grid_getchar(grid, row, col, NULL));
  size_t i = 0;
  do {
    int c = utf_ptr2char(buf + i);
    tv_list_append_number(rettv->vval.v_list, c);
    i += (size_t)utf_ptr2len(buf + i);
  } while (buf[i] != NUL);
}

void nvim_eval_screenstring(typval_T *argvars, typval_T *rettv)
{
  rettv->vval.v_string = NULL;
  rettv->v_type = VAR_STRING;
  int row = (int)tv_get_number_chk(&argvars[0], NULL) - 1;
  int col = (int)tv_get_number_chk(&argvars[1], NULL) - 1;
  ScreenGrid *grid;
  screenchar_adjust_inner(&grid, &row, &col);
  if (row < 0 || row >= grid->rows || col < 0 || col >= grid->cols) {
    return;
  }
  char buf[MAX_SCHAR_SIZE + 1];
  schar_get(buf, grid_getchar(grid, row, col, NULL));
  rettv->vval.v_string = xstrdup(buf);
}


// nvim_eval_environ: inlined into Rust (system.rs) — os_get_fullenv_size + os_copy_fullenv delegation

static void get_xdg_var_list_inner(const XDGVarType xdg, typval_T *rettv)
  FUNC_ATTR_NONNULL_ALL
{
  list_T *const list = tv_list_alloc(kListLenShouldKnow);
  rettv->v_type = VAR_LIST;
  rettv->vval.v_list = list;
  tv_list_ref(list);
  char *const dirs = stdpaths_get_xdg_var(xdg);
  if (dirs == NULL) {
    return;
  }
  const void *iter = NULL;
  const char *appname = get_appname(false);
  do {
    size_t dir_len;
    const char *dir;
    iter = vim_env_iter(ENV_SEPCHAR, dirs, iter, &dir, &dir_len);
    if (dir != NULL && dir_len > 0) {
      char *dir_with_nvim = xmemdupz(dir, dir_len);
      dir_with_nvim = concat_fnames_realloc(dir_with_nvim, appname, true);
      tv_list_append_allocated_string(list, dir_with_nvim);
    }
  } while (iter != NULL);
  xfree(dirs);
}

/// Public wrapper for inlining stdpath() list cases into Rust.
void nvim_eval_xdg_var_list(int xdg, typval_T *rettv)
{
  get_xdg_var_list_inner((XDGVarType)xdg, rettv);
}

void nvim_eval_ctxget(typval_T *argvars, typval_T *rettv)
{
  size_t index = 0;
  if (argvars[0].v_type == VAR_NUMBER) {
    index = (size_t)argvars[0].vval.v_number;
  } else if (argvars[0].v_type != VAR_UNKNOWN) {
    semsg(_(e_invarg2), "expected nothing or a Number as an argument");
    return;
  }

  Context *ctx = ctx_get(index);
  if (ctx == NULL) {
    semsg(_(e_invargNval), "index", "out of bounds");
    return;
  }

  Arena arena = ARENA_EMPTY;
  Dict ctx_dict = ctx_to_dict(ctx, &arena);
  Error err = ERROR_INIT;
  object_to_vim(DICT_OBJ(ctx_dict), rettv, &err);
  arena_mem_free(arena_finish(&arena));
  api_clear_error(&err);
}

void nvim_eval_ctxpush(typval_T *argvars, typval_T *rettv)
{
  int types = kCtxAll;
  if (argvars[0].v_type == VAR_LIST) {
    types = 0;
    TV_LIST_ITER(argvars[0].vval.v_list, li, {
      typval_T *tv_li = TV_LIST_ITEM_TV(li);
      if (tv_li->v_type == VAR_STRING) {
        if (strequal(tv_li->vval.v_string, "regs")) {
          types |= kCtxRegs;
        } else if (strequal(tv_li->vval.v_string, "jumps")) {
          types |= kCtxJumps;
        } else if (strequal(tv_li->vval.v_string, "bufs")) {
          types |= kCtxBufs;
        } else if (strequal(tv_li->vval.v_string, "gvars")) {
          types |= kCtxGVars;
        } else if (strequal(tv_li->vval.v_string, "sfuncs")) {
          types |= kCtxSFuncs;
        } else if (strequal(tv_li->vval.v_string, "funcs")) {
          types |= kCtxFuncs;
        }
      }
    });
  } else if (argvars[0].v_type != VAR_UNKNOWN) {
    semsg(_(e_invarg2), "expected nothing or a List as an argument");
    return;
  }
  ctx_save(NULL, types);
}

void nvim_eval_ctxset(typval_T *argvars, typval_T *rettv)
{
  if (argvars[0].v_type != VAR_DICT) {
    semsg(_(e_invarg2), "expected dictionary as first argument");
    return;
  }

  size_t index = 0;
  if (argvars[1].v_type == VAR_NUMBER) {
    index = (size_t)argvars[1].vval.v_number;
  } else if (argvars[1].v_type != VAR_UNKNOWN) {
    semsg(_(e_invarg2), "expected nothing or a Number as second argument");
    return;
  }

  Context *ctx = ctx_get(index);
  if (ctx == NULL) {
    semsg(_(e_invargNval), "index", "out of bounds");
    return;
  }

  const int save_did_emsg = did_emsg;
  did_emsg = false;

  Arena arena = ARENA_EMPTY;
  Dict dict = vim_to_object(&argvars[0], &arena, true).data.dict;
  Context tmp = CONTEXT_INIT;
  Error err = ERROR_INIT;
  ctx_from_dict(dict, &tmp, &err);

  if (ERROR_SET(&err)) {
    semsg("%s", err.msg);
    ctx_free(&tmp);
  } else {
    ctx_free(ctx);
    *ctx = tmp;
  }

  arena_mem_free(arena_finish(&arena));
  api_clear_error(&err);
  did_emsg = save_did_emsg;
}

// nvim_eval_getcharsearch: inlined into Rust (misc.rs) — last_csearch/forward/until delegation
// nvim_eval_setcharsearch: inlined into Rust (misc.rs) — set_last_csearch/direction/until delegation

/// Common between getreg(), getreginfo() and getregtype(): get the register
/// name from the first argument.
/// Returns zero on error.
static int nvim_eval_getreg_get_regname(typval_T *argvars)
{
  const char *strregname;

  if (argvars[0].v_type != VAR_UNKNOWN) {
    strregname = tv_get_string_chk(&argvars[0]);
    if (strregname == NULL) {  // type error; errmsg already given
      return 0;
    }
  } else {
    // Default to v:register
    strregname = get_vim_var_str(VV_REG);
  }

  return *strregname == 0 ? '"' : (uint8_t)(*strregname);
}

// nvim_eval_getreg: inlined into Rust (misc.rs) — get_reg_contents delegation
// nvim_eval_getregtype: inlined into Rust (misc.rs) — get_reg_type/format_reg_type delegation

void nvim_eval_getreginfo(typval_T *argvars, typval_T *rettv)
{
  int regname = nvim_eval_getreg_get_regname(argvars);
  if (regname == 0) {
    return;
  }

  if (regname == '@') {
    regname = '"';
  }

  tv_dict_alloc_ret(rettv);
  dict_T *const dict = rettv->vval.v_dict;

  list_T *const list = get_reg_contents(regname, kGRegExprSrc | kGRegList);
  if (list == NULL) {
    return;
  }
  tv_dict_add_list(dict, S_LEN("regcontents"), list);

  char buf[NUMBUFLEN + 2];
  buf[0] = NUL;
  buf[1] = NUL;
  colnr_T reglen = 0;
  switch (get_reg_type(regname, &reglen)) {
  case kMTLineWise:
    buf[0] = 'V';
    break;
  case kMTCharWise:
    buf[0] = 'v';
    break;
  case kMTBlockWise:
    vim_snprintf(buf, sizeof(buf), "%c%d", Ctrl_V, reglen + 1);
    break;
  case kMTUnknown:
    abort();
  }
  tv_dict_add_str(dict, S_LEN("regtype"), buf);

  buf[0] = (char)get_register_name(get_unname_register());
  buf[1] = NUL;
  if (regname == '"') {
    tv_dict_add_str(dict, S_LEN("points_to"), buf);
  } else {
    tv_dict_add_bool(dict, S_LEN("isunnamed"),
                     regname == buf[0] ? kBoolVarTrue : kBoolVarFalse);
  }
}

static void nvim_eval_may_add_state_char(garray_T *gap, const char *include, uint8_t c)
{
  if (include == NULL || vim_strchr(include, c) != NULL) {
    ga_append(gap, c);
  }
}

void nvim_eval_state(typval_T *argvars, typval_T *rettv)
{
  garray_T ga;
  ga_init(&ga, 1, 20);
  const char *include = NULL;

  if (argvars[0].v_type != VAR_UNKNOWN) {
    include = tv_get_string(&argvars[0]);
  }

  if (!(stuff_empty() && typebuf.tb_len == 0 && !using_script())) {
    nvim_eval_may_add_state_char(&ga, include, 'm');
  }
  if (rs_op_pending()) {
    nvim_eval_may_add_state_char(&ga, include, 'o');
  }
  if (autocmd_busy) {
    nvim_eval_may_add_state_char(&ga, include, 'x');
  }
  if (rs_ins_compl_active()) {
    nvim_eval_may_add_state_char(&ga, include, 'a');
  }
  if (!get_was_safe_state()) {
    nvim_eval_may_add_state_char(&ga, include, 'S');
  }
  for (int i = 0; i < rs_get_callback_depth() && i < 3; i++) {
    nvim_eval_may_add_state_char(&ga, include, 'c');
  }
  if (msg_scrolled > 0) {
    nvim_eval_may_add_state_char(&ga, include, 's');
  }

  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = ga.ga_data;
}

// nvim_eval_searchdecl: inlined into Rust (misc.rs) — find_decl delegation

void nvim_eval_searchpos(typval_T *argvars, typval_T *rettv)
{
  pos_T match_pos;
  int flags = 0;

  const int n = search_cmn(argvars, &match_pos, &flags);

  tv_list_alloc_ret(rettv, 2 + (!!(flags & SP_SUBPAT)));

  const int lnum = (n > 0 ? match_pos.lnum : 0);
  const int col = (n > 0 ? match_pos.col : 0);

  tv_list_append_number(rettv->vval.v_list, (varnumber_T)lnum);
  tv_list_append_number(rettv->vval.v_list, (varnumber_T)col);
  if (flags & SP_SUBPAT) {
    tv_list_append_number(rettv->vval.v_list, (varnumber_T)n);
  }
}


void nvim_eval_timer_info(typval_T *argvars, typval_T *rettv)
{
  tv_list_alloc_ret(rettv, kListLenUnknown);

  if (tv_check_for_opt_number_arg(argvars, 0) == FAIL) {
    return;
  }

  if (argvars[0].v_type != VAR_UNKNOWN) {
    timer_T *timer = find_timer_by_nr(tv_get_number(&argvars[0]));
    if (timer != NULL && (!timer->stopped || timer->refcount > 1)) {
      add_timer_info(rettv, timer);
    }
  } else {
    add_timer_info_all(rettv);
  }
}

void nvim_eval_timer_pause(typval_T *argvars, typval_T *rettv)
{
  if (argvars[0].v_type != VAR_NUMBER) {
    emsg(_(e_number_exp));
    return;
  }

  int paused = (bool)tv_get_number(&argvars[1]);
  timer_T *timer = find_timer_by_nr(tv_get_number(&argvars[0]));
  if (timer != NULL) {
    if (!timer->paused && paused) {
      time_watcher_stop(&timer->tw);
    } else if (timer->paused && !paused) {
      time_watcher_start(&timer->tw, timer_due_cb, (uint64_t)timer->timeout,
                         (uint64_t)timer->timeout);
    }
    timer->paused = paused;
  }
}

void nvim_eval_timer_start(typval_T *argvars, typval_T *rettv)
{
  int repeat = 1;

  rettv->vval.v_number = -1;
  if (rs_check_secure()) {
    return;
  }

  if (argvars[2].v_type != VAR_UNKNOWN) {
    if (tv_check_for_nonnull_dict_arg(argvars, 2) == FAIL) {
      return;
    }
    dict_T *dict = argvars[2].vval.v_dict;
    dictitem_T *const di = tv_dict_find(dict, S_LEN("repeat"));
    if (di != NULL) {
      repeat = (int)tv_get_number(&di->di_tv);
      if (repeat == 0) {
        repeat = 1;
      }
    }
  }

  Callback callback;
  if (!rs_callback_from_typval(&callback, &argvars[1])) {
    return;
  }
  rettv->vval.v_number = (varnumber_T)timer_start(tv_get_number(&argvars[0]), repeat, &callback);
}

void nvim_eval_timer_stop(typval_T *argvars, typval_T *rettv)
{
  if (tv_check_for_number_arg(argvars, 0) == FAIL) {
    return;
  }

  timer_T *timer = find_timer_by_nr(tv_get_number(&argvars[0]));
  if (timer == NULL) {
    return;
  }

  timer_stop(timer);
}


void nvim_eval_spellbadword(typval_T *argvars, typval_T *rettv)
{
  const int wo_spell_save = curwin->w_p_spell;

  if (!curwin->w_p_spell) {
    parse_spelllang(curwin);
    curwin->w_p_spell = true;
  }

  if (*curwin->w_s->b_p_spl == NUL) {
    emsg(_(e_no_spell));
    curwin->w_p_spell = wo_spell_save;
    return;
  }

  const char *word = "";
  hlf_T attr = HLF_COUNT;
  size_t len = 0;
  if (argvars[0].v_type == VAR_UNKNOWN) {
    // Find the start and length of the badly spelled word.
    len = spell_move_to(curwin, FORWARD, SMT_ALL, true, &attr);
    if (len != 0) {
      word = get_cursor_pos_ptr();
      curwin->w_set_curswant = true;
    }
  } else if (*curbuf->b_s.b_p_spl != NUL) {
    const char *str = tv_get_string_chk(&argvars[0]);
    int capcol = -1;

    if (str != NULL) {
      // Check the argument for spelling.
      while (*str != NUL) {
        len = spell_check(curwin, (char *)str, &attr, &capcol, false);
        if (attr != HLF_COUNT) {
          word = str;
          break;
        }
        str += len;
        capcol -= (int)len;
        len = 0;
      }
    }
  }
  curwin->w_p_spell = wo_spell_save;

  assert(len <= INT_MAX);
  tv_list_alloc_ret(rettv, 2);
  tv_list_append_string(rettv->vval.v_list, word, (ssize_t)len);
  tv_list_append_string(rettv->vval.v_list,
                        (attr == HLF_SPB
                         ? "bad" : (attr == HLF_SPR
                                    ? "rare" : (attr == HLF_SPL
                                                ? "local" : (attr == HLF_SPC
                                                             ? "caps" : NULL)))), -1);
}

void nvim_eval_spellsuggest(typval_T *argvars, typval_T *rettv)
{
  garray_T ga = GA_EMPTY_INIT_VALUE;
  const int wo_spell_save = curwin->w_p_spell;

  if (!curwin->w_p_spell) {
    parse_spelllang(curwin);
    curwin->w_p_spell = true;
  }

  if (*curwin->w_s->b_p_spl == NUL) {
    emsg(_(e_no_spell));
    curwin->w_p_spell = wo_spell_save;
    return;
  }

  int maxcount;
  bool need_capital = false;
  const char *const str = tv_get_string(&argvars[0]);
  if (argvars[1].v_type != VAR_UNKNOWN) {
    bool typeerr = false;
    maxcount = (int)tv_get_number_chk(&argvars[1], &typeerr);
    if (maxcount <= 0) {
      goto nvim_eval_spellsuggest_return;
    }
    if (argvars[2].v_type != VAR_UNKNOWN) {
      need_capital = tv_get_number_chk(&argvars[2], &typeerr);
      if (typeerr) {
        goto nvim_eval_spellsuggest_return;
      }
    }
  } else {
    maxcount = 25;
  }

  spell_suggest_list(&ga, (char *)str, maxcount, need_capital, false);

nvim_eval_spellsuggest_return:
  tv_list_alloc_ret(rettv, (ptrdiff_t)ga.ga_len);
  for (int i = 0; i < ga.ga_len; i++) {
    char *const p = ((char **)ga.ga_data)[i];
    tv_list_append_allocated_string(rettv->vval.v_list, p);
  }
  ga_clear(&ga);
  curwin->w_p_spell = wo_spell_save;
}


void nvim_eval_synID(typval_T *argvars, typval_T *rettv)
{
  // -1 on type error (both)
  const linenr_T lnum = tv_get_lnum(argvars);
  const colnr_T col = (colnr_T)tv_get_number(&argvars[1]) - 1;

  bool transerr = false;
  const int trans = (int)tv_get_number_chk(&argvars[2], &transerr);

  int id = 0;
  if (!transerr && lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count
      && col >= 0 && col < ml_get_len(lnum)) {
    id = syn_get_id(curwin, lnum, col, trans, NULL, false);
  }

  rettv->vval.v_number = id;
}


void nvim_eval_synconcealed(typval_T *argvars, typval_T *rettv)
{
  int syntax_flags = 0;
  int matchid = 0;
  char str[NUMBUFLEN];

  tv_list_set_ret(rettv, NULL);

  // -1 on type error (both)
  const linenr_T lnum = tv_get_lnum(argvars);
  const colnr_T col = (colnr_T)tv_get_number(&argvars[1]) - 1;

  CLEAR_FIELD(str);

  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count
      && col >= 0 && col <= ml_get_len(lnum) && curwin->w_p_cole > 0) {
    syn_get_id(curwin, lnum, col, false, NULL, false);
    syntax_flags = get_syntax_info(&matchid);

    // get the conceal character
    if ((syntax_flags & HL_CONCEAL) && curwin->w_p_cole < 3) {
      schar_T cchar = schar_from_char(syn_get_sub_char());
      if (cchar == NUL && curwin->w_p_cole == 1) {
        cchar = (curwin->w_p_lcs_chars.conceal == NUL)
                ? schar_from_ascii(' ') : curwin->w_p_lcs_chars.conceal;
      }
      if (cchar != NUL) {
        schar_get(str, cchar);
      }
    }
  }

  tv_list_alloc_ret(rettv, 3);
  tv_list_append_number(rettv->vval.v_list, (syntax_flags & HL_CONCEAL) != 0);
  // -1 to auto-determine strlen
  tv_list_append_string(rettv->vval.v_list, str, -1);
  tv_list_append_number(rettv->vval.v_list, matchid);
}

void nvim_eval_synstack(typval_T *argvars, typval_T *rettv)
{
  tv_list_set_ret(rettv, NULL);

  // -1 on type error (both)
  const linenr_T lnum = tv_get_lnum(argvars);
  const colnr_T col = (colnr_T)tv_get_number(&argvars[1]) - 1;

  if (lnum >= 1 && lnum <= curbuf->b_ml.ml_line_count
      && col >= 0 && col <= ml_get_len(lnum)) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    syn_get_id(curwin, lnum, col, false, NULL, true);

    int id;
    int i = 0;
    while ((id = syn_get_stack_item(i++)) >= 0) {
      tv_list_append_number(rettv->vval.v_list, id);
    }
  }
}


void nvim_eval_index(typval_T *argvars, typval_T *rettv)
{
  int idx = 0;
  bool ic = false;

  rettv->vval.v_number = -1;
  if (argvars[0].v_type == VAR_BLOB) {
    bool error = false;
    int start = 0;

    if (argvars[2].v_type != VAR_UNKNOWN) {
      start = (int)tv_get_number_chk(&argvars[2], &error);
      if (error) {
        return;
      }
    }
    blob_T *const b = argvars[0].vval.v_blob;
    if (b == NULL) {
      return;
    }
    if (start < 0) {
      start = tv_blob_len(b) + start;
      if (start < 0) {
        start = 0;
      }
    }
    for (idx = start; idx < tv_blob_len(b); idx++) {
      typval_T tv;
      tv.v_type = VAR_NUMBER;
      tv.vval.v_number = tv_blob_get(b, idx);
      if (tv_equal(&tv, &argvars[1], ic)) {
        rettv->vval.v_number = idx;
        return;
      }
    }
    return;
  } else if (argvars[0].v_type != VAR_LIST) {
    emsg(_(e_listblobreq));
    return;
  }

  list_T *const l = argvars[0].vval.v_list;
  if (l == NULL) {
    return;
  }

  listitem_T *item = tv_list_first(l);
  if (argvars[2].v_type != VAR_UNKNOWN) {
    bool error = false;

    // Start at specified item.
    idx = tv_list_uidx(l, (int)tv_get_number_chk(&argvars[2], &error));
    if (error || idx == -1) {
      item = NULL;
    } else {
      item = tv_list_find(l, idx);
      assert(item != NULL);
    }
    if (argvars[3].v_type != VAR_UNKNOWN) {
      ic = !!tv_get_number_chk(&argvars[3], &error);
      if (error) {
        item = NULL;
      }
    }
  }

  for (; item != NULL; item = TV_LIST_ITEM_NEXT(l, item), idx++) {
    if (tv_equal(TV_LIST_ITEM_TV(item), &argvars[1], ic)) {
      rettv->vval.v_number = idx;
      break;
    }
  }
}

static varnumber_T nvim_eval_indexof_eval_expr(typval_T *expr)
{
  typval_T argv[3];
  argv[0] = *get_vim_var_tv(VV_KEY);
  argv[1] = *get_vim_var_tv(VV_VAL);
  typval_T newtv;
  newtv.v_type = VAR_UNKNOWN;

  if (eval_expr_typval(expr, false, argv, 2, &newtv) == FAIL) {
    return false;
  }

  bool error = false;
  varnumber_T found = tv_get_bool_chk(&newtv, &error);
  tv_clear(&newtv);

  return error ? false : found;
}

static varnumber_T nvim_eval_indexof_blob(blob_T *b, varnumber_T startidx, typval_T *expr)
{
  if (b == NULL) {
    return -1;
  }

  if (startidx < 0) {
    startidx = tv_blob_len(b) + startidx;
    if (startidx < 0) {
      startidx = 0;
    }
  }

  set_vim_var_type(VV_KEY, VAR_NUMBER);
  set_vim_var_type(VV_VAL, VAR_NUMBER);

  const int called_emsg_start = called_emsg;
  for (varnumber_T idx = startidx; idx < tv_blob_len(b); idx++) {
    set_vim_var_nr(VV_KEY, idx);
    set_vim_var_nr(VV_VAL, tv_blob_get(b, (int)idx));

    if (nvim_eval_indexof_eval_expr(expr)) {
      return idx;
    }

    if (called_emsg != called_emsg_start) {
      return -1;
    }
  }

  return -1;
}

static varnumber_T nvim_eval_indexof_list(list_T *l, varnumber_T startidx, typval_T *expr)
{
  if (l == NULL) {
    return -1;
  }

  listitem_T *item;
  varnumber_T idx = 0;
  if (startidx == 0) {
    item = tv_list_first(l);
  } else {
    idx = tv_list_uidx(l, (int)startidx);
    if (idx == -1) {
      item = NULL;
    } else {
      item = tv_list_find(l, (int)idx);
      assert(item != NULL);
    }
  }

  set_vim_var_type(VV_KEY, VAR_NUMBER);

  const int called_emsg_start = called_emsg;
  for (; item != NULL; item = TV_LIST_ITEM_NEXT(l, item), idx++) {
    set_vim_var_nr(VV_KEY, idx);
    tv_copy(TV_LIST_ITEM_TV(item), get_vim_var_tv(VV_VAL));

    bool found = nvim_eval_indexof_eval_expr(expr);
    tv_clear(get_vim_var_tv(VV_VAL));

    if (found) {
      return idx;
    }

    if (called_emsg != called_emsg_start) {
      return -1;
    }
  }

  return -1;
}

void nvim_eval_indexof(typval_T *argvars, typval_T *rettv)
{
  rettv->vval.v_number = -1;

  if (tv_check_for_list_or_blob_arg(argvars, 0) == FAIL
      || tv_check_for_string_or_func_arg(argvars, 1) == FAIL
      || tv_check_for_opt_dict_arg(argvars, 2) == FAIL) {
    return;
  }

  if ((argvars[1].v_type == VAR_STRING
       && (argvars[1].vval.v_string == NULL || *argvars[1].vval.v_string == NUL))
      || (argvars[1].v_type == VAR_FUNC && argvars[1].vval.v_partial == NULL)) {
    return;
  }

  varnumber_T startidx = 0;
  if (argvars[2].v_type == VAR_DICT) {
    startidx = tv_dict_get_number_def(argvars[2].vval.v_dict, "startidx", 0);
  }

  typval_T save_val;
  typval_T save_key;
  prepare_vimvar(VV_VAL, &save_val);
  prepare_vimvar(VV_KEY, &save_key);

  const int save_did_emsg = did_emsg;
  did_emsg = false;

  if (argvars[0].v_type == VAR_BLOB) {
    rettv->vval.v_number = nvim_eval_indexof_blob(argvars[0].vval.v_blob, startidx, &argvars[1]);
  } else {
    rettv->vval.v_number = nvim_eval_indexof_list(argvars[0].vval.v_list, startidx, &argvars[1]);
  }

  restore_vimvar(VV_KEY, &save_key);
  restore_vimvar(VV_VAL, &save_val);
  did_emsg |= save_did_emsg;
}

// nvim_eval_range: inlined into Rust (misc.rs) — list construction


static void nvim_eval_reduce_list(typval_T *argvars, typval_T *expr, typval_T *rettv)
{
  list_T *const l = argvars[0].vval.v_list;
  const int called_emsg_start = called_emsg;

  typval_T initial;
  const listitem_T *li = NULL;
  if (argvars[2].v_type == VAR_UNKNOWN) {
    if (tv_list_len(l) == 0) {
      semsg(_(e_reduce_of_an_empty_str_with_no_initial_value), "List");
      return;
    }
    const listitem_T *const first = tv_list_first(l);
    initial = *TV_LIST_ITEM_TV(first);
    li = TV_LIST_ITEM_NEXT(l, first);
  } else {
    initial = argvars[2];
    li = tv_list_first(l);
  }

  tv_copy(&initial, rettv);

  if (l == NULL) {
    return;
  }

  const VarLockStatus prev_locked = tv_list_locked(l);

  tv_list_set_lock(l, VAR_FIXED);  // disallow the list changing here
  for (; li != NULL; li = TV_LIST_ITEM_NEXT(l, li)) {
    typval_T argv[3];
    argv[0] = *rettv;
    argv[1] = *TV_LIST_ITEM_TV(li);
    rettv->v_type = VAR_UNKNOWN;

    const int r = eval_expr_typval(expr, true, argv, 2, rettv);

    tv_clear(&argv[0]);
    if (r == FAIL || called_emsg != called_emsg_start) {
      break;
    }
  }
  tv_list_set_lock(l, prev_locked);
}

static void nvim_eval_reduce_string(typval_T *argvars, typval_T *expr, typval_T *rettv)
{
  const char *p = tv_get_string(&argvars[0]);
  int len;
  const int called_emsg_start = called_emsg;

  if (argvars[2].v_type == VAR_UNKNOWN) {
    if (*p == NUL) {
      semsg(_(e_reduce_of_an_empty_str_with_no_initial_value), "String");
      return;
    }
    len = utfc_ptr2len(p);
    *rettv = (typval_T){
      .v_type = VAR_STRING,
      .v_lock = VAR_UNLOCKED,
      .vval.v_string = xmemdupz(p, (size_t)len),
    };
    p += len;
  } else if (tv_check_for_string_arg(argvars, 2) == FAIL) {
    return;
  } else {
    tv_copy(&argvars[2], rettv);
  }

  for (; *p != NUL; p += len) {
    typval_T argv[3];
    argv[0] = *rettv;
    len = utfc_ptr2len(p);
    argv[1] = (typval_T){
      .v_type = VAR_STRING,
      .v_lock = VAR_UNLOCKED,
      .vval.v_string = xmemdupz(p, (size_t)len),
    };

    const int r = eval_expr_typval(expr, true, argv, 2, rettv);

    tv_clear(&argv[0]);
    tv_clear(&argv[1]);
    if (r == FAIL || called_emsg != called_emsg_start) {
      break;
    }
  }
}

static void nvim_eval_reduce_blob(typval_T *argvars, typval_T *expr, typval_T *rettv)
{
  const blob_T *const b = argvars[0].vval.v_blob;
  const int called_emsg_start = called_emsg;

  typval_T initial;
  int i;
  if (argvars[2].v_type == VAR_UNKNOWN) {
    if (tv_blob_len(b) == 0) {
      semsg(_(e_reduce_of_an_empty_str_with_no_initial_value), "Blob");
      return;
    }
    initial = (typval_T){
      .v_type = VAR_NUMBER,
      .v_lock = VAR_UNLOCKED,
      .vval.v_number = tv_blob_get(b, 0),
    };
    i = 1;
  } else if (tv_check_for_number_arg(argvars, 2) == FAIL) {
    return;
  } else {
    initial = argvars[2];
    i = 0;
  }

  tv_copy(&initial, rettv);
  for (; i < tv_blob_len(b); i++) {
    typval_T argv[3];
    argv[0] = *rettv;
    argv[1] = (typval_T){
      .v_type = VAR_NUMBER,
      .v_lock = VAR_UNLOCKED,
      .vval.v_number = tv_blob_get(b, i),
    };

    const int r = eval_expr_typval(expr, true, argv, 2, rettv);

    if (r == FAIL || called_emsg != called_emsg_start) {
      return;
    }
  }
}

void nvim_eval_reduce(typval_T *argvars, typval_T *rettv)
{
  if (argvars[0].v_type != VAR_STRING
      && argvars[0].v_type != VAR_LIST
      && argvars[0].v_type != VAR_BLOB) {
    emsg(_(e_string_list_or_blob_required));
    return;
  }

  const char *func_name;
  if (argvars[1].v_type == VAR_FUNC) {
    func_name = argvars[1].vval.v_string;
  } else if (argvars[1].v_type == VAR_PARTIAL) {
    func_name = rs_partial_name(argvars[1].vval.v_partial);
  } else {
    func_name = tv_get_string(&argvars[1]);
  }
  if (func_name == NULL || *func_name == NUL) {
    emsg(_(e_missing_function_argument));
    return;
  }

  if (argvars[0].v_type == VAR_LIST) {
    nvim_eval_reduce_list(argvars, &argvars[1], rettv);
  } else if (argvars[0].v_type == VAR_STRING) {
    nvim_eval_reduce_string(argvars, &argvars[1], rettv);
  } else {
    nvim_eval_reduce_blob(argvars, &argvars[1], rettv);
  }
}


// nvim_eval_eval: inlined into Rust (misc.rs)
// nvim_eval_exists: inlined into Rust (misc.rs)

// nvim_eval_has: migrated to Rust (rs_f_has in funcs/misc.rs)

// nvim_eval_json_decode: inlined into Rust (misc.rs)

void nvim_eval_printf(typval_T *argvars, typval_T *rettv)
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;
  {
    int saved_did_emsg = did_emsg;

    // Get the required length, allocate the buffer and do it for real.
    did_emsg = false;
    char buf[NUMBUFLEN];
    const char *fmt = tv_get_string_buf(&argvars[0], buf);
    int len = vim_vsnprintf_typval(NULL, 0, fmt, dummy_ap, argvars + 1);
    if (!did_emsg) {
      char *s = xmalloc((size_t)len + 1);
      rettv->vval.v_string = s;
      vim_vsnprintf_typval(s, (size_t)len + 1, fmt, dummy_ap, argvars + 1);
    }
    did_emsg |= saved_did_emsg;
  }
}


// f_getcmdcomplpat, f_getcmdcompltype, f_setcmdline, f_setcmdpos, f_wildtrigger:
// migrated to Rust in src/nvim-rs/eval/src/funcs/cmdline.rs

// Rust helper used by get_user_input
extern int rs_get_echo_hl_id(void);

/// This function is used by f_input() and f_inputdialog() functions. The third
/// argument to f_input() specifies the type of completion to use at the
/// prompt. The third argument to f_inputdialog() specifies the value to return
/// when the user cancels the prompt.
void get_user_input(const typval_T *const argvars, typval_T *const rettv, const bool inputdialog,
                    const bool secret)
  FUNC_ATTR_NONNULL_ALL
{
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  if (cmdpreview) {
    return;
  }

  const char *prompt;
  const char *defstr = "";
  typval_T *cancelreturn = NULL;
  typval_T cancelreturn_strarg2 = TV_INITIAL_VALUE;
  const char *xp_name = NULL;
  Callback input_callback = { .type = kCallbackNone };
  char prompt_buf[NUMBUFLEN];
  char defstr_buf[NUMBUFLEN];
  char cancelreturn_buf[NUMBUFLEN];
  char xp_name_buf[NUMBUFLEN];
  char def[1] = { 0 };
  if (argvars[0].v_type == VAR_DICT) {
    if (argvars[1].v_type != VAR_UNKNOWN) {
      emsg(_("E5050: {opts} must be the only argument"));
      return;
    }
    dict_T *const dict = argvars[0].vval.v_dict;
    prompt = tv_dict_get_string_buf_chk(dict, S_LEN("prompt"), prompt_buf, "");
    if (prompt == NULL) {
      return;
    }
    defstr = tv_dict_get_string_buf_chk(dict, S_LEN("default"), defstr_buf, "");
    if (defstr == NULL) {
      return;
    }
    dictitem_T *cancelreturn_di = tv_dict_find(dict, S_LEN("cancelreturn"));
    if (cancelreturn_di != NULL) {
      cancelreturn = &cancelreturn_di->di_tv;
    }
    xp_name = tv_dict_get_string_buf_chk(dict, S_LEN("completion"),
                                         xp_name_buf, def);
    if (xp_name == NULL) {  // error
      return;
    }
    if (xp_name == def) {  // default to NULL
      xp_name = NULL;
    }
    if (!tv_dict_get_callback(dict, S_LEN("highlight"), &input_callback)) {
      return;
    }
  } else {
    prompt = tv_get_string_buf_chk(&argvars[0], prompt_buf);
    if (prompt == NULL) {
      return;
    }
    if (argvars[1].v_type != VAR_UNKNOWN) {
      defstr = tv_get_string_buf_chk(&argvars[1], defstr_buf);
      if (defstr == NULL) {
        return;
      }
      if (argvars[2].v_type != VAR_UNKNOWN) {
        const char *const strarg2 = tv_get_string_buf_chk(&argvars[2], cancelreturn_buf);
        if (strarg2 == NULL) {
          return;
        }
        if (inputdialog) {
          cancelreturn_strarg2.v_type = VAR_STRING;
          cancelreturn_strarg2.vval.v_string = (char *)strarg2;
          cancelreturn = &cancelreturn_strarg2;
        } else {
          xp_name = strarg2;
        }
      }
    }
  }

  int xp_type = EXPAND_NOTHING;
  char *xp_arg = NULL;
  if (xp_name != NULL) {
    // input() with a third argument: completion
    const int xp_namelen = (int)strlen(xp_name);

    uint32_t argt = 0;
    if (parse_compl_arg(xp_name, xp_namelen, &xp_type,
                        &argt, &xp_arg) == FAIL) {
      return;
    }
  }

  // Only the part of the message after the last NL is considered as
  // prompt for the command line, unlsess cmdline is externalized
  const char *p = prompt;
  if (!ui_has(kUICmdline)) {
    const char *lastnl = strrchr(prompt, '\n');
    if (lastnl != NULL) {
      p = lastnl + 1;
      msg_start();
      msg_clr_eos();
      msg_puts_len(prompt, p - prompt, rs_get_echo_hl_id(), false);
      msg_didout = false;
      msg_starthere();
    }
  }
  cmdline_row = msg_row;

  stuffReadbuffSpec(defstr);

  const int save_ex_normal_busy = ex_normal_busy;
  ex_normal_busy = 0;
  rettv->vval.v_string = getcmdline_prompt(secret ? NUL : '@', p, rs_get_echo_hl_id(),
                                           xp_type, xp_arg, input_callback, false, NULL);
  ex_normal_busy = save_ex_normal_busy;
  callback_free(&input_callback);

  if (rettv->vval.v_string == NULL && cancelreturn != NULL) {
    tv_copy(cancelreturn, rettv);
  }

  xfree(xp_arg);

  // Since the user typed this, no need to wait for return.
  need_wait_return = false;
  msg_didout = false;
}
