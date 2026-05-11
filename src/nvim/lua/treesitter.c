// lua bindings for treesitter.
// NB: this file mostly contains a generic lua interface for treesitter
// trees and nodes, and could be broken out as a reusable lua package

#include <assert.h>
#include <ctype.h>
#include <lauxlib.h>
#include <limits.h>
#include <lua.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <tree_sitter/api.h>
#include <uv.h>

#include "nvim/os/time.h"

#ifdef HAVE_WASMTIME
# include <wasm.h>

# include "nvim/os/fs.h"
#endif

#include "nvim/api/private/helpers.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer_defs.h"
#include "nvim/globals.h"
#include "nvim/lua/treesitter.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/pos_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"

#define TS_META_PARSER "treesitter_parser"
#define TS_META_TREE "treesitter_tree"
#define TS_META_NODE "treesitter_node"
#define TS_META_QUERY "treesitter_query"
#define TS_META_QUERYCURSOR "treesitter_querycursor"
#define TS_META_QUERYMATCH "treesitter_querymatch"

typedef struct {
  LuaRef cb;
  lua_State *lstate;
  bool lex;
  bool parse;
} TSLuaLoggerOpts;

typedef struct {
  // We derive TSNode's, TSQueryCursor's, etc., from the TSTree, so it must not be mutated.
  const TSTree *tree;
} TSLuaTree;

typedef struct {
  uint64_t parse_start_time;
  uint64_t timeout_threshold_ns;
} TSLuaParserCallbackPayload;

#include "lua/treesitter.c.generated.h"

// Rust implementations (nvim-treesitter crate)
extern int rs_ts_node_tostring(lua_State *L);
extern int rs_ts_node_eq(lua_State *L);
extern int rs_ts_node_id(lua_State *L);
extern int rs_ts_node_named(lua_State *L);
extern int rs_ts_node_missing(lua_State *L);
extern int rs_ts_node_extra(lua_State *L);
extern int rs_ts_node_has_changes(lua_State *L);
extern int rs_ts_node_has_error(lua_State *L);
extern int rs_ts_node_type(lua_State *L);
extern int rs_ts_node_symbol(lua_State *L);
extern int rs_ts_node_child_count(lua_State *L);
extern int rs_ts_node_named_child_count(lua_State *L);
extern int rs_ts_node_byte_length(lua_State *L);
extern int rs_ts_node_equal(lua_State *L);
extern int rs_ts_node_sexpr(lua_State *L);
extern int rs_ts_node_range(lua_State *L);
extern int rs_ts_node_start(lua_State *L);
extern int rs_ts_node_end(lua_State *L);
extern int rs_ts_node_parent(lua_State *L);
extern int rs_ts_node_next_sibling(lua_State *L);
extern int rs_ts_node_prev_sibling(lua_State *L);
extern int rs_ts_node_next_named_sibling(lua_State *L);
extern int rs_ts_node_prev_named_sibling(lua_State *L);
extern int rs_ts_node_child(lua_State *L);
extern int rs_ts_node_named_child(lua_State *L);
extern int rs_ts_node_descendant_for_range(lua_State *L);
extern int rs_ts_node_named_descendant_for_range(lua_State *L);
extern int rs_ts_node_child_with_descendant(lua_State *L);
extern int rs_ts_node_root(lua_State *L);
extern int rs_ts_node_tree(lua_State *L);
extern int rs_ts_node_field(lua_State *L);
extern int rs_ts_node_named_children(lua_State *L);
extern int rs_ts_node_next_child(lua_State *L);
extern int rs_ts_node_iter_children(lua_State *L);
extern int rs_ts_has_ancestor(lua_State *L);
extern int rs_ts_tree_tostring(lua_State *L);
extern int rs_ts_tree_copy(lua_State *L);
extern int rs_ts_tree_get_ranges(lua_State *L);
extern int rs_ts_tree_gc(lua_State *L);
extern int rs_ts_tree_root(lua_State *L);
extern int rs_ts_tree_edit(lua_State *L);
extern int rs_ts_query_tostring(lua_State *L);
extern int rs_ts_query_gc(lua_State *L);
extern int rs_ts_query_disable_capture(lua_State *L);
extern int rs_ts_query_disable_pattern(lua_State *L);
extern int rs_ts_query_inspect(lua_State *L);
extern int rs_ts_querycursor_gc(lua_State *L);
extern int rs_ts_querycursor_remove_match(lua_State *L);
extern int rs_ts_querycursor_next_match(lua_State *L);
extern int rs_ts_querycursor_next_capture(lua_State *L);
extern int rs_ts_querymatch_info(lua_State *L);
extern int rs_ts_querymatch_captures(lua_State *L);
extern int rs_ts_parser_tostring(lua_State *L);
extern int rs_ts_parser_reset(lua_State *L);
extern void rs_ts_query_err_string(const char *src, int error_offset, uint32_t error_type,
                                   char *err_buf, size_t errlen);

static PMap(cstr_t) langs = MAP_INIT;

