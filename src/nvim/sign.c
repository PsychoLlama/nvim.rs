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
extern int rs_sign_row_cmp(int row1, int row2);
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

static PMap(cstr_t) sign_map = MAP_INIT;
static kvec_t(Integer) sign_ns = KV_INITIAL_VALUE;

static int sign_row_cmp(const void *p1, const void *p2)
{
  const MTKey *s1 = (MTKey *)p1;
  const MTKey *s2 = (MTKey *)p2;
  int row_cmp = rs_sign_row_cmp(s1->pos.row, s2->pos.row);
  if (row_cmp != 0) {
    return row_cmp;
  }
  DecorSignHighlight *sh1 = decor_find_sign(mt_decor(*s1));
  DecorSignHighlight *sh2 = decor_find_sign(mt_decor(*s2));
  assert(sh1 && sh2);
  SignItem si1 = { sh1, s1->id };
  SignItem si2 = { sh2, s2->id };
  return sign_item_cmp(&si1, &si2);
}

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
void nvim_sign_build_decor_and_set(buf_T *buf, uint32_t ns, uint32_t *id, int row, sign_T *sp, int prio)
{
  DecorSignHighlight sign = DECOR_SIGN_HIGHLIGHT_INIT;
  sign.flags |= kSHIsSign;
  memcpy(sign.text, sp->sn_text, SIGN_WIDTH * sizeof(schar_T));
  sign.sign_name = xstrdup(sp->sn_name);
  sign.hl_id = sp->sn_text_hl;
  sign.line_hl_id = sp->sn_line_hl;
  sign.number_hl_id = sp->sn_num_hl;
  sign.cursorline_hl_id = sp->sn_cul_hl;
  sign.priority = (DecorPriority)prio;
  bool has_hl = (sp->sn_line_hl || sp->sn_num_hl || sp->sn_cul_hl);
  uint16_t decor_flags = (sp->sn_text[0] ? MT_FLAG_DECOR_SIGNTEXT : 0)
                         | (has_hl ? MT_FLAG_DECOR_SIGNHL : 0);
  DecorInline decor = { .ext = true, .data.ext = { .vt = NULL, .sh_idx = decor_put_sh(sign) } };
  extmark_set(buf, ns, id, row, 0, -1, -1, decor, decor_flags, true, false, true, true, NULL);
}
linenr_T nvim_sign_marktree_lookup_row(buf_T *buf, uint32_t ns, uint32_t id)
{ MTKey mark = rs_marktree_lookup_ns(buf->b_marktree, ns, id, false, NULL); return mark.pos.row + 1; }
linenr_T nvim_sign_buf_line_count(buf_T *buf) { return buf ? buf->b_ml.ml_line_count : 0; }
void nvim_sign_ns_push(Integer ns) { kv_push(sign_ns, ns); }
int nvim_sign_create_namespace_cstr(const char *name) { return (int)nvim_create_namespace(cstr_as_string(name)); }
int nvim_sign_namespace_exists(const char *name) { return map_get(String, int)(&namespace_ids, cstr_as_string(name)) ? 1 : 0; }

