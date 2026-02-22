// sign.c: functions for managing with signs

#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/extmark.h"
#include "nvim/api/private/defs.h"
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
#include "nvim/fold.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/grid.h"
#include "nvim/highlight_defs.h"
#include "nvim/highlight_group.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/marktree.h"
#include "nvim/marktree_defs.h"
#include "nvim/mbyte.h"
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

// Rust fold FFI declaration
extern void rs_foldOpenCursor(void);

// Rust FFI declarations
extern int rs_sign_cmd_idx(const char *cmd);
extern int rs_sign_item_cmp(int priority1, uint32_t id1, uint32_t add_id1,
                            int priority2, uint32_t id2, uint32_t add_id2);
extern int rs_sign_name_valid(const char *name);
extern int rs_sign_priority_valid(int prio);
extern int rs_sign_effective_priority(int prio);
extern int rs_sign_row_cmp(int row1, int row2);
extern bool rs_sign_id_valid(int id);
extern int rs_sign_clamp_lnum(int lnum, int max_line);
extern bool rs_sign_lnum_valid(int lnum);
extern int64_t rs_group_get_ns(const char *group, int (*ns_lookup)(const char *));
extern const char *rs_sign_get_display_name(DecorSignHighlight *sh);
extern bool rs_sign_buffer_has_signs(const buf_T *buf);
extern size_t rs_describe_sign_text(char *buf, size_t buflen, const schar_T *sign_text);
extern int rs_init_sign_text(schar_T *sign_text, const char *text, int remove_backslash);
extern void rs_buf_set_sign(buf_T *buf, uint32_t *id, const char *group, int prio, linenr_T lnum,
                            sign_T *sp);
extern linenr_T rs_buf_mod_sign(buf_T *buf, uint32_t *id, const char *group, int prio, sign_T *sp);
extern int rs_buf_findsign(buf_T *buf, int id, const char *group);
extern int rs_buf_delete_signs(buf_T *buf, const char *group, int id, linenr_T atlnum);
extern int rs_sign_define_by_name(const char *name, const char *icon, const char *text,
                                  const char *linehl, const char *texthl,
                                  const char *culhl, const char *numhl, int prio);
extern int rs_sign_undefine_by_name(const char *name);
extern void rs_free_signs(void);
extern int rs_sign_place(uint32_t *id, const char *group, const char *name, buf_T *buf,
                         linenr_T lnum, int prio);
extern int rs_sign_unplace(buf_T *buf, int id, const char *group, linenr_T atlnum);
extern linenr_T rs_sign_jump(int id, const char *group, buf_T *buf);
extern void rs_sign_list_placed(buf_T *rbuf, const char *group);
extern void rs_sign_list_defined(sign_T *sp);
extern void rs_sign_list_by_name(const char *name);
extern void rs_sign_define_cmd(char *name, char *cmdline);
extern void rs_sign_place_cmd(buf_T *buf, linenr_T lnum, char *name, int id, char *group,
                               int prio);
extern void rs_sign_unplace_cmd(buf_T *buf, linenr_T lnum, const char *name, int id, char *group);
extern void rs_sign_jump_cmd(buf_T *buf, linenr_T lnum, const char *name, int id, char *group);
extern int rs_parse_sign_cmd_args(int cmd, char *arg, char **name, int *id, char **group,
                                   int *prio, buf_T **buf, linenr_T *lnum);
extern void rs_ex_sign(void *eap);
extern char *rs_get_sign_name(void *xp, int idx);
extern void rs_set_context_in_sign_cmd(void *xp, char *arg);
extern void *rs_sign_get_info_dict(sign_T *sp);
extern void *rs_sign_get_placed_info_dict(void *mark_ptr);
extern void *rs_get_buffer_signs(buf_T *buf);
extern void rs_sign_get_placed_in_buf(buf_T *buf, linenr_T lnum, int sign_id, const char *group,
                                       void *retlist);
extern void rs_sign_get_placed(buf_T *buf, linenr_T lnum, int id, const char *group,
                                void *retlist);
extern int rs_sign_define_from_dict(char *name, void *dict);
extern void rs_sign_define_multiple(void *l, void *retlist);
extern int rs_sign_place_from_dict(void *id_tv, void *group_tv, void *name_tv, void *buf_tv,
                                    void *dict);