#ifdef HAVE_WASMTIME
static wasm_engine_t *wasmengine;
static TSWasmStore *ts_wasmstore;
#endif

// TSLanguage

static int tslua_has_language(lua_State *L)
{
  const char *lang_name = luaL_checkstring(L, 1);
  lua_pushboolean(L, map_has(cstr_t, &langs, lang_name));
  return 1;
}

#ifdef HAVE_WASMTIME
static char *read_file(const char *path, size_t *len)
  FUNC_ATTR_MALLOC
{
  FILE *file = os_fopen(path, "r");
  if (file == NULL) {
    return NULL;
  }
  fseek(file, 0L, SEEK_END);
  *len = (size_t)ftell(file);
  fseek(file, 0L, SEEK_SET);
  char *data = xmalloc(*len);
  if (fread(data, *len, 1, file) != 1) {
    xfree(data);
    fclose(file);
    return NULL;
  }
  fclose(file);
  return data;
}

static const char *wasmerr_to_str(TSWasmErrorKind werr)
{
  switch (werr) {
  case TSWasmErrorKindParse:
    return "PARSE";
  case TSWasmErrorKindCompile:
    return "COMPILE";
  case TSWasmErrorKindInstantiate:
    return "INSTANTIATE";
  case TSWasmErrorKindAllocate:
    return "ALLOCATE";
  default:
    return "UNKNOWN";
  }
}
#endif

#ifdef HAVE_WASMTIME
static int tslua_add_language_from_wasm(lua_State *L) { return add_language(L, true); }
#endif

// Creates the language into the internal language map.
//
// Returns true if the language is correctly loaded in the language map
static int tslua_add_language_from_object(lua_State *L) { return add_language(L, false); }

static const TSLanguage *load_language_from_object(lua_State *L, const char *path,
                                                   const char *lang_name, const char *symbol)
{
  uv_lib_t lib;
  if (uv_dlopen(path, &lib)) {
    xstrlcpy(IObuff, uv_dlerror(&lib), sizeof(IObuff));
    uv_dlclose(&lib);
    luaL_error(L, "Failed to load parser for language '%s': uv_dlopen: %s", lang_name, IObuff);
  }

  char symbol_buf[128];
  snprintf(symbol_buf, sizeof(symbol_buf), "tree_sitter_%s", symbol);

  TSLanguage *(*lang_parser)(void);
  if (uv_dlsym(&lib, symbol_buf, (void **)&lang_parser)) {
    xstrlcpy(IObuff, uv_dlerror(&lib), sizeof(IObuff));
    uv_dlclose(&lib);
    luaL_error(L, "Failed to load parser: uv_dlsym: %s", IObuff);
  }

  TSLanguage *lang = lang_parser();

  if (lang == NULL) {
    uv_dlclose(&lib);
    luaL_error(L, "Failed to load parser %s: internal error", path);
  }

  return lang;
}

static const TSLanguage *load_language_from_wasm(lua_State *L, const char *path,
                                                 const char *lang_name)
{
#ifndef HAVE_WASMTIME
  luaL_error(L, "Not supported");
  return NULL;
#else
  if (wasmengine == NULL) {
    wasmengine = wasm_engine_new();
  }
  assert(wasmengine != NULL);

  TSWasmError werr = { 0 };
  if (ts_wasmstore == NULL) {
    ts_wasmstore = ts_wasm_store_new(wasmengine, &werr);
  }

  if (werr.kind > 0) {
    luaL_error(L, "Failed to create WASM store: (%s) %s", wasmerr_to_str(werr.kind), werr.message);
  }

  size_t file_size = 0;
  char *data = read_file(path, &file_size);

  if (data == NULL) {
    luaL_error(L, "Unable to read file", path);
  }

  const TSLanguage *lang = ts_wasm_store_load_language(ts_wasmstore, lang_name, data,
                                                       (uint32_t)file_size, &werr);

  xfree(data);

  if (werr.kind > 0) {
    luaL_error(L, "Failed to load WASM parser %s: (%s) %s", path, wasmerr_to_str(werr.kind),
               werr.message);
  }

  if (lang == NULL) {
    luaL_error(L, "Failed to load parser %s: internal error", path);
  }

  return lang;
#endif
}

static int add_language(lua_State *L, bool is_wasm)
{
  const char *path = luaL_checkstring(L, 1);
  const char *lang_name = luaL_checkstring(L, 2);
  const char *symbol_name = lang_name;

  if (!is_wasm && lua_gettop(L) >= 3 && !lua_isnil(L, 3)) {
    symbol_name = luaL_checkstring(L, 3);
  }

  if (map_has(cstr_t, &langs, lang_name)) {
    lua_pushboolean(L, true);
    return 1;
  }

  const TSLanguage *lang = is_wasm
                           ? load_language_from_wasm(L, path, lang_name)
                           : load_language_from_object(L, path, lang_name, symbol_name);

  uint32_t lang_version = ts_language_abi_version(lang);
  if (lang_version < TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION
      || lang_version > TREE_SITTER_LANGUAGE_VERSION) {
    return luaL_error(L,
                      "ABI version mismatch for %s: supported between %d and %d, found %d",
                      path,
                      TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
                      TREE_SITTER_LANGUAGE_VERSION, lang_version);
  }

  pmap_put(cstr_t)(&langs, xstrdup(lang_name), (TSLanguage *)lang);

  lua_pushboolean(L, true);
  return 1;
}