int nvim_sign_define_by_name_impl(const char *name, const char *icon, const char *text, const char *linehl, const char *texthl, const char *culhl, const char *numhl, int prio)
{
  cstr_t *key;
  bool new_sign = false;
  sign_T **sp = (sign_T **)pmap_put_ref(cstr_t)(&sign_map, name, &key, &new_sign);
  if (new_sign) {
    *key = xstrdup(name);
    *sp = xcalloc(1, sizeof(sign_T));
    (*sp)->sn_name = (char *)(*key);
  }
  if (icon != NULL) {
    xfree((*sp)->sn_icon);
    (*sp)->sn_icon = xstrdup(icon);
    backslash_halve((*sp)->sn_icon);
  }
  if (text != NULL && (init_sign_text(*sp, (*sp)->sn_text, (char *)text) == FAIL)) {
    return FAIL;
  }
  (*sp)->sn_priority = prio;
  const char *arg[] = { linehl, texthl, culhl, numhl };
  int *hl[] = { &(*sp)->sn_line_hl, &(*sp)->sn_text_hl, &(*sp)->sn_cul_hl, &(*sp)->sn_num_hl };
  for (int i = 0; i < 4; i++) {
    if (arg[i] != NULL) {
      *hl[i] = *arg[i] ? syn_check_group(arg[i], strlen(arg[i])) : 0;
    }
  }
  // Update already placed signs and redraw if necessary when modifying a sign.
  if (!new_sign) {
    bool did_redraw = false;
    for (size_t i = 0; i < kv_size(decor_items); i++) {
      DecorSignHighlight *sh = &kv_A(decor_items, i);
      if (sh->sign_name && strcmp(sh->sign_name, name) == 0) {
        memcpy(sh->text, (*sp)->sn_text, SIGN_WIDTH * sizeof(schar_T));
        sh->hl_id = (*sp)->sn_text_hl;
        sh->line_hl_id = (*sp)->sn_line_hl;
        sh->number_hl_id = (*sp)->sn_num_hl;
        sh->cursorline_hl_id = (*sp)->sn_cul_hl;
        if (!did_redraw) {
          FOR_ALL_WINDOWS_IN_TAB(wp, curtab) {
            if (rs_sign_buffer_has_signs(wp->w_buffer)) {
              redraw_buf_later(wp->w_buffer, UPD_NOT_VALID);
            }
          }
          did_redraw = true;
        }
      }
    }
  }
  return OK;
}
int nvim_sign_undefine_by_name_impl(const char *name)
{
  sign_T *sp = pmap_del(cstr_t)(&sign_map, name, NULL);
  if (sp == NULL) { return FAIL; }
  xfree(sp->sn_name); xfree(sp->sn_icon); xfree(sp);
  return OK;
}
linenr_T nvim_sign_jump_impl(int id, char *group, buf_T *buf)
{
  linenr_T lnum = rs_buf_findsign(buf, id, group);
  if (lnum <= 0) {
    semsg(_("E157: Invalid sign ID: %" PRId32), id);
    return -1;
  }
  if (buf_jump_open_win(buf) != NULL) {
    curwin->w_cursor.lnum = lnum;
    check_cursor_lnum(curwin);
    beginline(BL_WHITE);
  } else {
    if (buf->b_fname == NULL) {
      emsg(_("E934: Cannot jump to a buffer that does not have a name"));
      return -1;
    }
    size_t cmdlen = strlen(buf->b_fname) + 24;
    char *cmd = xmallocz(cmdlen);
    snprintf(cmd, cmdlen, "e +%" PRId64 " %s", (int64_t)lnum, buf->b_fname);
    do_cmdline_cmd(cmd);
    xfree(cmd);
  }
  rs_foldOpenCursor();
  return lnum;
}
void nvim_sign_free_all_impl(void)
{
  cstr_t name; kvec_t(cstr_t) names = KV_INITIAL_VALUE;
  map_foreach_key(&sign_map, name, { kv_push(names, name); });
  for (size_t i = 0; i < kv_size(names); i++) { nvim_sign_undefine_by_name_impl(kv_A(names, i)); }
  kv_destroy(names);
}
int nvim_sign_delete_signs_impl(buf_T *buf, int64_t ns, int id, linenr_T atlnum)
{
  MarkTreeIter itr[1];
  int row = atlnum > 0 ? atlnum - 1 : 0;
  kvec_t(MTKey) signs = KV_INITIAL_VALUE;
  // Store signs at a specific line number to remove one later.
  if (atlnum > 0) {
    if (!rs_marktree_itr_get_overlap(buf->b_marktree, row, 0, itr)) {
      return FAIL;
    }
    MTPair pair;
    while (rs_marktree_itr_step_overlap(buf->b_marktree, itr, &pair)) {
      if ((ns == UINT32_MAX || ns == pair.start.ns) && mt_decor_sign(pair.start)) {
        kv_push(signs, pair.start);
      }
    }
  } else {
    rs_marktree_itr_get(buf->b_marktree, 0, 0, itr);
  }
  while (itr->x) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (row && mark.pos.row > row) {
      break;
    }
    if (!mt_end(mark) && mt_decor_sign(mark)
        && (id == 0 || (int)mark.id == id)
        && (ns == UINT32_MAX || ns == mark.ns)) {
      if (atlnum > 0) {
        kv_push(signs, mark);
        rs_marktree_itr_next(buf->b_marktree, itr);
      } else {
        extmark_del(buf, itr, mark, true);
      }
    } else {
      rs_marktree_itr_next(buf->b_marktree, itr);
    }
  }
  // Sort to remove the highest priority sign at a specific line number.
  if (kv_size(signs)) {
    qsort((void *)&kv_A(signs, 0), kv_size(signs), sizeof(MTKey), sign_row_cmp);
    extmark_del_id(buf, kv_A(signs, 0).ns, kv_A(signs, 0).id);
    kv_destroy(signs);
  } else if (atlnum > 0) {
    return FAIL;
  }
  return OK;
}
void nvim_sign_list_placed_impl(buf_T *rbuf, const char *group)
{
  char lbuf[MSG_BUF_LEN];
  char namebuf[MSG_BUF_LEN];
  char groupbuf[MSG_BUF_LEN];
  buf_T *buf = rbuf ? rbuf : firstbuf;
  int64_t ns = group_get_ns(group);
  msg_puts_title(_("\n--- Signs ---"));
  msg_putchar('\n');
  while (buf != NULL && !got_int) {
    if (rs_sign_buffer_has_signs(buf)) {
      vim_snprintf(lbuf, MSG_BUF_LEN, _("Signs for %s:"), buf->b_fname);
      msg_puts_hl(lbuf, HLF_D, false);
      msg_putchar('\n');
    }
    if (ns >= 0) {
      MarkTreeIter itr[1];
      kvec_t(MTKey) signs = KV_INITIAL_VALUE;
      rs_marktree_itr_get(buf->b_marktree, 0, 0, itr);
      while (itr->x) {
        MTKey mark = rs_marktree_itr_current(itr);
        if (!mt_end(mark) && mt_decor_sign(mark)
            && (ns == UINT32_MAX || ns == mark.ns)) {
          kv_push(signs, mark);
        }
        rs_marktree_itr_next(buf->b_marktree, itr);
      }
      if (kv_size(signs)) {
        qsort((void *)&kv_A(signs, 0), kv_size(signs), sizeof(MTKey), sign_row_cmp);
        for (size_t i = 0; i < kv_size(signs); i++) {
          namebuf[0] = NUL;
          groupbuf[0] = NUL;
          MTKey mark = kv_A(signs, i);
          DecorSignHighlight *sh = decor_find_sign(mt_decor(mark));
          if (sh->sign_name != NULL) {
            vim_snprintf(namebuf, MSG_BUF_LEN, _("  name=%s"), rs_sign_get_display_name(sh));
          }
          if (mark.ns != 0) {
            vim_snprintf(groupbuf, MSG_BUF_LEN, _("  group=%s"), describe_ns((int)mark.ns, ""));
          }
          vim_snprintf(lbuf, MSG_BUF_LEN, _("    line=%" PRIdLINENR "  id=%u%s%s  priority=%d"),
                       mark.pos.row + 1, mark.id, groupbuf, namebuf, sh->priority);
          msg_puts(lbuf);
          msg_putchar('\n');
        }
        kv_destroy(signs);
      }
    }
    if (rbuf != NULL) {
      return;
    }
    buf = buf->b_next;
  }
}
void nvim_sign_list_defined_impl(sign_T *sp)
{
  smsg(0, "sign %s", sp->sn_name);
  if (sp->sn_icon != NULL) {
    msg_puts(" icon=");
    msg_outtrans(sp->sn_icon, 0, false);
    msg_puts(_(" (not supported)"));
  }
  if (sp->sn_text[0]) {
    msg_puts(" text=");
    char buf[SIGN_WIDTH * MAX_SCHAR_SIZE];
    rs_describe_sign_text(buf, SIGN_WIDTH * MAX_SCHAR_SIZE, sp->sn_text);
    msg_outtrans(buf, 0, false);
  }
  if (sp->sn_priority > 0) {
    char lbuf[MSG_BUF_LEN];
    vim_snprintf(lbuf, MSG_BUF_LEN, " priority=%d", sp->sn_priority);
    msg_puts(lbuf);
  }
  static char *arg[] = { " linehl=", " texthl=", " culhl=", " numhl=" };
  int hl[] = { sp->sn_line_hl, sp->sn_text_hl, sp->sn_cul_hl, sp->sn_num_hl };
  for (int i = 0; i < 4; i++) {
    if (hl[i] > 0) {
      msg_puts(arg[i]);
      const char *p = get_highlight_name_ext(NULL, hl[i] - 1, false);
      msg_puts(p ? p : "NONE");
    }
  }
}
dict_T *nvim_sign_get_info_dict_impl(sign_T *sp)
{
  dict_T *d = tv_dict_alloc();
  tv_dict_add_str(d, S_LEN("name"), sp->sn_name);
  if (sp->sn_icon != NULL) {
    tv_dict_add_str(d, S_LEN("icon"), sp->sn_icon);
  }
  if (sp->sn_text[0]) {
    char buf[SIGN_WIDTH * MAX_SCHAR_SIZE];
    rs_describe_sign_text(buf, SIGN_WIDTH * MAX_SCHAR_SIZE, sp->sn_text);
    tv_dict_add_str(d, S_LEN("text"), buf);
  }
  if (sp->sn_priority > 0) {
    tv_dict_add_nr(d, S_LEN("priority"), sp->sn_priority);
  }
  static char *arg[] = { "linehl", "texthl", "culhl", "numhl" };
  int hl[] = { sp->sn_line_hl, sp->sn_text_hl, sp->sn_cul_hl, sp->sn_num_hl };
  for (int i = 0; i < 4; i++) {
    if (hl[i] > 0) {
      const char *p = get_highlight_name_ext(NULL, hl[i] - 1, false);
      tv_dict_add_str(d, arg[i], strlen(arg[i]), p ? p : "NONE");
    }
  }
  return d;
}
dict_T *nvim_sign_get_placed_info_dict_impl(MTKey *mark)
{
  dict_T *d = tv_dict_alloc();
  DecorSignHighlight *sh = decor_find_sign(mt_decor(*mark));
  tv_dict_add_str(d, S_LEN("name"), rs_sign_get_display_name(sh));
  tv_dict_add_nr(d,  S_LEN("id"), (int)mark->id);
  tv_dict_add_str(d, S_LEN("group"), describe_ns((int)mark->ns, ""));
  tv_dict_add_nr(d,  S_LEN("lnum"), mark->pos.row + 1);
  tv_dict_add_nr(d,  S_LEN("priority"), sh->priority);
  return d;
}
list_T *nvim_get_buffer_signs_impl(buf_T *buf)
{
  list_T *const l = tv_list_alloc(kListLenMayKnow);
  MarkTreeIter itr[1];
  rs_marktree_itr_get(buf->b_marktree, 0, 0, itr);
  while (itr->x) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (!mt_end(mark) && mt_decor_sign(mark)) {
      tv_list_append_dict(l, nvim_sign_get_placed_info_dict_impl(&mark));
    }
    rs_marktree_itr_next(buf->b_marktree, itr);
  }
  return l;
}
void nvim_sign_get_placed_in_buf_impl(buf_T *buf, linenr_T lnum, int sign_id, const char *group, list_T *retlist)
{
  dict_T *d = tv_dict_alloc();
  tv_list_append_dict(retlist, d);
  tv_dict_add_nr(d, S_LEN("bufnr"), buf->b_fnum);
  list_T *l = tv_list_alloc(kListLenMayKnow);
  tv_dict_add_list(d, S_LEN("signs"), l);
  int64_t ns = group_get_ns(group);
  if (!rs_sign_buffer_has_signs(buf) || ns < 0) {
    return;
  }
  MarkTreeIter itr[1];
  kvec_t(MTKey) signs = KV_INITIAL_VALUE;
  rs_marktree_itr_get(buf->b_marktree, lnum ? lnum - 1 : 0, 0, itr);
  while (itr->x) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (lnum && mark.pos.row >= lnum) {
      break;
    }
    if (!mt_end(mark)
        && (ns == UINT32_MAX || ns == mark.ns)
        && ((lnum == 0 && sign_id == 0)
            || (sign_id == 0 && lnum == mark.pos.row + 1)
            || (lnum == 0 && sign_id == (int)mark.id)
            || (lnum == mark.pos.row + 1 && sign_id == (int)mark.id))) {
      if (mt_decor_sign(mark)) {
        kv_push(signs, mark);
      }
    }
    rs_marktree_itr_next(buf->b_marktree, itr);
  }
  if (kv_size(signs)) {
    qsort((void *)&kv_A(signs, 0), kv_size(signs), sizeof(MTKey), sign_row_cmp);
    for (size_t i = 0; i < kv_size(signs); i++) {
      tv_list_append_dict(l, nvim_sign_get_placed_info_dict_impl(&kv_A(signs, i)));
    }
    kv_destroy(signs);
  }
}
int nvim_sign_define_from_dict_impl(char *name, dict_T *dict)
{
  if (name == NULL) {
    name = tv_dict_get_string(dict, "name", false);
    if (name == NULL || name[0] == NUL) {
      return -1;
    }
  }
  char *icon = NULL;
  char *linehl = NULL;
  char *text = NULL;
  char *texthl = NULL;
  char *culhl = NULL;
  char *numhl = NULL;
  int prio = -1;
  if (dict != NULL) {
    icon = tv_dict_get_string(dict, "icon", false);
    linehl = tv_dict_get_string(dict, "linehl", false);
    text = tv_dict_get_string(dict, "text", false);
    texthl = tv_dict_get_string(dict, "texthl", false);
    culhl = tv_dict_get_string(dict, "culhl", false);
    numhl = tv_dict_get_string(dict, "numhl", false);
    prio = (int)tv_dict_get_number_def(dict, "priority", -1);
  }
  return rs_sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio) - 1;
}
void nvim_sign_define_multiple_impl(list_T *l, list_T *retlist)
{
  TV_LIST_ITER_CONST(l, li, {
    int retval = TV_LIST_ITEM_TV(li)->v_type == VAR_DICT
                 ? nvim_sign_define_from_dict_impl(NULL, TV_LIST_ITEM_TV(li)->vval.v_dict)
                 : (emsg(_(e_dictreq)), -1);
    tv_list_append_number(retlist, retval);
  });
}
int nvim_sign_place_from_dict_impl(typval_T *id_tv, typval_T *group_tv, typval_T *name_tv, typval_T *buf_tv, dict_T *dict)
{
  dictitem_T *di;
  int id = 0;
  bool notanum = false;
  if (id_tv == NULL) {
    di = tv_dict_find(dict, "id", -1);
    if (di != NULL) {
      id_tv = &di->di_tv;
    }
  }
  if (id_tv != NULL) {
    id = (int)tv_get_number_chk(id_tv, &notanum);
    if (notanum) {
      return -1;
    }
    if (id < 0) {
      emsg(_(e_invarg));
      return -1;
    }
  }
  char *group = NULL;
  if (group_tv == NULL) {
    di = tv_dict_find(dict, "group", -1);
    if (di != NULL) {
      group_tv = &di->di_tv;
    }
  }
  if (group_tv != NULL) {
    group = (char *)tv_get_string_chk(group_tv);
    if (group == NULL) {
      return -1;
    }
    if (group[0] == NUL) {
      group = NULL;
    }
  }
  char *name = NULL;
  if (name_tv == NULL) {
    di = tv_dict_find(dict, "name", -1);
    if (di != NULL) {
      name_tv = &di->di_tv;
    }
  }
  if (name_tv == NULL) {
    return -1;
  }
  name = (char *)tv_get_string_chk(name_tv);
  if (name == NULL) {
    return -1;
  }
  if (buf_tv == NULL) {
    di = tv_dict_find(dict, "buffer", -1);
    if (di != NULL) {
      buf_tv = &di->di_tv;
    }
  }
  if (buf_tv == NULL) {
    return -1;
  }
  buf_T *buf = get_buf_arg(buf_tv);
  if (buf == NULL) {
    return -1;
  }
  linenr_T lnum = 0;
  di = tv_dict_find(dict, "lnum", -1);
  if (di != NULL) {
    lnum = tv_get_lnum(&di->di_tv);
    if (lnum <= 0) {
      emsg(_(e_invarg));
      return -1;
    }
  }
  int prio = -1;
  di = tv_dict_find(dict, "priority", -1);
  if (di != NULL) {
    prio = (int)tv_get_number_chk(&di->di_tv, &notanum);
    if (notanum) {
      return -1;
    }
  }
  uint32_t uid = (uint32_t)id;
  if (rs_sign_place(&uid, group, name, buf, lnum, prio) == OK) {
    return (int)uid;
  }
  return -1;
}
int nvim_sign_unplace_from_dict_impl(typval_T *group_tv, dict_T *dict)
{
  dictitem_T *di;
  int id = 0;
  buf_T *buf = NULL;
  char *group = (group_tv != NULL) ? (char *)tv_get_string(group_tv)
                                   : tv_dict_get_string(dict, "group", false);
  if (group != NULL && group[0] == NUL) {
    group = NULL;
  }
  if (dict != NULL) {
    if ((di = tv_dict_find(dict, "buffer", -1)) != NULL) {
      buf = get_buf_arg(&di->di_tv);
      if (buf == NULL) {
        return -1;
      }
    }
    if (tv_dict_find(dict, "id", -1) != NULL) {
      id = (int)tv_dict_get_number(dict, "id");
      if (id <= 0) {
        emsg(_(e_invarg));
        return -1;
      }
    }
  }
  return rs_sign_unplace(buf, id, group, 0) - 1;
}
void nvim_sign_undefine_multiple_impl(list_T *l, list_T *retlist)
{
  TV_LIST_ITER_CONST(l, li, {
    char *name = (char *)tv_get_string_chk(TV_LIST_ITEM_TV(li));
    tv_list_append_number(retlist, name && rs_sign_undefine_by_name(name) == OK ? 0 : -1);
  });
}
void nvim_f_sign_define_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  if (argvars[0].v_type == VAR_LIST && argvars[1].v_type == VAR_UNKNOWN) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    nvim_sign_define_multiple_impl(argvars[0].vval.v_list, rettv->vval.v_list);
    return;
  }
  rettv->vval.v_number = -1;
  char *name = (char *)tv_get_string_chk(&argvars[0]);
  if (name == NULL) { return; }
  if (tv_check_for_opt_dict_arg(argvars, 1) == FAIL) { return; }
  dict_T *d = argvars[1].v_type == VAR_DICT ? argvars[1].vval.v_dict : NULL;
  rettv->vval.v_number = nvim_sign_define_from_dict_impl(name, d);
}
void nvim_f_sign_getdefined_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  tv_list_alloc_ret(rettv, 0);
  if (argvars[0].v_type == VAR_UNKNOWN) {
    sign_T *sp;
    map_foreach_value(&sign_map, sp, {
      tv_list_append_dict(rettv->vval.v_list, nvim_sign_get_info_dict_impl(sp));
    });
  } else {
    sign_T *sp = pmap_get(cstr_t)(&sign_map, tv_get_string(&argvars[0]));
    if (sp != NULL) {
      tv_list_append_dict(rettv->vval.v_list, nvim_sign_get_info_dict_impl(sp));
    }
  }
}
void nvim_f_sign_getplaced_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  buf_T *buf = NULL;
  linenr_T lnum = 0;
  int sign_id = 0;
  const char *group = NULL;
  bool notanum = false;
  tv_list_alloc_ret(rettv, 0);
  if (argvars[0].v_type != VAR_UNKNOWN) {
    buf = get_buf_arg(&argvars[0]);
    if (buf == NULL) {
      return;
    }
    if (argvars[1].v_type != VAR_UNKNOWN) {
      if (tv_check_for_nonnull_dict_arg(argvars, 1) == FAIL) {
        return;
      }
      dictitem_T *di;
      dict_T *dict = argvars[1].vval.v_dict;
      if ((di = tv_dict_find(dict, "lnum", -1)) != NULL) {
        lnum = tv_get_lnum(&di->di_tv);
        if (lnum <= 0) {
          return;
        }
      }
      if ((di = tv_dict_find(dict, "id", -1)) != NULL) {
        sign_id = (int)tv_get_number_chk(&di->di_tv, &notanum);
        if (notanum) {
          return;
        }
      }
      if ((di = tv_dict_find(dict, "group", -1)) != NULL) {
        group = tv_get_string_chk(&di->di_tv);
        if (group == NULL) {
          return;
        }
        if (*group == NUL) {
          group = NULL;
        }
      }
    }
  }
  sign_get_placed(buf, lnum, sign_id, group, rettv->vval.v_list);
}
void nvim_f_sign_jump_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  rettv->vval.v_number = -1;
  bool notanum = false;
  int id = (int)tv_get_number_chk(&argvars[0], &notanum);
  if (notanum) { return; }
  if (id <= 0) { emsg(_(e_invarg)); return; }
  char *group = (char *)tv_get_string_chk(&argvars[1]);
  if (group == NULL) { return; }
  if (group[0] == NUL) { group = NULL; }
  buf_T *buf = get_buf_arg(&argvars[2]);
  if (buf == NULL) { return; }
  rettv->vval.v_number = nvim_sign_jump_impl(id, group, buf);
}
void nvim_f_sign_place_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  dict_T *dict = NULL;
  rettv->vval.v_number = -1;
  if (argvars[4].v_type != VAR_UNKNOWN) {
    if (tv_check_for_nonnull_dict_arg(argvars, 4) == FAIL) {
      return;
    }
    dict = argvars[4].vval.v_dict;
  }
  rettv->vval.v_number = nvim_sign_place_from_dict_impl(&argvars[0], &argvars[1], &argvars[2], &argvars[3], dict);
}
void nvim_f_sign_placelist_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);
  if (argvars[0].v_type != VAR_LIST) { emsg(_(e_listreq)); return; }
  TV_LIST_ITER_CONST(argvars[0].vval.v_list, li, {
    int sign_id = TV_LIST_ITEM_TV(li)->v_type == VAR_DICT
                  ? nvim_sign_place_from_dict_impl(NULL, NULL, NULL, NULL, TV_LIST_ITEM_TV(li)->vval.v_dict)
                  : (emsg(_(e_dictreq)), -1);
    tv_list_append_number(rettv->vval.v_list, sign_id);
  });
}
void nvim_f_sign_undefine_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  if (argvars[0].v_type == VAR_LIST && argvars[1].v_type == VAR_UNKNOWN) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    nvim_sign_undefine_multiple_impl(argvars[0].vval.v_list, rettv->vval.v_list);
    return;
  }
  rettv->vval.v_number = -1;
  if (argvars[0].v_type == VAR_UNKNOWN) {
    nvim_sign_free_all_impl(); rettv->vval.v_number = 0;
  } else {
    const char *name = tv_get_string_chk(&argvars[0]);
    if (name == NULL) { return; }
    if (rs_sign_undefine_by_name(name) == OK) { rettv->vval.v_number = 0; }
  }
}
void nvim_f_sign_unplace_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  dict_T *dict = NULL;
  rettv->vval.v_number = -1;
  if (tv_check_for_string_arg(argvars, 0) == FAIL || tv_check_for_opt_dict_arg(argvars, 1) == FAIL) { return; }
  if (argvars[1].v_type != VAR_UNKNOWN) { dict = argvars[1].vval.v_dict; }
  rettv->vval.v_number = nvim_sign_unplace_from_dict_impl(&argvars[0], dict);
}
void nvim_f_sign_unplacelist_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);
  if (argvars[0].v_type != VAR_LIST) { emsg(_(e_listreq)); return; }
  TV_LIST_ITER_CONST(argvars[0].vval.v_list, li, {
    int retval = TV_LIST_ITEM_TV(li)->v_type == VAR_DICT
                 ? nvim_sign_unplace_from_dict_impl(NULL, TV_LIST_ITEM_TV(li)->vval.v_dict)
                 : (emsg(_(e_dictreq)), -1);
    tv_list_append_number(rettv->vval.v_list, retval);
  });
}
