// mapping.c: Code for mappings and abbreviations.

#include <assert.h>
#include <lauxlib.h>
#include <limits.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
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
#include "nvim/ex_session.h"
#include "nvim/fuzzy.h"
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
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/state_defs.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/ui_defs.h"
#include "nvim/vim_defs.h"

// Rust FFI declarations
extern int rs_map_to_exists_mode(const char *rhs, int mode, int abbr);
extern int rs_get_map_mode_string(const char *mode_string, int abbr);
extern void rs_map_mode_to_chars(int mode, char *buf);
extern int rs_langmap_adjust_mb(int c);
extern void rs_langmap_init(void);
extern int rs_langmap_parse(const char *langmap_str, char *errbuf, size_t errbuflen);
extern int rs_get_map_mode(char **cmdp, int forceit);
extern char *rs_check_map(char *keys, int mode, int exact, int ign_mod, int abbr,
                          mapblock_T **mp_ptr, int *local_ptr, int *rhs_lua);
extern int rs_map_to_exists_str(const char *str, const char *modechars, int abbr);
extern char *rs_translate_mapping(const char *str_in, const char *cpo_val);
extern void rs_map_clear_mode(buf_T *buf, int mode, int local, int abbr);
extern void rs_do_mapclear(char *cmdp, char *arg, int forceit, int abbr);
extern char *rs_set_context_in_map_cmd(void *xp, char *cmd, char *arg,
                                        int forceit, int isabbrev, int isunmap, int cmdidx);
extern int rs_expand_mappings(char *pat, void *regmatch,
                               int *numMatches, char ***matches);
extern int rs_makemap_should_skip(mapblock_T *mp);
extern int rs_makemap_needs_cpo(mapblock_T *mp);

// MakemapModeResult from Rust
typedef struct {
  char c1;
  char c2;
  char c3;
  int use_bang;
  int error;
} MakemapModeResult;
extern MakemapModeResult rs_makemap_mode_chars(int mode, int abbr);
extern int rs_put_escstr_escape_type(int what, int c, int is_first);

/// List used for abbreviations.
static mapblock_T *first_abbr = NULL;  // first entry in abbrlist

// Each mapping is put in one of the MAX_MAPHASH hash lists,
// to speed up finding it.
static mapblock_T *(maphash[MAX_MAPHASH]) = { 0 };

// Make a hash value for a mapping.
// "mode" is the lower 4 bits of the State for the mapping.
// "c1" is the first character of the "lhs".
// Returns a value between 0 and 255, index in maphash.
// Put Normal/Visual mode mappings mostly separately from Insert/Cmdline mode.
#define MAP_HASH(mode, \
                 c1) (((mode) & \
                       (MODE_NORMAL | MODE_VISUAL | MODE_SELECT | \
                        MODE_OP_PENDING | MODE_TERMINAL)) ? (c1) : ((c1) ^ 0x80))

/// All possible |:map-arguments| usable in a |:map| command.
///
/// The <special> argument has no effect on mappings and is excluded from this
/// struct declaration. |:noremap| is included, since it behaves like a map
/// argument when used in a mapping.
///
/// @see mapblock_T
struct map_arguments {
  bool buffer;
  bool expr;
  bool noremap;
  bool nowait;
  bool script;
  bool silent;
  bool unique;
  bool replace_keycodes;

  /// The {lhs} of the mapping.
  ///
  /// vim limits this to MAXMAPLEN characters, allowing us to use a static
  /// buffer. Setting lhs_len to a value larger than MAXMAPLEN can signal
  /// that {lhs} was too long and truncated.
  char lhs[MAXMAPLEN + 1];
  size_t lhs_len;