static int tslua_remove_lang(lua_State *L)
{
  const char *lang_name = luaL_checkstring(L, 1);
  bool present = map_has(cstr_t, &langs, lang_name);
  if (present) {
    cstr_t key;
    pmap_del(cstr_t)(&langs, lang_name, &key);
    xfree((void *)key);
  }
  lua_pushboolean(L, present);
  return 1;
}

static TSLanguage *lang_check(lua_State *L, int index)
{
  const char *lang_name = luaL_checkstring(L, index);
  TSLanguage *lang = pmap_get(cstr_t)(&langs, lang_name);
  if (!lang) {
    luaL_error(L, "no such language: %s", lang_name);
  }
  return lang;
}

static int tslua_inspect_lang(lua_State *L)
{
  TSLanguage *lang = lang_check(L, 1);

  lua_createtable(L, 0, 2);  // [retval]

  {  // Symbols
    uint32_t nsymbols = ts_language_symbol_count(lang);
    assert(nsymbols < INT_MAX);

    lua_createtable(L, (int)(nsymbols - 1), 1);  // [retval, symbols]
    for (uint32_t i = 0; i < nsymbols; i++) {
      TSSymbolType t = ts_language_symbol_type(lang, (TSSymbol)i);
      if (t == TSSymbolTypeAuxiliary) {
        // not used by the API
        continue;
      }
      const char *name = ts_language_symbol_name(lang, (TSSymbol)i);
      bool named = t != TSSymbolTypeAnonymous;
      lua_pushboolean(L, named);  // [retval, symbols, is_named]
      if (!named) {
        char buf[256];
        snprintf(buf, sizeof(buf), "\"%s\"", name);
        lua_setfield(L, -2, buf);  // [retval, symbols]
      } else {
        lua_setfield(L, -2, name);  // [retval, symbols]
      }
    }

    lua_setfield(L, -2, "symbols");  // [retval]
  }

  {  // Fields
    uint32_t nfields = ts_language_field_count(lang);
    lua_createtable(L, (int)nfields, 1);  // [retval, fields]
    // Field IDs go from 1 to nfields inclusive (extra index 0 maps to NULL)
    for (uint32_t i = 1; i <= nfields; i++) {
      lua_pushstring(L, ts_language_field_name_for_id(lang, (TSFieldId)i));
      lua_rawseti(L, -2, (int)i);  // [retval, fields]
    }

    lua_setfield(L, -2, "fields");  // [retval]
  }

  lua_pushboolean(L, ts_language_is_wasm(lang));
  lua_setfield(L, -2, "_wasm");

  lua_pushinteger(L, ts_language_abi_version(lang));  // [retval, version]
  lua_setfield(L, -2, "abi_version");

  {  // Metadata
    const TSLanguageMetadata *meta = ts_language_metadata(lang);

    if (meta != NULL) {
      lua_createtable(L, 0, 3);

      lua_pushinteger(L, meta->major_version);
      lua_setfield(L, -2, "major_version");
      lua_pushinteger(L, meta->minor_version);
      lua_setfield(L, -2, "minor_version");
      lua_pushinteger(L, meta->patch_version);
      lua_setfield(L, -2, "patch_version");

      lua_setfield(L, -2, "metadata");
    }
  }

  lua_pushinteger(L, ts_language_state_count(lang));
  lua_setfield(L, -2, "state_count");

  {  // Supertypes
    uint32_t nsupertypes;
    const TSSymbol *supertypes = ts_language_supertypes(lang, &nsupertypes);

    lua_createtable(L, 0, (int)nsupertypes);  // [retval, supertypes]
    for (uint32_t i = 0; i < nsupertypes; i++) {
      const TSSymbol supertype = *(supertypes + i);

      uint32_t nsubtypes;
      const TSSymbol *subtypes = ts_language_subtypes(lang, supertype, &nsubtypes);

      lua_createtable(L, (int)nsubtypes, 0);
      for (uint32_t j = 1; j <= nsubtypes; j++) {
        lua_pushstring(L, ts_language_symbol_name(lang, *(subtypes + j)));
        lua_rawseti(L, -2, (int)j);
      }

      lua_setfield(L, -2, ts_language_symbol_name(lang, supertype));
    }

    lua_setfield(L, -2, "supertypes");  // [retval]
  }

  return 1;
}

// TSParser

static struct luaL_Reg parser_meta[] = {
  { "__gc", parser_gc },
  { "__tostring", rs_ts_parser_tostring },
  { "parse", parser_parse },
  { "reset", rs_ts_parser_reset },
  { "set_included_ranges", parser_set_ranges },
  { "included_ranges", parser_get_ranges },
  { "_set_logger", parser_set_logger },
  { "_logger", parser_get_logger },
  { NULL, NULL }
};