extern int rs_sign_unplace_from_dict(void *group_tv, void *dict);
extern void rs_sign_undefine_multiple(void *l, void *retlist);
extern void rs_f_sign_define(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_getdefined(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_getplaced(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_jump(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_place(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_placelist(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_undefine(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_unplace(void *argvars, void *rettv, void *fptr);
extern void rs_f_sign_unplacelist(void *argvars, void *rettv, void *fptr);

static PMap(cstr_t) sign_map = MAP_INIT;
static kvec_t(Integer) sign_ns = KV_INITIAL_VALUE;

static char *cmds[] = {
  "define",
#define SIGNCMD_DEFINE  0
  "undefine",
#define SIGNCMD_UNDEFINE 1
  "list",
#define SIGNCMD_LIST    2
  "place",
#define SIGNCMD_PLACE   3
  "unplace",
#define SIGNCMD_UNPLACE 4
  "jump",
#define SIGNCMD_JUMP    5
  NULL
#define SIGNCMD_LAST    6
};

/// C accessor: look up namespace by name (wraps map_get for namespace_ids)
static int nvim_namespace_lookup_fn(const char *name)
{
  return map_get(String, int)(&namespace_ids, cstr_as_string(name));
}

// Convert the supplied "group" to a namespace filter
static int64_t group_get_ns(const char *group)
{
  return rs_group_get_ns(group, nvim_namespace_lookup_fn);
}

static const char *sign_get_name(DecorSignHighlight *sh)
{
  return rs_sign_get_display_name(sh);
}

/// Create or update a sign extmark.
static void buf_set_sign(buf_T *buf, uint32_t *id, char *group, int prio, linenr_T lnum, sign_T *sp)
{
  rs_buf_set_sign(buf, id, group, prio, lnum, sp);
}

/// For an existing, placed sign with "id", modify the sign, group or priority.
/// Returns the line number of the sign, or zero if the sign is not found.
static linenr_T buf_mod_sign(buf_T *buf, uint32_t *id, char *group, int prio, sign_T *sp)
{
  return rs_buf_mod_sign(buf, id, group, prio, sp);
}

/// Find the line number of the sign with the requested id in group 'group'.
static int buf_findsign(buf_T *buf, int id, char *group)
{
  return rs_buf_findsign(buf, id, group);
}

/// qsort() function to sort signs by line number, priority, id and recency.
static int sign_row_cmp(const void *p1, const void *p2)
{
  const MTKey *s1 = (MTKey *)p1;
  const MTKey *s2 = (MTKey *)p2;

  // Compare rows first using Rust helper
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

/// Delete the specified sign(s)
static int buf_delete_signs(buf_T *buf, char *group, int id, linenr_T atlnum)
{
  return rs_buf_delete_signs(buf, group, id, atlnum);
}

bool buf_has_signs(const buf_T *buf)
{
  return rs_sign_buffer_has_signs((const buf_T *)buf);
}

/// List placed signs for "rbuf".  If "rbuf" is NULL do it for all buffers.
static void sign_list_placed(buf_T *rbuf, char *group)
{
  rs_sign_list_placed(rbuf, group);
}

/// Find index of a ":sign" subcmd from its name.
/// "*end_cmd" must be writable.
///
/// @param begin_cmd  begin of sign subcmd
/// @param end_cmd  just after sign subcmd
static int sign_cmd_idx(char *begin_cmd, char *end_cmd)
{
  char save = *end_cmd;
  *end_cmd = NUL;
  int idx = rs_sign_cmd_idx(begin_cmd);
  *end_cmd = save;
  return idx;
}

/// buf must be SIGN_WIDTH * MAX_SCHAR_SIZE (no extra +1 needed)
size_t describe_sign_text(char *buf, schar_T *sign_text)
{
  return rs_describe_sign_text(buf, SIGN_WIDTH * MAX_SCHAR_SIZE, sign_text);
}

/// Initialize the "text" for a new sign and store in "sign_text".
/// "sp" is NULL for signs added through nvim_buf_set_extmark().
int init_sign_text(sign_T *sp, schar_T *sign_text, char *text)
{
  // sp != NULL means: remove backslashes (sign define cmd) and emit error on failure
  int result = rs_init_sign_text(sign_text, text, sp != NULL ? 1 : 0);
  if (result != 0 && sp != NULL) {
    semsg(_("E239: Invalid sign text: %s"), text);
  }
  return result == 0 ? OK : FAIL;
}

/// Define a new sign or update an existing sign
static int sign_define_by_name(char *name, char *icon, char *text, char *linehl, char *texthl,
                               char *culhl, char *numhl, int prio)
{
  return rs_sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio);
}

/// Free the sign specified by 'name'.
static int sign_undefine_by_name(const char *name)
{
  int result = rs_sign_undefine_by_name(name);
  if (result == FAIL) {
    semsg(_("E155: Unknown sign: %s"), name);
  }
  return result;
}

/// List one sign.
static void sign_list_defined(sign_T *sp)
{
  rs_sign_list_defined(sp);
}

/// List the signs matching 'name'
static void sign_list_by_name(char *name)
{
  rs_sign_list_by_name(name);
}

/// Place a sign at the specified file location or update a sign.
static int sign_place(uint32_t *id, char *group, char *name, buf_T *buf, linenr_T lnum, int prio)
{
  return rs_sign_place(id, group, name, buf, lnum, prio);
}

/// Unplace the specified sign for a single or all buffers
static int sign_unplace(buf_T *buf, int id, char *group, linenr_T atlnum)
{
  return rs_sign_unplace(buf, id, group, atlnum);
}

/// Jump to a sign.
static linenr_T sign_jump(int id, char *group, buf_T *buf)
{
  return rs_sign_jump(id, group, buf);
}

/// ":sign define {name} ..." command
static void sign_define_cmd(char *name, char *cmdline)
{
  rs_sign_define_cmd(name, cmdline);
}

/// ":sign place" command
static void sign_place_cmd(buf_T *buf, linenr_T lnum, char *name, int id, char *group, int prio)
{
  rs_sign_place_cmd(buf, lnum, name, id, group, prio);
}

/// ":sign unplace" command
static void sign_unplace_cmd(buf_T *buf, linenr_T lnum, const char *name, int id, char *group)
{
  rs_sign_unplace_cmd(buf, lnum, name, id, group);
}

/// Jump to a placed sign commands
static void sign_jump_cmd(buf_T *buf, linenr_T lnum, const char *name, int id, char *group)
{
  rs_sign_jump_cmd(buf, lnum, name, id, group);
}

/// Parse the command line arguments for the ":sign place", ":sign unplace" and
/// ":sign jump" commands.
static int parse_sign_cmd_args(int cmd, char *arg, char **name, int *id, char **group, int *prio,
                               buf_T **buf, linenr_T *lnum)
{
  return rs_parse_sign_cmd_args(cmd, arg, name, id, group, prio, buf, lnum);
}

/// ":sign" command
void ex_sign(exarg_T *eap)
{
  rs_ex_sign(eap);
}

/// Get dictionary of information for a defined sign "sp"
static dict_T *sign_get_info_dict(sign_T *sp)
{
  return rs_sign_get_info_dict(sp);
}

/// Get dictionary of information for placed sign "mark"
static dict_T *sign_get_placed_info_dict(MTKey mark)
{
  return rs_sign_get_placed_info_dict(&mark);
}

/// Returns information about signs placed in a buffer as list of dicts.
list_T *get_buffer_signs(buf_T *buf)
  FUNC_ATTR_NONNULL_RET FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT
{
  return rs_get_buffer_signs(buf);
}

/// @return  information about all the signs placed in a buffer
static void sign_get_placed_in_buf(buf_T *buf, linenr_T lnum, int sign_id, const char *group,
                                   list_T *retlist)
{
  rs_sign_get_placed_in_buf(buf, lnum, sign_id, group, retlist);
}

/// Get a list of signs placed in buffer 'buf'.
static void sign_get_placed(buf_T *buf, linenr_T lnum, int id, const char *group, list_T *retlist)
{
  rs_sign_get_placed(buf, lnum, id, group, retlist);
}

void free_signs(void)
{
  rs_free_signs();
}

static enum {
  EXP_SUBCMD,   // expand :sign sub-commands
  EXP_DEFINE,   // expand :sign define {name} args
  EXP_PLACE,    // expand :sign place {id} args
  EXP_LIST,     // expand :sign place args
  EXP_UNPLACE,  // expand :sign unplace"
  EXP_SIGN_NAMES,   // expand with name of placed signs
  EXP_SIGN_GROUPS,  // expand with name of placed sign groups
} expand_what;

/// Function given to ExpandGeneric() to obtain the sign command expansion.
char *get_sign_name(expand_T *xp, int idx)
{
  return rs_get_sign_name(xp, idx);
}

/// Handle command line completion for :sign command.
void set_context_in_sign_cmd(expand_T *xp, char *arg)
{
  rs_set_context_in_sign_cmd(xp, arg);
}

/// Define a sign from dict
static int sign_define_from_dict(char *name, dict_T *dict)
{
  return rs_sign_define_from_dict(name, dict);
}

/// Define multiple signs
static void sign_define_multiple(list_T *l, list_T *retlist)
{
  rs_sign_define_multiple(l, retlist);
}

/// "sign_define()" function
void f_sign_define(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_define(argvars, rettv, &fptr);
}

/// "sign_getdefined()" function
void f_sign_getdefined(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_getdefined(argvars, rettv, &fptr);
}

/// "sign_getplaced()" function
void f_sign_getplaced(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_getplaced(argvars, rettv, &fptr);
}

/// "sign_jump()" function
void f_sign_jump(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_jump(argvars, rettv, &fptr);
}

/// Place a sign from dict
static int sign_place_from_dict(typval_T *id_tv, typval_T *group_tv, typval_T *name_tv,
                                typval_T *buf_tv, dict_T *dict)
{
  return rs_sign_place_from_dict(id_tv, group_tv, name_tv, buf_tv, dict);
}

/// "sign_place()" function
void f_sign_place(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_place(argvars, rettv, &fptr);
}

/// "sign_placelist()" function
void f_sign_placelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_placelist(argvars, rettv, &fptr);
}

/// Undefine multiple signs
static void sign_undefine_multiple(list_T *l, list_T *retlist)
{
  rs_sign_undefine_multiple(l, retlist);
}

/// "sign_undefine()" function
void f_sign_undefine(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_undefine(argvars, rettv, &fptr);
}

/// Unplace a sign from dict
static int sign_unplace_from_dict(typval_T *group_tv, dict_T *dict)
{
  return rs_sign_unplace_from_dict(group_tv, dict);
}

/// "sign_unplace()" function
void f_sign_unplace(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_unplace(argvars, rettv, &fptr);
}

/// "sign_unplacelist()" function
void f_sign_unplacelist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  rs_f_sign_unplacelist(argvars, rettv, &fptr);
}

// =============================================================================
// Accessor functions for Rust sign crate
// =============================================================================

/// Get sign by name from the sign map
sign_T *nvim_sign_map_get(const char *name)
{
  if (name == NULL) {
    return NULL;
  }
  return pmap_get(cstr_t)(&sign_map, name);
}

/// Check if sign exists in the map
int nvim_sign_map_has(const char *name)
{
  if (name == NULL) {
    return 0;
  }
  return map_has(cstr_t, &sign_map, name) ? 1 : 0;
}

/// Get sign text highlight ID
int nvim_sign_get_text_hl(sign_T *sp)
{
  return sp ? sp->sn_text_hl : 0;
}

/// Get sign line highlight ID
int nvim_sign_get_line_hl(sign_T *sp)
{
  return sp ? sp->sn_line_hl : 0;
}

/// Get sign number highlight ID
int nvim_sign_get_num_hl(sign_T *sp)
{
  return sp ? sp->sn_num_hl : 0;
}

/// Get sign cursorline highlight ID
int nvim_sign_get_cul_hl(sign_T *sp)
{
  return sp ? sp->sn_cul_hl : 0;
}

/// Get sign icon
char *nvim_sign_get_icon(sign_T *sp)
{
  return sp ? sp->sn_icon : NULL;
}

/// Get sign name
char *nvim_sign_get_name(sign_T *sp)
{
  return sp ? sp->sn_name : NULL;
}

/// Get sign priority
int nvim_sign_get_priority(sign_T *sp)
{
  return sp ? sp->sn_priority : -1;
}

/// Build a DecorSignHighlight from sign properties and place/update an extmark.
/// This composite accessor keeps complex struct construction on the C side.
void nvim_sign_build_decor_and_set(buf_T *buf, uint32_t ns, uint32_t *id, int row,
                                   sign_T *sp, int prio)
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

/// Look up a sign in the marktree by namespace and ID.
/// Returns the 1-based line number, or 0 if not found.
linenr_T nvim_sign_marktree_lookup_row(buf_T *buf, uint32_t ns, uint32_t id)
{
  MTKey mark = rs_marktree_lookup_ns(buf->b_marktree, ns, id, false, NULL);
  return mark.pos.row + 1;
}

/// Get the line count of a buffer (for sign operations).
linenr_T nvim_sign_buf_line_count(buf_T *buf)
{
  return buf ? buf->b_ml.ml_line_count : 0;
}

/// Push a namespace ID onto the sign_ns kvec.
void nvim_sign_ns_push(Integer ns)
{
  kv_push(sign_ns, ns);
}

/// Create a namespace from a C string name.
int nvim_sign_create_namespace_cstr(const char *name)
{
  return (int)nvim_create_namespace(cstr_as_string(name));
}

/// Check if a namespace with the given name exists.
int nvim_sign_namespace_exists(const char *name)
{
  return map_get(String, int)(&namespace_ids, cstr_as_string(name)) ? 1 : 0;
}

/// Implement sign definition — allocate/update in sign_map, set all fields.
/// Returns OK on success, FAIL on failure.
/// Error messages are handled by caller.
int nvim_sign_define_by_name_impl(const char *name, const char *icon, const char *text,
                                  const char *linehl, const char *texthl,
                                  const char *culhl, const char *numhl, int prio)
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
            if (buf_has_signs(wp->w_buffer)) {
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

/// Undefine a sign by name — remove from map and free.
/// Returns OK on success, FAIL if not found (no error message emitted).
int nvim_sign_undefine_by_name_impl(const char *name)
{
  sign_T *sp = pmap_del(cstr_t)(&sign_map, name, NULL);
  if (sp == NULL) {
    return FAIL;
  }
  xfree(sp->sn_name);
  xfree(sp->sn_icon);
  xfree(sp);
  return OK;
}

/// Place a sign — composite accessor implementing the core logic.
/// Returns OK on success, FAIL on failure. Error messages stay in C.
int nvim_sign_place_impl(uint32_t *id, char *group, char *name, buf_T *buf, linenr_T lnum,
                         int prio)
{
  // Check for reserved character '*' in group name
  if (group != NULL && (*group == '*' || *group == NUL)) {
    return FAIL;
  }

  sign_T *sp = pmap_get(cstr_t)(&sign_map, name);
  if (sp == NULL) {
    semsg(_("E155: Unknown sign: %s"), name);
    return FAIL;
  }

  // Use the default priority value for this sign.
  prio = rs_sign_effective_priority(prio == -1 && sp->sn_priority != -1 ? sp->sn_priority : prio);

  if (lnum > 0) {
    buf_set_sign(buf, id, group, prio, lnum, sp);
  } else {
    lnum = buf_mod_sign(buf, id, group, prio, sp);
  }
  if (lnum <= 0) {
    semsg(_("E885: Not possible to change sign %s"), name);
    return FAIL;
  }

  return OK;
}

/// Unplace sign(s) from a single buffer — composite accessor.
int nvim_sign_unplace_inner_impl(buf_T *buf, int id, char *group, linenr_T atlnum)
{
  if (!buf_has_signs(buf)) {
    return FAIL;
  }

  if (id == 0 || atlnum > 0 || (group != NULL && *group == '*')) {
    if (!buf_delete_signs(buf, group, id, atlnum)) {
      return FAIL;
    }
  } else {
    int64_t ns = group_get_ns(group);
    if (ns < 0 || !extmark_del_id(buf, (uint32_t)ns, (uint32_t)id)) {
      return FAIL;
    }
  }

  return OK;
}

/// Unplace sign(s) from a single buffer or all buffers.
int nvim_sign_unplace_impl(buf_T *buf, int id, char *group, linenr_T atlnum)
{
  if (buf != NULL) {
    return nvim_sign_unplace_inner_impl(buf, id, group, atlnum);
  } else {
    int retval = OK;
    FOR_ALL_BUFFERS(cbuf) {
      if (!nvim_sign_unplace_inner_impl(cbuf, id, group, atlnum)) {
        retval = FAIL;
      }
    }
    return retval;
  }
}

/// Jump to a sign — composite accessor.
/// Returns lnum on success, -1 on failure. Error messages stay in C.
linenr_T nvim_sign_jump_impl(int id, char *group, buf_T *buf)
{
  linenr_T lnum = buf_findsign(buf, id, group);

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

/// Free all signs — iterate map and undefine each.
void nvim_sign_free_all_impl(void)
{
  cstr_t name;
  kvec_t(cstr_t) names = KV_INITIAL_VALUE;
  map_foreach_key(&sign_map, name, {
    kv_push(names, name);
  });
  for (size_t i = 0; i < kv_size(names); i++) {
    nvim_sign_undefine_by_name_impl(kv_A(names, i));
  }
  kv_destroy(names);
}

/// Perform the marktree iteration + sorting + deletion for sign removal.
/// This composite accessor keeps MarkTreeIter on the C side.
///
/// @param buf  buffer
/// @param ns  namespace filter (0 = global, UINT32_MAX = all)
/// @param id  sign ID filter (0 = all matching)
/// @param atlnum  line number (> 0 = specific line, <= 0 = any line)
/// @return OK on success, FAIL on failure
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

/// List placed signs for a buffer or all buffers — composite accessor.
/// Performs marktree iteration, sorting, and message output.
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
    if (buf_has_signs(buf)) {
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
            vim_snprintf(namebuf, MSG_BUF_LEN, _("  name=%s"), sign_get_name(sh));
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

/// List a sign definition — composite accessor.
/// Formats and outputs a single sign definition using message APIs.
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
    describe_sign_text(buf, sp->sn_text);
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

/// List sign by name — composite accessor.
/// Looks up sign in sign_map and delegates to nvim_sign_list_defined_impl.
/// Emits E155 error for unknown sign names.
void nvim_sign_list_by_name_impl(const char *name)
{
  sign_T *sp = pmap_get(cstr_t)(&sign_map, name);
  if (sp != NULL) {
    nvim_sign_list_defined_impl(sp);
  } else {
    semsg(_("E155: Unknown sign: %s"), name);
  }
}

/// ":sign define {name} ..." command — composite accessor.
/// Parses key=value pairs and calls sign_define_by_name.
void nvim_sign_define_cmd_impl(char *name, char *cmdline)
{
  char *icon = NULL;
  char *text = NULL;
  char *linehl = NULL;
  char *texthl = NULL;
  char *culhl = NULL;
  char *numhl = NULL;
  int prio = -1;

  // set values for a defined sign.
  while (true) {
    char *arg = skipwhite(cmdline);
    if (*arg == NUL) {
      break;
    }
    cmdline = skiptowhite_esc(arg);
    if (strncmp(arg, "icon=", 5) == 0) {
      icon = arg + 5;
    } else if (strncmp(arg, "text=", 5) == 0) {
      text = arg + 5;
    } else if (strncmp(arg, "linehl=", 7) == 0) {
      linehl = arg + 7;
    } else if (strncmp(arg, "texthl=", 7) == 0) {
      texthl = arg + 7;
    } else if (strncmp(arg, "culhl=", 6) == 0) {
      culhl = arg + 6;
    } else if (strncmp(arg, "numhl=", 6) == 0) {
      numhl = arg + 6;
    } else if (strncmp(arg, "priority=", 9) == 0) {
      prio = atoi(arg + 9);
    } else {
      semsg(_(e_invarg2), arg);
      return;
    }
    if (*cmdline == NUL) {
      break;
    }
    *cmdline++ = NUL;
  }

  sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio);
}

/// ":sign place" command — composite accessor.
void nvim_sign_place_cmd_impl(buf_T *buf, linenr_T lnum, char *name, int id, char *group, int prio)
{
  if (id <= 0) {
    if (lnum >= 0 || name != NULL || (group != NULL && *group == NUL)) {
      emsg(_(e_invarg));
    } else {
      sign_list_placed(buf, group);
    }
  } else {
    if (name == NULL || buf == NULL || (group != NULL && *group == NUL)) {
      emsg(_(e_invarg));
      return;
    }
    uint32_t uid = (uint32_t)id;
    sign_place(&uid, group, name, buf, lnum, prio);
  }
}

/// ":sign unplace" command — composite accessor.
void nvim_sign_unplace_cmd_impl(buf_T *buf, linenr_T lnum, const char *name, int id, char *group)
{
  if (lnum >= 0 || name != NULL || (group != NULL && *group == NUL)) {
    emsg(_(e_invarg));
    return;
  }

  if (id == -1) {
    lnum = curwin->w_cursor.lnum;
    buf = curwin->w_buffer;
  }

  if (!sign_unplace(buf, MAX(0, id), group, lnum) && lnum > 0) {
    emsg(_("E159: Missing sign number"));
  }
}

/// ":sign jump" command — composite accessor.
void nvim_sign_jump_cmd_impl(buf_T *buf, linenr_T lnum, const char *name, int id, char *group)
{
  if (name == NULL && group == NULL && id == -1) {
    emsg(_(e_argreq));
    return;
  }

  if (buf == NULL || (group != NULL && *group == NUL) || lnum >= 0 || name != NULL) {
    emsg(_(e_invarg));
    return;
  }

  sign_jump(id, group, buf);
}

/// Parse command line arguments — composite accessor.
int nvim_parse_sign_cmd_args_impl(int cmd, char *arg, char **name, int *id, char **group, int *prio,
                                  buf_T **buf, linenr_T *lnum)
{
  char *arg1 = arg;
  char *filename = NULL;
  bool lnum_arg = false;

  // first arg could be placed sign id
  if (ascii_isdigit(*arg)) {
    *id = getdigits_int(&arg, true, 0);
    if (!ascii_iswhite(*arg) && *arg != NUL) {
      *id = -1;
      arg = arg1;
    } else {
      arg = skipwhite(arg);
    }
  }

  while (*arg != NUL) {
    if (strncmp(arg, "line=", 5) == 0) {
      arg += 5;
      *lnum = atoi(arg);
      arg = skiptowhite(arg);
      lnum_arg = true;
    } else if (strncmp(arg, "*", 1) == 0 && cmd == SIGNCMD_UNPLACE) {
      if (*id != -1) {
        emsg(_(e_invarg));
        return FAIL;
      }
      *id = -2;
      arg = skiptowhite(arg + 1);
    } else if (strncmp(arg, "name=", 5) == 0) {
      arg += 5;
      char *namep = arg;
      arg = skiptowhite(arg);
      if (*arg != NUL) {
        *arg++ = NUL;
      }
      while (namep[0] == '0' && namep[1] != NUL) {
        namep++;
      }
      *name = namep;
    } else if (strncmp(arg, "group=", 6) == 0) {
      arg += 6;
      *group = arg;
      arg = skiptowhite(arg);
      if (*arg != NUL) {
        *arg++ = NUL;
      }
    } else if (strncmp(arg, "priority=", 9) == 0) {
      arg += 9;
      *prio = atoi(arg);
      arg = skiptowhite(arg);
    } else if (strncmp(arg, "file=", 5) == 0) {
      arg += 5;
      filename = arg;
      *buf = buflist_findname_exp(arg);
      break;
    } else if (strncmp(arg, "buffer=", 7) == 0) {
      arg += 7;
      filename = arg;
      *buf = buflist_findnr(getdigits_int(&arg, true, 0));
      if (*skipwhite(arg) != NUL) {
        semsg(_(e_trailing_arg), arg);
      }
      break;
    } else {
      emsg(_(e_invarg));
      return FAIL;
    }
    arg = skipwhite(arg);
  }

  if (filename != NULL && *buf == NULL) {
    semsg(_(e_invalid_buffer_name_str), filename);
    return FAIL;
  }

  // If the filename is not supplied for the sign place or the sign jump
  // command, then use the current buffer.
  if (filename == NULL && ((cmd == SIGNCMD_PLACE && lnum_arg) || cmd == SIGNCMD_JUMP)) {
    *buf = curwin->w_buffer;
  }
  return OK;
}

/// ":sign" command — composite accessor.
void nvim_ex_sign_impl(exarg_T *eap)
{
  char *arg = eap->arg;

  // Parse the subcommand.
  char *p = skiptowhite(arg);
  int idx = sign_cmd_idx(arg, p);
  if (idx == SIGNCMD_LAST) {
    semsg(_("E160: Unknown sign command: %s"), arg);
    return;
  }
  arg = skipwhite(p);

  if (idx <= SIGNCMD_LIST) {
    // Define, undefine or list signs.
    if (idx == SIGNCMD_LIST && *arg == NUL) {
      // ":sign list": list all defined signs
      sign_T *sp;
      map_foreach_value(&sign_map, sp, {
        sign_list_defined(sp);
      });
    } else if (*arg == NUL) {
      emsg(_("E156: Missing sign name"));
    } else {
      // Isolate the sign name.  If it's a number skip leading zeroes,
      // so that "099" and "99" are the same sign.  But keep "0".
      p = skiptowhite(arg);
      if (*p != NUL) {
        *p++ = NUL;
      }
      while (arg[0] == '0' && arg[1] != NUL) {
        arg++;
      }

      if (idx == SIGNCMD_DEFINE) {
        sign_define_cmd(arg, p);
      } else if (idx == SIGNCMD_LIST) {
        // ":sign list {name}"
        sign_list_by_name(arg);
      } else {
        // ":sign undefine {name}"
        sign_undefine_by_name(arg);
      }

      return;
    }
  } else {
    int id = -1;
    linenr_T lnum = -1;
    char *name = NULL;
    char *group = NULL;
    int prio = -1;
    buf_T *buf = NULL;

    // Parse command line arguments
    if (parse_sign_cmd_args(idx, arg, &name, &id, &group, &prio, &buf, &lnum) == FAIL) {
      return;
    }

    if (idx == SIGNCMD_PLACE) {
      sign_place_cmd(buf, lnum, name, id, group, prio);
    } else if (idx == SIGNCMD_UNPLACE) {
      sign_unplace_cmd(buf, lnum, name, id, group);
    } else if (idx == SIGNCMD_JUMP) {
      sign_jump_cmd(buf, lnum, name, id, group);
    }
  }
}

// =============================================================================
// Command completion composite accessors
// =============================================================================

/// Get the n'th sign name from the sign_map.
char *nvim_sign_get_nth_name(int idx)
{
  cstr_t name;
  int current_idx = 0;
  map_foreach_key(&sign_map, name, {
    if (current_idx++ == idx) {
      return (char *)name;
    }
  });
  return NULL;
}

/// Get the n'th sign group name.
char *nvim_sign_get_nth_group_name(int idx)
{
  if (idx < (int)kv_size(sign_ns)) {
    return (char *)describe_ns((NS)kv_A(sign_ns, idx), "");
  }
  return NULL;
}

/// Set the expand_what static variable.
void nvim_sign_set_expand_what(int val)
{
  expand_what = val;
}

/// Get the expand_what static variable.
int nvim_sign_get_expand_what(void)
{
  return expand_what;
}

/// Get a sign subcommand name by index.
char *nvim_sign_get_cmd_name(int idx)
{
  return cmds[idx];
}

/// Set expand_T xp_context field.
void nvim_sign_set_xp_context(expand_T *xp, int ctx)
{
  xp->xp_context = ctx;
}

/// Get expand_T xp_context field.
int nvim_sign_get_xp_context(expand_T *xp)
{
  return xp->xp_context;
}

/// Set expand_T xp_pattern field.
void nvim_sign_set_xp_pattern(expand_T *xp, char *pat)
{
  xp->xp_pattern = pat;
}

/// Get expand_T xp_pattern field.
char *nvim_sign_get_xp_pattern(expand_T *xp)
{
  return xp->xp_pattern;
}

/// Wrap skipwhite for Rust.
char *nvim_sign_skipwhite(char *p)
{
  return skipwhite(p);
}

/// Wrap skiptowhite for Rust.
char *nvim_sign_skiptowhite(char *p)
{
  return skiptowhite(p);
}

/// Wrap vim_strchr for Rust.
char *nvim_sign_vim_strchr(char *p, int c)
{
  return vim_strchr(p, c);
}

/// Wrap ascii_isdigit for Rust.
int nvim_sign_ascii_isdigit(int c)
{
  return ascii_isdigit(c);
}

/// get_sign_name composite accessor — does the full switch on expand_what.
char *nvim_get_sign_name_impl(expand_T *xp, int idx)
{
  switch (expand_what) {
  case EXP_SUBCMD:
    return cmds[idx];
  case EXP_DEFINE: {
    char *define_arg[] = { "culhl=", "icon=", "linehl=", "numhl=", "text=", "texthl=",
                           "priority=", NULL };
    return define_arg[idx];
  }
  case EXP_PLACE: {
    char *place_arg[] = { "line=", "name=", "group=", "priority=", "file=", "buffer=", NULL };
    return place_arg[idx];
  }
  case EXP_LIST: {
    char *list_arg[] = { "group=", "file=", "buffer=", NULL };
    return list_arg[idx];
  }
  case EXP_UNPLACE: {
    char *unplace_arg[] = { "group=", "file=", "buffer=", NULL };
    return unplace_arg[idx];
  }
  case EXP_SIGN_NAMES:
    return nvim_sign_get_nth_name(idx);
  case EXP_SIGN_GROUPS:
    return nvim_sign_get_nth_group_name(idx);
  default:
    return NULL;
  }
}

/// set_context_in_sign_cmd composite accessor — full completion context logic.
void nvim_set_context_in_sign_cmd_impl(expand_T *xp, char *arg)
{
  // Default: expand subcommands.
  xp->xp_context = EXPAND_SIGN;
  expand_what = EXP_SUBCMD;
  xp->xp_pattern = arg;

  char *end_subcmd = skiptowhite(arg);
  if (*end_subcmd == NUL) {
    return;
  }

  int cmd_idx = sign_cmd_idx(arg, end_subcmd);
  char *begin_subcmd_args = skipwhite(end_subcmd);

  // Loop until reaching last argument.
  char *last;
  char *p = begin_subcmd_args;
  do {
    p = skipwhite(p);
    last = p;
    p = skiptowhite(p);
  } while (*p != NUL);

  p = vim_strchr(last, '=');

  if (p == NULL) {
    xp->xp_pattern = last;
    switch (cmd_idx) {
    case SIGNCMD_DEFINE:
      expand_what = EXP_DEFINE;
      break;
    case SIGNCMD_PLACE:
      if (ascii_isdigit(*begin_subcmd_args)) {
        expand_what = EXP_PLACE;
      } else {
        expand_what = EXP_LIST;
      }
      break;
    case SIGNCMD_LIST:
    case SIGNCMD_UNDEFINE:
      expand_what = EXP_SIGN_NAMES;
      break;
    case SIGNCMD_JUMP:
    case SIGNCMD_UNPLACE:
      expand_what = EXP_UNPLACE;
      break;
    default:
      xp->xp_context = EXPAND_NOTHING;
    }
  } else {
    xp->xp_pattern = p + 1;
    switch (cmd_idx) {
    case SIGNCMD_DEFINE:
      if (strncmp(last, "texthl", 6) == 0
          || strncmp(last, "linehl", 6) == 0
          || strncmp(last, "culhl", 5) == 0
          || strncmp(last, "numhl", 5) == 0) {
        xp->xp_context = EXPAND_HIGHLIGHT;
      } else if (strncmp(last, "icon", 4) == 0) {
        xp->xp_context = EXPAND_FILES;
      } else {
        xp->xp_context = EXPAND_NOTHING;
      }
      break;
    case SIGNCMD_PLACE:
      if (strncmp(last, "name", 4) == 0) {
        expand_what = EXP_SIGN_NAMES;
      } else if (strncmp(last, "group", 5) == 0) {
        expand_what = EXP_SIGN_GROUPS;
      } else if (strncmp(last, "file", 4) == 0) {
        xp->xp_context = EXPAND_BUFFERS;
      } else {
        xp->xp_context = EXPAND_NOTHING;
      }
      break;
    case SIGNCMD_UNPLACE:
    case SIGNCMD_JUMP:
      if (strncmp(last, "group", 5) == 0) {
        expand_what = EXP_SIGN_GROUPS;
      } else if (strncmp(last, "file", 4) == 0) {
        xp->xp_context = EXPAND_BUFFERS;
      } else {
        xp->xp_context = EXPAND_NOTHING;
      }
      break;
    default:
      xp->xp_context = EXPAND_NOTHING;
    }
  }
}

// =============================================================================
// Phase 10: VimL function composite accessors
// =============================================================================

/// Build info dict for a defined sign — composite accessor.
dict_T *nvim_sign_get_info_dict_impl(sign_T *sp)
{
  dict_T *d = tv_dict_alloc();

  tv_dict_add_str(d, S_LEN("name"), sp->sn_name);

  if (sp->sn_icon != NULL) {
    tv_dict_add_str(d, S_LEN("icon"), sp->sn_icon);
  }
  if (sp->sn_text[0]) {
    char buf[SIGN_WIDTH * MAX_SCHAR_SIZE];
    describe_sign_text(buf, sp->sn_text);
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

/// Build info dict for a placed sign — composite accessor.
dict_T *nvim_sign_get_placed_info_dict_impl(MTKey *mark)
{
  dict_T *d = tv_dict_alloc();

  DecorSignHighlight *sh = decor_find_sign(mt_decor(*mark));

  tv_dict_add_str(d, S_LEN("name"), sign_get_name(sh));
  tv_dict_add_nr(d,  S_LEN("id"), (int)mark->id);
  tv_dict_add_str(d, S_LEN("group"), describe_ns((int)mark->ns, ""));
  tv_dict_add_nr(d,  S_LEN("lnum"), mark->pos.row + 1);
  tv_dict_add_nr(d,  S_LEN("priority"), sh->priority);
  return d;
}

/// Get buffer signs as list — composite accessor.
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

/// Get placed signs in buffer — composite accessor.
void nvim_sign_get_placed_in_buf_impl(buf_T *buf, linenr_T lnum, int sign_id, const char *group,
                                      list_T *retlist)
{
  dict_T *d = tv_dict_alloc();
  tv_list_append_dict(retlist, d);

  tv_dict_add_nr(d, S_LEN("bufnr"), buf->b_fnum);

  list_T *l = tv_list_alloc(kListLenMayKnow);
  tv_dict_add_list(d, S_LEN("signs"), l);

  int64_t ns = group_get_ns(group);
  if (!buf_has_signs(buf) || ns < 0) {
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

/// Get placed signs — composite accessor.
void nvim_sign_get_placed_impl(buf_T *buf, linenr_T lnum, int id, const char *group,
                               list_T *retlist)
{
  if (buf != NULL) {
    nvim_sign_get_placed_in_buf_impl(buf, lnum, id, group, retlist);
  } else {
    FOR_ALL_BUFFERS(cbuf) {
      if (buf_has_signs(cbuf)) {
        nvim_sign_get_placed_in_buf_impl(cbuf, 0, id, group, retlist);
      }
    }
  }
}

/// Define sign from dict — composite accessor.
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

  return sign_define_by_name(name, icon, text, linehl, texthl, culhl, numhl, prio) - 1;
}

/// Define multiple signs — composite accessor.
void nvim_sign_define_multiple_impl(list_T *l, list_T *retlist)
{
  TV_LIST_ITER_CONST(l, li, {
    int retval = -1;
    if (TV_LIST_ITEM_TV(li)->v_type == VAR_DICT) {
      retval = nvim_sign_define_from_dict_impl(NULL, TV_LIST_ITEM_TV(li)->vval.v_dict);
    } else {
      emsg(_(e_dictreq));
    }
    tv_list_append_number(retlist, retval);
  });
}

/// Place sign from dict — composite accessor.
int nvim_sign_place_from_dict_impl(typval_T *id_tv, typval_T *group_tv, typval_T *name_tv,
                                   typval_T *buf_tv, dict_T *dict)
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
  if (sign_place(&uid, group, name, buf, lnum, prio) == OK) {
    return (int)uid;
  }

  return -1;
}

/// Unplace sign from dict — composite accessor.
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

  return sign_unplace(buf, id, group, 0) - 1;
}

/// Undefine multiple signs — composite accessor.
void nvim_sign_undefine_multiple_impl(list_T *l, list_T *retlist)
{
  TV_LIST_ITER_CONST(l, li, {
    int retval = -1;
    char *name = (char *)tv_get_string_chk(TV_LIST_ITEM_TV(li));
    if (name != NULL && (sign_undefine_by_name(name) == OK)) {
      retval = 0;
    }
    tv_list_append_number(retlist, retval);
  });
}

/// f_sign_define — composite accessor.
void nvim_f_sign_define_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  if (argvars[0].v_type == VAR_LIST && argvars[1].v_type == VAR_UNKNOWN) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    sign_define_multiple(argvars[0].vval.v_list, rettv->vval.v_list);
    return;
  }

  rettv->vval.v_number = -1;

  char *name = (char *)tv_get_string_chk(&argvars[0]);
  if (name == NULL) {
    return;
  }

  if (tv_check_for_opt_dict_arg(argvars, 1) == FAIL) {
    return;
  }

  dict_T *d = argvars[1].v_type == VAR_DICT ? argvars[1].vval.v_dict : NULL;
  rettv->vval.v_number = sign_define_from_dict(name, d);
}

/// f_sign_getdefined — composite accessor.
void nvim_f_sign_getdefined_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  tv_list_alloc_ret(rettv, 0);

  if (argvars[0].v_type == VAR_UNKNOWN) {
    sign_T *sp;
    map_foreach_value(&sign_map, sp, {
      tv_list_append_dict(rettv->vval.v_list, sign_get_info_dict(sp));
    });
  } else {
    sign_T *sp = pmap_get(cstr_t)(&sign_map, tv_get_string(&argvars[0]));
    if (sp != NULL) {
      tv_list_append_dict(rettv->vval.v_list, sign_get_info_dict(sp));
    }
  }
}

/// f_sign_getplaced — composite accessor.
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

/// f_sign_jump — composite accessor.
void nvim_f_sign_jump_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  rettv->vval.v_number = -1;

  bool notanum = false;
  int id = (int)tv_get_number_chk(&argvars[0], &notanum);
  if (notanum) {
    return;
  }
  if (id <= 0) {
    emsg(_(e_invarg));
    return;
  }

  char *group = (char *)tv_get_string_chk(&argvars[1]);
  if (group == NULL) {
    return;
  }
  if (group[0] == NUL) {
    group = NULL;
  }

  buf_T *buf = get_buf_arg(&argvars[2]);
  if (buf == NULL) {
    return;
  }

  rettv->vval.v_number = sign_jump(id, group, buf);
}

/// f_sign_place — composite accessor.
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

  rettv->vval.v_number = sign_place_from_dict(&argvars[0], &argvars[1],
                                              &argvars[2], &argvars[3], dict);
}

/// f_sign_placelist — composite accessor.
void nvim_f_sign_placelist_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (argvars[0].v_type != VAR_LIST) {
    emsg(_(e_listreq));
    return;
  }

  TV_LIST_ITER_CONST(argvars[0].vval.v_list, li, {
    int sign_id = -1;
    if (TV_LIST_ITEM_TV(li)->v_type == VAR_DICT) {
      sign_id = sign_place_from_dict(NULL, NULL, NULL, NULL, TV_LIST_ITEM_TV(li)->vval.v_dict);
    } else {
      emsg(_(e_dictreq));
    }
    tv_list_append_number(rettv->vval.v_list, sign_id);
  });
}

/// f_sign_undefine — composite accessor.
void nvim_f_sign_undefine_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  if (argvars[0].v_type == VAR_LIST && argvars[1].v_type == VAR_UNKNOWN) {
    tv_list_alloc_ret(rettv, kListLenMayKnow);
    sign_undefine_multiple(argvars[0].vval.v_list, rettv->vval.v_list);
    return;
  }

  rettv->vval.v_number = -1;

  if (argvars[0].v_type == VAR_UNKNOWN) {
    free_signs();
    rettv->vval.v_number = 0;
  } else {
    const char *name = tv_get_string_chk(&argvars[0]);
    if (name == NULL) {
      return;
    }

    if (sign_undefine_by_name(name) == OK) {
      rettv->vval.v_number = 0;
    }
  }
}

/// f_sign_unplace — composite accessor.
void nvim_f_sign_unplace_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  dict_T *dict = NULL;

  rettv->vval.v_number = -1;

  if (tv_check_for_string_arg(argvars, 0) == FAIL
      || tv_check_for_opt_dict_arg(argvars, 1) == FAIL) {
    return;
  }

  if (argvars[1].v_type != VAR_UNKNOWN) {
    dict = argvars[1].vval.v_dict;
  }

  rettv->vval.v_number = sign_unplace_from_dict(&argvars[0], dict);
}

/// f_sign_unplacelist — composite accessor.
void nvim_f_sign_unplacelist_impl(typval_T *argvars, typval_T *rettv, EvalFuncData *fptr)
{
  tv_list_alloc_ret(rettv, kListLenMayKnow);

  if (argvars[0].v_type != VAR_LIST) {
    emsg(_(e_listreq));
    return;
  }

  TV_LIST_ITER_CONST(argvars[0].vval.v_list, li, {
    int retval = -1;
    if (TV_LIST_ITEM_TV(li)->v_type == VAR_DICT) {
      retval = sign_unplace_from_dict(NULL, TV_LIST_ITEM_TV(li)->vval.v_dict);
    } else {
      emsg(_(e_dictreq));
    }
    tv_list_append_number(rettv->vval.v_list, retval);
  });
}