  /// Unsimplifed {lhs} of the mapping. If no simplification has been done then alt_lhs_len is 0.
  char alt_lhs[MAXMAPLEN + 1];
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
extern int rs_do_map(int maptype, char *arg, int mode, int is_abbrev);

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

/// Get the start of the hashed map list for "state" and first character "c".
mapblock_T *get_maphash_list(int state, int c)
{
  return maphash[MAP_HASH(state, c)];
}

/// Get the buffer-local hashed map list for "state" and first character "c".
mapblock_T *get_buf_maphash_list(int state, int c)
{
  return curbuf->b_maphash[MAP_HASH(state, c)];
}

/// Delete one entry from the abbrlist or maphash[].
/// "mpp" is a pointer to the m_next field of the PREVIOUS entry!
static void mapblock_free(mapblock_T **mpp)
{
  mapblock_T *mp = *mpp;
  xfree(mp->m_keys);
  if (mp->m_alt != NULL) {
    mp->m_alt->m_alt = NULL;
  } else {
    NLUA_CLEAR_REF(mp->m_luaref);
    xfree(mp->m_str);
    xfree(mp->m_orig_str);
    xfree(mp->m_desc);
  }
  *mpp = mp->m_next;
  xfree(mp);
}

/// put characters to represent the map mode in a string buffer
///
/// @param[out] buf must be at least 7 bytes (including NUL)
void map_mode_to_chars(int mode, char *buf)
  FUNC_ATTR_NONNULL_ALL
{
  rs_map_mode_to_chars(mode, buf);
}

/// @param local  true for buffer-local map
static void showmap(mapblock_T *mp, bool local)
{
  if (message_filtered(mp->m_keys) && message_filtered(mp->m_str)
      && (mp->m_desc == NULL || message_filtered(mp->m_desc))) {
    return;
  }

  // When ext_messages is active, msg_didout is never set.
  if (msg_didout || msg_silent != 0 || ui_has(kUIMessages)) {
    msg_putchar('\n');
    if (got_int) {          // 'q' typed at MORE prompt
      return;
    }
  }

  char mapchars[7];
  map_mode_to_chars(mp->m_mode, mapchars);
  msg_puts(mapchars);
  size_t len = strlen(mapchars);

  while (++len <= 3) {
    msg_putchar(' ');
  }

  // Display the LHS.  Get length of what we write.
  len = (size_t)msg_outtrans_special(mp->m_keys, true, 0);
  do {
    msg_putchar(' ');                   // pad with blanks
    len++;
  } while (len < 12);

  if (mp->m_noremap == REMAP_NONE) {
    msg_puts_hl("*", HLF_8, false);
  } else if (mp->m_noremap == REMAP_SCRIPT) {
    msg_puts_hl("&", HLF_8, false);
  } else {
    msg_putchar(' ');
  }

  if (local) {
    msg_putchar('@');
  } else {
    msg_putchar(' ');
  }

  // Use false below if we only want things like <Up> to show up as such on
  // the rhs, and not M-x etc, true gets both -- webb
  if (mp->m_luaref != LUA_NOREF) {
    char *str = nlua_funcref_str(mp->m_luaref, NULL);
    msg_puts_hl(str, HLF_8, false);
    xfree(str);
  } else if (mp->m_str[0] == NUL) {
    msg_puts_hl("<Nop>", HLF_8, false);
  } else {
    msg_outtrans_special(mp->m_str, false, 0);
  }

  if (mp->m_desc != NULL) {
    msg_puts("\n                 ");  // Shift line to same level as rhs.
    msg_puts(mp->m_desc);
  }
  if (p_verbose > 0) {
    last_set_msg(mp->m_script_ctx);
  }
  msg_clr_eos();
}

// Argument parsing — now implemented in Rust (src/nvim-rs/mapping/src/args.rs).
// These thin C wrappers delegate to the Rust implementations.

/// Replace termcodes in the given LHS and RHS and store the results into mapargs.
static bool set_maparg_lhs_rhs(const char *const orig_lhs, const size_t orig_lhs_len,
                               const char *const orig_rhs, const size_t orig_rhs_len,
                               const LuaRef rhs_lua, const char *const cpo_val,
                               MapArguments *const mapargs)
{
  return rs_set_maparg_lhs_rhs(orig_lhs, orig_lhs_len, orig_rhs, orig_rhs_len,
                               rhs_lua, cpo_val, mapargs) != 0;
}

/// @see set_maparg_lhs_rhs
static void set_maparg_rhs(const char *const orig_rhs, const size_t orig_rhs_len,
                           const LuaRef rhs_lua, const scid_T sid, const char *const cpo_val,
                           MapArguments *const mapargs)
{
  rs_set_maparg_rhs(orig_rhs, orig_rhs_len, rhs_lua, sid, cpo_val, mapargs);
}

/// Parse a string of |:map-arguments| into a MapArguments struct.
static int str_to_mapargs(const char *strargs, bool is_unmap, MapArguments *mapargs)
{
  return rs_str_to_mapargs(strargs, is_unmap ? 1 : 0, mapargs);
}

/// @param args  "rhs", "rhs_lua", "orig_rhs", "expr", "silent", "nowait",
///              "replace_keycodes" and "desc" fields are used.
/// @param sid  0 to use current_sctx
static mapblock_T *map_add(buf_T *buf, mapblock_T **map_table, mapblock_T **abbr_table,
                           const char *keys, MapArguments *args, int noremap, int mode,
                           bool is_abbr, scid_T sid, linenr_T lnum, bool simplified)
  FUNC_ATTR_NONNULL_RET
{
  mapblock_T *mp = xcalloc(1, sizeof(mapblock_T));

  // If CTRL-C has been mapped, don't always use it for Interrupting.
  if (*keys == Ctrl_C) {
    if (map_table == buf->b_maphash) {
      buf->b_mapped_ctrl_c |= mode;
    } else {
      mapped_ctrl_c |= mode;
    }
  }

  mp->m_keys = xstrdup(keys);
  mp->m_str = args->rhs;
  mp->m_orig_str = args->orig_rhs;
  mp->m_luaref = args->rhs_lua;
  mp->m_keylen = (int)strlen(mp->m_keys);
  mp->m_noremap = noremap;
  mp->m_nowait = args->nowait;
  mp->m_silent = args->silent;
  mp->m_mode = mode;
  mp->m_simplified = simplified;
  mp->m_expr = args->expr;
  mp->m_replace_keycodes = args->replace_keycodes;
  if (sid != 0) {
    mp->m_script_ctx.sc_sid = sid;
    mp->m_script_ctx.sc_lnum = lnum;
  } else {
    mp->m_script_ctx = current_sctx;
    mp->m_script_ctx.sc_lnum += SOURCING_LNUM;
    nlua_set_sctx(&mp->m_script_ctx);
  }
  mp->m_desc = args->desc;

  // add the new entry in front of the abbrlist or maphash[] list
  if (is_abbr) {
    mp->m_next = *abbr_table;
    *abbr_table = mp;
  } else {
    const int n = MAP_HASH(mp->m_mode, (uint8_t)mp->m_keys[0]);
    mp->m_next = map_table[n];
    map_table[n] = mp;
  }
  return mp;
}

/// Sets or removes a mapping or abbreviation in buffer `buf`.
///
/// @param maptype    @see do_map
/// @param args  Fully parsed and "preprocessed" arguments for the
///              (un)map/abbrev command. Termcodes should have already been
///              replaced; whitespace, `<` and `>` signs, etc. in {lhs} and
///              {rhs} are assumed to be literal components of the mapping.
/// @param mode       @see do_map
/// @param is_abbrev  @see do_map
/// @param buf        Target Buffer
static int buf_do_map(int maptype, MapArguments *args, int mode, bool is_abbrev, buf_T *buf)
{
  return rs_buf_do_map(maptype, args, mode, is_abbrev ? 1 : 0, buf);
}

/// Set or remove a mapping or an abbreviation in the current buffer, OR
/// display (matching) mappings/abbreviations.
///
/// ```vim
/// map[!]                          " show all key mappings
/// map[!] {lhs}                    " show key mapping for {lhs}
/// map[!] {lhs} {rhs}              " set key mapping for {lhs} to {rhs}
/// noremap[!] {lhs} {rhs}          " same, but no remapping for {rhs}
/// unmap[!] {lhs}                  " remove key mapping for {lhs}
/// abbr                            " show all abbreviations
/// abbr {lhs}                      " show abbreviations for {lhs}
/// abbr {lhs} {rhs}                " set abbreviation for {lhs} to {rhs}
/// noreabbr {lhs} {rhs}            " same, but no remapping for {rhs}
/// unabbr {lhs}                    " remove abbreviation for {lhs}
///
/// for :map   mode is MODE_NORMAL | MODE_VISUAL | MODE_SELECT | MODE_OP_PENDING
/// for :map!  mode is MODE_INSERT | MODE_CMDLINE
/// for :cmap  mode is MODE_CMDLINE
/// for :imap  mode is MODE_INSERT
/// for :lmap  mode is MODE_LANGMAP
/// for :nmap  mode is MODE_NORMAL
/// for :vmap  mode is MODE_VISUAL | MODE_SELECT
/// for :xmap  mode is MODE_VISUAL
/// for :smap  mode is MODE_SELECT
/// for :omap  mode is MODE_OP_PENDING
/// for :tmap  mode is MODE_TERMINAL
///
/// for :abbr  mode is MODE_INSERT | MODE_CMDLINE
/// for :iabbr mode is MODE_INSERT
/// for :cabbr mode is MODE_CMDLINE
/// ```
///
/// @param maptype  MAPTYPE_MAP for |:map| or |:abbr|
///                 MAPTYPE_UNMAP for |:unmap| or |:unabbr|
///                 MAPTYPE_NOREMAP for |:noremap| or |:noreabbr|
///                 MAPTYPE_UNMAP_LHS is like MAPTYPE_UNMAP, but doesn't try to match
///                 with {rhs} if there is no match with {lhs}.
/// @param arg      C-string containing the arguments of the map/abbrev
///                 command, i.e. everything except the initial `:[X][nore]map`.
///                 - Cannot be a read-only string; it will be modified.
/// @param mode   Bitflags representing the mode in which to set the mapping.
///               See @ref get_map_mode.
/// @param is_abbrev  True if setting an abbreviation, false otherwise.
///
/// @return 0 on success. On failure, will return one of the following:
///         - 1 for invalid arguments
///         - 2 for no match
///         - 4 for out of mem (deprecated, WON'T HAPPEN)
///         - 5 for entry not unique
///         - 6 for buflocal unique entry conflicts with global entry
///
int do_map(int maptype, char *arg, int mode, bool is_abbrev)
{
  return rs_do_map(maptype, arg, mode, is_abbrev ? 1 : 0);
}

/// Get the mapping mode from the command name.
/// Now implemented in Rust (rs_get_map_mode).
static int get_map_mode(char **cmdp, bool forceit)
{
  return rs_get_map_mode(cmdp, forceit ? 1 : 0);
}

/// Clear all mappings (":mapclear") or abbreviations (":abclear").
/// "abbr" should be false for mappings, true for abbreviations.
/// This function used to be called map_clear().
static void do_mapclear(char *cmdp, char *arg, int forceit, int abbr)
{
  rs_do_mapclear(cmdp, arg, forceit, abbr);
}

/// Clear all mappings in "mode".
void map_clear_mode(buf_T *buf, int mode, bool local, bool abbr)
{
  rs_map_clear_mode(buf, mode, local ? 1 : 0, abbr ? 1 : 0);
}

/// Check if a map exists that has given string in the rhs
///
/// Also checks mappings local to the current buffer.
///
/// @param[in]  str  String which mapping must have in the rhs. Termcap codes
///                  are recognized in this argument.
/// @param[in]  modechars  Mode(s) in which mappings are checked.
/// @param[in]  abbr  true if checking abbreviations in place of mappings.
///
/// @return true if there is at least one mapping with given parameters.
bool map_to_exists(const char *const str, const char *const modechars, const bool abbr)
  FUNC_ATTR_NONNULL_ALL FUNC_ATTR_WARN_UNUSED_RESULT FUNC_ATTR_PURE
{
  return rs_map_to_exists_str(str, modechars, abbr ? 1 : 0) != 0;
}

/// Check if a map exists that has given string in the rhs
///
/// Also checks mappings local to the current buffer.
///
/// @param[in]  rhs  String which mapping must have in the rhs. Termcap codes
///                  are recognized in this argument.
/// @param[in]  mode  Mode(s) in which mappings are checked.
/// @param[in]  abbr  true if checking abbreviations in place of mappings.
///
/// @return true if there is at least one mapping with given parameters.
bool map_to_exists_mode(const char *const rhs, const int mode, const bool abbr)
{
  // Delegate to Rust implementation
  return rs_map_to_exists_mode(rhs, mode, abbr ? 1 : 0) != 0;
}

/// Used below when expanding mapping/abbreviation names.
static int expand_mapmodes = 0;
static bool expand_isabbrev = false;
static bool expand_buffer = false;

/// Translate an internal mapping/abbreviation representation into the
/// corresponding external one recognized by :map/:abbrev commands.
///
/// This function is called when expanding mappings/abbreviations on the
/// command-line.
///
/// It uses a growarray to build the translation string since the latter can be
/// wider than the original description. The caller has to free the string
/// afterwards.
///
/// @param[in] cpo_val  See param docs for @ref replace_termcodes.
///
/// @return  NULL when there is a problem.
static char *translate_mapping(const char *const str_in, const char *const cpo_val)
{
  return rs_translate_mapping(str_in, cpo_val);
}

/// Work out what to complete when doing command line completion of mapping
/// or abbreviation names.
///
/// @param forceit  true if '!' given
/// @param isabbrev  true if abbreviation
/// @param isunmap  true if unmap/unabbrev command
char *set_context_in_map_cmd(expand_T *xp, char *cmd, char *arg, bool forceit, bool isabbrev,
                             bool isunmap, cmdidx_T cmdidx)
{
  return rs_set_context_in_map_cmd(xp, cmd, arg,
                                   forceit ? 1 : 0,
                                   isabbrev ? 1 : 0,
                                   isunmap ? 1 : 0,
                                   (int)cmdidx);
}

/// Find all mapping/abbreviation names that match regexp "regmatch".
/// For command line expansion of ":[un]map" and ":[un]abbrev" in all modes.
/// @return OK if matches found, FAIL otherwise.
int ExpandMappings(char *pat, regmatch_T *regmatch, int *numMatches, char ***matches)
{
  return rs_expand_mappings(pat, regmatch, numMatches, matches);
}

// Check for an abbreviation.
// Cursor is at ptr[col].
// When inserting, mincol is where insert started.
// For the command line, mincol is what is to be skipped over.
// "c" is the character typed before check_abbr was called.  It may have
// ABBR_OFF added to avoid prepending a CTRL-V to it.
//
// Historic vi practice: The last character of an abbreviation must be an id
// character ([a-zA-Z0-9_]). The characters in front of it must be all id
// characters or all non-id characters. This allows for abbr. "#i" to
// "#include".
//
// Vim addition: Allow for abbreviations that end in a non-keyword character.
// Then there must be white space before the abbr.
//
// Return true if there is an abbreviation, false if not.
bool check_abbr(int c, char *ptr, int col, int mincol)
{
  uint8_t tb[MB_MAXBYTES + 4];
  int clen = 0;                 // length in characters

  if (typebuf.tb_no_abbr_cnt) {  // abbrev. are not recursive
    return false;
  }

  // no remapping implies no abbreviation, except for CTRL-]
  if (noremap_keys() && c != Ctrl_RSB) {
    return false;
  }

  // Check for word before the cursor: If it ends in a keyword char all
  // chars before it must be keyword chars or non-keyword chars, but not
  // white space. If it ends in a non-keyword char we accept any characters
  // before it except white space.
  if (col == 0) {  // cannot be an abbr.
    return false;
  }

  int scol;  // starting column of the abbr.

  {
    bool is_id = true;
    bool vim_abbr;
    char *p = mb_prevptr(ptr, ptr + col);
    if (!vim_iswordp(p)) {
      vim_abbr = true;    // Vim added abbr.
    } else {
      vim_abbr = false;   // vi compatible abbr.
      if (p > ptr) {
        is_id = vim_iswordp(mb_prevptr(ptr, p));
      }
    }
    clen = 1;
    while (p > ptr + mincol) {
      p = mb_prevptr(ptr, p);
      if (ascii_isspace(*p) || (!vim_abbr && is_id != vim_iswordp(p))) {
        p += utfc_ptr2len(p);
        break;
      }
      clen++;
    }
    scol = (int)(p - ptr);
  }

  if (scol < mincol) {
    scol = mincol;
  }
  if (scol < col) {             // there is a word in front of the cursor
    ptr += scol;
    int len = col - scol;
    mapblock_T *mp = curbuf->b_first_abbr;
    mapblock_T *mp2 = first_abbr;
    if (mp == NULL) {
      mp = mp2;
      mp2 = NULL;
    }
    for (; mp;
         mp->m_next == NULL ? (mp = mp2, mp2 = NULL)
                            : (mp = mp->m_next)) {
      int qlen = mp->m_keylen;
      char *q = mp->m_keys;

      if (strchr(mp->m_keys, K_SPECIAL) != NULL) {
        // Might have K_SPECIAL escaped mp->m_keys.
        q = xstrdup(mp->m_keys);
        vim_unescape_ks(q);
        qlen = (int)strlen(q);
      }
      // find entries with right mode and keys
      int match = (mp->m_mode & State)
                  && qlen == len
                  && !strncmp(q, ptr, (size_t)len);
      if (q != mp->m_keys) {
        xfree(q);
      }
      if (match) {
        break;
      }
    }
    if (mp != NULL) {
      // Found a match:
      // Insert the rest of the abbreviation in typebuf.tb_buf[].
      // This goes from end to start.
      //
      // Characters 0x000 - 0x100: normal chars, may need CTRL-V,
      // except K_SPECIAL: Becomes K_SPECIAL KS_SPECIAL KE_FILLER
      // Characters where IS_SPECIAL() == true: key codes, need
      // K_SPECIAL. Other characters (with ABBR_OFF): don't use CTRL-V.
      //
      // Character CTRL-] is treated specially - it completes the
      // abbreviation, but is not inserted into the input stream.
      int j = 0;
      if (c != Ctrl_RSB) {
        // special key code, split up
        if (IS_SPECIAL(c) || c == K_SPECIAL) {
          tb[j++] = K_SPECIAL;
          tb[j++] = (uint8_t)K_SECOND(c);
          tb[j++] = (uint8_t)K_THIRD(c);
        } else {
          if (c < ABBR_OFF && (c < ' ' || c > '~')) {
            tb[j++] = Ctrl_V;                   // special char needs CTRL-V
          }
          // if ABBR_OFF has been added, remove it here.
          if (c >= ABBR_OFF) {
            c -= ABBR_OFF;
          }
          int newlen = utf_char2bytes(c, (char *)tb + j);
          tb[j + newlen] = NUL;
          // Need to escape K_SPECIAL.
          char *escaped = vim_strsave_escape_ks((char *)tb + j);
          if (escaped != NULL) {
            newlen = (int)strlen(escaped);
            memmove(tb + j, escaped, (size_t)newlen);
            j += newlen;
            xfree(escaped);
          }
        }
        tb[j] = NUL;
        // insert the last typed char
        ins_typebuf((char *)tb, 1, 0, true, mp->m_silent);
      }

      // copy values here, calling eval_map_expr() may make "mp" invalid!
      const int noremap = mp->m_noremap;
      const bool silent = mp->m_silent;
      const bool expr = mp->m_expr;

      char *s;
      if (expr) {
        s = eval_map_expr(mp, c);
      } else {
        s = mp->m_str;
      }
      if (s != NULL) {
        // insert the to string
        ins_typebuf(s, noremap, 0, true, silent);
        // no abbrev. for these chars
        typebuf.tb_no_abbr_cnt += (int)strlen(s) + j + 1;
        if (expr) {
          xfree(s);
        }
      }

      tb[0] = Ctrl_H;
      tb[1] = NUL;
      len = clen;  // Delete characters instead of bytes
      while (len-- > 0) {  // delete the from string
        ins_typebuf((char *)tb, 1, 0, true, silent);
      }
      return true;
    }
  }
  return false;
}

/// Evaluate the RHS of a mapping or abbreviations and take care of escaping
/// special characters.
/// Careful: after this "mp" will be invalid if the mapping was deleted.
///
/// @param c  NUL or typed character for abbreviation
char *eval_map_expr(mapblock_T *mp, int c)
{
  char *p = NULL;
  char *expr = NULL;

  // Remove escaping of K_SPECIAL, because "str" is in a format to be used as
  // typeahead.
  if (mp->m_luaref == LUA_NOREF) {
    expr = xstrdup(mp->m_str);
    vim_unescape_ks(expr);
  }

  const bool replace_keycodes = mp->m_replace_keycodes;

  // Forbid changing text or using ":normal" to avoid most of the bad side
  // effects.  Also restore the cursor position.
  expr_map_lock++;
  set_vim_var_char(c);    // set v:char to the typed character
  const pos_T save_cursor = curwin->w_cursor;
  const int save_msg_col = msg_col;
  const int save_msg_row = msg_row;
  if (mp->m_luaref != LUA_NOREF) {
    Error err = ERROR_INIT;
    Array args = ARRAY_DICT_INIT;
    Object ret = nlua_call_ref(mp->m_luaref, NULL, args, kRetObject, NULL, &err);
    if (ret.type == kObjectTypeString) {
      p = string_to_cstr(ret.data.string);
    }
    api_free_object(ret);
    if (ERROR_SET(&err)) {
      semsg_multiline("emsg", "E5108: %s", err.msg);
      api_clear_error(&err);
    }
  } else {
    p = eval_to_string(expr, false, false);
    xfree(expr);
  }
  expr_map_lock--;
  curwin->w_cursor = save_cursor;
  msg_col = save_msg_col;
  msg_row = save_msg_row;

  if (p == NULL) {
    return NULL;
  }

  char *res = NULL;

  if (replace_keycodes) {
    replace_termcodes(p, strlen(p), &res, 0, REPTERM_DO_LT, NULL, p_cpo);
  } else {
    // Escape K_SPECIAL in the result to be able to use the string as typeahead.
    res = vim_strsave_escape_ks(p);
  }
  xfree(p);

  return res;
}

/// Write map commands for the current mappings to an .exrc file.
/// Return FAIL on error, OK otherwise.
///
/// @param buf  buffer for local mappings or NULL
int makemap(FILE *fd, buf_T *buf)
{
  bool did_cpo = false;

  // Do the loop twice: Once for mappings, once for abbreviations.
  // Then loop over all map hash lists.
  for (int abbr = 0; abbr < 2; abbr++) {
    for (int hash = 0; hash < 256; hash++) {
      mapblock_T *mp;
      if (abbr) {
        if (hash > 0) {                 // there is only one abbr list
          break;
        }
        if (buf != NULL) {
          mp = buf->b_first_abbr;
        } else {
          mp = first_abbr;
        }
      } else {
        if (buf != NULL) {
          mp = buf->b_maphash[hash];
        } else {
          mp = maphash[hash];
        }
      }

      for (; mp; mp = mp->m_next) {
        if (rs_makemap_should_skip(mp)) {
          continue;
        }

        // Decompose mode into prefix characters (Rust)
        MakemapModeResult mr = rs_makemap_mode_chars(mp->m_mode, abbr);
        if (mr.error) {
          iemsg(_("E228: makemap: Illegal mode"));
          return FAIL;
        }
        char c1 = mr.c1;
        char c2 = mr.c2;
        char c3 = mr.c3;
        char *cmd = mr.use_bang ? "map!" : (abbr ? "abbr" : "map");

        do {  // do this twice if c2 is set, 3 times with c3
          if (!did_cpo && rs_makemap_needs_cpo(mp)) {
            did_cpo = true;
            if (fprintf(fd, "let s:cpo_save=&cpo") < 0
                || put_eol(fd) < 0
                || fprintf(fd, "set cpo&vim") < 0
                || put_eol(fd) < 0) {
              return FAIL;
            }
          }
          if (c1 && putc(c1, fd) < 0) {
            return FAIL;
          }
          if (mp->m_noremap != REMAP_YES && fprintf(fd, "nore") < 0) {
            return FAIL;
          }
          if (fputs(cmd, fd) < 0) {
            return FAIL;
          }
          if (buf != NULL && fputs(" <buffer>", fd) < 0) {
            return FAIL;
          }
          if (mp->m_nowait && fputs(" <nowait>", fd) < 0) {
            return FAIL;
          }
          if (mp->m_silent && fputs(" <silent>", fd) < 0) {
            return FAIL;
          }
          if (mp->m_expr && fputs(" <expr>", fd) < 0) {
            return FAIL;
          }

          if (putc(' ', fd) < 0
              || put_escstr(fd, mp->m_keys, 0) == FAIL
              || putc(' ', fd) < 0
              || put_escstr(fd, mp->m_str, 1) == FAIL
              || put_eol(fd) < 0) {
            return FAIL;
          }
          c1 = c2;
          c2 = c3;
          c3 = NUL;
        } while (c1 != NUL);
      }
    }
  }
  if (did_cpo) {
    if (fprintf(fd, "let &cpo=s:cpo_save") < 0
        || put_eol(fd) < 0
        || fprintf(fd, "unlet s:cpo_save") < 0
        || put_eol(fd) < 0) {
      return FAIL;
    }
  }
  return OK;
}

// write escape string to file
// "what": 0 for :map lhs, 1 for :map rhs, 2 for :set
//
// return FAIL for failure, OK otherwise
int put_escstr(FILE *fd, const char *strstart, int what)
{
  uint8_t *str = (uint8_t *)strstart;

  // :map xx <Nop>
  if (*str == NUL && what == 1) {
    if (fprintf(fd, "<Nop>") < 0) {
      return FAIL;
    }
    return OK;
  }

  for (; *str != NUL; str++) {
    // Check for a multi-byte character, which may contain escaped
    // K_SPECIAL bytes.
    const char *p = mb_unescape((const char **)&str);
    if (p != NULL) {
      while (*p != NUL) {
        if (fputc(*p++, fd) < 0) {
          return FAIL;
        }
      }
      str--;
      continue;
    }

    int c = *str;
    // Special key codes have to be translated to be able to make sense
    // when they are read back.
    if (c == K_SPECIAL && what != 2) {
      int modifiers = 0;
      if (str[1] == KS_MODIFIER) {
        modifiers = str[2];
        str += 3;

        // Modifiers can be applied too to multi-byte characters.
        p = mb_unescape((const char **)&str);

        if (p == NULL) {
          c = *str;
        } else {
          // retrieve codepoint (character number) from unescaped string
          c = utf_ptr2char(p);
          str--;
        }
      }
      if (c == K_SPECIAL) {
        c = TO_SPECIAL(str[1], str[2]);
        str += 2;
      }
      if (IS_SPECIAL(c) || modifiers) {         // special key
        if (fputs(get_special_key_name(c, modifiers), fd) < 0) {
          return FAIL;
        }
        continue;
      }
    }

    // A '\n' in a map command should be written as <NL>.
    // A '\n' in a set command should be written as \^V^J.
    if (c == NL) {
      if (what == 2) {
        if (fprintf(fd, "\\\026\n") < 0) {
          return FAIL;
        }
      } else {
        if (fprintf(fd, "<NL>") < 0) {
          return FAIL;
        }
      }
      continue;
    }

    int esc = rs_put_escstr_escape_type(what, c, str == (uint8_t *)strstart ? 1 : 0);
    if (esc == 1) {
      if (putc('\\', fd) < 0) {
        return FAIL;
      }
    } else if (esc == 2) {
      if (putc(Ctrl_V, fd) < 0) {
        return FAIL;
      }
    }
    if (putc(c, fd) < 0) {
      return FAIL;
    }
  }
  return OK;
}

/// Check the string "keys" against the lhs of all mappings.
/// Return pointer to rhs of mapping (mapblock->m_str).
/// NULL when no mapping found.
///
/// @param exact  require exact match
/// @param ign_mod  ignore preceding modifier
/// @param abbr  do abbreviations
/// @param mp_ptr  return: pointer to mapblock or NULL
/// @param local_ptr  return: buffer-local mapping or NULL
char *check_map(char *keys, int mode, int exact, int ign_mod, int abbr, mapblock_T **mp_ptr,
                int *local_ptr, int *rhs_lua)
{
  return rs_check_map(keys, mode, exact, ign_mod, abbr, mp_ptr, local_ptr, rhs_lua);
}

/// "hasmapto()" function
void f_hasmapto(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  const char *mode;
  const char *const name = tv_get_string(&argvars[0]);
  bool abbr = false;
  char buf[NUMBUFLEN];
  if (argvars[1].v_type == VAR_UNKNOWN) {
    mode = "nvo";
  } else {
    mode = tv_get_string_buf(&argvars[1], buf);
    if (argvars[2].v_type != VAR_UNKNOWN) {
      abbr = tv_get_number(&argvars[2]);
    }
  }

  rettv->vval.v_number = map_to_exists(name, mode, abbr);
}

/// Fill a Dict with all applicable maparg() like dictionaries
///
/// @param mp            The maphash that contains the mapping information
/// @param buffer_value  The "buffer" value
/// @param abbr          True if abbreviation
/// @param compatible    True for compatible with old maparg() dict
///
/// @return  Dict.
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
    // Keep old compatible behavior
    // This is unable to determine whether a mapping is a <script> mapping
    noremap_value = !!mp->m_noremap;
  } else {
    // Distinguish between <script> mapping
    // If it's not a <script> mapping, check if it's a noremap
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
    // Also add the value for the simplified entry.
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
  const int mode = get_map_mode((char **)&which, 0);

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

/// Get the mapping mode from the mode string.
/// It may contain multiple characters, eg "nox", or "!", or ' '
/// Return 0 if there is an error.
static int get_map_mode_string(const char *const mode_string, const bool abbr)
{
  // Delegate to Rust implementation
  return rs_get_map_mode_string(mode_string, abbr ? 1 : 0);
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
  const int mode = get_map_mode_string(which, is_abbr);
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

  set_maparg_rhs(orig_rhs, strlen(orig_rhs), rhs_lua, sid, p_cpo, &args);

  mapblock_T **map_table = buffer ? curbuf->b_maphash : maphash;
  mapblock_T **abbr_table = buffer ? &curbuf->b_first_abbr : &first_abbr;

  // Delete any existing mapping for this lhs and mode.
  MapArguments unmap_args = MAP_ARGUMENTS_INIT;
  set_maparg_lhs_rhs(lhs, strlen(lhs), "", 0, LUA_NOREF, p_cpo, &unmap_args);
  unmap_args.buffer = buffer;
  buf_do_map(MAPTYPE_UNMAP_LHS, &unmap_args, mode, is_abbr, curbuf);
  xfree(unmap_args.rhs);
  xfree(unmap_args.orig_rhs);

  mapblock_T *mp_result[2] = { NULL, NULL };

  mp_result[0] = map_add(curbuf, map_table, abbr_table, lhsraw, &args,
                         noremap, mode, is_abbr, sid, lnum, false);
  if (lhsrawalt != NULL) {
    mp_result[1] = map_add(curbuf, map_table, abbr_table, lhsrawalt, &args,
                           noremap, mode, is_abbr, sid, lnum, true);
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
void f_maparg(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  get_maparg(argvars, rettv, true);
}

/// "mapcheck()" function
void f_mapcheck(typval_T *argvars, typval_T *rettv, EvalFuncData fptr)
{
  get_maparg(argvars, rettv, false);
}

/// Add a mapping. Unlike @ref do_map this copies the string arguments, so
/// static or read-only strings can be used.
///
/// @param lhs  C-string containing the lhs of the mapping
/// @param rhs  C-string containing the rhs of the mapping
/// @param mode  Bitflags representing the mode in which to set the mapping.
///              See @ref get_map_mode.
/// @param buffer  If true, make a buffer-local mapping for curbuf
void add_map(char *lhs, char *rhs, int mode, bool buffer)
{
  MapArguments args = MAP_ARGUMENTS_INIT;
  set_maparg_lhs_rhs(lhs, strlen(lhs), rhs, strlen(rhs), LUA_NOREF, p_cpo, &args);
  args.buffer = buffer;

  buf_do_map(MAPTYPE_NOREMAP, &args, mode, false, curbuf);
  xfree(args.rhs);
  xfree(args.orig_rhs);
}

// Langmap subsystem — now implemented in Rust (src/nvim-rs/mapping/src/langmap.rs).
// These thin C wrappers delegate to the Rust implementations.

/// Apply 'langmap' to multi-byte character "c" and return the result.
int langmap_adjust_mb(int c)
{
  return rs_langmap_adjust_mb(c);
}

void langmap_init(void)
{
  rs_langmap_init();
}

/// Called when langmap option is set; the language map can be
/// changed at any time!
const char *did_set_langmap(optset_T *args)
{
  int err = rs_langmap_parse(p_langmap, args->os_errbuf, args->os_errbuflen);
  if (err != 0) {
    return args->os_errbuf;
  }
  return NULL;
}

static void do_exmap(exarg_T *eap, int isabbrev)
{
  char *cmdp = eap->cmd;
  int mode = get_map_mode(&cmdp, eap->forceit || isabbrev);

  int maptype;
  if (*cmdp == 'n') {
    maptype = MAPTYPE_NOREMAP;
  } else if (*cmdp == 'u') {
    maptype = MAPTYPE_UNMAP;
  } else {
    maptype = MAPTYPE_MAP;
  }
  MapArguments parsed_args;
  int result = str_to_mapargs(eap->arg, maptype == MAPTYPE_UNMAP, &parsed_args);
  switch (result) {
  case 0:
    break;
  case 1:
    emsg(_(e_invarg));
    goto free_rhs;
    break;
  default:
    assert(false && "Unknown return code from str_to_mapargs!");
    goto free_rhs;
  }
  switch (buf_do_map(maptype, &parsed_args, mode, isabbrev, curbuf)) {
  case 1:
    emsg(_(e_invarg));
    break;
  case 2:
    emsg(isabbrev ? _(e_noabbr) : _(e_nomap));
    break;
  case 5:
    semsg(isabbrev ? _(e_abbreviation_already_exists_for_str)
                   : _(e_mapping_already_exists_for_str),
          parsed_args.lhs);
    break;
  case 6:
    semsg(isabbrev ? _(e_global_abbreviation_already_exists_for_str)
                   : _(e_global_mapping_already_exists_for_str),
          parsed_args.lhs);
  }
free_rhs:
  xfree(parsed_args.rhs);
  xfree(parsed_args.orig_rhs);
}

/// ":abbreviate" and friends.
void ex_abbreviate(exarg_T *eap)
{
  do_exmap(eap, true);          // almost the same as mapping
}

/// ":map" and friends.
void ex_map(exarg_T *eap)
{
  // If we are in a secure mode we print the mappings for security reasons.
  if (secure) {
    secure = 2;
    msg_outtrans(eap->cmd, 0, false);
    msg_putchar('\n');
  }
  do_exmap(eap, false);
}

/// ":unmap" and friends.
void ex_unmap(exarg_T *eap)
{
  do_exmap(eap, false);
}

/// ":mapclear" and friends.
void ex_mapclear(exarg_T *eap)
{
  do_mapclear(eap->cmd, eap->arg, eap->forceit, false);
}

/// ":abclear" and friends.
void ex_abclear(exarg_T *eap)
{
  do_mapclear(eap->cmd, eap->arg, true, true);
}

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

  if (!set_maparg_lhs_rhs(lhs.data, lhs.size,
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
  int mode_val = get_map_mode(&p, forceit);
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

  // buf_do_map() reads noremap/unmap as its own argument.
  int maptype_val = MAPTYPE_MAP;
  if (is_unmap) {
    maptype_val = MAPTYPE_UNMAP;
  } else if (is_noremap) {
    maptype_val = MAPTYPE_NOREMAP;
  }

  switch (buf_do_map(maptype_val, &parsed_args, mode_val, is_abbrev, target_buf)) {
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
  int int_mode = get_map_mode(&p, forceit);
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

// =============================================================================
// Rust FFI accessor functions
// =============================================================================

// Field accessors for mapblock_T (used by Rust via FFI)

mapblock_T *nvim_mapblock_get_next(mapblock_T *mp)
{
  return mp ? mp->m_next : NULL;
}

mapblock_T *nvim_mapblock_get_alt(mapblock_T *mp)
{
  return mp ? mp->m_alt : NULL;
}

const char *nvim_mapblock_get_keys(mapblock_T *mp)
{
  return mp ? mp->m_keys : NULL;
}

const char *nvim_mapblock_get_str(mapblock_T *mp)
{
  return mp ? mp->m_str : NULL;
}

const char *nvim_mapblock_get_orig_str(mapblock_T *mp)
{
  return mp ? mp->m_orig_str : NULL;
}

int nvim_mapblock_get_keylen(mapblock_T *mp)
{
  return mp ? mp->m_keylen : 0;
}

int nvim_mapblock_get_mode(mapblock_T *mp)
{
  return mp ? mp->m_mode : 0;
}

int nvim_mapblock_get_simplified(mapblock_T *mp)
{
  return mp ? mp->m_simplified : 0;
}

int nvim_mapblock_get_noremap(mapblock_T *mp)
{
  return mp ? mp->m_noremap : 0;
}

int nvim_mapblock_is_silent(mapblock_T *mp)
{
  return mp ? mp->m_silent : 0;
}

int nvim_mapblock_is_nowait(mapblock_T *mp)
{
  return mp ? mp->m_nowait : 0;
}

int nvim_mapblock_is_expr(mapblock_T *mp)
{
  return mp ? mp->m_expr : 0;
}

LuaRef nvim_mapblock_get_luaref(mapblock_T *mp)
{
  return mp ? mp->m_luaref : LUA_NOREF;
}

const char *nvim_mapblock_get_desc(mapblock_T *mp)
{
  return mp ? mp->m_desc : NULL;
}

int nvim_mapblock_get_replace_keycodes(mapblock_T *mp)
{
  return mp ? mp->m_replace_keycodes : 0;
}

// Hash table accessors

mapblock_T *nvim_get_maphash_entry(int index)
{
  if (index < 0 || index >= MAX_MAPHASH) {
    return NULL;
  }
  return maphash[index];
}

mapblock_T *nvim_get_first_abbr(void)
{
  return first_abbr;
}

mapblock_T *nvim_buf_get_maphash_entry(buf_T *buf, int index)
{
  if (!buf || index < 0 || index >= MAX_MAPHASH) {
    return NULL;
  }
  return buf->b_maphash[index];
}

mapblock_T *nvim_buf_get_first_abbr(buf_T *buf)
{
  return buf ? buf->b_first_abbr : NULL;
}

// p_cpo accessor for Rust
const char *nvim_mapping_get_p_cpo(void)
{
  return p_cpo;
}

// Static assertions for MapArguments struct layout (Rust #[repr(C)] must match)
_Static_assert(sizeof(MapArguments) == 184,
               "MapArguments size mismatch with Rust");
_Static_assert(offsetof(struct map_arguments, buffer) == 0,
               "MapArguments.buffer offset mismatch");
_Static_assert(offsetof(struct map_arguments, lhs) == 8,
               "MapArguments.lhs offset mismatch");
_Static_assert(offsetof(struct map_arguments, rhs) == 136,
               "MapArguments.rhs offset mismatch");
_Static_assert(offsetof(struct map_arguments, rhs_lua) == 152,
               "MapArguments.rhs_lua offset mismatch");
_Static_assert(offsetof(struct map_arguments, desc) == 176,
               "MapArguments.desc offset mismatch");

// Langmap C accessors for Rust

uint8_t nvim_langmap_mapchar_get(int index)
{
  if (index < 0 || index >= 256) {
    return (uint8_t)index;
  }
  return langmap_mapchar[index];
}

void nvim_langmap_mapchar_set(int index, uint8_t value)
{
  if (index >= 0 && index < 256) {
    langmap_mapchar[index] = value;
  }
}

int nvim_mapping_utf_ptr2char(const char *p)
{
  return utf_ptr2char(p);
}

int nvim_mapping_utfc_ptr2len(const char *p)
{
  return utfc_ptr2len(p);
}

// Write accessors for Rust mutation primitives (Phase 5)

void nvim_mapping_emsg_invarg(void)
{
  emsg(_(e_invarg));
}

void nvim_set_maphash_entry(int index, mapblock_T *mp)
{
  if (index >= 0 && index < MAX_MAPHASH) {
    maphash[index] = mp;
  }
}

void nvim_set_first_abbr(mapblock_T *mp)
{
  first_abbr = mp;
}

void nvim_buf_set_maphash_entry(buf_T *buf, int index, mapblock_T *mp)
{
  if (buf && index >= 0 && index < MAX_MAPHASH) {
    buf->b_maphash[index] = mp;
  }
}

void nvim_buf_set_first_abbr(buf_T *buf, mapblock_T *mp)
{
  if (buf) {
    buf->b_first_abbr = mp;
  }
}

void nvim_mapblock_set_next(mapblock_T *mp, mapblock_T *next)
{
  if (mp) {
    mp->m_next = next;
  }
}

void nvim_mapblock_set_mode(mapblock_T *mp, int mode)
{
  if (mp) {
    mp->m_mode = mode;
  }
}

/// Free a mapblock and advance the pointer to the next entry.
/// This is a C-side helper because it calls xfree and NLUA_CLEAR_REF.
void nvim_mapblock_free(mapblock_T *mp)
{
  if (!mp) {
    return;
  }
  xfree(mp->m_keys);
  if (mp->m_alt != NULL) {
    mp->m_alt->m_alt = NULL;
  } else {
    NLUA_CLEAR_REF(mp->m_luaref);
    xfree(mp->m_str);
    xfree(mp->m_orig_str);
    xfree(mp->m_desc);
  }
  xfree(mp);
}

// =========================================================================
// Phase 6 C accessors for buf_do_map / do_map Rust migration
// =========================================================================

/// Wrapper for showmap() callable from Rust.
void nvim_showmap(mapblock_T *mp, int local)
{
  showmap(mp, local != 0);
}

/// Wrapper for map_add() callable from Rust.
/// @param is_buf_local  if true, uses buf->b_maphash / buf->b_first_abbr;
///                      otherwise uses global maphash / first_abbr.
mapblock_T *nvim_map_add(buf_T *buf, int is_buf_local, const char *keys,
                         void *args_ptr, int noremap, int mode,
                         int is_abbr, int sid, int lnum, int simplified)
{
  MapArguments *args = (MapArguments *)args_ptr;
  mapblock_T **mt = is_buf_local ? buf->b_maphash : maphash;
  mapblock_T **at = is_buf_local ? &buf->b_first_abbr : &first_abbr;
  return map_add(buf, mt, at, keys, args, noremap, mode,
                 is_abbr != 0, (scid_T)sid, (linenr_T)lnum, simplified != 0);
}

/// Reuse an existing mapblock entry with new RHS data from args.
/// This handles the complex field-update pattern from buf_do_map (lines 598-624).
void nvim_mapblock_reuse(mapblock_T *mp, void *args_ptr,
                         int noremap, int mode, int simplified)
{
  MapArguments *args = (MapArguments *)args_ptr;
  if (!mp || !args) {
    return;
  }
  if (mp->m_alt != NULL) {
    mp->m_alt = mp->m_alt->m_alt = NULL;
  } else {
    NLUA_CLEAR_REF(mp->m_luaref);
    xfree(mp->m_str);
    xfree(mp->m_orig_str);
    xfree(mp->m_desc);
  }
  mp->m_str = args->rhs;
  mp->m_orig_str = args->orig_rhs;
  mp->m_luaref = args->rhs_lua;
  mp->m_noremap = noremap;
  mp->m_nowait = args->nowait;
  mp->m_silent = args->silent;
  mp->m_mode = mode;
  mp->m_simplified = simplified != 0;
  mp->m_expr = args->expr;
  mp->m_replace_keycodes = args->replace_keycodes;
  mp->m_script_ctx = current_sctx;
  mp->m_script_ctx.sc_lnum += SOURCING_LNUM;
  nlua_set_sctx(&mp->m_script_ctx);
  mp->m_desc = args->desc;
}

/// Link two mapblocks as alternates of each other.
void nvim_mapblock_set_alt(mapblock_T *a, mapblock_T *b)
{
  if (a) {
    a->m_alt = b;
  }
  if (b) {
    b->m_alt = a;
  }
}

/// Nullify args ownership fields (rhs, orig_rhs, rhs_lua, desc) so that
/// do_map's cleanup won't free memory now owned by the mapblock.
void nvim_mapargs_take_ownership(void *args_ptr)
{
  MapArguments *args = (MapArguments *)args_ptr;
  if (!args) {
    return;
  }
  args->rhs = NULL;
  args->orig_rhs = NULL;
  args->rhs_lua = LUA_NOREF;
  args->desc = NULL;
}

// nvim_get_got_int, nvim_get_mapped_ctrl_c, nvim_set_mapped_ctrl_c,
// nvim_msg_start already defined in other files — reuse those.

/// Set the global no_abbr flag.
void nvim_mapping_set_no_abbr(int val)
{
  no_abbr = val != 0;
}

/// Get the buffer-local b_mapped_ctrl_c.
int nvim_mapping_buf_get_mapped_ctrl_c(buf_T *buf)
{
  return buf ? buf->b_mapped_ctrl_c : 0;
}

/// Set the buffer-local b_mapped_ctrl_c.
void nvim_mapping_buf_set_mapped_ctrl_c(buf_T *buf, int val)
{
  if (buf) {
    buf->b_mapped_ctrl_c = val;
  }
}

/// Call msg_ext_set_kind("list_cmd") from Rust.
void nvim_mapping_msg_ext_set_kind_list_cmd(void)
{
  msg_ext_set_kind("list_cmd");
}

/// Display "No mapping found" or "No abbreviation found" message.
void nvim_mapping_msg_no_mapping(int is_abbr)
{
  if (is_abbr) {
    msg(_("No abbreviation found"), 0);
  } else {
    msg(_("No mapping found"), 0);
  }
}

/// Wrapper for vim_iswordp().
int nvim_vim_iswordp(const char *p)
{
  return vim_iswordp(p);
}

/// Get strlen(mp->m_str) for round-2 unmap matching.
int nvim_mapblock_get_str_len(mapblock_T *mp)
{
  return mp ? (int)strlen(mp->m_str) : 0;
}

/// Check if map_table for this operation is buffer-local.
/// Used to determine "local" display flag in showmap.
int nvim_mapping_is_buf_maphash(buf_T *buf, int is_buf_local)
{
  // In the C code: map_table != maphash  means buffer-local
  // is_buf_local directly tells us this
  return is_buf_local;
}

/// Wrapper for mapblock_free() using pointer-to-pointer pattern.
/// Unlinks from the specified list and frees.
/// Returns the new current node (*mpp after deletion).
mapblock_T *nvim_mapblock_free_in_list(buf_T *buf, int hash, int is_abbr,
                                       int is_buf_local, mapblock_T *mp)
{
  // Find the pointer-to-pointer that points to mp
  mapblock_T **mpp;
  if (is_abbr) {
    mpp = is_buf_local ? &buf->b_first_abbr : &first_abbr;
  } else {
    mapblock_T **mt = is_buf_local ? buf->b_maphash : maphash;
    mpp = &mt[hash];
  }

  // Walk to find the entry
  while (*mpp != NULL) {
    if (*mpp == mp) {
      mapblock_free(mpp);
      return *mpp;  // mpp now points to the next entry
    }
    mpp = &((*mpp)->m_next);
  }
  return NULL;
}

/// Move a mapblock from one hash bucket to another.
/// Unlinks from old bucket and inserts at head of new bucket.
void nvim_mapblock_rehash(buf_T *buf, int is_buf_local,
                          int old_hash, int new_hash, mapblock_T *mp)
{
  mapblock_T **mt = is_buf_local ? buf->b_maphash : maphash;

  // Unlink from old hash bucket
  mapblock_T **mpp = &mt[old_hash];
  while (*mpp != NULL) {
    if (*mpp == mp) {
      *mpp = mp->m_next;
      break;
    }
    mpp = &((*mpp)->m_next);
  }

  // Insert at head of new hash bucket
  mp->m_next = mt[new_hash];
  mt[new_hash] = mp;
}

/// Format a langmap error message into the provided buffer.
/// msgid: 357 = E357 (matching missing), 358 = E358 (extra chars)
void nvim_langmap_format_error(char *buf, size_t buflen, int msgid, const char *arg)
{
  if (msgid == 357) {
    snprintf(buf, buflen,
             _("E357: 'langmap': Matching character missing for %s"), arg);
  } else if (msgid == 358) {
    snprintf(buf, buflen,
             _("E358: 'langmap': Extra characters after semicolon: %s"), arg);
  }
}

// =============================================================================
// Phase 7 C accessor functions for completion + serialization
// =============================================================================

// expand_T field accessors: reuse nvim_expand_set_context and
// nvim_expand_set_pattern from cmdexpand.c (already exist).

// Global expand state accessors
void nvim_mapping_set_expand_mapmodes(int val)
{
  expand_mapmodes = val;
}

void nvim_mapping_set_expand_isabbrev(int val)
{
  expand_isabbrev = (val != 0);
}

void nvim_mapping_set_expand_buffer(int val)
{
  expand_buffer = (val != 0);
}

int nvim_mapping_get_expand_mapmodes(void)
{
  return expand_mapmodes;
}

int nvim_mapping_get_expand_isabbrev(void)
{
  return expand_isabbrev ? 1 : 0;
}

int nvim_mapping_get_expand_buffer(void)
{
  return expand_buffer ? 1 : 0;
}

int nvim_mapping_get_cmd_map(void)
{
  return (int)CMD_map;
}

int nvim_mapping_get_cmd_unmap(void)
{
  return (int)CMD_unmap;
}

char *nvim_mapping_skipwhite(const char *p)
{
  return skipwhite(p);
}

// ExpandMappings helper: call vim_regexec on regmatch (passed opaquely)
int nvim_mapping_vim_regexec(void *regmatch, const char *s)
{
  return vim_regexec((regmatch_T *)regmatch, s, 0) ? 1 : 0;
}

int nvim_mapping_fuzzy_match_str(const char *s, const char *pat)
{
  return fuzzy_match_str((char *)s, pat);
}

int nvim_mapping_cmdline_fuzzy_complete(const char *pat)
{
  return cmdline_fuzzy_complete(pat) ? 1 : 0;
}

// garray_T wrappers (garray is passed opaquely)
void nvim_mapping_ga_init_str(void *ga)
{
  ga_init((garray_T *)ga, (int)sizeof(char *), 3);
}

void nvim_mapping_ga_init_fuzmatch(void *ga)
{
  ga_init((garray_T *)ga, (int)sizeof(fuzmatch_str_T), 3);
}

void nvim_mapping_ga_append_str(void *ga, const char *s)
{
  GA_APPEND(char *, (garray_T *)ga, xstrdup(s));
}

void nvim_mapping_ga_append_fuzmatch(void *ga, const char *s, int score)
{
  garray_T *g = (garray_T *)ga;
  GA_APPEND(fuzmatch_str_T, g, ((fuzmatch_str_T){
    .idx = g->ga_len,
    .str = xstrdup(s),
    .score = score,
  }));
}

int nvim_mapping_ga_len(const void *ga)
{
  return ((const garray_T *)ga)->ga_len;
}

/// Finish ExpandMappings: sort, dedup, set output.
/// Returns OK or FAIL.
int nvim_mapping_expand_finish(void *ga_ptr, int fuzzy,
                               int *numMatches, char ***matches)
{
  garray_T *ga = (garray_T *)ga_ptr;

  if (ga->ga_len == 0) {
    return FAIL;
  }

  if (!fuzzy) {
    *matches = ga->ga_data;
    *numMatches = ga->ga_len;
  } else {
    fuzzymatches_to_strmatches(ga->ga_data, matches, ga->ga_len, false);
    *numMatches = ga->ga_len;
  }

  int count = *numMatches;
  if (count > 1) {
    // Sort the matches (fuzzy matching already sorts)
    if (!fuzzy) {
      sort_strings(*matches, count);
    }

    // Remove duplicates
    char **ptr1 = *matches;
    char **ptr2 = ptr1 + 1;
    char **ptr3 = ptr1 + count;

    while (ptr2 < ptr3) {
      if (strcmp(*ptr1, *ptr2) != 0) {
        *++ptr1 = *ptr2++;
      } else {
        xfree(*ptr2++);
        count--;
      }
    }
  }

  *numMatches = count;
  return count == 0 ? FAIL : OK;
}