static int tslua_push_parser(lua_State *L)
{
  TSLanguage *lang = lang_check(L, 1);

  TSParser **parser = lua_newuserdata(L, sizeof(TSParser *));
  *parser = ts_parser_new();

#ifdef HAVE_WASMTIME
  if (ts_language_is_wasm(lang)) {
    assert(wasmengine != NULL);
    ts_parser_set_wasm_store(*parser, ts_wasmstore);
  }
#endif

  if (!ts_parser_set_language(*parser, lang)) {
    ts_parser_delete(*parser);
    const char *lang_name = luaL_checkstring(L, 1);
    return luaL_error(L, "Failed to load language : %s", lang_name);
  }

  lua_getfield(L, LUA_REGISTRYINDEX, TS_META_PARSER);  // [udata, meta]
  lua_setmetatable(L, -2);  // [udata]
  return 1;
}

static TSParser *parser_check(lua_State *L, uint16_t index)
{
  TSParser **ud = luaL_checkudata(L, index, TS_META_PARSER);
  luaL_argcheck(L, *ud, index, "TSParser expected");
  return *ud;
}

static void logger_gc(TSLogger logger)
{
  if (!logger.log) {
    return;
  }

  TSLuaLoggerOpts *opts = (TSLuaLoggerOpts *)logger.payload;
  luaL_unref(opts->lstate, LUA_REGISTRYINDEX, opts->cb);
  xfree(opts);
}

static int parser_gc(lua_State *L)
{
  TSParser *p = parser_check(L, 1);
  logger_gc(ts_parser_logger(p));
  ts_parser_delete(p);
  return 0;
}

static const char *input_cb(void *payload, uint32_t byte_index, TSPoint position,
                            uint32_t *bytes_read)
{
  buf_T *bp = payload;
#define BUFSIZE 256
  static char buf[BUFSIZE];

  if ((linenr_T)position.row >= bp->b_ml.ml_line_count) {
    *bytes_read = 0;
    return "";
  }
  linenr_T lnum = (linenr_T)position.row + 1;
  char *line = ml_get_buf(bp, lnum);
  size_t len = (size_t)ml_get_buf_len(bp, lnum);
  if (position.column > len) {
    *bytes_read = 0;
    return "";
  }
  size_t tocopy = MIN(len - position.column, BUFSIZE);

  memcpy(buf, line + position.column, tocopy);
  // Translate embedded \n to NUL
  memchrsub(buf, '\n', NUL, tocopy);
  *bytes_read = (uint32_t)tocopy;
  if (tocopy < BUFSIZE) {
    // now add the final \n, if it is meant to be present for this buffer. If it didn't fit,
    // input_cb will be called again on the same line with advanced column.
    if (lnum != bp->b_ml.ml_line_count || (!bp->b_p_bin && bp->b_p_fixeol)
        || (lnum != bp->b_no_eol_lnum && bp->b_p_eol)) {
      buf[tocopy] = '\n';
      (*bytes_read)++;
    }
  }
  return buf;
#undef BUFSIZE
}

static void push_ranges(lua_State *L, const TSRange *ranges, const size_t length,
                        bool include_bytes)
{
  lua_createtable(L, (int)length, 0);
  for (size_t i = 0; i < length; i++) {
    lua_createtable(L, include_bytes ? 6 : 4, 0);
    int j = 1;
    lua_pushinteger(L, ranges[i].start_point.row);
    lua_rawseti(L, -2, j++);
    lua_pushinteger(L, ranges[i].start_point.column);
    lua_rawseti(L, -2, j++);
    if (include_bytes) {
      lua_pushinteger(L, ranges[i].start_byte);
      lua_rawseti(L, -2, j++);
    }
    lua_pushinteger(L, ranges[i].end_point.row);
    lua_rawseti(L, -2, j++);
    lua_pushinteger(L, ranges[i].end_point.column);
    lua_rawseti(L, -2, j++);
    if (include_bytes) {
      lua_pushinteger(L, ranges[i].end_byte);
      lua_rawseti(L, -2, j++);
    }

    lua_rawseti(L, -2, (int)(i + 1));
  }
}

static bool on_parser_progress(TSParseState *state)
{
  TSLuaParserCallbackPayload *payload = state->payload;
  uint64_t parse_time = os_hrtime() - payload->parse_start_time;
  return parse_time >= payload->timeout_threshold_ns;
}

