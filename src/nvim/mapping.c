// mapping.c: Code for mappings and abbreviations.

#include <assert.h>
#include <lauxlib.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/api/keysets_defs.h"
#include "nvim/api/private/converter.h"
#include "nvim/api/private/defs.h"
#include "nvim/api/private/dispatch.h"
#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/cmdexpand.h"
#include "nvim/cmdexpand_defs.h"
#include "nvim/errors.h"
#include "nvim/eval.h"
#include "nvim/eval/typval.h"
#include "nvim/eval/typval_defs.h"
#include "nvim/eval/userfunc.h"
#include "nvim/eval/vars.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/getchar.h"
#include "nvim/getchar_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/highlight_defs.h"
#include "nvim/keycodes.h"
#include "nvim/lua/executor.h"
#include "nvim/macros_defs.h"
#include "nvim/mapping.h"
#include "nvim/mapping_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memory.h"
#include "nvim/memory_defs.h"
#include "nvim/message.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/pos_defs.h"
#include "nvim/runtime.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"

// Rust FFI declarations
extern int rs_get_map_mode(char **cmdp, int forceit);
extern int rs_get_map_mode_string(const char *mode_string, int abbr);

// Rust-implemented dict/query functions (Phase 2)
extern Dict mapblock_fill_dict(const mapblock_T *mp, const char *lhsrawalt,
                               int buffer_value, bool abbr, bool compatible,
                               Arena *arena);
// Rust-implemented VimL eval functions (Phase 2)
void f_maparg(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_mapcheck(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
// Rust-implemented VimL eval functions (Phase 3)
void f_mapset(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
void f_maplist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);

/// List used for abbreviations.
static mapblock_T *first_abbr = NULL;  // first entry in abbrlist

// Each mapping is put in one of the MAX_MAPHASH hash lists,
// to speed up finding it.
static mapblock_T *(maphash[MAX_MAPHASH]) = { 0 };
// Hash index for mode+first-char; NV modes use c1, IC modes use c1^0x80.
#define MAP_HASH(mode, \
                 c1) (((mode) & \
                       (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | \
                        MODE_OP_PENDING | MODE_TERMINAL)) ? (c1) : ((c1) ^ 0x80))

/// All possible |:map-arguments| usable in a |:map| command. @see mapblock_T
struct map_arguments {
  bool buffer;
  bool expr;
  bool noremap;
  bool nowait;
  bool script;
  bool silent;
  bool unique;
  bool replace_keycodes;
  char lhs[MAXMAPLEN + 1];  ///< {lhs}; lhs_len > MAXMAPLEN signals truncation
  size_t lhs_len;
  char alt_lhs[MAXMAPLEN + 1];  ///< unsimplified {lhs}; alt_lhs_len==0 if no simplification
  size_t alt_lhs_len;
  char *rhs;  ///< The {rhs} of the mapping.
  size_t rhs_len;
  LuaRef rhs_lua;  ///< lua function as {rhs}
  bool rhs_is_noop;  ///< True when the {rhs} should be <Nop>.
  char *orig_rhs;  ///< The original text of the {rhs}.
  size_t orig_rhs_len;
  char *desc;  ///< map description
};
typedef struct map_arguments MapArguments;

// Rust FFI declarations that depend on MapArguments
extern int rs_str_to_mapargs(const char *strargs, int is_unmap, MapArguments *mapargs);
extern int rs_set_maparg_lhs_rhs(const char *orig_lhs, size_t orig_lhs_len,
                                  const char *orig_rhs, size_t orig_rhs_len,
                                  LuaRef rhs_lua, const char *cpo_val,
                                  MapArguments *mapargs);
extern void rs_set_maparg_rhs(const char *orig_rhs, size_t orig_rhs_len,
                               LuaRef rhs_lua, int sid, const char *cpo_val,
                               MapArguments *mapargs);
extern int rs_buf_do_map(int maptype, MapArguments *args, int mode, int is_abbrev, buf_T *buf);
extern mapblock_T *rs_map_add(buf_T *buf, int is_buf_local, const char *keys,
                              MapArguments *args, int noremap, int mode,
                              int is_abbr, int sid, int lnum, int simplified);

#define MAP_ARGUMENTS_INIT { false, false, false, false, false, false, false, false, \
                             { 0 }, 0, { 0 }, 0, NULL, 0, LUA_NOREF, false, NULL, 0, NULL }

