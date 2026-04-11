// spell.c: code for spell checking
//
// See spellfile.c for the Vim spell file format.
//
// The spell checking mechanism uses a tree (aka trie).  Each node in the tree
// has a list of bytes that can appear (siblings).  For each byte there is a
// pointer to the node with the byte that follows in the word (child).
//
// A NUL byte is used where the word may end.  The bytes are sorted, so that
// binary searching can be used and the NUL bytes are at the start.  The
// number of possible bytes is stored before the list of bytes.
//
// The tree uses two arrays: "byts" stores the characters, "idxs" stores
// either the next index or flags.  The tree starts at index 0.  For example,
// to lookup "vi" this sequence is followed:
//      i = 0
//      len = byts[i]
//      n = where "v" appears in byts[i + 1] to byts[i + len]
//      i = idxs[n]
//      len = byts[i]
//      n = where "i" appears in byts[i + 1] to byts[i + len]
//      i = idxs[n]
//      len = byts[i]
//      find that byts[i + 1] is 0, idxs[i + 1] has flags for "vi".
//
// There are two word trees: one with case-folded words and one with words in
// original case.  The second one is only used for keep-case words and is
// usually small.
//
// There is one additional tree for when not all prefixes are applied when
// generating the .spl file.  This tree stores all the possible prefixes, as
// if they were words.  At each word (prefix) end the prefix nr is stored, the
// following word must support this prefix nr.  And the condition nr is
// stored, used to lookup the condition that the word must match with.
//
// Thanks to Olaf Seibert for providing an example implementation of this tree
// and the compression mechanism.
// LZ trie ideas, original link (now dead)
//      irb.hr/hr/home/ristov/papers/RistovLZtrieRevision1.pdf
// More papers: http://www-igm.univ-mlv.fr/~laporte/publi_en.html
//
// Matching involves checking the caps type: Onecap ALLCAP KeepCap.
//
// Why doesn't Vim use aspell/ispell/myspell/etc.?
// See ":help develop-spell".

// Use SPELL_PRINTTREE for debugging: dump the word tree after adding a word.
// Only use it for small word lists!

// Use SPELL_COMPRESS_ALWAYS for debugging: compress the word tree after
// adding a word.  Only use it for small word lists!

// Use DEBUG_TRIEWALK to print the changes made in suggest_trie_walk() for a
// specific word.

#include <assert.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

#include "nvim/ascii_defs.h"
#include "nvim/autocmd.h"
#include "nvim/autocmd_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/change.h"
#include "nvim/charset.h"
#include "nvim/cursor.h"
#include "nvim/decoration.h"
#include "nvim/decoration_provider.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/ex_docmd.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/highlight_defs.h"
#include "nvim/insexpand.h"
#include "nvim/log.h"
#include "nvim/macros_defs.h"
#include "nvim/mark_defs.h"
#include "nvim/mbyte.h"
#include "nvim/mbyte_defs.h"
#include "nvim/memline.h"
#include "nvim/memory.h"
#include "nvim/message.h"
#include "nvim/option.h"
#include "nvim/option_defs.h"
#include "nvim/option_vars.h"
#include "nvim/os/fs.h"
#include "nvim/os/input.h"
#include "nvim/os/os_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/regexp_defs.h"
#include "nvim/runtime.h"
#include "nvim/search.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/spellfile.h"
#include "nvim/spellsuggest.h"
#include "nvim/strings.h"
#include "nvim/syntax.h"
#include "nvim/types_defs.h"
#include "nvim/undo.h"
#include "nvim/vim_defs.h"
#include "nvim/window.h"

// First language that is loaded, start of the linked list of loaded
// languages.
slang_T *first_lang = NULL;

// file used for "zG" and "zW"
char *int_wordlist = NULL;

// spelload_T typedef moved to spell_shim.c (used by nvim_spell_load_lang).

spelltab_T spelltab;
bool did_set_spelltab;

#include "spell.c.generated.h"
extern int rs_win_valid_any_tab(win_T *win);
// Rust implementations of spell functions
extern int rs_find_region(const char *rp, const char *region);

// Static assertions to validate Rust repr(C) struct layout matches C struct layout.
// These catch layout mismatches at compile time before they cause silent bugs.
_Static_assert(sizeof(spelltab_T) == 1024, "spelltab_T size mismatch");
_Static_assert(sizeof(langp_T) == 32, "langp_T size mismatch");
_Static_assert(sizeof(slang_T) == 4344, "slang_T size mismatch");
_Static_assert(offsetof(slang_T, sl_fbyts) == 32, "sl_fbyts offset mismatch");
_Static_assert(offsetof(slang_T, sl_fidxs) == 48, "sl_fidxs offset mismatch");
_Static_assert(offsetof(slang_T, sl_sal) == 1088, "sl_sal offset mismatch");
_Static_assert(offsetof(slang_T, sl_sal_first) == 1112, "sl_sal_first offset mismatch");
_Static_assert(offsetof(slang_T, sl_rep) == 552, "sl_rep offset mismatch");
_Static_assert(offsetof(slang_T, sl_rep_first) == 576, "sl_rep_first offset mismatch");
_Static_assert(offsetof(slang_T, sl_sofo) == 2139, "sl_sofo offset mismatch");
_Static_assert(offsetof(slang_T, sl_compmax) == 424, "sl_compmax offset mismatch");
_Static_assert(offsetof(slang_T, sl_has_map) == 2721, "sl_has_map offset mismatch");
_Static_assert(offsetof(slang_T, sl_map_hash) == 2728, "sl_map_hash offset mismatch");
_Static_assert(offsetof(slang_T, sl_map_array) == 3024, "sl_map_array offset mismatch");
_Static_assert(offsetof(slang_T, sl_sounddone) == 4048, "sl_sounddone offset mismatch");
_Static_assert(offsetof(slang_T, sl_sbyts) == 2696, "sl_sbyts offset mismatch");
_Static_assert(offsetof(slang_T, sl_sidxs) == 2704, "sl_sidxs offset mismatch");

char *e_format = N_("E759: Format error in spell file");

// Remember what "z?" replaced.
char *repl_from = NULL;
char *repl_to = NULL;






// spell_move_to, decor_spell_nav_start, decor_spell_nav_col, can_syn_spell:
//   migrated to Rust: src/nvim-rs/spell/src/navigate.rs
// parse_spelllang, clear_midword, use_midword:
//   migrated to Rust: src/nvim-rs/spell/src/lang.rs
// spell_load_lang, int_wordlist_spl, spell_load_cb:
//   kept as C in spell_shim.c (complex C-only dependencies)

// Open a spell buffer.  This is a nameless buffer that is not in the buffer
// list and only contains text lines.  Can use a swapfile to reduce memory
// use.
// Most other fields are invalid!  Esp. watch out for string options being
// NULL and there is no undo info.
buf_T *open_spellbuf(void)
{
  buf_T *buf = xcalloc(1, sizeof(buf_T));

  buf->b_spell = true;
  buf->b_p_swf = true;        // may create a swap file
  if (ml_open(buf) == FAIL) {
    ELOG("Error opening a new memline");
  }
  ml_open_file(buf);          // create swap file now

  return buf;
}

// Close the buffer used for spell info.
void close_spellbuf(buf_T *buf)
{
  if (buf == NULL) {
    return;
  }

  ml_close(buf, true);
  xfree(buf);
}