static int parser_parse(lua_State *L)
{
  TSParser *p = parser_check(L, 1);
  const TSTree *old_tree = NULL;
  if (!lua_isnil(L, 2)) {
    TSLuaTree *ud = luaL_checkudata(L, 2, TS_META_TREE);
    old_tree = ud ? ud->tree : NULL;
  }

  if (lua_type(L, 3) != LUA_TNUMBER) {
    return luaL_argerror(L, 3, "expected buffer handle");
  }

  handle_T bufnr = (handle_T)lua_tointeger(L, 3);
  buf_T *buf = handle_get_buffer(bufnr);

  if (!buf) {
#define BUFSIZE 256
    char ebuf[BUFSIZE] = { 0 };
    vim_snprintf(ebuf, BUFSIZE, "invalid buffer handle: %d", bufnr);
    return luaL_argerror(L, 3, ebuf);
#undef BUFSIZE
  }

  TSInput input = (TSInput){ (void *)buf, input_cb, TSInputEncodingUTF8, NULL };
  TSTree *new_tree = NULL;

  if (!lua_isnil(L, 5)) {
    uint64_t timeout_ns = (uint64_t)lua_tointeger(L, 5);
    TSLuaParserCallbackPayload payload =
      (TSLuaParserCallbackPayload){ .parse_start_time = os_hrtime(),
                                    .timeout_threshold_ns = timeout_ns };
    TSParseOptions parse_options = { .payload = &payload,
                                     .progress_callback = on_parser_progress };
    new_tree = ts_parser_parse_with_options(p, old_tree, input, parse_options);
  } else {
    new_tree = ts_parser_parse(p, old_tree, input);
  }

  bool include_bytes = (lua_gettop(L) >= 4) && lua_toboolean(L, 4);

  if (!new_tree) {
    // Sometimes parsing fails (no language was set, or it was set to one with an incompatible ABI)
    // In those cases, just return an error.
    if (!ts_parser_language(p)) {
      return luaL_error(L, "Language was unset, or has an incompatible ABI.");
    }
    return 0;
  }

  // The new tree will be pushed to the stack, without copy, ownership is now to the lua GC.
  // Old tree is owned by lua GC since before
  uint32_t n_ranges = 0;
  TSRange *changed = old_tree ? ts_tree_get_changed_ranges(old_tree, new_tree, &n_ranges)
                              : ts_tree_included_ranges(new_tree, &n_ranges);

  push_tree(L, new_tree);  // [tree]

  push_ranges(L, changed, n_ranges, include_bytes);  // [tree, ranges]

  xfree(changed);
  return 2;
}

static void range_err(lua_State *L) { luaL_error(L, "Ranges can only be made from 6 element long tables or nodes."); }

// Use the top of the stack (without popping it) to create a TSRange, it can be
// either a lua table or a TSNode
static void range_from_lua(lua_State *L, TSRange *range)
{
  TSNode node;

  if (lua_istable(L, -1)) {
    // should be a table of 6 elements
    if (lua_objlen(L, -1) != 6) {
      range_err(L);
    }

    lua_rawgeti(L, -1, 1);  // [ range, start_row]
    uint32_t start_row = (uint32_t)luaL_checkinteger(L, -1);
    lua_pop(L, 1);

    lua_rawgeti(L, -1, 2);  // [ range, start_col]
    uint32_t start_col = (uint32_t)luaL_checkinteger(L, -1);
    lua_pop(L, 1);

    lua_rawgeti(L, -1, 3);  // [ range, start_byte]
    uint32_t start_byte = (uint32_t)luaL_checkinteger(L, -1);
    lua_pop(L, 1);

    lua_rawgeti(L, -1, 4);  // [ range, end_row]
    uint32_t end_row = (uint32_t)luaL_checkinteger(L, -1);
    lua_pop(L, 1);

    lua_rawgeti(L, -1, 5);  // [ range, end_col]
    uint32_t end_col = (uint32_t)luaL_checkinteger(L, -1);
    lua_pop(L, 1);

    lua_rawgeti(L, -1, 6);  // [ range, end_byte]
    uint32_t end_byte = (uint32_t)luaL_checkinteger(L, -1);
    lua_pop(L, 1);  // [ range ]

    *range = (TSRange) {
      .start_point = (TSPoint) {
        .row = start_row,
        .column = start_col
      },
      .end_point = (TSPoint) {
        .row = end_row,
        .column = end_col
      },
      .start_byte = start_byte,
      .end_byte = end_byte,
    };
  } else if (node_check_opt(L, -1, &node)) {
    *range = (TSRange) {
      .start_point = ts_node_start_point(node),
      .end_point = ts_node_end_point(node),
      .start_byte = ts_node_start_byte(node),
      .end_byte = ts_node_end_byte(node)
    };
  } else {
    range_err(L);
  }
}

static int parser_set_ranges(lua_State *L)
{
  if (lua_gettop(L) < 2) {
    return luaL_error(L, "not enough args to parser:set_included_ranges()");
  }

  TSParser *p = parser_check(L, 1);

  luaL_argcheck(L, lua_istable(L, 2), 2, "table expected.");

  size_t tbl_len = lua_objlen(L, 2);
  TSRange *ranges = xmalloc(sizeof(TSRange) * tbl_len);

  // [ parser, ranges ]
  for (size_t index = 0; index < tbl_len; index++) {
    lua_rawgeti(L, 2, (int)index + 1);  // [ parser, ranges, range ]
    range_from_lua(L, ranges + index);
    lua_pop(L, 1);
  }

  // This memcpies ranges, thus we can free it afterwards
  ts_parser_set_included_ranges(p, ranges, (uint32_t)tbl_len);
  xfree(ranges);

  return 0;
}