#include "mapping.c.generated.h"

static const char e_global_abbreviation_already_exists_for_str[]
  = N_("E224: Global abbreviation already exists for %s");
static const char e_global_mapping_already_exists_for_str[]
  = N_("E225: Global mapping already exists for %s");
static const char e_abbreviation_already_exists_for_str[]
  = N_("E226: Abbreviation already exists for %s");
static const char e_mapping_already_exists_for_str[]
  = N_("E227: Mapping already exists for %s");
static const char e_entries_missing_in_mapset_dict_argument[]
  = N_("E460: Entries missing in mapset() dict argument");
static const char e_illegal_map_mode_string_str[]
  = N_("E1276: Illegal map mode string: '%s'");



// f_mapset body deleted; implemented in Rust (Phase 3). See query.rs.

// f_maplist body deleted; implemented in Rust (Phase 3). See query.rs.



/// Set, tweak, or remove a mapping in a mode. Acts as the implementation for
/// functions like @ref nvim_buf_set_keymap.
///
/// Arguments are handled like @ref nvim_set_keymap unless noted.
/// @param  buffer    Buffer handle for a specific buffer, or 0 for the current
///                   buffer, or -1 to signify global behavior ("all buffers")
/// @param  is_unmap  When true, removes the mapping that matches {lhs}.
void modify_keymap(uint64_t channel_id, Buffer buffer, bool is_unmap, String mode, String lhs,
                   String rhs, Dict(keymap) *opts, Error *err)
{
  LuaRef lua_funcref = LUA_NOREF;
  bool global = (buffer == -1);
  if (global) {
    buffer = 0;
  }
  buf_T *target_buf = find_buffer_by_handle(buffer, err);

  if (!target_buf) {
    return;
  }

  const sctx_T save_current_sctx = api_set_sctx(channel_id);

  MapArguments parsed_args = MAP_ARGUMENTS_INIT;
  if (opts) {
    parsed_args.nowait = opts->nowait;
    parsed_args.noremap = opts->noremap;
    parsed_args.silent = opts->silent;
    parsed_args.script = opts->script;
    parsed_args.expr = opts->expr;
    parsed_args.unique = opts->unique;
    parsed_args.replace_keycodes = opts->replace_keycodes;
    if (HAS_KEY(opts, keymap, callback)) {
      lua_funcref = opts->callback;
      opts->callback = LUA_NOREF;
    }
    if (HAS_KEY(opts, keymap, desc)) {
      parsed_args.desc = string_to_cstr(opts->desc);
    }
  }
  parsed_args.buffer = !global;

  if (parsed_args.replace_keycodes && !parsed_args.expr) {
    api_set_error(err, kErrorTypeValidation,  "\"replace_keycodes\" requires \"expr\"");
    goto fail_and_free;
  }

  if (!rs_set_maparg_lhs_rhs(lhs.data, lhs.size,
                             rhs.data, rhs.size, lua_funcref,
                             p_cpo, &parsed_args)) {
    api_set_error(err, kErrorTypeValidation,  "LHS exceeds maximum map length: %s", lhs.data);
    goto fail_and_free;
  }

  if (parsed_args.lhs_len > MAXMAPLEN || parsed_args.alt_lhs_len > MAXMAPLEN) {
    api_set_error(err, kErrorTypeValidation,  "LHS exceeds maximum map length: %s", lhs.data);
    goto fail_and_free;
  }

  char *p = mode.size > 0 ? mode.data : "m";
  bool forceit = *p == '!';
  // integer value of the mapping mode, to be passed to do_map()
  int mode_val = rs_get_map_mode(&p, forceit);
  if (forceit) {
    assert(p == mode.data);
    p++;
  }
  bool is_abbrev = (mode_val & (MODE_INSERT | MODE_CMDLINE)) != 0 && *p == 'a';
  if (is_abbrev) {
    p++;
  }
  if (mode.size > 0 && (size_t)(p - mode.data) != mode.size) {
    api_set_error(err, kErrorTypeValidation, "Invalid mode shortname: \"%s\"", mode.data);
    goto fail_and_free;
  }

  if (parsed_args.lhs_len == 0) {
    api_set_error(err, kErrorTypeValidation, "Invalid (empty) LHS");
    goto fail_and_free;
  }

  bool is_noremap = parsed_args.noremap;
  assert(!(is_unmap && is_noremap));

  if (!is_unmap && lua_funcref == LUA_NOREF
      && (parsed_args.rhs_len == 0 && !parsed_args.rhs_is_noop)) {
    if (rhs.size == 0) {  // assume that the user wants RHS to be a <Nop>
      parsed_args.rhs_is_noop = true;
    } else {
      abort();  // should never happen
    }
  } else if (is_unmap && (parsed_args.rhs_len || parsed_args.rhs_lua != LUA_NOREF)) {
    if (parsed_args.rhs_len) {
      api_set_error(err, kErrorTypeValidation,
                    "Gave nonempty RHS in unmap command: %s", parsed_args.rhs);
    } else {
      api_set_error(err, kErrorTypeValidation, "Gave nonempty RHS for unmap");
    }
    goto fail_and_free;
  }

  // rs_buf_do_map() reads noremap/unmap as its own argument.
  int maptype_val = MAPTYPE_MAP;
  if (is_unmap) {
    maptype_val = MAPTYPE_UNMAP;
  } else if (is_noremap) {
    maptype_val = MAPTYPE_NOREMAP;
  }

  switch (rs_buf_do_map(maptype_val, &parsed_args, mode_val, is_abbrev ? 1 : 0, target_buf)) {
  case 0:
    break;
  case 1:
    api_set_error(err, kErrorTypeException, e_invarg, 0);
    goto fail_and_free;
  case 2:
    api_set_error(err, kErrorTypeException, e_nomap, 0);
    goto fail_and_free;
  case 5:
    api_set_error(err, kErrorTypeException,
                  is_abbrev ? e_abbreviation_already_exists_for_str
                            : e_mapping_already_exists_for_str, lhs.data);
    goto fail_and_free;
    break;
  case 6:
    api_set_error(err, kErrorTypeException,
                  is_abbrev ? e_global_abbreviation_already_exists_for_str
                            : e_global_mapping_already_exists_for_str, lhs.data);
    goto fail_and_free;
  default:
    assert(false && "Unrecognized return code!");
    goto fail_and_free;
  }  // switch

fail_and_free:
  current_sctx = save_current_sctx;
  NLUA_CLEAR_REF(parsed_args.rhs_lua);
  xfree(parsed_args.rhs);
  xfree(parsed_args.orig_rhs);
  xfree(parsed_args.desc);
}

