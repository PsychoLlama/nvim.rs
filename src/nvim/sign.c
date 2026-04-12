// sign.c: functions for managing with signs

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/decoration_defs.h"
#include "nvim/drawscreen.h"
#include "nvim/edit.h"
#include "nvim/errors.h"
#include "nvim/eval/funcs.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/extmark.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/marktree.h"
#include "nvim/marktree_defs.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/pos_defs.h"
#include "nvim/sign.h"
#include "nvim/sign_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

#include "sign.c.generated.h"

// Rust FFI
extern void rs_foldOpenCursor(void);
extern void sign_get_placed(buf_T *buf, linenr_T lnum, int id, const char *group, list_T *retlist);
extern int64_t group_get_ns(const char *group);
extern int rs_sign_cmd_idx(const char *cmd);
extern const char *rs_sign_get_display_name(DecorSignHighlight *sh);
extern bool rs_sign_buffer_has_signs(const buf_T *buf);
extern int rs_buf_findsign(buf_T *buf, int id, const char *group);
extern void rs_sign_list_defined(sign_T *sp);
extern void rs_sign_list_by_name(const char *name);
extern int rs_sign_define_by_name(const char *name, const char *icon, const char *text, const char *linehl, const char *texthl, const char *culhl, const char *numhl, int prio);
extern int rs_sign_undefine_by_name(const char *name);
extern size_t rs_describe_sign_text(char *buf, size_t buf_len, schar_T *sign_text);
extern int rs_sign_place(uint32_t *id, const char *group, const char *name, buf_T *buf, linenr_T lnum, int prio);
extern int rs_sign_unplace(buf_T *buf, int id, const char *group, linenr_T atlnum);
// Phase 1: these functions now live in Rust (nvim-sign crate)
extern dict_T *nvim_sign_get_placed_info_dict_impl(MTKey *mark);
// Phase 2: these functions now live in Rust (nvim-sign crate)
extern int nvim_sign_delete_signs_impl(buf_T *buf, int64_t ns, int id, linenr_T atlnum);
extern void nvim_sign_build_decor_and_set(buf_T *buf, uint32_t ns, uint32_t *id, int row, sign_T *sp, int prio);
extern void nvim_sign_define_update_placed(const char *name, sign_T *sp);
// Phase 3: these functions now live in Rust (nvim-sign crate)
extern void nvim_sign_get_placed_in_buf_impl(buf_T *buf, linenr_T lnum, int sign_id, const char *group, list_T *retlist);

static PMap(cstr_t) sign_map = MAP_INIT;
static kvec_t(Integer) sign_ns = KV_INITIAL_VALUE;


sign_T *nvim_sign_map_get(const char *name)
{ return name ? pmap_get(cstr_t)(&sign_map, name) : NULL; }
int nvim_sign_map_has(const char *name)
{ return name ? (map_has(cstr_t, &sign_map, name) ? 1 : 0) : 0; }
char *nvim_sign_map_get_nth_key(int idx)
{
  cstr_t name; int current_idx = 0;
  map_foreach_key(&sign_map, name, { if (current_idx++ == idx) { return (char *)name; } });
  return NULL;
}
int nvim_sign_ns_size(void) { return (int)kv_size(sign_ns); }
Integer nvim_sign_ns_get(int idx) { return idx < (int)kv_size(sign_ns) ? kv_A(sign_ns, idx) : -1; }
char *nvim_sign_ns_get_name(int idx)
{ return idx < (int)kv_size(sign_ns) ? (char *)describe_ns((NS)kv_A(sign_ns, idx), "") : NULL; }
linenr_T nvim_sign_marktree_lookup_row(buf_T *buf, uint32_t ns, uint32_t id)
{ MTKey mark = rs_marktree_lookup_ns(buf->b_marktree, ns, id, false, NULL); return mark.pos.row + 1; }
linenr_T nvim_sign_buf_line_count(buf_T *buf) { return buf ? buf->b_ml.ml_line_count : 0; }
void nvim_sign_ns_push(Integer ns) { kv_push(sign_ns, ns); }
int nvim_sign_create_namespace_cstr(const char *name) { return (int)nvim_create_namespace(cstr_as_string(name)); }
int nvim_sign_namespace_exists(const char *name) { return map_get(String, int)(&namespace_ids, cstr_as_string(name)) ? 1 : 0; }

// typval_T / list_T / dict_T accessor shims for Rust FFI
// These let Rust access C typval internals without knowing the struct layout.
// Note: nvim_tv_get_type, nvim_tv_get_dict, nvim_tv_get_list are in typval.c
// Note: nvim_tv_list_len is in eval_shim.c
varnumber_T nvim_tv_get_number_val(const typval_T *tv) { return tv->vval.v_number; }
void nvim_rettv_set_number(typval_T *rettv, varnumber_T num)
{ rettv->v_type = VAR_NUMBER; rettv->vval.v_number = num; }
void nvim_rettv_alloc_list(typval_T *rettv, int len) { tv_list_alloc_ret(rettv, len); }
list_T *nvim_rettv_get_list(typval_T *rettv) { return rettv->vval.v_list; }
// Indexed access to argvars / rettv (pointer arithmetic as C shim)
typval_T *nvim_argvars_at(typval_T *argvars, int idx) { return &argvars[idx]; }
// sign_map iteration by value (for getdefined)
sign_T *nvim_sign_map_get_nth_value(int idx)
{
  sign_T *sp; int current_idx = 0;
  map_foreach_value(&sign_map, sp, { if (current_idx++ == idx) { return sp; } });
  return NULL;
}
// TV list indexed item typval access
typval_T *nvim_tv_list_item_tv_at(list_T *l, int idx)
{
  int i = 0;
  TV_LIST_ITER_CONST(l, li, { if (i++ == idx) { return (typval_T *)TV_LIST_ITEM_TV(li); } });
  return NULL;
}
// sign_map iteration size (for free_all)
int nvim_sign_map_size(void) { return (int)map_size(&sign_map); }