static int parser_get_ranges(lua_State *L)
{
  TSParser *p = parser_check(L, 1);

  bool include_bytes = (lua_gettop(L) >= 2) && lua_toboolean(L, 2);

  uint32_t len;
  const TSRange *ranges = ts_parser_included_ranges(p, &len);

  push_ranges(L, ranges, len, include_bytes);
  return 1;
}

static void logger_cb(void *payload, TSLogType logtype, const char *s)
{
  TSLuaLoggerOpts *opts = (TSLuaLoggerOpts *)payload;
  if ((!opts->lex && logtype == TSLogTypeLex)
      || (!opts->parse && logtype == TSLogTypeParse)) {
    return;
  }

  lua_State *lstate = opts->lstate;

  lua_rawgeti(lstate, LUA_REGISTRYINDEX, opts->cb);
  lua_pushstring(lstate, logtype == TSLogTypeParse ? "parse" : "lex");
  lua_pushstring(lstate, s);
  if (lua_pcall(lstate, 2, 0, 0)) {
    luaL_error(lstate, "treesitter logger callback failed");
  }
}

static int parser_set_logger(lua_State *L)
{
  TSParser *p = parser_check(L, 1);

  luaL_argcheck(L, lua_isboolean(L, 2), 2, "boolean expected");
  luaL_argcheck(L, lua_isboolean(L, 3), 3, "boolean expected");
  luaL_argcheck(L, lua_isfunction(L, 4), 4, "function expected");

  TSLuaLoggerOpts *opts = xmalloc(sizeof(TSLuaLoggerOpts));
  lua_pushvalue(L, 4);
  LuaRef ref = luaL_ref(L, LUA_REGISTRYINDEX);

  *opts = (TSLuaLoggerOpts){
    .lex = lua_toboolean(L, 2),
    .parse = lua_toboolean(L, 3),
    .cb = ref,
    .lstate = L
  };

  TSLogger logger = {
    .payload = (void *)opts,
    .log = logger_cb
  };

  ts_parser_set_logger(p, logger);
  return 0;
}

static int parser_get_logger(lua_State *L)
{
  TSParser *p = parser_check(L, 1);
  TSLogger logger = ts_parser_logger(p);
  if (logger.log) {
    TSLuaLoggerOpts *opts = (TSLuaLoggerOpts *)logger.payload;
    lua_rawgeti(L, LUA_REGISTRYINDEX, opts->cb);
  } else {
    lua_pushnil(L);
  }

  return 1;
}

// TSTree

static struct luaL_Reg tree_meta[] = {
  { "__gc", rs_ts_tree_gc },
  { "__tostring", rs_ts_tree_tostring },
  { "root", rs_ts_tree_root },
  { "edit", rs_ts_tree_edit },
  { "included_ranges", rs_ts_tree_get_ranges },
  { "copy", rs_ts_tree_copy },
  { NULL, NULL }
};

/// Push tree interface on to the lua stack.
///
/// The tree is not copied. Ownership of the tree is transferred from C to
/// Lua. If needed use ts_tree_copy() in the caller.
static void push_tree(lua_State *L, const TSTree *tree)
{
  if (tree == NULL) {
    lua_pushnil(L);
    return;
  }

  TSLuaTree *ud = lua_newuserdata(L, sizeof(TSLuaTree));  // [udata]
  ud->tree = tree;
  lua_getfield(L, LUA_REGISTRYINDEX, TS_META_TREE);  // [udata, meta]
  lua_setmetatable(L, -2);  // [udata]
}


// TSNode
static struct luaL_Reg node_meta[] = {
  { "__tostring", rs_ts_node_tostring },
  { "__eq", rs_ts_node_eq },
  { "__len", rs_ts_node_child_count },
  { "id", rs_ts_node_id },
  { "range", rs_ts_node_range },
  { "start", rs_ts_node_start },
  { "end_", rs_ts_node_end },
  { "type", rs_ts_node_type },
  { "symbol", rs_ts_node_symbol },
  { "field", rs_ts_node_field },
  { "named", rs_ts_node_named },
  { "missing", rs_ts_node_missing },
  { "extra", rs_ts_node_extra },
  { "has_changes", rs_ts_node_has_changes },
  { "has_error", rs_ts_node_has_error },
  { "sexpr", rs_ts_node_sexpr },
  { "child_count", rs_ts_node_child_count },
  { "named_child_count", rs_ts_node_named_child_count },
  { "child", rs_ts_node_child },
  { "named_child", rs_ts_node_named_child },
  { "descendant_for_range", rs_ts_node_descendant_for_range },
  { "named_descendant_for_range", rs_ts_node_named_descendant_for_range },
  { "parent", rs_ts_node_parent },
  { "__has_ancestor", rs_ts_has_ancestor },
  { "child_with_descendant", rs_ts_node_child_with_descendant },
  { "iter_children", rs_ts_node_iter_children },
  { "next_sibling", rs_ts_node_next_sibling },
  { "prev_sibling", rs_ts_node_prev_sibling },
  { "next_named_sibling", rs_ts_node_next_named_sibling },
  { "prev_named_sibling", rs_ts_node_prev_named_sibling },
  { "named_children", rs_ts_node_named_children },
  { "root", rs_ts_node_root },
  { "tree", rs_ts_node_tree },
  { "byte_length", rs_ts_node_byte_length },
  { "equal", rs_ts_node_equal },