/// Get an array containing dictionaries describing mappings
/// based on mode and buffer id
///
/// @param  mode  The abbreviation for the mode
/// @param  buf  The buffer to get the mapping array. NULL for global
/// @returns Array of maparg()-like dictionaries describing mappings
ArrayOf(Dict) keymap_array(String mode, buf_T *buf, Arena *arena)
{
  ArrayBuilder mappings = KV_INITIAL_VALUE;
  kvi_init(mappings);

  char *p = mode.size > 0 ? mode.data : "m";
  bool forceit = *p == '!';
  // Convert the string mode to the integer mode stored within each mapblock.
  int int_mode = rs_get_map_mode(&p, forceit);
  if (forceit) {
    assert(p == mode.data);
    p++;
  }
  bool is_abbrev = (int_mode & (MODE_INSERT | MODE_CMDLINE)) != 0 && *p == 'a';

  // Determine the desired buffer value
  int buffer_value = (buf == NULL) ? 0 : buf->handle;

  for (int i = 0; i < (is_abbrev ? 1 : MAX_MAPHASH); i++) {
    for (const mapblock_T *current_maphash = is_abbrev
                                             ? (buf ? buf->b_first_abbr : first_abbr)
                                             : (buf ? buf->b_maphash[i] : maphash[i]);
         current_maphash;
         current_maphash = current_maphash->m_next) {
      if (current_maphash->m_simplified) {
        continue;
      }
      // Check for correct mode
      if (int_mode & current_maphash->m_mode) {
        kvi_push(mappings, DICT_OBJ(mapblock_fill_dict(current_maphash,
                                                       current_maphash->m_alt
                                                       ? current_maphash->m_alt->m_keys : NULL,
                                                       buffer_value,
                                                       is_abbrev, false, arena)));
      }
    }
  }

  return arena_take_arraybuilder(arena, &mappings);
}

