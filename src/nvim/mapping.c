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


/// Fill a Dict with all applicable maparg() like dictionaries.
static Dict mapblock_fill_dict(const mapblock_T *const mp, const char *lhsrawalt,
                               const int buffer_value, const bool abbr, const bool compatible,
                               Arena *arena)
  FUNC_ATTR_NONNULL_ARG(1)
{
  Dict dict = arena_dict(arena, 19);
  char *const lhs = str2special_arena(mp->m_keys, compatible, !compatible, arena);
  char *mapmode = arena_alloc(arena, 7, false);
  map_mode_to_chars(mp->m_mode, mapmode);
  int noremap_value;

  if (compatible) {
    noremap_value = !!mp->m_noremap;
  } else {
    noremap_value = mp->m_noremap == REMAP_SCRIPT ? 2 : !!mp->m_noremap;
  }

  if (mp->m_luaref != LUA_NOREF) {
    PUT_C(dict, "callback", LUAREF_OBJ(api_new_luaref(mp->m_luaref)));
  } else {
    String rhs = cstr_as_string(compatible
                                ? mp->m_orig_str
                                : str2special_arena(mp->m_str, false, true, arena));
    PUT_C(dict, "rhs", STRING_OBJ(rhs));
  }
  if (mp->m_desc != NULL) {
    PUT_C(dict, "desc", CSTR_AS_OBJ(mp->m_desc));
  }
  PUT_C(dict, "lhs", CSTR_AS_OBJ(lhs));
  PUT_C(dict, "lhsraw", CSTR_AS_OBJ(mp->m_keys));
  if (lhsrawalt != NULL) {
    PUT_C(dict, "lhsrawalt", CSTR_AS_OBJ(lhsrawalt));
  }
  PUT_C(dict, "noremap", INTEGER_OBJ(noremap_value));
  PUT_C(dict, "script", INTEGER_OBJ(mp->m_noremap == REMAP_SCRIPT ? 1 : 0));
  PUT_C(dict, "expr", INTEGER_OBJ(mp->m_expr ? 1 : 0));
  PUT_C(dict, "silent", INTEGER_OBJ(mp->m_silent ? 1 : 0));
  PUT_C(dict, "sid", INTEGER_OBJ(mp->m_script_ctx.sc_sid));
  PUT_C(dict, "scriptversion", INTEGER_OBJ(1));
  PUT_C(dict, "lnum", INTEGER_OBJ(mp->m_script_ctx.sc_lnum));
  PUT_C(dict, "buffer", INTEGER_OBJ(buffer_value));
  PUT_C(dict, "nowait", INTEGER_OBJ(mp->m_nowait ? 1 : 0));
  if (mp->m_replace_keycodes) {
    PUT_C(dict, "replace_keycodes", INTEGER_OBJ(1));
  }
  PUT_C(dict, "mode", CSTR_AS_OBJ(mapmode));
  PUT_C(dict, "abbr", INTEGER_OBJ(abbr ? 1 : 0));
  PUT_C(dict, "mode_bits", INTEGER_OBJ(mp->m_mode));

  return dict;
}