// Phase 3 accessors: sign_map insert/delete
sign_T *nvim_sign_map_del(const char *name)
{
  sign_T *sp = pmap_del(cstr_t)(&sign_map, name, NULL);
  return sp;
}
// Get or create sign entry in sign_map. Returns sign_T*, sets *is_new if created.
// If new, also allocates and initializes the sign_T and copies the name.
sign_T *nvim_sign_map_get_or_create(const char *name, bool *is_new)
{
  cstr_t *key;
  bool new_sign = false;
  sign_T **sp_ptr = (sign_T **)pmap_put_ref(cstr_t)(&sign_map, name, &key, &new_sign);
  if (new_sign) {
    *key = xstrdup(name);
    *sp_ptr = xcalloc(1, sizeof(sign_T));
    (*sp_ptr)->sn_name = (char *)(*key);
  }
  *is_new = new_sign;
  return *sp_ptr;
}
// init_sign_text wrapper (expose to Rust)
int nvim_init_sign_text(sign_T *sp, schar_T *out, const char *text)
{ return (int)init_sign_text(sp, out, (char *)text); }
// nvim_backslash_halve is in ex_docmd.c
// Phase 3: jump accessors
win_T *nvim_buf_jump_open_win(buf_T *buf) { return buf_jump_open_win(buf); }
// nvim_curwin_set_cursor_lnum is in ex_cmds_shim.c
void nvim_curwin_check_and_beginline(void) { check_cursor_lnum(curwin); beginline(BL_WHITE); }
const char *nvim_buf_get_fname(buf_T *buf) { return buf ? buf->b_fname : NULL; }
void nvim_do_cmdline_cmd_str(const char *cmd) { do_cmdline_cmd((char *)cmd); }
// Phase 3: list_defined message accessors
void nvim_smsg0(const char *msg) { smsg(0, "%s", msg); }
void nvim_msg_puts(const char *s) { msg_puts(s); }
void nvim_msg_outtrans(const char *s) { msg_outtrans((char *)s, 0, false); }
void nvim_msg_putchar_nl(void) { msg_putchar('\n'); }
// Format a number into a buffer for list_defined priority
void nvim_msg_puts_priority(int prio) {
  char lbuf[MSG_BUF_LEN];
  vim_snprintf(lbuf, MSG_BUF_LEN, " priority=%d", prio);
  msg_puts(lbuf);
}
// Phase 3: delete_signs (marktree iteration from Rust)
void nvim_marktree_free_itr(MarkTreeIter *itr) { xfree(itr); }
bool nvim_mtitr_has_x(const MarkTreeIter *itr) { return itr->x != NULL; }
MTKey nvim_mtitr_current_key(MarkTreeIter *itr) { return rs_marktree_itr_current(itr); }
bool nvim_mtitr_next(buf_T *buf, MarkTreeIter *itr) { return rs_marktree_itr_next(buf->b_marktree, itr); }
bool nvim_mt_end(MTKey key) { return mt_end(key); }
bool nvim_mt_decor_sign(MTKey key) { return mt_decor_sign(key); }
bool nvim_mtitr_get_overlap(buf_T *buf, int row, int col, MarkTreeIter *itr) { return rs_marktree_itr_get_overlap(buf->b_marktree, row, col, itr); }
bool nvim_mtitr_step_overlap(buf_T *buf, MarkTreeIter *itr, MTPair *pair) { return rs_marktree_itr_step_overlap(buf->b_marktree, itr, pair); }
void nvim_mtitr_get(buf_T *buf, int row, int col, MarkTreeIter *itr) { rs_marktree_itr_get(buf->b_marktree, row, col, itr); }
void nvim_extmark_del(buf_T *buf, MarkTreeIter *itr, MTKey mark, bool end) { extmark_del(buf, itr, mark, end); }
MTKey nvim_mtpair_start(MTPair pair) { return pair.start; }
uint32_t nvim_ns_all(void) { return UINT32_MAX; }
// Phase 2: C helper — trigger sign redraw for all windows in current tab
void nvim_redraw_sign_buffers_in_curtab(void)
{
  FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
    if (rs_sign_buffer_has_signs(wp->w_buffer)) {
      redraw_buf_later(wp->w_buffer, UPD_NOT_VALID);
    }
  }
}
// Phase 2: C helper for extmark_set (to avoid exposing Error* to Rust)
void nvim_sign_extmark_set(buf_T *buf, uint32_t ns, uint32_t *id, int row,
                           DecorInline decor, uint16_t decor_flags)
{
  extmark_set(buf, ns, id, row, 0, -1, -1, decor, decor_flags, true, false, true, true, NULL);
}