// Rust FFI accessor functions
mapblock_T *nvim_get_maphash_entry(int index) { return (index >= 0 && index < MAX_MAPHASH) ? maphash[index] : NULL; }
mapblock_T *nvim_get_first_abbr(void) { return first_abbr; }
mapblock_T *nvim_buf_get_maphash_entry(buf_T *buf, int index) { return (buf && index >= 0 && index < MAX_MAPHASH) ? buf->b_maphash[index] : NULL; }
mapblock_T *nvim_buf_get_first_abbr(buf_T *buf) { return buf ? buf->b_first_abbr : NULL; }
const char *nvim_mapping_get_p_cpo(void) { return p_cpo; }

// Error string accessors for Rust (local static strings not accessible from Rust)
const char *nvim_mapping_e_abbr_exists(int abbr)
{
  return abbr ? _(e_abbreviation_already_exists_for_str)
              : _(e_mapping_already_exists_for_str);
}
const char *nvim_mapping_e_global_abbr_exists(int abbr)
{
  return abbr ? _(e_global_abbreviation_already_exists_for_str)
              : _(e_global_mapping_already_exists_for_str);
}

// Static assertions for MapArguments struct layout (Rust #[repr(C)] must match)
_Static_assert(sizeof(MapArguments) == 184, "MapArguments size mismatch with Rust");
_Static_assert(offsetof(struct map_arguments, buffer) == 0, "MapArguments.buffer offset mismatch");
_Static_assert(offsetof(struct map_arguments, lhs) == 8, "MapArguments.lhs offset mismatch");
_Static_assert(offsetof(struct map_arguments, rhs) == 136, "MapArguments.rhs offset mismatch");
_Static_assert(offsetof(struct map_arguments, rhs_lua) == 152, "MapArguments.rhs_lua offset mismatch");
_Static_assert(offsetof(struct map_arguments, desc) == 176, "MapArguments.desc offset mismatch");
// Static assertions for mapblock_T layout (Rust MapblockT #[repr(C)] must match)
_Static_assert(sizeof(mapblock_T) == 104, "mapblock_T size mismatch with Rust MapblockT");
_Static_assert(offsetof(mapblock_T, m_next) == 0, "mapblock_T.m_next offset mismatch");
_Static_assert(offsetof(mapblock_T, m_alt) == 8, "mapblock_T.m_alt offset mismatch");
_Static_assert(offsetof(mapblock_T, m_keys) == 16, "mapblock_T.m_keys offset mismatch");
_Static_assert(offsetof(mapblock_T, m_str) == 24, "mapblock_T.m_str offset mismatch");
_Static_assert(offsetof(mapblock_T, m_orig_str) == 32, "mapblock_T.m_orig_str offset mismatch");
_Static_assert(offsetof(mapblock_T, m_luaref) == 40, "mapblock_T.m_luaref offset mismatch");
_Static_assert(offsetof(mapblock_T, m_keylen) == 44, "mapblock_T.m_keylen offset mismatch");
_Static_assert(offsetof(mapblock_T, m_mode) == 48, "mapblock_T.m_mode offset mismatch");
_Static_assert(offsetof(mapblock_T, m_simplified) == 52, "mapblock_T.m_simplified offset mismatch");
_Static_assert(offsetof(mapblock_T, m_noremap) == 56, "mapblock_T.m_noremap offset mismatch");
_Static_assert(offsetof(mapblock_T, m_silent) == 60, "mapblock_T.m_silent offset mismatch");
_Static_assert(offsetof(mapblock_T, m_nowait) == 61, "mapblock_T.m_nowait offset mismatch");
_Static_assert(offsetof(mapblock_T, m_expr) == 62, "mapblock_T.m_expr offset mismatch");
_Static_assert(offsetof(mapblock_T, m_script_ctx) == 64, "mapblock_T.m_script_ctx offset mismatch");
_Static_assert(offsetof(mapblock_T, m_desc) == 88, "mapblock_T.m_desc offset mismatch");
_Static_assert(offsetof(mapblock_T, m_replace_keycodes) == 96, "mapblock_T.m_replace_keycodes offset mismatch");

