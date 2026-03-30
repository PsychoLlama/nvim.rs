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

// VimL functions moved to funcs_shim.c (Phase 23)
extern void f_line(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_serverlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_swapname(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_tabpagebuflist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_virtcol(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 24)
extern void f_getchangelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getjumplist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getmarklist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_gettagstack(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_prompt_getprompt(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_prompt_getinput(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 25)
extern void f_expandcmd(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_islocked(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_settagstack(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 26)
extern void f_call(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_expand(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_split(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 27)
extern void f_get(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_setreg(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 28)
extern void f_getregion(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getregionpos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 29)
extern void f_wait(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_inputlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_inputrestore(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_inputsave(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_inputsecret(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 30)
extern void f_matchbufline(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchstrlist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_msgpackdump(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_msgpackparse(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 31)
extern void f_rpcnotify(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_rpcrequest(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_sockconnect(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_stdioopen(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// VimL functions moved to funcs_shim.c (Phase 32)
extern void f_chansend(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_jobpid(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_jobresize(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_jobstart(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_jobstop(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_jobwait(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust fs.c Phase 3 VimL function declarations (fs.rs in nvim-eval crate)
extern void f_chdir(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getcwd(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_haslocaldir(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_mkdir(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_readdir(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust fs.c Phase 2 VimL function declarations (fs.rs in nvim-eval crate)
extern void f_getfsize(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getftype(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_filecopy(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_delete(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_pathshorten(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

// Rust fs.c Phase 1 VimL function declarations (fs.rs in nvim-eval crate)
extern void f_browse(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_browsedir(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_isabsolutepath(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_isdirectory(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_filereadable(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_filewritable(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_executable(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_tempname(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_simplify(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_rename(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getftime(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_glob2regpat(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_exepath(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_getfperm(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

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

// e_invalwindow: moved to funcs_shim.c (Phase 32)
// e_string_list_or_blob_required: moved to funcs_shim.c
// e_missing_function_argument: moved to funcs_shim.c
// dummy_ap (static va_list): moved to funcs_shim.c as dummy_ap_shim


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


/// "chanclose(id[, stream])" function


/// json_decode() function

/// json_encode() function
/// "keytrans()" function


/// "libcall()" function

/// "nextnonblank()" function


/// "printf()" function
/// "py3eval()" and "pyxeval()" functions (always python3)


/// "perleval()" function
/// "range()" function

/// "getreginfo()" function


/// "reltimestr()" function


// SP_NOMOVE and other SP_* macros, get_search_arg, search_cmn,
// nvim_curbuf_get_did_filetype, nvim_curbuf_get_u_seq_cur,
// nvim_get_reg_recorded: moved to funcs_shim.c (Phase 33)
// nvim_eval_ui_current_col: inlined into Rust (misc.rs) — direct ui_current_col() call
// nvim_eval_ui_current_row: inlined into Rust (misc.rs) — direct ui_current_row() call
// nvim_eval_pum_visible: inlined into Rust (misc.rs) — direct pum_visible() call
// nvim_eval_os_get_pid: inlined into Rust (misc.rs) — direct os_get_pid() call
// nvim_eval_get_col: moved to funcs_shim.c
// nvim_eval_getpos_both: moved to funcs_shim.c
// nvim_eval_get_windows_version: moved to funcs_shim.c


// nvim_eval_find_some_match: moved to funcs_shim.c
// nvim_eval_max_min: moved to funcs_shim.c
/// Used by searchpair() and searchpairpos()
// searchpair_cmn, do_searchpair, nvim_eval_searchpair_cmn,
// nvim_eval_search, nvim_eval_searchpairpos: moved to funcs_shim.c (Phase 33)
// nvim_eval_swapfilelist: inlined into Rust (misc.rs) — rs_recover_names delegation
// nvim_eval_swapinfo: inlined into Rust (misc.rs) — swapfile_dict delegation


// nvim_eval_api_info: moved to funcs_shim.c

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


// screenchar_adjust_inner, nvim_eval_screenattr, nvim_eval_screenchar,
// nvim_eval_screenchars, nvim_eval_screenstring:
// moved to funcs_shim.c


// nvim_eval_environ: inlined into Rust (system.rs) — os_get_fullenv_size + os_copy_fullenv delegation

// get_xdg_var_list_inner, nvim_eval_xdg_var_list,
// nvim_eval_searchpos: moved to funcs_shim.c (Phase 33)


// nvim_eval_timer_info: moved to funcs_shim.c

// nvim_eval_timer_pause: moved to funcs_shim.c

// nvim_eval_timer_start: moved to funcs_shim.c

// nvim_eval_timer_stop: moved to funcs_shim.c

// nvim_eval_spellbadword: moved to funcs_shim.c

// nvim_eval_spellsuggest: moved to funcs_shim.c


// nvim_eval_synID: moved to funcs_shim.c


// nvim_eval_synconcealed: moved to funcs_shim.c

// nvim_eval_synstack: moved to funcs_shim.c


// nvim_eval_index, nvim_eval_indexof_*: moved to funcs_shim.c

// nvim_eval_range: inlined into Rust (misc.rs) — list construction


// nvim_eval_reduce_list/string/blob, nvim_eval_reduce: moved to funcs_shim.c


// nvim_eval_eval: inlined into Rust (misc.rs)
// nvim_eval_exists: inlined into Rust (misc.rs)

// nvim_eval_has: migrated to Rust (rs_f_has in funcs/misc.rs)

// nvim_eval_json_decode: inlined into Rust (misc.rs)
// nvim_eval_printf: moved to funcs_shim.c (with dummy_ap_shim)


// f_getcmdcomplpat, f_getcmdcompltype, f_setcmdline, f_setcmdpos, f_wildtrigger:
// migrated to Rust in src/nvim-rs/eval/src/funcs/cmdline.rs

// get_user_input: moved to funcs_shim.c (Phase 29)