static void get_maparg(typval_T *argvars, typval_T *rettv, int exact)
{
  // Return empty string for failure.
  rettv->v_type = VAR_STRING;
  rettv->vval.v_string = NULL;

  char *keys = (char *)tv_get_string(&argvars[0]);
  if (*keys == NUL) {
    return;
  }

  const char *which;
  char buf[NUMBUFLEN];
  bool abbr = false;
  bool get_dict = false;

  if (argvars[1].v_type != VAR_UNKNOWN) {
    which = tv_get_string_buf_chk(&argvars[1], buf);
    if (argvars[2].v_type != VAR_UNKNOWN) {
      abbr = (bool)tv_get_number(&argvars[2]);
      if (argvars[3].v_type != VAR_UNKNOWN) {
        get_dict = (bool)tv_get_number(&argvars[3]);
      }
    }
  } else {
    which = "";
  }
  if (which == NULL) {
    return;
  }

  char *keys_buf = NULL;
  char *alt_keys_buf = NULL;
  bool did_simplify = false;
  const int flags = REPTERM_FROM_PART | REPTERM_DO_LT;
  const int mode = rs_get_map_mode((char **)&which, 0);

  char *keys_simplified = replace_termcodes(keys, strlen(keys), &keys_buf, 0,
                                            flags, &did_simplify, p_cpo);
  mapblock_T *mp = NULL;
  int buffer_local;
  LuaRef rhs_lua;
  char *rhs = check_map(keys_simplified, mode, exact, false, abbr, &mp, &buffer_local,
                        &rhs_lua);
  if (did_simplify) {
    // When the lhs is being simplified the not-simplified keys are
    // preferred for printing, like in do_map().
    replace_termcodes(keys, strlen(keys), &alt_keys_buf, 0,
                      flags | REPTERM_NO_SIMPLIFY, NULL, p_cpo);
    rhs = check_map(alt_keys_buf, mode, exact, false, abbr, &mp, &buffer_local, &rhs_lua);
  }

  if (!get_dict) {
    // Return a string.
    if (rhs != NULL) {
      if (*rhs == NUL) {
        rettv->vval.v_string = xstrdup("<Nop>");
      } else {
        rettv->vval.v_string = str2special_save(rhs, false, false);
      }
    } else if (rhs_lua != LUA_NOREF) {
      rettv->vval.v_string = nlua_funcref_str(mp->m_luaref, NULL);
    }
  } else {
    // Return a dictionary.
    if (mp != NULL && (rhs != NULL || rhs_lua != LUA_NOREF)) {
      Arena arena = ARENA_EMPTY;
      Dict dict = mapblock_fill_dict(mp, did_simplify ? keys_simplified : NULL,
                                     buffer_local, abbr, true, &arena);
      object_to_vim_take_luaref(&DICT_OBJ(dict), rettv, true, NULL);
      arena_mem_free(arena_finish(&arena));
    } else {
      // Return an empty dictionary.
      tv_dict_alloc_ret(rettv);
    }
  }

  xfree(keys_buf);
  xfree(alt_keys_buf);
}

/// "mapset()" function
void f_mapset(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const char *which;
  char buf[NUMBUFLEN];
  int is_abbr;
  dict_T *d;

  // If first arg is a dict, then that's the only arg permitted.
  const bool dict_only = argvars[0].v_type == VAR_DICT;

  if (dict_only) {
    d = argvars[0].vval.v_dict;
    which = tv_dict_get_string(d, "mode", false);
    is_abbr = (int)tv_dict_get_bool(d, "abbr", -1);
    if (which == NULL || is_abbr < 0) {
      emsg(_(e_entries_missing_in_mapset_dict_argument));
      return;
    }
  } else {
    which = tv_get_string_buf_chk(&argvars[0], buf);
    if (which == NULL) {
      return;
    }
    is_abbr = (int)tv_get_bool(&argvars[1]);
    if (tv_check_for_dict_arg(argvars, 2) == FAIL) {
      return;
    }
    d = argvars[2].vval.v_dict;
  }
  const int mode = rs_get_map_mode_string(which, is_abbr);
  if (mode == 0) {
    semsg(_(e_illegal_map_mode_string_str), which);
    return;
  }

  // Get the values in the same order as above in get_maparg().
  char *lhs = tv_dict_get_string(d, "lhs", false);
  char *lhsraw = tv_dict_get_string(d, "lhsraw", false);
  char *lhsrawalt = tv_dict_get_string(d, "lhsrawalt", false);
  char *orig_rhs = tv_dict_get_string(d, "rhs", false);
  LuaRef rhs_lua = LUA_NOREF;
  dictitem_T *callback_di = tv_dict_find(d, S_LEN("callback"));
  if (callback_di != NULL) {
    if (callback_di->di_tv.v_type == VAR_FUNC) {
      ufunc_T *fp = find_func(callback_di->di_tv.vval.v_string);
      if (fp != NULL && (fp->uf_flags & FC_LUAREF)) {
        rhs_lua = api_new_luaref(fp->uf_luaref);
        orig_rhs = "";
      }
    }
  }
  if (lhs == NULL || lhsraw == NULL || orig_rhs == NULL) {
    emsg(_(e_entries_missing_in_mapset_dict_argument));
    api_free_luaref(rhs_lua);
    return;
  }

  int noremap = tv_dict_get_number(d, "noremap") != 0 ? REMAP_NONE : 0;
  if (tv_dict_get_number(d, "script") != 0) {
    noremap = REMAP_SCRIPT;
  }
  MapArguments args = {
    .expr = tv_dict_get_number(d, "expr") != 0,
    .silent = tv_dict_get_number(d, "silent") != 0,
    .nowait = tv_dict_get_number(d, "nowait") != 0,
    .replace_keycodes = tv_dict_get_number(d, "replace_keycodes") != 0,
    .desc = tv_dict_get_string(d, "desc", true),
  };
  scid_T sid = (scid_T)tv_dict_get_number(d, "sid");
  linenr_T lnum = (linenr_T)tv_dict_get_number(d, "lnum");
  bool buffer = tv_dict_get_number(d, "buffer") != 0;
  // mode from the dict is not used

  rs_set_maparg_rhs(orig_rhs, strlen(orig_rhs), rhs_lua, sid, p_cpo, &args);

  // Delete any existing mapping for this lhs and mode.
  MapArguments unmap_args = MAP_ARGUMENTS_INIT;
  rs_set_maparg_lhs_rhs(lhs, strlen(lhs), "", 0, LUA_NOREF, p_cpo, &unmap_args);
  unmap_args.buffer = buffer;
  rs_buf_do_map(MAPTYPE_UNMAP_LHS, &unmap_args, mode, is_abbr ? 1 : 0, curbuf);
  xfree(unmap_args.rhs);
  xfree(unmap_args.orig_rhs);

  mapblock_T *mp_result[2] = { NULL, NULL };

  mp_result[0] = rs_map_add(curbuf, buffer ? 1 : 0, lhsraw, &args,
                            noremap, mode, is_abbr, sid, lnum, 0);
  if (lhsrawalt != NULL) {
    mp_result[1] = rs_map_add(curbuf, buffer ? 1 : 0, lhsrawalt, &args,
                              noremap, mode, is_abbr, sid, lnum, 1);
  }

  if (mp_result[0] != NULL && mp_result[1] != NULL) {
    mp_result[0]->m_alt = mp_result[1];
    mp_result[1]->m_alt = mp_result[0];
  }
}