// Langmap C accessors for Rust
uint8_t nvim_langmap_mapchar_get(int index) { return (index >= 0 && index < 256) ? langmap_mapchar[index] : (uint8_t)index; }
void nvim_langmap_mapchar_set(int index, uint8_t value) { if (index >= 0 && index < 256) { langmap_mapchar[index] = value; } }
int nvim_mapping_utf_ptr2char(const char *p) { return utf_ptr2char(p); }
int nvim_mapping_utfc_ptr2len(const char *p) { return utfc_ptr2len(p); }

// Phase 3 accessors: f_mapset / f_maplist helpers
LuaRef nvim_ufunc_get_luaref(const ufunc_T *fp) { return fp->uf_luaref; }
int nvim_mapping_dictitem_tv_type(const dictitem_T *di) { return (int)di->di_tv.v_type; }
const char *nvim_mapping_dictitem_tv_vstring(const dictitem_T *di) { return di->di_tv.vval.v_string; }
// Error emitters for f_mapset (avoid exposing variadic semsg to Rust)
void nvim_mapping_emsg_entries_missing(void) { emsg(_(e_entries_missing_in_mapset_dict_argument)); }
void nvim_mapping_semsg_illegal_map_mode(const char *which) { semsg(_(e_illegal_map_mode_string_str), which); }
void nvim_mapping_emsg_invarg(void) { emsg(_(e_invarg)); }
void nvim_set_maphash_entry(int index, mapblock_T *mp) { if (index >= 0 && index < MAX_MAPHASH) { maphash[index] = mp; } }
void nvim_set_first_abbr(mapblock_T *mp) { first_abbr = mp; }
void nvim_buf_set_maphash_entry(buf_T *buf, int index, mapblock_T *mp) { if (buf && index >= 0 && index < MAX_MAPHASH) { buf->b_maphash[index] = mp; } }
void nvim_buf_set_first_abbr(buf_T *buf, mapblock_T *mp) { if (buf) { buf->b_first_abbr = mp; } }

void nvim_mapping_set_no_abbr(int val) { no_abbr = val != 0; }
int nvim_mapping_buf_get_mapped_ctrl_c(buf_T *buf) { return buf ? buf->b_mapped_ctrl_c : 0; }
void nvim_mapping_buf_set_mapped_ctrl_c(buf_T *buf, int val) { if (buf) { buf->b_mapped_ctrl_c = val; } }
void nvim_mapping_msg_ext_set_kind_list_cmd(void) { msg_ext_set_kind("list_cmd"); }
void nvim_mapping_msg_no_mapping(int is_abbr)
{
  msg(is_abbr ? _("No abbreviation found") : _("No mapping found"), 0);
}
int nvim_vim_iswordp(const char *p) { return vim_iswordp(p); }
void nvim_langmap_format_error(char *buf, size_t buflen, int msgid, const char *arg)
{
  if (msgid == 357) {
    snprintf(buf, buflen, _("E357: 'langmap': Matching character missing for %s"), arg);
  } else if (msgid == 358) {
    snprintf(buf, buflen, _("E358: 'langmap': Extra characters after semicolon: %s"), arg);
  }
}
void nvim_mapping_vim_unescape_ks(char *s) { vim_unescape_ks(s); }
int nvim_mapping_get_state(void) { return State; }

// Accessors for eval_map_expr / check_abbr (Phase 3)
int nvim_mapping_get_typebuf_no_abbr_cnt(void) { return typebuf.tb_no_abbr_cnt; }
void nvim_mapping_add_typebuf_no_abbr_cnt(int delta) { typebuf.tb_no_abbr_cnt += delta; }
void nvim_mapping_get_curwin_cursor(int32_t *lnum, int *col, int *coladd)
{
  *lnum = curwin->w_cursor.lnum;
  *col = curwin->w_cursor.col;
  *coladd = curwin->w_cursor.coladd;
}
void nvim_mapping_set_curwin_cursor(int32_t lnum, int col, int coladd)
{
  curwin->w_cursor.lnum = lnum;
  curwin->w_cursor.col = col;
  curwin->w_cursor.coladd = coladd;
}
void nvim_mapping_semsg_lua_err(char *msg)
{
  semsg_multiline("emsg", "E5108: %s", msg);
}