  { NULL, NULL }
};

/// Push node interface on to the Lua stack
///
/// Stack at `uindex` must have a value with a fenv with a reference to node's
/// tree. This value is not popped. Can only be called inside a cfunction with
/// the tslua environment.
static void push_node(lua_State *L, TSNode node, int uindex)
{
  assert(uindex > 0 || uindex < -LUA_MINSTACK);
  if (ts_node_is_null(node)) {
    lua_pushnil(L);  // [nil]
    return;
  }

  TSNode *ud = lua_newuserdata(L, sizeof(TSNode));  // [udata]
  *ud = node;
  lua_getfield(L, LUA_REGISTRYINDEX, TS_META_NODE);  // [udata, meta]
  lua_setmetatable(L, -2);  // [udata]

  // Copy the fenv to keep alive a reference to the node's tree.
  lua_getfenv(L, uindex);  // [udata, reftable]
  lua_setfenv(L, -2);  // [udata]
}

static bool node_check_opt(lua_State *L, int index, TSNode *res)
{
  TSNode *ud = luaL_checkudata(L, index, TS_META_NODE);
  if (ud) {
    *res = *ud;
    return true;
  }
  return false;
}

static TSNode node_check(lua_State *L, int index)
{
  TSNode *ud = luaL_checkudata(L, index, TS_META_NODE);
  return *ud;
}

// TSQueryCursor

static struct luaL_Reg querycursor_meta[] = {
  { "remove_match", rs_ts_querycursor_remove_match },
  { "next_capture", rs_ts_querycursor_next_capture },
  { "next_match", rs_ts_querycursor_next_match },
  { "__gc", rs_ts_querycursor_gc },
  { NULL, NULL }
};

static int tslua_push_querycursor(lua_State *L)
{
  TSNode node = node_check(L, 1);

  TSQuery *query = query_check(L, 2);
  TSQueryCursor *cursor = ts_query_cursor_new();

  if (lua_gettop(L) >= 3 && !lua_isnil(L, 3)) {
    luaL_argcheck(L, lua_istable(L, 3), 3, "table expected");
  }

  lua_getfield(L, 3, "start_row");
  uint32_t start_row = (uint32_t)luaL_checkinteger(L, -1);
  lua_pop(L, 1);

  lua_getfield(L, 3, "start_col");
  uint32_t start_col = (uint32_t)luaL_checkinteger(L, -1);
  lua_pop(L, 1);

  lua_getfield(L, 3, "end_row");
  uint32_t end_row = (uint32_t)luaL_checkinteger(L, -1);
  lua_pop(L, 1);

  lua_getfield(L, 3, "end_col");
  uint32_t end_col = (uint32_t)luaL_checkinteger(L, -1);
  lua_pop(L, 1);

  ts_query_cursor_set_point_range(cursor, (TSPoint){ start_row, start_col },
                                  (TSPoint){ end_row, end_col });

  lua_getfield(L, 3, "max_start_depth");
  if (!lua_isnil(L, -1)) {
    uint32_t max_start_depth = (uint32_t)luaL_checkinteger(L, -1);
    ts_query_cursor_set_max_start_depth(cursor, max_start_depth);
  }
  lua_pop(L, 1);

  lua_getfield(L, 3, "match_limit");
  if (!lua_isnil(L, -1)) {
    uint32_t match_limit = (uint32_t)luaL_checkinteger(L, -1);
    ts_query_cursor_set_match_limit(cursor, match_limit);
  }
  lua_pop(L, 1);

  ts_query_cursor_exec(cursor, query, node);

  TSQueryCursor **ud = lua_newuserdata(L, sizeof(*ud));  // [node, query, ..., udata]
  *ud = cursor;
  lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERYCURSOR);  // [node, query, ..., udata, meta]
  lua_setmetatable(L, -2);  // [node, query, ..., udata]

  // Copy the fenv which contains the nodes tree.
  lua_getfenv(L, 1);  // [udata, reftable]
  lua_setfenv(L, -2);  // [udata]

  return 1;
}

static TSQueryCursor *querycursor_check(lua_State *L, int index)
{
  TSQueryCursor **ud = luaL_checkudata(L, index, TS_META_QUERYCURSOR);
  luaL_argcheck(L, *ud, index, "TSQueryCursor expected");
  return *ud;
}

// TSQueryMatch

