// spellfile.c: read/write Neovim spell files (.spl, .sug, .add).
// Most logic has been migrated to Rust in src/nvim-rs/spell/.
// File format documentation: see src/nvim-rs/spell/src/spellfile.rs
// and the original Vim source (runtime/doc/spell.txt, spellfile.c).
#include <stdint.h>
#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <limits.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "nvim/arglist.h"
#include "nvim/ascii_defs.h"
#include "nvim/buffer.h"
#include "nvim/buffer_defs.h"
#include "nvim/charset.h"
#include "nvim/drawscreen.h"
#include "nvim/errors.h"
#include "nvim/ex_cmds_defs.h"
#include "nvim/fileio.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"
#include "nvim/gettext_defs.h"
#include "nvim/globals.h"
#include "nvim/hashtab.h"
#include "nvim/hashtab_defs.h"
#include "nvim/macros_defs.h"
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
#include "nvim/os/os.h"
#include "nvim/os/os_defs.h"
#include "nvim/os/stdpaths_defs.h"
#include "nvim/os/time.h"
#include "nvim/os/time_defs.h"
#include "nvim/path.h"
#include "nvim/pos_defs.h"
#include "nvim/regexp.h"
#include "nvim/runtime.h"
#include "nvim/runtime_defs.h"
#include "nvim/spell.h"
#include "nvim/spell_defs.h"
#include "nvim/spellfile.h"
#include "nvim/strings.h"
#include "nvim/types_defs.h"
#include "nvim/ui.h"
#include "nvim/vim_defs.h"
#include "nvim/undo.h"

// All spell file I/O logic has been migrated to Rust (src/nvim-rs/spell/).
// Remaining content: struct definitions (sblock_T, wordnode_S, spellinfo_S),
// includes, and the nvim_spell_toupper macro wrapper.


// Arena allocator (getroom/free_blocks) and spell file I/O (spell_load_file,
// suggest_load_files) are now exported directly from Rust with their C names.

// Arena block for word tree; full definition required because free_blocks() takes sblock_T*.
typedef struct sblock_S sblock_T;
struct sblock_S {
  int sb_used;                  // nr of bytes already in use
  sblock_T *sb_next;         // next block in list
  char sb_data[];            // data
};

// A node in the tree.
// (Forward declaration is in spellfile.h; full definition here.)
struct wordnode_S {
  union {   // shared to save space
    uint8_t hashkey[6];         // the hash key, only used while compressing
    int index;                  // index in written nodes (valid after first
                                // round)
  } wn_u1;
  union {   // shared to save space
    wordnode_T *next;           // next node with same hash key
    wordnode_T *wnode;          // parent node that will write this node
  } wn_u2;
  wordnode_T *wn_child;        // child (next byte in word)
  wordnode_T *wn_sibling;      // next sibling (alternate byte in word,
                               //   always sorted)
  int wn_refs;                 // Nr. of references to this node.  Only
                               //   relevant for first node in a list of
                               //   siblings, in following siblings it is
                               //   always one.
  uint8_t wn_byte;             // Byte for this node. NUL for word end

  // Info for when "wn_byte" is NUL.
  // In PREFIXTREE "wn_region" is used for the prefcondnr.
  // In the soundfolded word tree "wn_flags" has the MSW of the wordnr and
  // "wn_region" the LSW of the wordnr.
  uint8_t wn_affixID;           // supported/required prefix ID or 0
  uint16_t wn_flags;            // WF_ flags
  int16_t wn_region;            // region mask

};

// Info used while reading the spell files.
// (struct tag spellinfo_S is forward-declared in spellfile.h for opaque access.)
typedef struct spellinfo_S {
  wordnode_T *si_foldroot;     // tree with case-folded words
  int si_foldwcount;           // nr of words in si_foldroot

  wordnode_T *si_keeproot;     // tree with keep-case words
  int si_keepwcount;           // nr of words in si_keeproot

  wordnode_T *si_prefroot;     // tree with postponed prefixes

  int si_sugtree;              // creating the soundfolding trie

  sblock_T *si_blocks;       // memory blocks used
  int si_blocks_cnt;           // memory blocks allocated
  int si_did_emsg;              // true when ran out of memory

  int si_compress_cnt;         // words to add before lowering
                               // compression limit
  wordnode_T *si_first_free;   // List of nodes that have been freed during
                               // compression, linked by "wn_child" field.
  int si_free_count;           // number of nodes in si_first_free
  buf_T *si_spellbuf;     // buffer used to store soundfold word table

  int si_ascii;                 // handling only ASCII words
  int si_add;                   // addition file
  int si_clear_chartab;             // when true clear char tables
  int si_region;                // region mask
  vimconv_T si_conv;            // for conversion to 'encoding'
  int si_memtot;                // runtime memory used
  int si_verbose;               // verbose messages
  int si_msg_count;             // number of words added since last message
  char *si_info;                // info text chars or NULL
  int si_region_count;          // number of regions supported (1 when there
                                // are no regions)
  char si_region_name[MAXREGIONS * 2 + 1];
  // region names; used only if
  // si_region_count > 1)

  garray_T si_rep;              // list of fromto_T entries from REP lines
  garray_T si_repsal;           // list of fromto_T entries from REPSAL lines
  garray_T si_sal;              // list of fromto_T entries from SAL lines
  char *si_sofofr;              // SOFOFROM text
  char *si_sofoto;              // SOFOTO text
  int si_nosugfile;             // NOSUGFILE item found
  int si_nosplitsugs;           // NOSPLITSUGS item found
  int si_nocompoundsugs;        // NOCOMPOUNDSUGS item found
  int si_followup;              // soundsalike: ?
  int si_collapse;              // soundsalike: ?
  hashtab_T si_commonwords;     // hashtable for common words
  time_t si_sugtime;            // timestamp for .sug file
  int si_rem_accents;           // soundsalike: remove accents
  garray_T si_map;              // MAP info concatenated
  char *si_midword;             // MIDWORD chars or NULL
  int si_compmax;               // max nr of words for compounding
  int si_compminlen;            // minimal length for compounding
  int si_compsylmax;            // max nr of syllables for compounding
  int si_compoptions;           // COMP_ flags
  garray_T si_comppat;          // CHECKCOMPOUNDPATTERN items, each stored as
                                // a string
  char *si_compflags;           // flags used for compounding
  char si_nobreak;              // NOBREAK
  char *si_syllable;            // syllable string
  garray_T si_prefcond;         // table with conditions for postponed
                                // prefixes, each stored as a string
  int si_newprefID;             // current value for ah_newID
  int si_newcompID;             // current value for compound ID
} spellinfo_T;

#include "spellfile.c.generated.h"

// ex_mkspell, ex_spell, write_vim_spell, mkspell, spell_add_word, and
// init_spellfile are all implemented in Rust and exported with their C names.

/// C wrapper for SPELL_TOUPPER macro (used by Rust via FFI).
int nvim_spell_toupper(int c) { return SPELL_TOUPPER(c); }

/// Set the spell character tables from strings in the .spl file.
///
/// @param cnt  length of "flags"