/// "maplist()" function
void f_maplist(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const int flags = REPTERM_FROM_PART | REPTERM_DO_LT;
  const bool abbr = argvars[0].v_type != VAR_UNKNOWN && tv_get_bool(&argvars[0]);

  tv_list_alloc_ret(rettv, kListLenUnknown);

  // Do it twice: once for global maps and once for local maps.
  for (int buffer_local = 0; buffer_local <= 1; buffer_local++) {
    for (int hash = 0; hash < 256; hash++) {
      mapblock_T *mp;
      if (abbr) {
        if (hash > 0) {  // there is only one abbr list
          break;
        }
        if (buffer_local) {
          mp = curbuf->b_first_abbr;
        } else {
          mp = first_abbr;
        }
      } else if (buffer_local) {
        mp = curbuf->b_maphash[hash];
      } else {
        mp = maphash[hash];
      }
      for (; mp; mp = mp->m_next) {
        if (mp->m_simplified) {
          continue;
        }

        char *keys_buf = NULL;
        bool did_simplify = false;

        Arena arena = ARENA_EMPTY;
        char *lhs = str2special_arena(mp->m_keys, true, false, &arena);
        replace_termcodes(lhs, strlen(lhs), &keys_buf, 0, flags, &did_simplify,
                          p_cpo);

        Dict dict = mapblock_fill_dict(mp, did_simplify ? keys_buf : NULL, buffer_local, abbr, true,
                                       &arena);
        typval_T d = TV_INITIAL_VALUE;
        object_to_vim_take_luaref(&DICT_OBJ(dict), &d, true, NULL);
        assert(d.v_type == VAR_DICT);
        tv_list_append_dict(rettv->vval.v_list, d.vval.v_dict);
        arena_mem_free(arena_finish(&arena));
        xfree(keys_buf);
      }
    }
  }
}

/// "maparg()" function
void f_maparg(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { get_maparg(argvars, rettv, true); }

/// "mapcheck()" function
void f_mapcheck(typval_T *argvars, typval_T *rettv, EvalFuncData fptr) { get_maparg(argvars, rettv, false); }


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