static struct luaL_Reg querymatch_meta[] = {
  { "info", rs_ts_querymatch_info },
  { "captures", rs_ts_querymatch_captures },
  { NULL, NULL }
};

// TSQuery

static struct luaL_Reg query_meta[] = {
  { "__gc", rs_ts_query_gc },
  { "__tostring", rs_ts_query_tostring },
  { "inspect", rs_ts_query_inspect },
  { "disable_capture", rs_ts_query_disable_capture },
  { "disable_pattern", rs_ts_query_disable_pattern },
  { NULL, NULL }
};

static int tslua_parse_query(lua_State *L)
{
  if (lua_gettop(L) < 2 || !lua_isstring(L, 1) || !lua_isstring(L, 2)) {
    return luaL_error(L, "string expected");
  }

  TSLanguage *lang = lang_check(L, 1);

  size_t len;
  const char *src = lua_tolstring(L, 2, &len);

  tslua_query_parse_count++;
  uint32_t error_offset;
  TSQueryError error_type;
  TSQuery *query = ts_query_new(lang, src, (uint32_t)len, &error_offset, &error_type);

  if (!query) {
    char err_msg[IOSIZE];
    rs_ts_query_err_string(src, (int)error_offset, (uint32_t)error_type, err_msg, sizeof(err_msg));
    return luaL_error(L, "%s", err_msg);
  }

  TSQuery **ud = lua_newuserdata(L, sizeof(TSQuery *));  // [udata]
  *ud = query;
  lua_getfield(L, LUA_REGISTRYINDEX, TS_META_QUERY);  // [udata, meta]
  lua_setmetatable(L, -2);  // [udata]
  return 1;
}

static TSQuery *query_check(lua_State *L, int index)
{
  TSQuery **ud = luaL_checkudata(L, index, TS_META_QUERY);
  luaL_argcheck(L, *ud, index, "TSQuery expected");
  return *ud;
}

// Library init

static void build_meta(lua_State *L, const char *tname, const luaL_Reg *meta)
{
  if (luaL_newmetatable(L, tname)) {  // [meta]
    luaL_register(L, NULL, meta);

    lua_pushvalue(L, -1);  // [meta, meta]
    lua_setfield(L, -2, "__index");  // [meta]
  }
  lua_pop(L, 1);  // [] (don't use it now)
}

/// Init the tslua library.
///
/// All global state is stored in the registry of the lua_State.
static void tslua_init(lua_State *L)
{
  // type metatables
  build_meta(L, TS_META_PARSER, parser_meta);
  build_meta(L, TS_META_TREE, tree_meta);
  build_meta(L, TS_META_NODE, node_meta);
  build_meta(L, TS_META_QUERY, query_meta);
  build_meta(L, TS_META_QUERYCURSOR, querycursor_meta);
  build_meta(L, TS_META_QUERYMATCH, querymatch_meta);

  ts_set_allocator(xmalloc, xcalloc, xrealloc, xfree);
}

static int tslua_get_language_version(lua_State *L)
{
  lua_pushnumber(L, TREE_SITTER_LANGUAGE_VERSION);
  return 1;
}

static int tslua_get_minimum_language_version(lua_State *L)
{
  lua_pushnumber(L, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION);
  return 1;
}

void nlua_treesitter_free(void)
{
#ifdef HAVE_WASMTIME
  if (wasmengine != NULL) {
    wasm_engine_delete(wasmengine);
  }
  if (ts_wasmstore != NULL) {
    ts_wasm_store_delete(ts_wasmstore);
  }
#endif
}

void nlua_treesitter_init(lua_State *const lstate) FUNC_ATTR_NONNULL_ALL
{
  tslua_init(lstate);

  lua_pushcfunction(lstate, tslua_push_parser);
  lua_setfield(lstate, -2, "_create_ts_parser");

  lua_pushcfunction(lstate, tslua_push_querycursor);
  lua_setfield(lstate, -2, "_create_ts_querycursor");

  lua_pushcfunction(lstate, tslua_add_language_from_object);
  lua_setfield(lstate, -2, "_ts_add_language_from_object");

#ifdef HAVE_WASMTIME
  lua_pushcfunction(lstate, tslua_add_language_from_wasm);
  lua_setfield(lstate, -2, "_ts_add_language_from_wasm");
#endif

  lua_pushcfunction(lstate, tslua_has_language);
  lua_setfield(lstate, -2, "_ts_has_language");

  lua_pushcfunction(lstate, tslua_remove_lang);
  lua_setfield(lstate, -2, "_ts_remove_language");

  lua_pushcfunction(lstate, tslua_inspect_lang);
  lua_setfield(lstate, -2, "_ts_inspect_language");

  lua_pushcfunction(lstate, tslua_parse_query);
  lua_setfield(lstate, -2, "_ts_parse_query");

  lua_pushcfunction(lstate, tslua_get_language_version);
  lua_setfield(lstate, -2, "_ts_get_language_version");

  lua_pushcfunction(lstate, tslua_get_minimum_language_version);
  lua_setfield(lstate, -2, "_ts_get_minimum_language_version");
}
