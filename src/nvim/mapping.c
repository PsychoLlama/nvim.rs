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
// Rust-implemented API functions (Phase 4)
void modify_keymap(uint64_t channel_id, Buffer buffer, bool is_unmap, String mode, String lhs,
                   String rhs, Dict(keymap) *opts, Error *err);
ArrayOf(Dict) keymap_array(String mode, buf_T *buf, Arena *arena);

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



// modify_keymap body deleted; implemented in Rust (Phase 4). See api.rs.

// keymap_array body deleted; implemented in Rust (Phase 4). See api.rs.

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

// Phase 4 accessors: modify_keymap / keymap_array helpers

// Dict(keymap) field accessors
bool nvim_mapping_keymap_opts_get_noremap(const Dict(keymap) *opts) { return opts->noremap; }
bool nvim_mapping_keymap_opts_get_nowait(const Dict(keymap) *opts) { return opts->nowait; }
bool nvim_mapping_keymap_opts_get_silent(const Dict(keymap) *opts) { return opts->silent; }
bool nvim_mapping_keymap_opts_get_script(const Dict(keymap) *opts) { return opts->script; }
bool nvim_mapping_keymap_opts_get_expr(const Dict(keymap) *opts) { return opts->expr; }
bool nvim_mapping_keymap_opts_get_unique(const Dict(keymap) *opts) { return opts->unique; }
bool nvim_mapping_keymap_opts_get_replace_keycodes(const Dict(keymap) *opts) { return opts->replace_keycodes; }
bool nvim_mapping_keymap_opts_has_callback(const Dict(keymap) *opts) { return HAS_KEY(opts, keymap, callback); }
LuaRef nvim_mapping_keymap_opts_take_callback(Dict(keymap) *opts) { LuaRef r = opts->callback; opts->callback = LUA_NOREF; return r; }
bool nvim_mapping_keymap_opts_has_desc(const Dict(keymap) *opts) { return HAS_KEY(opts, keymap, desc); }
const char *nvim_mapping_keymap_opts_get_desc_data(const Dict(keymap) *opts) { return opts->desc.data; }
size_t nvim_mapping_keymap_opts_get_desc_size(const Dict(keymap) *opts) { return opts->desc.size; }

// api_set_error wrappers (avoid variadic from Rust)
void nvim_mapping_api_set_error_validation(Error *err, const char *msg) { api_set_error(err, kErrorTypeValidation, "%s", msg); }
void nvim_mapping_api_set_error_validation_lhs(Error *err, const char *lhs) { api_set_error(err, kErrorTypeValidation, "LHS exceeds maximum map length: %s", lhs); }
void nvim_mapping_api_set_error_validation_mode(Error *err, const char *mode) { api_set_error(err, kErrorTypeValidation, "Invalid mode shortname: \"%s\"", mode); }
void nvim_mapping_api_set_error_validation_rhs_lhs(Error *err, const char *rhs) { api_set_error(err, kErrorTypeValidation, "Gave nonempty RHS in unmap command: %s", rhs); }
void nvim_mapping_api_set_error_exception_invarg(Error *err) { api_set_error(err, kErrorTypeException, "%s", e_invarg); }
void nvim_mapping_api_set_error_exception_nomap(Error *err) { api_set_error(err, kErrorTypeException, "%s", e_nomap); }
void nvim_mapping_api_set_error_exception(Error *err, const char *msg) { api_set_error(err, kErrorTypeException, "%s", msg); }
void nvim_mapping_api_set_error_exception_abbr(Error *err, int is_abbrev, int is_global, const char *lhs)
{
  if (is_global) {
    api_set_error(err, kErrorTypeException, is_abbrev
      ? _(e_global_abbreviation_already_exists_for_str)
      : _(e_global_mapping_already_exists_for_str), lhs);
  } else {
    api_set_error(err, kErrorTypeException, is_abbrev
      ? _(e_abbreviation_already_exists_for_str)
      : _(e_mapping_already_exists_for_str), lhs);
  }
}

// api_set_sctx / current_sctx save-restore helpers
sctx_T nvim_mapping_api_set_sctx(uint64_t channel_id) { return api_set_sctx(channel_id); }
void nvim_mapping_restore_sctx(sctx_T sctx) { current_sctx = sctx; }

// find_buffer_by_handle wrapper (returns NULL on error)
buf_T *nvim_mapping_find_buffer_by_handle(int buffer, Error *err) { return find_buffer_by_handle(buffer, err); }

// buf->handle accessor
int nvim_mapping_buf_handle(const buf_T *buf) { return buf->handle; }

// string_to_cstr wrapper (for desc field)
char *nvim_mapping_string_to_cstr_len(const char *data, size_t size) { return string_to_cstr((String){ .data = (char *)data, .size = size }); }

// ArrayBuilder wrappers for keymap_array (opaque void* handle)
void *nvim_mapping_array_builder_new(void) { ArrayBuilder *b = xmalloc(sizeof(*b)); kvi_init(*b); return b; }
void nvim_mapping_array_builder_push_dict(void *b, Dict d) { kvi_push(*(ArrayBuilder *)b, DICT_OBJ(d)); }
Array nvim_mapping_array_builder_finish(Arena *arena, void *b) { Array r = arena_take_arraybuilder(arena, (ArrayBuilder *)b); xfree(b); return r; }

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
