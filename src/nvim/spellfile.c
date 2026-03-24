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
#include "nvim/undo.h"
#include "nvim/vim_defs.h"

// =============================================================================
// Rust FFI declarations
// =============================================================================

// Rust implementations (direct exports)
extern int offset2bytes(int nr, uint8_t *buf);
extern bool sal_to_bool(const char *s);

// Section reading functions (Rust implementations)
// SOFO section parsing
typedef struct {
  uint8_t from[512];
  uint16_t from_len;
  uint8_t to[512];
  uint16_t to_len;
} RsSofoSection;

extern int rs_parse_sofo_section(const uint8_t *buf, size_t buf_len, RsSofoSection *section_out,
                                 size_t *consumed_out);

// WORDS section parsing
extern int rs_parse_words_entry(const uint8_t *buf, size_t buf_len, uint8_t *output,
                                size_t output_len, size_t *consumed_out);

// Tree reading functions
extern int rs_read_tree(const uint8_t *buf, size_t buf_len, uint8_t *byts, int32_t *idxs,
                        size_t array_len, bool prefixtree, int prefixcnt,
                        size_t *bytes_consumed_out, int *node_count_out);
extern int rs_read_tree_peek_nodecount(const uint8_t *buf, size_t buf_len, uint32_t *nodecount_out);

// Phase 4: Dictionary and wordfile line parsers

typedef struct {
  uint16_t word_len;       // length of unescaped word written into word_buf
  uint16_t affix_offset;   // byte offset in original line where affix starts (0xFFFF = absent)
  uint16_t affix_len;      // length of affix string in original line
} RsDicLineResult;

extern int rs_parse_dic_line(const uint8_t *line, size_t line_len,
                              uint8_t *word_buf, size_t word_buf_cap,
                              RsDicLineResult *result_out);

typedef struct {
  uint8_t  directive[16];  // "encoding\0" or "regions\0" or empty for word lines
  uint16_t word_len;       // word length (or directive value length)
  uint16_t word_end_offset; // byte offset in line where word ends (before '/')
  int      flags;          // WF_* flags (word lines only)
  int      regionmask;     // region bitmask (word lines only)
  uint8_t  region_count;   // for /regions= directives
} RsWordfileLineResult;

extern int rs_parse_wordfile_line(const uint8_t *line, size_t line_len,
                                  int region_count,
                                  RsWordfileLineResult *result_out);

extern int rs_spell_add_word_format(const uint8_t *word, size_t word_len,
                                    int what, uint8_t *buf_out, size_t buf_cap);

extern bool rs_spell_find_duplicate_word(const uint8_t *file_content, size_t content_len,
                                         const uint8_t *word, size_t word_len,
                                         size_t *offset_out);

// Phase 5: mkspell argument parsing helpers

typedef struct {
  uint8_t  fname[4096];    // NUL-terminated output filename
  uint16_t fname_len;      // length of fname (excluding NUL)
  bool     is_add;         // detected .add.spl output
  bool     is_ascii;       // detected .ascii.spl output
} RsMkspellFnameResult;

extern int rs_mkspell_output_fname(const uint8_t *const *fnames, int fcount,
                                   const uint8_t *enc,
                                   RsMkspellFnameResult *result_out);

extern int rs_mkspell_validate_args(const uint8_t *const *innames, int incount,
                                    uint8_t *region_name_out);

// Phase 1 & 2: New Rust replacements for spellfile.c utility functions
extern int rs_set_spell_charflags(const uint8_t *flags_in, int cnt, const char *fol);
extern int *rs_mb_str2wide(const char *s);
extern void rs_tree_count_words(const uint8_t *byts, int *idxs, int len);
extern void rs_set_sal_first(slang_T *slang);
extern void rs_set_map_str(slang_T *slang, const char *map);
// spell_check_msm is now in Rust via #[export_name = "spell_check_msm"]; no extern needed.
// Phase 4: set_sofo replacement
extern int rs_set_sofo(slang_T *slang, const char *from, const char *to);
// Phase 4: read_compound replacement
extern int rs_read_compound(const uint8_t *buf, size_t len, slang_T *slang);
// Phase 4: read_sal_section replacement
extern int rs_read_sal_section(const uint8_t *buf, size_t len, slang_T *slang);
// Phase 4: read_prefcond_section replacement
extern int rs_read_prefcond_section(FILE *fd, slang_T *lp);
// Phase 4: read_rep_section replacement
extern int rs_read_rep_section(FILE *fd, garray_T *gap, int16_t *first);

// Phase 2: Spell section writing (write_vim_spell helper)
// SpellSectionParams is filled from spellinfo_T and passed to Rust.
// Rust writes all sections (except SN_CHARFLAGS and SN_WORDS) to a buffer.
typedef struct {
  const char *si_info;
  int si_region_count;
  const uint8_t *si_region_name;
  bool si_skip_charflags;
  const char *si_midword;
  const uint8_t **si_prefcond_strs;
  int si_prefcond_count;
  const uint8_t **si_rep_from;
  const uint8_t **si_rep_to;
  int si_rep_count;
  bool si_use_sal;
  const uint8_t **si_sal_from;
  const uint8_t **si_sal_to;
  int si_sal_count;
  uint8_t si_sal_flags;
  const uint8_t **si_repsal_from;
  const uint8_t **si_repsal_to;
  int si_repsal_count;
  const char *si_sofofr;
  const char *si_sofoto;
  const uint8_t *si_map_data;
  int si_map_len;
  int64_t si_sugtime;
  bool si_nosplitsugs;
  bool si_nocompoundsugs;
  const char *si_compflags;
  uint8_t si_compmax;
  uint8_t si_compminlen;
  uint8_t si_compsylmax;
  uint16_t si_compoptions;
  const uint8_t **si_comppat_strs;
  int si_comppat_count;
  bool si_nobreak;
  const char *si_syllable;
} SpellSectionParams;

extern int rs_write_spell_sections(const SpellSectionParams *params,
                                   uint8_t *buf, size_t buf_len,
                                   size_t *written_out);

// Phase 3: Rust tree serialization - rs_put_node writes tree to buffer and
// returns the nodecount; rs_clear_node resets index/wnode fields.
// (Accessor function definitions appear after the full wordnode_S struct below.)
extern int rs_put_node(wordnode_T *node, uint8_t *buf, size_t buf_len,
                       int idx, int regionmask, bool prefixtree, size_t *written_out);
extern void rs_clear_node(wordnode_T *node);

// Phase 6: Rust tree compression.
// rs_node_compress compresses a sibling list (first sibling of root->wn_sibling).
// Returns compressed node count; sets *tot_out to total nodes visited.
// spellinfo_T is forward-declared as an opaque pointer from Rust's perspective.
extern int rs_node_compress(spellinfo_T *spin, wordnode_T *node, int *tot_out);

// Phase 5: Tree-building functions migrated to Rust.
extern int rs_tree_add_word(spellinfo_T *spin, const char *word, wordnode_T *root, int flags,
                            int region, int affixID);
extern int rs_store_word(spellinfo_T *spin, const char *word, int flags, int region,
                         const char *pfxlist, bool need_affix);
#define store_word rs_store_word
#define tree_add_word rs_tree_add_word

// is_aff_rule, spell_info_item, str_equal, rep_compare are implemented in Rust
// (spellfile.rs) via #[export_name].  Declare for C call sites:
extern bool is_aff_rule(char **items, int itemcnt, const char *rulename, int mincount);
extern bool spell_info_item(const char *s);
extern bool str_equal(const char *s1, const char *s2);
extern int rep_compare(const void *s1, const void *s2);

// Special byte values for <byte>.  Some are only used in the tree for
// postponed prefixes, some only in the other trees.  This is a bit messy...
enum {
  BY_NOFLAGS = 0,  // end of word without flags or region; for postponed prefix: no <pflags>
  BY_INDEX = 1,    // child is shared, index follows
  BY_FLAGS = 2,    // end of word, <flags> byte follows; for postponed prefix: <pflags> follows
  BY_FLAGS2 = 3,   // end of word, <flags> and <flags2> bytes follow; never used in prefix tree
  BY_SPECIAL = BY_FLAGS2,  // highest special byte value
};

#define ZERO_FLAG   65009       // used when flag is zero: "0"

// Flags used in .spl file for soundsalike flags.
enum {
  SAL_F0LLOWUP = 1,
  SAL_COLLAPSE = 2,
  SAL_REM_ACCENTS = 4,
};

#define VIMSPELLMAGIC "VIMspell"  // string at start of Vim spell file
#define VIMSPELLMAGICL (sizeof(VIMSPELLMAGIC) - 1)
#define VIMSPELLVERSION 50

// Section IDs.  Only renumber them when VIMSPELLVERSION changes!
enum {
  SN_REGION = 0,           // <regionname> section
  SN_CHARFLAGS = 1,        // charflags section
  SN_MIDWORD = 2,          // <midword> section
  SN_PREFCOND = 3,         // <prefcond> section
  SN_REP = 4,              // REP items section
  SN_SAL = 5,              // SAL items section
  SN_SOFO = 6,             // soundfolding section
  SN_MAP = 7,              // MAP items section
  SN_COMPOUND = 8,         // compound words section
  SN_SYLLABLE = 9,         // syllable section
  SN_NOBREAK = 10,         // NOBREAK section
  SN_SUGFILE = 11,         // timestamp for .sug file
  SN_REPSAL = 12,          // REPSAL items section
  SN_WORDS = 13,           // common words
  SN_NOSPLITSUGS = 14,     // don't split word for suggestions
  SN_INFO = 15,            // info section
  SN_NOCOMPOUNDSUGS = 16,  // don't compound for suggestions
  SN_END = 255,            // end of sections
};

#define SNF_REQUIRED    1       // <sectionflags>: required section

enum {
  CF_WORD = 0x01,
  CF_UPPER = 0x02,
};

static const char *e_spell_trunc = N_("E758: Truncated spell file");
static const char e_error_while_reading_sug_file_str[]
  = N_("E782: Error while reading .sug file: %s");
static const char *e_illegal_character_in_word = N_("E1280: Illegal character in word");
static const char *e_afftrailing = N_("Trailing text in %s line %d: %s");
static const char *e_affname = N_("Affix name too long in %s line %d: %s");
static const char *msg_compressing = N_("Compressing word tree...");

#define MAXLINELEN  500         // Maximum length in bytes of a line in a .aff
                                // and .dic file.
// Main structure to store the contents of a ".aff" file.
typedef struct {
  char *af_enc;                 // "SET", normalized, alloc'ed string or NULL
  int af_flagtype;              // AFT_CHAR, AFT_LONG, AFT_NUM or AFT_CAPLONG
  unsigned af_rare;             // RARE ID for rare word
  unsigned af_keepcase;         // KEEPCASE ID for keep-case word
  unsigned af_bad;              // BAD ID for banned word
  unsigned af_needaffix;        // NEEDAFFIX ID
  unsigned af_circumfix;        // CIRCUMFIX ID
  unsigned af_needcomp;         // NEEDCOMPOUND ID
  unsigned af_comproot;         // COMPOUNDROOT ID
  unsigned af_compforbid;       // COMPOUNDFORBIDFLAG ID
  unsigned af_comppermit;       // COMPOUNDPERMITFLAG ID
  unsigned af_nosuggest;        // NOSUGGEST ID
  int af_pfxpostpone;           // postpone prefixes without chop string and
                                // without flags
  bool af_ignoreextra;          // IGNOREEXTRA present
  hashtab_T af_pref;            // hashtable for prefixes, affheader_T
  hashtab_T af_suff;            // hashtable for suffixes, affheader_T
  hashtab_T af_comp;            // hashtable for compound flags, compitem_T
} afffile_T;

#define AFT_CHAR        0       // flags are one character
#define AFT_LONG        1       // flags are two characters
#define AFT_CAPLONG     2       // flags are one or two characters
#define AFT_NUM         3       // flags are numbers, comma separated

typedef struct affentry_S affentry_T;
// Affix entry from ".aff" file.  Used for prefixes and suffixes.
struct affentry_S {
  affentry_T *ae_next;          // next affix with same name/number
  char *ae_chop;                // text to chop off basic word (can be NULL)
  char *ae_add;                 // text to add to basic word (can be NULL)
  char *ae_flags;               // flags on the affix (can be NULL)
  char *ae_cond;                // condition (NULL for ".")
  regprog_T *ae_prog;           // regexp program for ae_cond or NULL
  char ae_compforbid;           // COMPOUNDFORBIDFLAG found
  char ae_comppermit;           // COMPOUNDPERMITFLAG found
};

#define AH_KEY_LEN 17          // 2 x 8 bytes + NUL

// Affix header from ".aff" file.  Used for af_pref and af_suff.
typedef struct {
  char ah_key[AH_KEY_LEN];      // key for hashtab == name of affix
  unsigned ah_flag;             // affix name as number, uses "af_flagtype"
  int ah_newID;                 // prefix ID after renumbering; 0 if not used
  int ah_combine;               // suffix may combine with prefix
  int ah_follows;               // another affix block should be following
  affentry_T *ah_first;         // first affix entry
} affheader_T;

#define HI2AH(hi)   ((affheader_T *)(hi)->hi_key)

// Flag used in compound items.
typedef struct {
  char ci_key[AH_KEY_LEN];      // key for hashtab == name of compound
  unsigned ci_flag;             // affix name as number, uses "af_flagtype"
  int ci_newID;                 // affix ID after renumbering.
} compitem_T;

#define HI2CI(hi)   ((compitem_T *)(hi)->hi_key)

// Structure that is used to store the items in the word tree.  This avoids
// the need to keep track of each allocated thing, everything is freed all at
// once after ":mkspell" is done.
// Note: "sb_next" must be just before "sb_data" to make sure the alignment of
// "sb_data" is correct for systems where pointers must be aligned on
// pointer-size boundaries and sizeof(pointer) > sizeof(int) (e.g., Sparc).
#define  SBLOCKSIZE 16000       // size of sb_data
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

#define WN_MASK  0xffff         // mask relevant bits of "wn_flags"

#define HI2WN(hi)    (wordnode_T *)((hi)->hi_key)


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

// Rust-implemented functions (use extern linkage, not static).
extern void spell_message(const spellinfo_T *spin, char *str);
extern wordnode_T *wordtree_alloc(spellinfo_T *spin);
extern bool valid_spell_word(const char *word, const char *end);
extern wordnode_T *get_wordnode(spellinfo_T *spin);
extern void wordtree_compress(spellinfo_T *spin, wordnode_T *root, const char *name);
#include "spellfile.c.generated.h"

/// Load one spell file and store the info into a slang_T.
///
/// This is invoked in three ways:
/// - From spell_load_cb() to load a spell file for the first time.  "lang" is
///   the language name, "old_lp" is NULL.  Will allocate an slang_T.
/// - To reload a spell file that was changed.  "lang" is NULL and "old_lp"
///   points to the existing slang_T.
/// - Just after writing a .spl file; it's read back to produce the .sug file.
///   "old_lp" is NULL and "lang" is NULL.  Will allocate an slang_T.
///
/// @param silent  no error if file doesn't exist
///
/// @return  the slang_T the spell file was loaded into.  NULL for error.
slang_T *spell_load_file(char *fname, char *lang, slang_T *old_lp, bool silent)
{
  char *p;
  slang_T *lp = NULL;
  int res;
  bool did_estack_push = false;

  FILE *fd = os_fopen(fname, "r");
  if (fd == NULL) {
    if (!silent) {
      semsg(_(e_notopen), fname);
    } else if (p_verbose > 2) {
      verbose_enter();
      smsg(0, e_notopen, fname);
      verbose_leave();
    }
    goto endFAIL;
  }
  if (p_verbose > 2) {
    verbose_enter();
    smsg(0, _("Reading spell file \"%s\""), fname);
    verbose_leave();
  }

  if (old_lp == NULL) {
    lp = slang_alloc(lang);

    // Remember the file name, used to reload the file when it's updated.
    lp->sl_fname = xstrdup(fname);

    // Check for .add.spl.
    lp->sl_add = strstr(path_tail(fname), SPL_FNAME_ADD) != NULL;
  } else {
    lp = old_lp;
  }

  // Set sourcing_name, so that error messages mention the file name.
  estack_push(ETYPE_SPELL, fname, 0);
  did_estack_push = true;

  // <HEADER>: <fileID> -- validate magic string
  {
    char magic_buf[VIMSPELLMAGICL];
    if (fread(magic_buf, 1, VIMSPELLMAGICL, fd) != VIMSPELLMAGICL) {
      if (feof(fd)) {
        semsg("%s", _("E757: This does not look like a spell file"));
      } else {
        semsg(_("E5042: Failed to read spell file %s: %s"),
              fname, strerror(ferror(fd)));
      }
      goto endFAIL;
    }
    if (memcmp(magic_buf, VIMSPELLMAGIC, VIMSPELLMAGICL) != 0) {
      semsg("%s", _("E757: This does not look like a spell file"));
      goto endFAIL;
    }
  }
  int c = getc(fd);                                         // <versionnr>
  if (c < VIMSPELLVERSION) {
    emsg(_("E771: Old spell file, needs to be updated"));
    goto endFAIL;
  } else if (c > VIMSPELLVERSION) {
    emsg(_("E772: Spell file is for newer version of Vim"));
    goto endFAIL;
  }

  // <SECTIONS>: <section> ... <sectionend>
  // <section>: <sectionID> <sectionflags> <sectionlen> (section contents)
  while (true) {
    int n = getc(fd);                           // <sectionID> or <sectionend>
    if (n == SN_END) {
      break;
    }
    c = getc(fd);                                       // <sectionflags>
    int len = get4c(fd);                                    // <sectionlen>
    if (len < 0) {
      goto truncerr;
    }

    res = 0;
    switch (n) {
    case SN_INFO:
      XFREE_CLEAR(lp->sl_info);
      lp->sl_info = read_string(fd, (size_t)len);  // <infotext>
      if (lp->sl_info == NULL) {
        goto endFAIL;
      }
      break;

    case SN_REGION:
      // Inline of deleted read_region_section():
      if (len > MAXREGIONS * 2) {
        res = SP_FORMERROR;
      } else {
        if (fread(lp->sl_regions, 1, (size_t)len, fd) != (size_t)len) {
          res = feof(fd) ? SP_TRUNCERROR : SP_OTHERERROR;
        } else if (memchr(lp->sl_regions, NUL, (size_t)len)) {
          res = SP_FORMERROR;
        } else {
          lp->sl_regions[len] = NUL;
        }
      }
      break;

    case SN_CHARFLAGS: {
      // Read charflags section inline (replaces read_charflags_section +
      // set_spell_charflags + set_spell_finish, all now in Rust).
      // Read 1-byte length prefix for flags.
      int flagslen;
      {
        int fc = getc(fd);
        if (fc == EOF) { res = SP_TRUNCERROR; break; }
        flagslen = fc;
      }
      char *flags = flagslen == 0 ? NULL : read_string(fd, (size_t)flagslen);
      if (flagslen > 0 && flags == NULL) { res = SP_OTHERERROR; break; }

      // Read 2-byte length prefix for fol.
      int follen;
      {
        int fc1 = getc(fd), fc2 = getc(fd);
        if (fc1 == EOF || fc2 == EOF) { xfree(flags); res = SP_TRUNCERROR; break; }
        follen = (fc1 << 8) | fc2;
      }
      char *fol = follen == 0 ? NULL : read_string(fd, (size_t)follen);
      if (follen > 0 && fol == NULL) { xfree(flags); res = SP_OTHERERROR; break; }

      if ((flags == NULL) != (fol == NULL)) {
        xfree(flags);
        xfree(fol);
        res = SP_FORMERROR;
        break;
      }
      if (flags != NULL && fol != NULL) {
        if (rs_set_spell_charflags((uint8_t *)flags, flagslen, fol) != 0) {
          res = SP_OTHERERROR;
        }
      }
      xfree(flags);
      xfree(fol);
      break;
    }

    case SN_MIDWORD:
      lp->sl_midword = read_string(fd, (size_t)len);  // <midword>
      if (lp->sl_midword == NULL) {
        goto endFAIL;
      }
      break;

    case SN_PREFCOND:
      res = rs_read_prefcond_section(fd, lp);
      break;

    case SN_REP:
      res = rs_read_rep_section(fd, &lp->sl_rep, lp->sl_rep_first);
      break;

    case SN_REPSAL:
      res = rs_read_rep_section(fd, &lp->sl_repsal, lp->sl_repsal_first);
      break;

    case SN_SAL: {
      uint8_t *sal_buf = xmalloc((size_t)len);
      if (fread(sal_buf, 1, (size_t)len, fd) != (size_t)len) {
        xfree(sal_buf);
        res = feof(fd) ? SP_TRUNCERROR : SP_OTHERERROR;
        break;
      }
      res = rs_read_sal_section(sal_buf, (size_t)len, lp);
      xfree(sal_buf);
      break;
    }

    case SN_SOFO: {
      lp->sl_sofo = true;
      if (len <= 0) {
        res = SP_FORMERROR;
        break;
      }
      uint8_t *sofo_buf = xmalloc((size_t)len);
      if (fread(sofo_buf, 1, (size_t)len, fd) != (size_t)len) {
        xfree(sofo_buf);
        res = feof(fd) ? SP_TRUNCERROR : SP_OTHERERROR;
        break;
      }
      RsSofoSection sofo_section;
      size_t sofo_consumed;
      res = rs_parse_sofo_section(sofo_buf, (size_t)len, &sofo_section, &sofo_consumed);
      xfree(sofo_buf);
      if (res == 0) {
        if (sofo_section.from_len == 0 && sofo_section.to_len == 0) {
          // empty, OK
        } else if (sofo_section.from_len == 0 || sofo_section.to_len == 0) {
          res = SP_FORMERROR;
        } else {
          char sofo_from[513], sofo_to[513];
          memcpy(sofo_from, sofo_section.from, sofo_section.from_len);
          sofo_from[sofo_section.from_len] = '\0';
          memcpy(sofo_to, sofo_section.to, sofo_section.to_len);
          sofo_to[sofo_section.to_len] = '\0';
          res = rs_set_sofo(lp, sofo_from, sofo_to);
        }
      }
      break;
    }

    case SN_MAP:
      p = read_string(fd, (size_t)len);  // <mapstr>
      if (p == NULL) {
        goto endFAIL;
      }
      rs_set_map_str(lp, p);
      xfree(p);
      break;

    case SN_WORDS: {
      // Inline of deleted read_words_section():
      uint8_t *wbuf = xmalloc((size_t)len);
      if (fread(wbuf, 1, (size_t)len, fd) != (size_t)len) {
        xfree(wbuf);
        res = feof(fd) ? SP_TRUNCERROR : SP_OTHERERROR;
        goto someerror;
      }
      size_t woff = 0;
      uint8_t wword[MAXWLEN];
      while (woff < (size_t)len) {
        size_t wconsumed;
        int wlen = rs_parse_words_entry(wbuf + woff, (size_t)len - woff,
                                        wword, MAXWLEN, &wconsumed);
        if (wlen < 0) {
          xfree(wbuf);
          res = wlen;
          goto someerror;
        }
        count_common_word(lp, (char *)wword, -1, 10);
        woff += wconsumed;
      }
      xfree(wbuf);
      break;
    }

    case SN_SUGFILE:
      lp->sl_sugtime = get8ctime(fd);                   // <timestamp>
      break;

    case SN_NOSPLITSUGS:
      lp->sl_nosplitsugs = true;
      break;

    case SN_NOCOMPOUNDSUGS:
      lp->sl_nocompoundsugs = true;
      break;

    case SN_COMPOUND: {
      uint8_t *cmp_buf = xmalloc((size_t)len);
      if (fread(cmp_buf, 1, (size_t)len, fd) != (size_t)len) {
        xfree(cmp_buf);
        res = feof(fd) ? SP_TRUNCERROR : SP_OTHERERROR;
        break;
      }
      res = rs_read_compound(cmp_buf, (size_t)len, lp);
      xfree(cmp_buf);
      break;
    }

    case SN_NOBREAK:
      lp->sl_nobreak = true;
      break;

    case SN_SYLLABLE:
      lp->sl_syllable = read_string(fd, (size_t)len);  // <syllable>
      if (lp->sl_syllable == NULL) {
        goto endFAIL;
      }
      if (init_syl_tab(lp) != OK) {
        goto endFAIL;
      }
      break;

    default:
      // Unsupported section.  When it's required give an error
      // message.  When it's not required skip the contents.
      if (c & SNF_REQUIRED) {
        emsg(_("E770: Unsupported section in spell file"));
        goto endFAIL;
      }
      while (--len >= 0) {
        if (getc(fd) < 0) {
          goto truncerr;
        }
      }
      break;
    }
someerror:
    if (res == SP_FORMERROR) {
      emsg(_(e_format));
      goto endFAIL;
    }
    if (res == SP_TRUNCERROR) {
truncerr:
      emsg(_(e_spell_trunc));
      goto endFAIL;
    }
    if (res == SP_OTHERERROR) {
      goto endFAIL;
    }
  }

  // Read all remaining tree data (<LWORDTREE> <KWORDTREE> <PREFIXTREE>) into
  // a single buffer and process with Rust (rs_read_tree).
  {
    // Read from current position to EOF.
    long pos_before = ftell(fd);
    fseek(fd, 0, SEEK_END);
    long pos_end = ftell(fd);
    fseek(fd, pos_before, SEEK_SET);

    size_t tree_data_size = (pos_end > pos_before) ? (size_t)(pos_end - pos_before) : 0;
    if (tree_data_size == 0) {
      goto truncerr;
    }

    uint8_t *tree_data = xmalloc(tree_data_size);
    if (fread(tree_data, 1, tree_data_size, fd) != tree_data_size) {
      xfree(tree_data);
      goto truncerr;
    }

    size_t toff = 0;  // offset into tree_data

    // Helper macro: read one tree from the buffer.
    // Reads nodecount, allocates arrays, calls rs_read_tree.
    // bytsp_len_p may be NULL (for trees that don't need the length stored).
    #define READ_TREE_FROM_BUF(bytsp, bytsp_len_p, idxsp, is_prefix, prefcnt) \
      do { \
        size_t _remaining = tree_data_size - toff; \
        if (_remaining < 4) { xfree(tree_data); goto truncerr; } \
        uint32_t _nodecount = 0; \
        if (rs_read_tree_peek_nodecount(tree_data + toff, _remaining, &_nodecount) != 0) { \
          xfree(tree_data); goto truncerr; \
        } \
        if (_nodecount == 0) { \
          (bytsp) = NULL; \
          if ((bytsp_len_p) != NULL) { *(int *)(bytsp_len_p) = 0; } \
          (idxsp) = NULL; \
          toff += 4; \
        } else { \
          (bytsp) = xmalloc((size_t)_nodecount); \
          if ((bytsp_len_p) != NULL) { *(int *)(bytsp_len_p) = (int)_nodecount; } \
          (idxsp) = xcalloc((size_t)_nodecount, sizeof(idx_T)); \
          size_t _consumed = 0; \
          int _ncnt = 0; \
          res = rs_read_tree(tree_data + toff, _remaining, (bytsp), \
                             (int32_t *)(idxsp), (size_t)_nodecount, \
                             (is_prefix), (prefcnt), &_consumed, &_ncnt); \
          if (res != 0) { xfree(tree_data); goto someerror; } \
          toff += _consumed; \
        } \
      } while (0)

    // <LWORDTREE>
    READ_TREE_FROM_BUF(lp->sl_fbyts, &lp->sl_fbyts_len, lp->sl_fidxs, false, 0);
    // <KWORDTREE>
    READ_TREE_FROM_BUF(lp->sl_kbyts, NULL, lp->sl_kidxs, false, 0);
    // <PREFIXTREE>
    READ_TREE_FROM_BUF(lp->sl_pbyts, NULL, lp->sl_pidxs, true, lp->sl_prefixcnt);

    #undef READ_TREE_FROM_BUF

    xfree(tree_data);
  }

  // For a new file link it in the list of spell files.
  if (old_lp == NULL && lang != NULL) {
    lp->sl_next = first_lang;
    first_lang = lp;
  }

  goto endOK;

endFAIL:
  if (lang != NULL) {
    // truncating the name signals the error to spell_load_lang()
    *lang = NUL;
  }
  if (lp != NULL && old_lp == NULL) {
    slang_free(lp);
  }
  lp = NULL;

endOK:
  if (fd != NULL) {
    fclose(fd);
  }
  if (did_estack_push) {
    estack_pop();
  }

  return lp;
}


/// Load the .sug files for languages that have one and weren't loaded yet.
void suggest_load_files(void)
{
  char buf[MAXWLEN];
  garray_T ga;

  // Do this for all languages that support sound folding.
  for (int lpi = 0; lpi < curwin->w_s->b_langp.ga_len; lpi++) {
    langp_T *lp = LANGP_ENTRY(curwin->w_s->b_langp, lpi);
    slang_T *slang = lp->lp_slang;
    if (slang->sl_sugtime != 0 && !slang->sl_sugloaded) {
      // Change ".spl" to ".sug" and open the file.  When the file isn't
      // found silently skip it.  Do set "sl_sugloaded" so that we
      // don't try again and again.
      slang->sl_sugloaded = true;

      char *dotp = strrchr(slang->sl_fname, '.');
      if (dotp == NULL || path_fnamecmp(dotp, ".spl") != 0) {
        continue;
      }
      STRCPY(dotp, ".sug");
      FILE *fd = os_fopen(slang->sl_fname, "r");
      if (fd == NULL) {
        goto nextone;
      }

      // <SUGHEADER>: <fileID> <versionnr> <timestamp>
      for (int i = 0; i < VIMSUGMAGICL; i++) {
        buf[i] = (char)getc(fd);                              // <fileID>
      }
      if (strncmp(buf, VIMSUGMAGIC, VIMSUGMAGICL) != 0) {
        semsg(_("E778: This does not look like a .sug file: %s"),
              slang->sl_fname);
        goto nextone;
      }
      int c = getc(fd);                                     // <versionnr>
      if (c < VIMSUGVERSION) {
        semsg(_("E779: Old .sug file, needs to be updated: %s"),
              slang->sl_fname);
        goto nextone;
      } else if (c > VIMSUGVERSION) {
        semsg(_("E780: .sug file is for newer version of Vim: %s"),
              slang->sl_fname);
        goto nextone;
      }

      // Check the timestamp, it must be exactly the same as the one in
      // the .spl file.  Otherwise the word numbers won't match.
      time_t timestamp = get8ctime(fd);                        // <timestamp>
      if (timestamp != slang->sl_sugtime) {
        semsg(_("E781: .sug file doesn't match .spl file: %s"),
              slang->sl_fname);
        goto nextone;
      }

      // <SUGWORDTREE>: <wordtree>
      // Read the trie with the soundfolded words (inlined spell_read_tree).
      {
        uint8_t rt_hdr[4];
        int rt_res = 0;
        if (fread(rt_hdr, 1, 4, fd) != 4) {
          rt_res = feof(fd) ? SP_TRUNCERROR : SP_OTHERERROR;
        } else {
          int rt_len = ((int)rt_hdr[0] << 24) | ((int)rt_hdr[1] << 16)
                       | ((int)rt_hdr[2] << 8) | rt_hdr[3];
          if (rt_len < 0) {
            rt_res = SP_TRUNCERROR;
          } else if ((size_t)rt_len >= SIZE_MAX / sizeof(int)) {
            rt_res = SP_FORMERROR;
          } else if (rt_len > 0) {
            uint8_t *rt_bp = xmalloc((size_t)rt_len);
            slang->sl_sbyts = rt_bp;
            idx_T *rt_ip = xcalloc((size_t)rt_len, sizeof(*rt_ip));
            slang->sl_sidxs = rt_ip;
            size_t rt_buf_max = 4 + (size_t)rt_len * 6 + 64;
            uint8_t *rt_buf = xmalloc(rt_buf_max);
            memcpy(rt_buf, rt_hdr, 4);
            size_t rt_data = fread(rt_buf + 4, 1, rt_buf_max - 4, fd);
            if (rt_data == 0) {
              xfree(rt_buf);
              rt_res = SP_TRUNCERROR;
            } else {
              size_t rt_consumed = 0;
              int rt_nc = 0;
              rt_res = rs_read_tree(rt_buf, 4 + rt_data, rt_bp, (int32_t *)rt_ip,
                                    (size_t)rt_len, false, 0, &rt_consumed, &rt_nc);
              long rt_over = (long)(4 + rt_data) - (long)rt_consumed;
              if (rt_over > 0) {
                fseek(fd, -rt_over, SEEK_CUR);
              }
              xfree(rt_buf);
            }
          }
        }
        if (rt_res != 0) {
          goto someerror;
        }
      }
      if (0) {
someerror:
        semsg(_(e_error_while_reading_sug_file_str),
              slang->sl_fname);
        slang_clear_sug(slang);
        goto nextone;
      }

      // <SUGTABLE>: <sugwcount> <sugline> ...
      //
      // Read the table with word numbers.  We use a file buffer for
      // this, because it's so much like a file with lines.  Makes it
      // possible to swap the info and save on memory use.
      slang->sl_sugbuf = open_spellbuf();

      // <sugwcount>
      int wcount = get4c(fd);
      if (wcount < 0) {
        goto someerror;
      }

      // Read all the wordnr lists into the buffer, one NUL terminated
      // list per line.
      ga_init(&ga, 1, 100);
      for (int wordnr = 0; wordnr < wcount; wordnr++) {
        ga.ga_len = 0;
        while (true) {
          c = getc(fd);                                     // <sugline>
          if (c < 0) {
            goto someerror;
          }
          GA_APPEND(uint8_t, &ga, (uint8_t)c);
          if (c == NUL) {
            break;
          }
        }
        if (ml_append_buf(slang->sl_sugbuf, (linenr_T)wordnr,
                          ga.ga_data, ga.ga_len, true) == FAIL) {
          goto someerror;
        }
      }
      ga_clear(&ga);

      // Need to put word counts in the word tries, so that we can find
      // a word by its number.
      rs_tree_count_words(slang->sl_fbyts, slang->sl_fidxs, slang->sl_fbyts_len);
      // Soundfold tree has no stored length; pass INT_MAX (tree structure
      // provides its own termination via sibling counts).
      rs_tree_count_words(slang->sl_sbyts, slang->sl_sidxs, INT_MAX);

nextone:
      if (fd != NULL) {
        fclose(fd);
      }
      STRCPY(dotp, ".spl");
    }
  }
}

/// Reload the spell file "fname" if it's loaded.
///
/// @param added_word  invoked through "zg"
static void spell_reload_one(char *fname, bool added_word)
{
  bool didit = false;

  for (slang_T *slang = first_lang; slang != NULL; slang = slang->sl_next) {
    if (path_full_compare(fname, slang->sl_fname, false, true) == kEqualFiles) {
      slang_clear(slang);
      if (spell_load_file(fname, NULL, slang, false) == NULL) {
        // reloading failed, clear the language
        slang_clear(slang);
      }
      redraw_all_later(UPD_SOME_VALID);
      didit = true;
    }
  }

  // When "zg" was used and the file wasn't loaded yet, should redo
  // 'spelllang' to load it now.
  if (added_word && !didit) {
    parse_spelllang(curwin);
  }
}

// Functions for ":mkspell".

// In the postponed prefixes tree wn_flags is used to store the WFP_ flags,
// but it must be negative to indicate the prefix tree to tree_add_word().
// Use a negative number with the lower 8 bits zero.
#define PFX_FLAGS       (-256)

// flags for "condit" argument of store_aff_word()
#define CONDIT_COMB     1       // affix must combine
#define CONDIT_CFIX     2       // affix must have CIRCUMFIX flag
#define CONDIT_SUF      4       // add a suffix for matching flags
#define CONDIT_AFF      8       // word already has an affix

// Compression tuning globals (compress_start/inc/added) have moved to Rust
// (COMPRESS_START/INC/ADDED in spellfile.rs). spell_check_msm() sets them.

// spell_check_msm() is implemented in Rust (rs_do_spell_check_msm) via
// #[export_name = "spell_check_msm"].  The declaration appears in the
// generated header via the forward declarations below.

// Reads the affix file "fname".
// Returns an afffile_T, NULL for complete failure.
static afffile_T *spell_read_aff(spellinfo_T *spin, char *fname)
{
  char rline[MAXLINELEN];
  char *line;
  char *pc = NULL;
#define MAXITEMCNT  30
  char *(items[MAXITEMCNT]);
  char *p;
  int lnum = 0;
  affheader_T *cur_aff = NULL;
  bool did_postpone_prefix = false;
  int aff_todo = 0;
  hashtab_T *tp;
  char *low = NULL;
  char *fol = NULL;
  char *upp = NULL;
  bool found_map = false;
  hashitem_T *hi;
  int compminlen = 0;              // COMPOUNDMIN value
  int compsylmax = 0;              // COMPOUNDSYLMAX value
  int compoptions = 0;             // COMP_ flags
  int compmax = 0;                 // COMPOUNDWORDMAX value
  char *compflags = NULL;          // COMPOUNDFLAG and COMPOUNDRULE
                                   // concatenated
  char *midword = NULL;            // MIDWORD value
  char *syllable = NULL;           // SYLLABLE value
  char *sofofrom = NULL;           // SOFOFROM value
  char *sofoto = NULL;             // SOFOTO value

  // Open the file.
  FILE *fd = os_fopen(fname, "r");
  if (fd == NULL) {
    semsg(_(e_notopen), fname);
    return NULL;
  }

  vim_snprintf(IObuff, IOSIZE, _("Reading affix file %s..."), fname);
  spell_message(spin, IObuff);

  // Only do REP lines when not done in another .aff file already.
  bool do_rep = GA_EMPTY(&spin->si_rep);

  // Only do REPSAL lines when not done in another .aff file already.
  bool do_repsal = GA_EMPTY(&spin->si_repsal);

  // Only do SAL lines when not done in another .aff file already.
  bool do_sal = GA_EMPTY(&spin->si_sal);

  // Only do MAP lines when not done in another .aff file already.
  bool do_mapline = GA_EMPTY(&spin->si_map);

  // Allocate and init the afffile_T structure.
  afffile_T *aff = getroom(spin, sizeof(*aff), true);
  hash_init(&aff->af_pref);
  hash_init(&aff->af_suff);
  hash_init(&aff->af_comp);

  // Read all the lines in the file one by one.
  while (!vim_fgets(rline, MAXLINELEN, fd) && !got_int) {
    line_breakcheck();
    lnum++;

    // Skip comment lines.
    if (*rline == '#') {
      continue;
    }

    // Convert from "SET" to 'encoding' when needed.
    xfree(pc);
    if (spin->si_conv.vc_type != CONV_NONE) {
      pc = string_convert(&spin->si_conv, rline, NULL);
      if (pc == NULL) {
        smsg(0, _("Conversion failure for word in %s line %d: %s"),
             fname, lnum, rline);
        continue;
      }
      line = pc;
    } else {
      pc = NULL;
      line = rline;
    }

    // Split the line up in white separated items.  Put a NUL after each
    // item.
    int itemcnt = 0;
    for (p = line;;) {
      while (*p != NUL && (uint8_t)(*p) <= ' ') {  // skip white space and CR/NL
        p++;
      }
      if (*p == NUL) {
        break;
      }
      if (itemcnt == MAXITEMCNT) {          // too many items
        break;
      }
      items[itemcnt++] = p;
      // A few items have arbitrary text argument, don't split them.
      if (itemcnt == 2 && spell_info_item(items[0])) {
        while ((uint8_t)(*p) >= ' ' || *p == TAB) {  // skip until CR/NL
          p++;
        }
      } else {
        while ((uint8_t)(*p) > ' ') {  // skip until white space or CR/NL
          p++;
        }
      }
      if (*p == NUL) {
        break;
      }
      *p++ = NUL;
    }

    // Handle non-empty lines.
    if (itemcnt > 0) {
      if (is_aff_rule(items, itemcnt, "SET", 2) && aff->af_enc == NULL) {
        // Setup for conversion from "ENC" to 'encoding'.
        aff->af_enc = enc_canonize(items[1]);
        if (!spin->si_ascii
            && convert_setup(&spin->si_conv, aff->af_enc, p_enc) == FAIL) {
          smsg(0, _("Conversion in %s not supported: from %s to %s"),
               fname, aff->af_enc, p_enc);
        }
        spin->si_conv.vc_fail = true;
      } else if (is_aff_rule(items, itemcnt, "FLAG", 2)
                 && aff->af_flagtype == AFT_CHAR) {
        if (strcmp(items[1], "long") == 0) {
          aff->af_flagtype = AFT_LONG;
        } else if (strcmp(items[1], "num") == 0) {
          aff->af_flagtype = AFT_NUM;
        } else if (strcmp(items[1], "caplong") == 0) {
          aff->af_flagtype = AFT_CAPLONG;
        } else {
          smsg(0, _("Invalid value for FLAG in %s line %d: %s"),
               fname, lnum, items[1]);
        }
        if (aff->af_rare != 0
            || aff->af_keepcase != 0
            || aff->af_bad != 0
            || aff->af_needaffix != 0
            || aff->af_circumfix != 0
            || aff->af_needcomp != 0
            || aff->af_comproot != 0
            || aff->af_nosuggest != 0
            || compflags != NULL
            || aff->af_suff.ht_used > 0
            || aff->af_pref.ht_used > 0) {
          smsg(0, _("FLAG after using flags in %s line %d: %s"),
               fname, lnum, items[1]);
        }
      } else if (spell_info_item(items[0]) && itemcnt > 1) {
        p = getroom(spin,
                    (spin->si_info == NULL ? 0 : strlen(spin->si_info))
                    + strlen(items[0])
                    + strlen(items[1]) + 3, false);
        if (spin->si_info != NULL) {
          STRCPY(p, spin->si_info);
          strcat(p, "\n");
        }
        strcat(p, items[0]);
        strcat(p, " ");
        strcat(p, items[1]);
        spin->si_info = p;
      } else if (is_aff_rule(items, itemcnt, "MIDWORD", 2) && midword == NULL) {
        midword = getroom_save(spin, items[1]);
      } else if (is_aff_rule(items, itemcnt, "TRY", 2)) {
        // ignored, we look in the tree for what chars may appear
      } else if ((is_aff_rule(items, itemcnt, "RAR", 2)  // TODO(vim): remove "RAR" later
                  || is_aff_rule(items, itemcnt, "RARE", 2))
                 && aff->af_rare == 0) {
        aff->af_rare = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if ((is_aff_rule(items, itemcnt, "KEP", 2)  // TODO(vim): remove "KEP" later
                  || is_aff_rule(items, itemcnt, "KEEPCASE", 2))
                 && aff->af_keepcase == 0) {
        aff->af_keepcase = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if ((is_aff_rule(items, itemcnt, "BAD", 2)
                  || is_aff_rule(items, itemcnt, "FORBIDDENWORD", 2))
                 && aff->af_bad == 0) {
        aff->af_bad = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if (is_aff_rule(items, itemcnt, "NEEDAFFIX", 2)
                 && aff->af_needaffix == 0) {
        aff->af_needaffix = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if (is_aff_rule(items, itemcnt, "CIRCUMFIX", 2)
                 && aff->af_circumfix == 0) {
        aff->af_circumfix = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if (is_aff_rule(items, itemcnt, "NOSUGGEST", 2)
                 && aff->af_nosuggest == 0) {
        aff->af_nosuggest = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if ((is_aff_rule(items, itemcnt, "NEEDCOMPOUND", 2)
                  || is_aff_rule(items, itemcnt, "ONLYINCOMPOUND", 2))
                 && aff->af_needcomp == 0) {
        aff->af_needcomp = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDROOT", 2)
                 && aff->af_comproot == 0) {
        aff->af_comproot = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDFORBIDFLAG", 2)
                 && aff->af_compforbid == 0) {
        aff->af_compforbid = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
        if (aff->af_pref.ht_used > 0) {
          smsg(0,
               _("Defining COMPOUNDFORBIDFLAG after PFX item may give wrong results in %s line %d"),
               fname, lnum);
        }
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDPERMITFLAG", 2)
                 && aff->af_comppermit == 0) {
        aff->af_comppermit = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
        if (aff->af_pref.ht_used > 0) {
          smsg(0,
               _("Defining COMPOUNDPERMITFLAG after PFX item may give wrong results in %s line %d"),
               fname, lnum);
        }
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDFLAG", 2)
                 && compflags == NULL) {
        // Turn flag "c" into COMPOUNDRULE compatible string "c+",
        // "Na" into "Na+", "1234" into "1234+".
        p = getroom(spin, strlen(items[1]) + 2, false);
        STRCPY(p, items[1]);
        strcat(p, "+");
        compflags = p;
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDRULES", 2)) {
        // We don't use the count, but do check that it's a number and
        // not COMPOUNDRULE mistyped.
        if (atoi(items[1]) == 0) {
          smsg(0, _("Wrong COMPOUNDRULES value in %s line %d: %s"),
               fname, lnum, items[1]);
        }
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDRULE", 2)) {
        // Don't use the first rule if it is a number.
        if (compflags != NULL || *skipdigits(items[1]) != NUL) {
          // Concatenate this string to previously defined ones,
          // using a slash to separate them.
          int l = (int)strlen(items[1]) + 1;
          if (compflags != NULL) {
            l += (int)strlen(compflags) + 1;
          }
          p = getroom(spin, (size_t)l, false);
          if (compflags != NULL) {
            STRCPY(p, compflags);
            strcat(p, "/");
          }
          strcat(p, items[1]);
          compflags = p;
        }
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDWORDMAX", 2)
                 && compmax == 0) {
        compmax = atoi(items[1]);
        if (compmax == 0) {
          smsg(0, _("Wrong COMPOUNDWORDMAX value in %s line %d: %s"),
               fname, lnum, items[1]);
        }
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDMIN", 2)
                 && compminlen == 0) {
        compminlen = atoi(items[1]);
        if (compminlen == 0) {
          smsg(0, _("Wrong COMPOUNDMIN value in %s line %d: %s"),
               fname, lnum, items[1]);
        }
      } else if (is_aff_rule(items, itemcnt, "COMPOUNDSYLMAX", 2)
                 && compsylmax == 0) {
        compsylmax = atoi(items[1]);
        if (compsylmax == 0) {
          smsg(0, _("Wrong COMPOUNDSYLMAX value in %s line %d: %s"),
               fname, lnum, items[1]);
        }
      } else if (is_aff_rule(items, itemcnt, "CHECKCOMPOUNDDUP", 1)) {
        compoptions |= COMP_CHECKDUP;
      } else if (is_aff_rule(items, itemcnt, "CHECKCOMPOUNDREP", 1)) {
        compoptions |= COMP_CHECKREP;
      } else if (is_aff_rule(items, itemcnt, "CHECKCOMPOUNDCASE", 1)) {
        compoptions |= COMP_CHECKCASE;
      } else if (is_aff_rule(items, itemcnt, "CHECKCOMPOUNDTRIPLE", 1)) {
        compoptions |= COMP_CHECKTRIPLE;
      } else if (is_aff_rule(items, itemcnt, "CHECKCOMPOUNDPATTERN", 2)) {
        if (atoi(items[1]) == 0) {
          smsg(0, _("Wrong CHECKCOMPOUNDPATTERN value in %s line %d: %s"),
               fname, lnum, items[1]);
        }
      } else if (is_aff_rule(items, itemcnt, "CHECKCOMPOUNDPATTERN", 3)) {
        garray_T *gap = &spin->si_comppat;
        int i;

        // Only add the couple if it isn't already there.
        for (i = 0; i < gap->ga_len - 1; i += 2) {
          if (strcmp(((char **)(gap->ga_data))[i], items[1]) == 0
              && strcmp(((char **)(gap->ga_data))[i + 1], items[2]) == 0) {
            break;
          }
        }
        if (i >= gap->ga_len) {
          ga_grow(gap, 2);
          ((char **)(gap->ga_data))[gap->ga_len++] = getroom_save(spin, items[1]);
          ((char **)(gap->ga_data))[gap->ga_len++] = getroom_save(spin, items[2]);
        }
      } else if (is_aff_rule(items, itemcnt, "SYLLABLE", 2)
                 && syllable == NULL) {
        syllable = getroom_save(spin, items[1]);
      } else if (is_aff_rule(items, itemcnt, "NOBREAK", 1)) {
        spin->si_nobreak = true;
      } else if (is_aff_rule(items, itemcnt, "NOSPLITSUGS", 1)) {
        spin->si_nosplitsugs = true;
      } else if (is_aff_rule(items, itemcnt, "NOCOMPOUNDSUGS", 1)) {
        spin->si_nocompoundsugs = true;
      } else if (is_aff_rule(items, itemcnt, "NOSUGFILE", 1)) {
        spin->si_nosugfile = true;
      } else if (is_aff_rule(items, itemcnt, "PFXPOSTPONE", 1)) {
        aff->af_pfxpostpone = true;
      } else if (is_aff_rule(items, itemcnt, "IGNOREEXTRA", 1)) {
        aff->af_ignoreextra = true;
      } else if ((strcmp(items[0], "PFX") == 0
                  || strcmp(items[0], "SFX") == 0)
                 && aff_todo == 0
                 && itemcnt >= 4) {
        int lasti = 4;
        char key[AH_KEY_LEN];

        if (*items[0] == 'P') {
          tp = &aff->af_pref;
        } else {
          tp = &aff->af_suff;
        }

        // Myspell allows the same affix name to be used multiple
        // times.  The affix files that do this have an undocumented
        // "S" flag on all but the last block, thus we check for that
        // and store it in ah_follows.
        xstrlcpy(key, items[1], AH_KEY_LEN);
        hi = hash_find(tp, key);
        if (!HASHITEM_EMPTY(hi)) {
          cur_aff = HI2AH(hi);
          if (cur_aff->ah_combine != (*items[2] == 'Y')) {
            smsg(0, _("Different combining flag in continued affix block in %s line %d: %s"),
                 fname, lnum, items[1]);
          }
          if (!cur_aff->ah_follows) {
            smsg(0, _("Duplicate affix in %s line %d: %s"),
                 fname, lnum, items[1]);
          }
        } else {
          // New affix letter.
          cur_aff = getroom(spin, sizeof(*cur_aff), true);
          cur_aff->ah_flag = affitem2flag(aff->af_flagtype, items[1], fname, lnum);
          if (cur_aff->ah_flag == 0 || strlen(items[1]) >= AH_KEY_LEN) {
            break;
          }
          if (cur_aff->ah_flag == aff->af_bad
              || cur_aff->ah_flag == aff->af_rare
              || cur_aff->ah_flag == aff->af_keepcase
              || cur_aff->ah_flag == aff->af_needaffix
              || cur_aff->ah_flag == aff->af_circumfix
              || cur_aff->ah_flag == aff->af_nosuggest
              || cur_aff->ah_flag == aff->af_needcomp
              || cur_aff->ah_flag == aff->af_comproot) {
            smsg(0, _("Affix also used for BAD/RARE/KEEPCASE/NEEDAFFIX/NEEDCOMPOUND/NOSUGGEST "
                      "in %s line %d: %s"),
                 fname, lnum, items[1]);
          }
          STRCPY(cur_aff->ah_key, items[1]);
          hash_add(tp, cur_aff->ah_key);

          cur_aff->ah_combine = (*items[2] == 'Y');
        }

        // Check for the "S" flag, which apparently means that another
        // block with the same affix name is following.
        if (itemcnt > lasti && strcmp(items[lasti], "S") == 0) {
          lasti++;
          cur_aff->ah_follows = true;
        } else {
          cur_aff->ah_follows = false;
        }

        // Myspell allows extra text after the item, but that might
        // mean mistakes go unnoticed.  Require a comment-starter,
        // unless IGNOREEXTRA is used.  Hunspell uses a "-" item.
        if (itemcnt > lasti
            && !aff->af_ignoreextra
            && *items[lasti] != '#') {
          smsg(0, _(e_afftrailing), fname, lnum, items[lasti]);
        }

        if (strcmp(items[2], "Y") != 0 && strcmp(items[2], "N") != 0) {
          smsg(0, _("Expected Y or N in %s line %d: %s"),
               fname, lnum, items[2]);
        }

        if (*items[0] == 'P' && aff->af_pfxpostpone) {
          if (cur_aff->ah_newID == 0) {
            // Use a new number in the .spl file later, to be able
            // to handle multiple .aff files.
            check_renumber(spin);
            cur_aff->ah_newID = ++spin->si_newprefID;

            // We only really use ah_newID if the prefix is
            // postponed.  We know that only after handling all
            // the items.
            did_postpone_prefix = false;
          } else {
            // Did use the ID in a previous block.
            did_postpone_prefix = true;
          }
        }

        aff_todo = atoi(items[3]);
      } else if ((strcmp(items[0], "PFX") == 0
                  || strcmp(items[0], "SFX") == 0)
                 && aff_todo > 0
                 && strcmp(cur_aff->ah_key, items[1]) == 0
                 && itemcnt >= 5) {
        affentry_T *aff_entry;
        int lasti = 5;

        // Myspell allows extra text after the item, but that might
        // mean mistakes go unnoticed.  Require a comment-starter.
        // Hunspell uses a "-" item.
        if (itemcnt > lasti && *items[lasti] != '#'
            && (strcmp(items[lasti], "-") != 0
                || itemcnt != lasti + 1)) {
          smsg(0, _(e_afftrailing), fname, lnum, items[lasti]);
        }

        // New item for an affix letter.
        aff_todo--;
        aff_entry = getroom(spin, sizeof(*aff_entry), true);

        if (strcmp(items[2], "0") != 0) {
          aff_entry->ae_chop = getroom_save(spin, items[2]);
        }
        if (strcmp(items[3], "0") != 0) {
          aff_entry->ae_add = getroom_save(spin, items[3]);

          // Recognize flags on the affix: abcd/XYZ
          aff_entry->ae_flags = vim_strchr(aff_entry->ae_add, '/');
          if (aff_entry->ae_flags != NULL) {
            *aff_entry->ae_flags++ = NUL;
            aff_process_flags(aff, aff_entry);
          }
        }

        // Don't use an affix entry with non-ASCII characters when
        // "spin->si_ascii" is true.
        if (!spin->si_ascii || !(has_non_ascii(aff_entry->ae_chop)
                                 || has_non_ascii(aff_entry->ae_add))) {
          aff_entry->ae_next = cur_aff->ah_first;
          cur_aff->ah_first = aff_entry;

          if (strcmp(items[4], ".") != 0) {
            char buf[MAXLINELEN];

            aff_entry->ae_cond = getroom_save(spin, items[4]);
            snprintf(buf, sizeof(buf), *items[0] == 'P' ? "^%s" : "%s$", items[4]);
            aff_entry->ae_prog = vim_regcomp(buf, RE_MAGIC + RE_STRING + RE_STRICT);
            if (aff_entry->ae_prog == NULL) {
              smsg(0, _("Broken condition in %s line %d: %s"),
                   fname, lnum, items[4]);
            }
          }

          // For postponed prefixes we need an entry in si_prefcond
          // for the condition.  Use an existing one if possible.
          // Can't be done for an affix with flags, ignoring
          // COMPOUNDFORBIDFLAG and COMPOUNDPERMITFLAG.
          if (*items[0] == 'P' && aff->af_pfxpostpone
              && aff_entry->ae_flags == NULL) {
            bool upper = false;
            // When the chop string is one lower-case letter and
            // the add string ends in the upper-case letter we set
            // the "upper" flag, clear "ae_chop" and remove the
            // letters from "ae_add".  The condition must either
            // be empty or start with the same letter.
            if (aff_entry->ae_chop != NULL
                && aff_entry->ae_add != NULL
                && aff_entry->ae_chop[utfc_ptr2len(aff_entry->ae_chop)] ==
                NUL) {
              int c = utf_ptr2char(aff_entry->ae_chop);
              int c_up = SPELL_TOUPPER(c);
              if (c_up != c
                  && (aff_entry->ae_cond == NULL
                      || utf_ptr2char(aff_entry->ae_cond) == c)) {
                p = aff_entry->ae_add + strlen(aff_entry->ae_add);
                MB_PTR_BACK(aff_entry->ae_add, p);
                if (utf_ptr2char(p) == c_up) {
                  upper = true;
                  aff_entry->ae_chop = NULL;
                  *p = NUL;

                  // The condition is matched with the
                  // actual word, thus must check for the
                  // upper-case letter.
                  if (aff_entry->ae_cond != NULL) {
                    char buf[MAXLINELEN];
                    onecap_copy(items[4], buf, true);
                    aff_entry->ae_cond = getroom_save(spin, buf);
                    if (aff_entry->ae_cond != NULL) {
                      snprintf(buf, MAXLINELEN, "^%s", aff_entry->ae_cond);
                      vim_regfree(aff_entry->ae_prog);
                      aff_entry->ae_prog = vim_regcomp(buf, RE_MAGIC + RE_STRING);
                    }
                  }
                }
              }
            }

            if (aff_entry->ae_chop == NULL) {
              int idx;

              // Find a previously used condition.
              for (idx = spin->si_prefcond.ga_len - 1; idx >= 0; idx--) {
                p = ((char **)spin->si_prefcond.ga_data)[idx];
                if (str_equal(p, aff_entry->ae_cond)) {
                  break;
                }
              }
              if (idx < 0) {
                // Not found, add a new condition.
                idx = spin->si_prefcond.ga_len;
                char **pp = GA_APPEND_VIA_PTR(char *, &spin->si_prefcond);
                *pp = (aff_entry->ae_cond == NULL)
                      ? NULL : getroom_save(spin, aff_entry->ae_cond);
              }

              // Add the prefix to the prefix tree.
              if (aff_entry->ae_add == NULL) {
                p = "";
              } else {
                p = aff_entry->ae_add;
              }

              // PFX_FLAGS is a negative number, so that
              // tree_add_word() knows this is the prefix tree.
              int n = PFX_FLAGS;
              if (!cur_aff->ah_combine) {
                n |= WFP_NC;
              }
              if (upper) {
                n |= WFP_UP;
              }
              if (aff_entry->ae_comppermit) {
                n |= WFP_COMPPERMIT;
              }
              if (aff_entry->ae_compforbid) {
                n |= WFP_COMPFORBID;
              }
              tree_add_word(spin, p, spin->si_prefroot, n,
                            idx, cur_aff->ah_newID);
              did_postpone_prefix = true;
            }

            // Didn't actually use ah_newID, backup si_newprefID.
            if (aff_todo == 0 && !did_postpone_prefix) {
              spin->si_newprefID--;
              cur_aff->ah_newID = 0;
            }
          }
        }
      } else if (is_aff_rule(items, itemcnt, "FOL", 2) && fol == NULL) {
        fol = xstrdup(items[1]);
      } else if (is_aff_rule(items, itemcnt, "LOW", 2) && low == NULL) {
        low = xstrdup(items[1]);
      } else if (is_aff_rule(items, itemcnt, "UPP", 2) && upp == NULL) {
        upp = xstrdup(items[1]);
      } else if (is_aff_rule(items, itemcnt, "REP", 2)
                 || is_aff_rule(items, itemcnt, "REPSAL", 2)) {
        // Ignore REP/REPSAL count
        if (!isdigit((uint8_t)(*items[1]))) {
          smsg(0, _("Expected REP(SAL) count in %s line %d"),
               fname, lnum);
        }
      } else if ((strcmp(items[0], "REP") == 0
                  || strcmp(items[0], "REPSAL") == 0)
                 && itemcnt >= 3) {
        // REP/REPSAL item
        // Myspell ignores extra arguments, we require it starts with
        // # to detect mistakes.
        if (itemcnt > 3 && items[3][0] != '#') {
          smsg(0, _(e_afftrailing), fname, lnum, items[3]);
        }
        if (items[0][3] == 'S' ? do_repsal : do_rep) {
          // Replace underscore with space (can't include a space
          // directly).
          for (p = items[1]; *p != NUL; MB_PTR_ADV(p)) {
            if (*p == '_') {
              *p = ' ';
            }
          }
          for (p = items[2]; *p != NUL; MB_PTR_ADV(p)) {
            if (*p == '_') {
              *p = ' ';
            }
          }
          add_fromto(spin, items[0][3] == 'S'
                     ? &spin->si_repsal
                     : &spin->si_rep, items[1], items[2]);
        }
      } else if (is_aff_rule(items, itemcnt, "MAP", 2)) {
        // MAP item or count
        if (!found_map) {
          // First line contains the count.
          found_map = true;
          if (!isdigit((uint8_t)(*items[1]))) {
            smsg(0, _("Expected MAP count in %s line %d"),
                 fname, lnum);
          }
        } else if (do_mapline) {
          // Check that every character appears only once.
          for (p = items[1]; *p != NUL;) {
            int c = mb_ptr2char_adv((const char **)&p);
            if ((!GA_EMPTY(&spin->si_map)
                 && vim_strchr(spin->si_map.ga_data, c)
                 != NULL)
                || vim_strchr(p, c) != NULL) {
              smsg(0, _("Duplicate character in MAP in %s line %d"),
                   fname, lnum);
            }
          }

          // We simply concatenate all the MAP strings, separated by
          // slashes.
          ga_concat(&spin->si_map, items[1]);
          ga_append(&spin->si_map, '/');
        }
      }
      // Accept "SAL from to" and "SAL from to  #comment".
      else if (is_aff_rule(items, itemcnt, "SAL", 3)) {
        if (do_sal) {
          // SAL item (sounds-a-like)
          // Either one of the known keys or a from-to pair.
          if (strcmp(items[1], "followup") == 0) {
            spin->si_followup = sal_to_bool(items[2]);
          } else if (strcmp(items[1], "collapse_result") == 0) {
            spin->si_collapse = sal_to_bool(items[2]);
          } else if (strcmp(items[1], "remove_accents") == 0) {
            spin->si_rem_accents = sal_to_bool(items[2]);
          } else {
            // when "to" is "_" it means empty
            add_fromto(spin, &spin->si_sal, items[1],
                       strcmp(items[2], "_") == 0 ? ""
                                                  : items[2]);
          }
        }
      } else if (is_aff_rule(items, itemcnt, "SOFOFROM", 2)
                 && sofofrom == NULL) {
        sofofrom = getroom_save(spin, items[1]);
      } else if (is_aff_rule(items, itemcnt, "SOFOTO", 2)
                 && sofoto == NULL) {
        sofoto = getroom_save(spin, items[1]);
      } else if (strcmp(items[0], "COMMON") == 0) {
        for (int i = 1; i < itemcnt; i++) {
          if (HASHITEM_EMPTY(hash_find(&spin->si_commonwords, items[i]))) {
            p = xstrdup(items[i]);
            hash_add(&spin->si_commonwords, p);
          }
        }
      } else {
        smsg(0, _("Unrecognized or duplicate item in %s line %d: %s"),
             fname, lnum, items[0]);
      }
    }
  }

  if (fol != NULL || low != NULL || upp != NULL) {
    if (spin->si_clear_chartab) {
      // Clear the char type tables, don't want to use any of the
      // currently used spell properties.
      init_spell_chartab();
      spin->si_clear_chartab = false;
    }

    xfree(fol);
    xfree(low);
    xfree(upp);
  }

  // Use compound specifications of the .aff file for the spell info.
  if (compmax != 0) {
    aff_check_number(spin->si_compmax, compmax, "COMPOUNDWORDMAX");
    spin->si_compmax = compmax;
  }

  if (compminlen != 0) {
    aff_check_number(spin->si_compminlen, compminlen, "COMPOUNDMIN");
    spin->si_compminlen = compminlen;
  }

  if (compsylmax != 0) {
    if (syllable == NULL) {
      smsg(0, "%s", _("COMPOUNDSYLMAX used without SYLLABLE"));
    }
    aff_check_number(spin->si_compsylmax, compsylmax, "COMPOUNDSYLMAX");
    spin->si_compsylmax = compsylmax;
  }

  if (compoptions != 0) {
    aff_check_number(spin->si_compoptions, compoptions, "COMPOUND options");
    spin->si_compoptions |= compoptions;
  }

  if (compflags != NULL) {
    process_compflags(spin, aff, compflags);
  }

  // Check that we didn't use too many renumbered flags.
  if (spin->si_newcompID < spin->si_newprefID) {
    if (spin->si_newcompID == 127 || spin->si_newcompID == 255) {
      msg(_("Too many postponed prefixes"), 0);
    } else if (spin->si_newprefID == 0 || spin->si_newprefID == 127) {
      msg(_("Too many compound flags"), 0);
    } else {
      msg(_("Too many postponed prefixes and/or compound flags"), 0);
    }
  }

  if (syllable != NULL) {
    aff_check_string(spin->si_syllable, syllable, "SYLLABLE");
    spin->si_syllable = syllable;
  }

  if (sofofrom != NULL || sofoto != NULL) {
    if (sofofrom == NULL || sofoto == NULL) {
      smsg(0, _("Missing SOFO%s line in %s"),
           sofofrom == NULL ? "FROM" : "TO", fname);
    } else if (!GA_EMPTY(&spin->si_sal)) {
      smsg(0, _("Both SAL and SOFO lines in %s"), fname);
    } else {
      aff_check_string(spin->si_sofofr, sofofrom, "SOFOFROM");
      aff_check_string(spin->si_sofoto, sofoto, "SOFOTO");
      spin->si_sofofr = sofofrom;
      spin->si_sofoto = sofoto;
    }
  }

  if (midword != NULL) {
    aff_check_string(spin->si_midword, midword, "MIDWORD");
    spin->si_midword = midword;
  }

  xfree(pc);
  fclose(fd);
  return aff;
}

// For affix "entry" move COMPOUNDFORBIDFLAG and COMPOUNDPERMITFLAG from
// ae_flags to ae_comppermit and ae_compforbid.
static void aff_process_flags(afffile_T *affile, affentry_T *entry)
{
  if (entry->ae_flags != NULL
      && (affile->af_compforbid != 0 || affile->af_comppermit != 0)) {
    for (char *p = entry->ae_flags; *p != NUL;) {
      char *prevp = p;
      unsigned flag = get_affitem(affile->af_flagtype, &p);
      if (flag == affile->af_comppermit || flag == affile->af_compforbid) {
        STRMOVE(prevp, p);
        p = prevp;
        if (flag == affile->af_comppermit) {
          entry->ae_comppermit = true;
        } else {
          entry->ae_compforbid = true;
        }
      }
      if (affile->af_flagtype == AFT_NUM && *p == ',') {
        p++;
      }
    }
    if (*entry->ae_flags == NUL) {
      entry->ae_flags = NULL;           // nothing left
    }
  }
}

// Turn an affix flag name into a number, according to the FLAG type.
// returns zero for failure.
static unsigned affitem2flag(int flagtype, char *item, char *fname, int lnum)
{
  char *p = item;

  unsigned res = get_affitem(flagtype, &p);
  if (res == 0) {
    if (flagtype == AFT_NUM) {
      smsg(0, _("Flag is not a number in %s line %d: %s"),
           fname, lnum, item);
    } else {
      smsg(0, _("Illegal flag in %s line %d: %s"),
           fname, lnum, item);
    }
  }
  if (*p != NUL) {
    smsg(0, _(e_affname), fname, lnum, item);
    return 0;
  }

  return res;
}

// Get one affix name from "*pp" and advance the pointer.
// Returns ZERO_FLAG for "0".
// Returns zero for an error, still advances the pointer then.
static unsigned get_affitem(int flagtype, char **pp)
{
  int res;

  if (flagtype == AFT_NUM) {
    if (!ascii_isdigit(**pp)) {
      (*pp)++;            // always advance, avoid getting stuck
      return 0;
    }
    res = getdigits_int(pp, true, 0);
    if (res == 0) {
      res = ZERO_FLAG;
    }
  } else {
    res = mb_ptr2char_adv((const char **)pp);
    if (flagtype == AFT_LONG || (flagtype == AFT_CAPLONG
                                 && res >= 'A' && res <= 'Z')) {
      if (**pp == NUL) {
        return 0;
      }
      res = mb_ptr2char_adv((const char **)pp) + (res << 16);
    }
  }
  return (unsigned)res;
}

/// Process the "compflags" string used in an affix file and append it to
/// spin->si_compflags.
/// The processing involves changing the affix names to ID numbers, so that
/// they fit in one byte.
static void process_compflags(spellinfo_T *spin, afffile_T *aff, char *compflags)
{
  compitem_T *ci;
  int id;
  char key[AH_KEY_LEN];

  // Make room for the old and the new compflags, concatenated with a / in
  // between.  Processing it makes it shorter, but we don't know by how
  // much, thus allocate the maximum.
  int len = (int)strlen(compflags) + 1;
  if (spin->si_compflags != NULL) {
    len += (int)strlen(spin->si_compflags) + 1;
  }
  char *p = getroom(spin, (size_t)len, false);
  if (spin->si_compflags != NULL) {
    STRCPY(p, spin->si_compflags);
    strcat(p, "/");
  }
  spin->si_compflags = p;
  uint8_t *tp = (uint8_t *)p + strlen(p);

  for (p = compflags; *p != NUL;) {
    if (vim_strchr("/?*+[]", (uint8_t)(*p)) != NULL) {
      // Copy non-flag characters directly.
      *tp++ = (uint8_t)(*p++);
    } else {
      // First get the flag number, also checks validity.
      char *prevp = p;
      unsigned flag = get_affitem(aff->af_flagtype, &p);
      if (flag != 0) {
        // Find the flag in the hashtable.  If it was used before, use
        // the existing ID.  Otherwise add a new entry.
        xmemcpyz(key, prevp, (size_t)(p - prevp));
        hashitem_T *hi = hash_find(&aff->af_comp, key);
        if (!HASHITEM_EMPTY(hi)) {
          id = HI2CI(hi)->ci_newID;
        } else {
          ci = getroom(spin, sizeof(compitem_T), true);
          STRCPY(ci->ci_key, key);
          ci->ci_flag = flag;
          // Avoid using a flag ID that has a special meaning in a
          // regexp (also inside []).
          do {
            check_renumber(spin);
            id = spin->si_newcompID--;
          } while (vim_strchr("/?*+[]\\-^", id) != NULL);
          ci->ci_newID = id;
          hash_add(&aff->af_comp, ci->ci_key);
        }
        *tp++ = (uint8_t)id;
      }
      if (aff->af_flagtype == AFT_NUM && *p == ',') {
        p++;
      }
    }
  }

  *tp = NUL;
}

// Check that the new IDs for postponed affixes and compounding don't overrun
// each other.  We have almost 255 available, but start at 0-127 to avoid
// using two bytes for utf-8.  When the 0-127 range is used up go to 128-255.
// When that is used up an error message is given.
static void check_renumber(spellinfo_T *spin)
{
  if (spin->si_newprefID == spin->si_newcompID && spin->si_newcompID < 128) {
    spin->si_newprefID = 127;
    spin->si_newcompID = 255;
  }
}

// Returns true if flag "flag" appears in affix list "afflist".
static bool flag_in_afflist(int flagtype, char *afflist, unsigned flag)
{
  switch (flagtype) {
  case AFT_CHAR:
    return vim_strchr(afflist, (int)flag) != NULL;

  case AFT_CAPLONG:
  case AFT_LONG:
    for (char *p = afflist; *p != NUL;) {
      unsigned n = (unsigned)mb_ptr2char_adv((const char **)&p);
      if ((flagtype == AFT_LONG || (n >= 'A' && n <= 'Z'))
          && *p != NUL) {
        n = (unsigned)mb_ptr2char_adv((const char **)&p) + (n << 16);
      }
      if (n == flag) {
        return true;
      }
    }
    break;

  case AFT_NUM:
    for (char *p = afflist; *p != NUL;) {
      int digits = getdigits_int(&p, true, 0);
      assert(digits >= 0);
      unsigned n = (unsigned)digits;
      if (n == 0) {
        n = ZERO_FLAG;
      }
      if (n == flag) {
        return true;
      }
      if (*p != NUL) {          // skip over comma
        p++;
      }
    }
    break;
  }
  return false;
}

// Give a warning when "spinval" and "affval" numbers are set and not the same.
static void aff_check_number(int spinval, int affval, char *name)
{
  if (spinval != 0 && spinval != affval) {
    smsg(0, _("%s value differs from what is used in another .aff file"),
         name);
  }
}

/// Give a warning when "spinval" and "affval" strings are set and not the same.
static void aff_check_string(char *spinval, char *affval, char *name)
{
  if (spinval != NULL && strcmp(spinval, affval) != 0) {
    smsg(0, _("%s value differs from what is used in another .aff file"),
         name);
  }
}

/// Add a from-to item to "gap".  Used for REP and SAL items.
/// They are stored case-folded.
static void add_fromto(spellinfo_T *spin, garray_T *gap, char *from, char *to)
{
  char word[MAXWLEN];

  fromto_T *ftp = GA_APPEND_VIA_PTR(fromto_T, gap);
  spell_casefold(curwin, from, (int)strlen(from), word, MAXWLEN);
  ftp->ft_from = getroom_save(spin, word);
  spell_casefold(curwin, to, (int)strlen(to), word, MAXWLEN);
  ftp->ft_to = getroom_save(spin, word);
}

// Free the structure filled by spell_read_aff().
static void spell_free_aff(afffile_T *aff)
{
  xfree(aff->af_enc);

  // All this trouble to free the "ae_prog" items...
  for (hashtab_T *ht = &aff->af_pref;; ht = &aff->af_suff) {
    int todo = (int)ht->ht_used;
    for (hashitem_T *hi = ht->ht_array; todo > 0; hi++) {
      if (!HASHITEM_EMPTY(hi)) {
        todo--;
        affheader_T *ah = HI2AH(hi);
        for (affentry_T *ae = ah->ah_first; ae != NULL; ae = ae->ae_next) {
          vim_regfree(ae->ae_prog);
        }
      }
    }
    if (ht == &aff->af_suff) {
      break;
    }
  }

  hash_clear(&aff->af_pref);
  hash_clear(&aff->af_suff);
  hash_clear(&aff->af_comp);
}

// Read dictionary file "fname".
// Returns OK or FAIL;
static int spell_read_dic(spellinfo_T *spin, char *fname, afffile_T *affile)
{
  hashtab_T ht;
  char line[MAXLINELEN];
  char word_buf[MAXLINELEN];    // Rust writes unescaped word here
  char store_afflist[MAXWLEN];
  char *pc;
  int lnum = 1;
  int non_ascii = 0;
  int retval = OK;
  char message[MAXLINELEN + MAXWLEN];
  int duplicate = 0;
  Timestamp last_msg_time = 0;

  // Open the file.
  FILE *fd = os_fopen(fname, "r");
  if (fd == NULL) {
    semsg(_(e_notopen), fname);
    return FAIL;
  }

  // The hashtable is only used to detect duplicated words.
  hash_init(&ht);

  vim_snprintf(IObuff, IOSIZE,
               _("Reading dictionary file %s..."), fname);
  spell_message(spin, IObuff);

  // start with a message for the first line
  spin->si_msg_count = 999999;

  // Read and ignore the first line: word count.
  if (vim_fgets(line, MAXLINELEN, fd) || !ascii_isdigit(*skipwhite(line))) {
    semsg(_("E760: No word count in %s"), fname);
  }

  // Read all the lines in the file one by one.
  // The words are converted to 'encoding' here, before being added to
  // the hashtable.
  while (!vim_fgets(line, MAXLINELEN, fd) && !got_int) {
    line_breakcheck();
    lnum++;

    // Convert from "SET" to 'encoding' when needed (before Rust parsing).
    if (spin->si_conv.vc_type != CONV_NONE) {
      pc = string_convert(&spin->si_conv, line, NULL);
      if (pc == NULL) {
        smsg(0, _("Conversion failure for word in %s line %d: %s"),
             fname, lnum, line);
        continue;
      }
    } else {
      pc = NULL;
    }
    const char *src_line = (pc != NULL) ? pc : line;
    size_t src_len = strlen(src_line);

    // Use Rust to: skip comments/empty lines, strip CR/LF, handle escapes,
    // and split word from affix list.
    RsDicLineResult rs_res;
    int parse_ret = rs_parse_dic_line((const uint8_t *)src_line, src_len,
                                      (uint8_t *)word_buf, sizeof(word_buf),
                                      &rs_res);
    if (parse_ret != 0) {
      // 1 = skip (comment/empty), <0 = error
      xfree(pc);
      continue;
    }

    word_buf[rs_res.word_len] = NUL;
    char *w = word_buf;

    // Reconstruct afflist pointer into src_line buffer.
    char *afflist = NULL;
    if (rs_res.affix_offset != 0xFFFF) {
      afflist = (char *)src_line + rs_res.affix_offset;
    }

    // Skip non-ASCII words when "spin->si_ascii" is true.
    if (spin->si_ascii && has_non_ascii(w)) {
      non_ascii++;
      xfree(pc);
      continue;
    }

    // This takes time, print a message every 10000 words, but not more
    // often than once per second.
    if (spin->si_verbose && spin->si_msg_count > 10000) {
      spin->si_msg_count = 0;
      if (os_time() > last_msg_time) {
        last_msg_time = os_time();
        vim_snprintf(message, sizeof(message),
                     _("line %6d, word %6d - %s"),
                     lnum, spin->si_foldwcount + spin->si_keepwcount, w);
        msg_start();
        msg_outtrans_long(message, 0);
        msg_clr_eos();
        msg_didout = false;
        msg_col = 0;
        ui_flush();
      }
    }

    // Store the word in the hashtable to be able to find duplicates.
    char *dw = getroom_save(spin, w);
    if (dw == NULL) {
      retval = FAIL;
      xfree(pc);
      break;
    }

    hash_T hash = hash_hash(dw);
    hashitem_T *hi = hash_lookup(&ht, dw, strlen(dw), hash);
    if (!HASHITEM_EMPTY(hi)) {
      if (p_verbose > 0) {
        smsg(0, _("Duplicate word in %s line %d: %s"),
             fname, lnum, dw);
      } else if (duplicate == 0) {
        smsg(0, _("First duplicate word in %s line %d: %s"),
             fname, lnum, dw);
      }
      duplicate++;
    } else {
      hash_add_item(&ht, hi, dw, hash);
    }

    int flags = 0;
    store_afflist[0] = NUL;
    int pfxlen = 0;
    bool need_affix = false;
    if (afflist != NULL) {
      // Extract flags from the affix list.
      flags |= get_affix_flags(affile, afflist);

      if (affile->af_needaffix != 0
          && flag_in_afflist(affile->af_flagtype, afflist,
                             affile->af_needaffix)) {
        need_affix = true;
      }

      if (affile->af_pfxpostpone) {
        // Need to store the list of prefix IDs with the word.
        pfxlen = get_pfxlist(affile, afflist, store_afflist);
      }

      if (spin->si_compflags != NULL) {
        // Need to store the list of compound flags with the word.
        // Concatenate them to the list of prefix IDs.
        get_compflags(affile, afflist, store_afflist + pfxlen);
      }
    }

    // Add the word to the word tree(s).
    if (store_word(spin, dw, flags, spin->si_region,
                   store_afflist, need_affix) == FAIL) {
      retval = FAIL;
    }

    if (afflist != NULL) {
      // Find all matching suffixes and add the resulting words.
      // Additionally do matching prefixes that combine.
      if (store_aff_word(spin, dw, afflist, affile,
                         &affile->af_suff, &affile->af_pref,
                         CONDIT_SUF, flags, store_afflist, pfxlen) == FAIL) {
        retval = FAIL;
      }

      // Find all matching prefixes and add the resulting words.
      if (store_aff_word(spin, dw, afflist, affile,
                         &affile->af_pref, NULL,
                         CONDIT_SUF, flags, store_afflist, pfxlen) == FAIL) {
        retval = FAIL;
      }
    }

    xfree(pc);
  }

  if (duplicate > 0) {
    smsg(0, _("%d duplicate word(s) in %s"), duplicate, fname);
  }
  if (spin->si_ascii && non_ascii > 0) {
    smsg(0, _("Ignored %d word(s) with non-ASCII characters in %s"),
         non_ascii, fname);
  }
  hash_clear(&ht);

  fclose(fd);
  return retval;
}

// Check for affix flags in "afflist" that are turned into word flags.
// Return WF_ flags.
static int get_affix_flags(afffile_T *affile, char *afflist)
{
  int flags = 0;

  if (affile->af_keepcase != 0
      && flag_in_afflist(affile->af_flagtype, afflist,
                         affile->af_keepcase)) {
    flags |= WF_KEEPCAP | WF_FIXCAP;
  }
  if (affile->af_rare != 0
      && flag_in_afflist(affile->af_flagtype, afflist, affile->af_rare)) {
    flags |= WF_RARE;
  }
  if (affile->af_bad != 0
      && flag_in_afflist(affile->af_flagtype, afflist, affile->af_bad)) {
    flags |= WF_BANNED;
  }
  if (affile->af_needcomp != 0
      && flag_in_afflist(affile->af_flagtype, afflist,
                         affile->af_needcomp)) {
    flags |= WF_NEEDCOMP;
  }
  if (affile->af_comproot != 0
      && flag_in_afflist(affile->af_flagtype, afflist,
                         affile->af_comproot)) {
    flags |= WF_COMPROOT;
  }
  if (affile->af_nosuggest != 0
      && flag_in_afflist(affile->af_flagtype, afflist,
                         affile->af_nosuggest)) {
    flags |= WF_NOSUGGEST;
  }
  return flags;
}

// Get the list of prefix IDs from the affix list "afflist".
// Used for PFXPOSTPONE.
// Put the resulting flags in "store_afflist[MAXWLEN]" with a terminating NUL
// and return the number of affixes.
static int get_pfxlist(afffile_T *affile, char *afflist, char *store_afflist)
{
  int cnt = 0;
  char key[AH_KEY_LEN];

  for (char *p = afflist; *p != NUL;) {
    char *prevp = p;
    if (get_affitem(affile->af_flagtype, &p) != 0) {
      // A flag is a postponed prefix flag if it appears in "af_pref"
      // and its ID is not zero.
      xmemcpyz(key, prevp, (size_t)(p - prevp));
      hashitem_T *hi = hash_find(&affile->af_pref, key);
      if (!HASHITEM_EMPTY(hi)) {
        int id = HI2AH(hi)->ah_newID;
        if (id != 0) {
          store_afflist[cnt++] = (char)(uint8_t)id;
        }
      }
    }
    if (affile->af_flagtype == AFT_NUM && *p == ',') {
      p++;
    }
  }

  store_afflist[cnt] = NUL;
  return cnt;
}

// Get the list of compound IDs from the affix list "afflist" that are used
// for compound words.
// Puts the flags in "store_afflist[]".
static void get_compflags(afffile_T *affile, char *afflist, char *store_afflist)
{
  int cnt = 0;
  char key[AH_KEY_LEN];

  for (char *p = afflist; *p != NUL;) {
    char *prevp = p;
    if (get_affitem(affile->af_flagtype, &p) != 0) {
      // A flag is a compound flag if it appears in "af_comp".
      xmemcpyz(key, prevp, (size_t)(p - prevp));
      hashitem_T *hi = hash_find(&affile->af_comp, key);
      if (!HASHITEM_EMPTY(hi)) {
        store_afflist[cnt++] = (char)(uint8_t)HI2CI(hi)->ci_newID;
      }
    }
    if (affile->af_flagtype == AFT_NUM && *p == ',') {
      p++;
    }
  }

  store_afflist[cnt] = NUL;
}

/// Apply affixes to a word and store the resulting words.
/// "ht" is the hashtable with affentry_T that need to be applied, either
/// prefixes or suffixes.
/// "xht", when not NULL, is the prefix hashtable, to be used additionally on
/// the resulting words for combining affixes.
///
/// @param spin  spell info
/// @param word  basic word start
/// @param afflist  list of names of supported affixes
/// @param condit  CONDIT_SUF et al.
/// @param flags  flags for the word
/// @param pfxlist  list of prefix IDs
/// @param pfxlen  nr of flags in "pfxlist" for prefixes, rest is compound flags
///
/// @return  FAIL when out of memory.
static int store_aff_word(spellinfo_T *spin, char *word, char *afflist, afffile_T *affile,
                          hashtab_T *ht, hashtab_T *xht, int condit, int flags, char *pfxlist,
                          int pfxlen)
{
  affentry_T *ae;
  char newword[MAXWLEN];
  int retval = OK;
  int j;
  char store_afflist[MAXWLEN];
  char pfx_pfxlist[MAXWLEN];
  size_t wordlen = strlen(word);

  int todo = (int)ht->ht_used;
  for (hashitem_T *hi = ht->ht_array; todo > 0 && retval == OK; hi++) {
    if (!HASHITEM_EMPTY(hi)) {
      todo--;
      affheader_T *ah = HI2AH(hi);

      // Check that the affix combines, if required, and that the word
      // supports this affix.
      if (((condit & CONDIT_COMB) == 0 || ah->ah_combine)
          && flag_in_afflist(affile->af_flagtype, afflist,
                             ah->ah_flag)) {
        // Loop over all affix entries with this name.
        for (ae = ah->ah_first; ae != NULL; ae = ae->ae_next) {
          // Check the condition.  It's not logical to match case
          // here, but it is required for compatibility with
          // Myspell.
          // Another requirement from Myspell is that the chop
          // string is shorter than the word itself.
          // For prefixes, when "PFXPOSTPONE" was used, only do
          // prefixes with a chop string and/or flags.
          // When a previously added affix had CIRCUMFIX this one
          // must have it too, if it had not then this one must not
          // have one either.
          if ((xht != NULL || !affile->af_pfxpostpone
               || ae->ae_chop != NULL
               || ae->ae_flags != NULL)
              && (ae->ae_chop == NULL
                  || strlen(ae->ae_chop) < wordlen)
              && (ae->ae_prog == NULL
                  || vim_regexec_prog(&ae->ae_prog, false, word, 0))
              && (((condit & CONDIT_CFIX) == 0)
                  == ((condit & CONDIT_AFF) == 0
                      || ae->ae_flags == NULL
                      || !flag_in_afflist(affile->af_flagtype,
                                          ae->ae_flags, affile->af_circumfix)))) {
            // Match.  Remove the chop and add the affix.
            if (xht == NULL) {
              // prefix: chop/add at the start of the word
              if (ae->ae_add == NULL) {
                *newword = NUL;
              } else {
                xstrlcpy(newword, ae->ae_add, MAXWLEN);
              }
              char *p = word;
              if (ae->ae_chop != NULL) {
                // Skip chop string.
                int i = mb_charlen(ae->ae_chop);
                for (; i > 0; i--) {
                  MB_PTR_ADV(p);
                }
              }
              strcat(newword, p);
            } else {
              // suffix: chop/add at the end of the word
              xstrlcpy(newword, word, MAXWLEN);
              if (ae->ae_chop != NULL) {
                // Remove chop string.
                char *p = newword + strlen(newword);
                int i = mb_charlen(ae->ae_chop);
                for (; i > 0; i--) {
                  MB_PTR_BACK(newword, p);
                }
                *p = NUL;
              }
              if (ae->ae_add != NULL) {
                strcat(newword, ae->ae_add);
              }
            }

            int use_flags = flags;
            char *use_pfxlist = pfxlist;
            int use_pfxlen = pfxlen;
            bool need_affix = false;
            int use_condit = condit | CONDIT_COMB | CONDIT_AFF;
            if (ae->ae_flags != NULL) {
              // Extract flags from the affix list.
              use_flags |= get_affix_flags(affile, ae->ae_flags);

              if (affile->af_needaffix != 0
                  && flag_in_afflist(affile->af_flagtype, ae->ae_flags,
                                     affile->af_needaffix)) {
                need_affix = true;
              }

              // When there is a CIRCUMFIX flag the other affix
              // must also have it and we don't add the word
              // with one affix.
              if (affile->af_circumfix != 0
                  && flag_in_afflist(affile->af_flagtype, ae->ae_flags,
                                     affile->af_circumfix)) {
                use_condit |= CONDIT_CFIX;
                if ((condit & CONDIT_CFIX) == 0) {
                  need_affix = true;
                }
              }

              if (affile->af_pfxpostpone
                  || spin->si_compflags != NULL) {
                if (affile->af_pfxpostpone) {
                  // Get prefix IDS from the affix list.
                  use_pfxlen = get_pfxlist(affile, ae->ae_flags, store_afflist);
                } else {
                  use_pfxlen = 0;
                }
                use_pfxlist = store_afflist;

                // Combine the prefix IDs. Avoid adding the
                // same ID twice.
                for (int i = 0; i < pfxlen; i++) {
                  for (j = 0; j < use_pfxlen; j++) {
                    if (pfxlist[i] == use_pfxlist[j]) {
                      break;
                    }
                  }
                  if (j == use_pfxlen) {
                    use_pfxlist[use_pfxlen++] = pfxlist[i];
                  }
                }

                if (spin->si_compflags != NULL) {
                  // Get compound IDS from the affix list.
                  get_compflags(affile, ae->ae_flags,
                                use_pfxlist + use_pfxlen);
                } else {
                  use_pfxlist[use_pfxlen] = NUL;
                }

                // Combine the list of compound flags.
                // Concatenate them to the prefix IDs list.
                // Avoid adding the same ID twice.
                for (int i = pfxlen; pfxlist[i] != NUL; i++) {
                  for (j = use_pfxlen; use_pfxlist[j] != NUL; j++) {
                    if (pfxlist[i] == use_pfxlist[j]) {
                      break;
                    }
                  }
                  if (use_pfxlist[j] == NUL) {
                    use_pfxlist[j++] = pfxlist[i];
                    use_pfxlist[j] = NUL;
                  }
                }
              }
            }

            // Obey a "COMPOUNDFORBIDFLAG" of the affix: don't
            // use the compound flags.
            if (use_pfxlist != NULL && ae->ae_compforbid) {
              xmemcpyz(pfx_pfxlist, use_pfxlist, (size_t)use_pfxlen);
              use_pfxlist = pfx_pfxlist;
            }

            // When there are postponed prefixes...
            if (spin->si_prefroot != NULL
                && spin->si_prefroot->wn_sibling != NULL) {
              // ... add a flag to indicate an affix was used.
              use_flags |= WF_HAS_AFF;

              // ... don't use a prefix list if combining
              // affixes is not allowed.  But do use the
              // compound flags after them.
              if (!ah->ah_combine && use_pfxlist != NULL) {
                use_pfxlist += use_pfxlen;
              }
            }

            // When compounding is supported and there is no
            // "COMPOUNDPERMITFLAG" then forbid compounding on the
            // side where the affix is applied.
            if (spin->si_compflags != NULL && !ae->ae_comppermit) {
              if (xht != NULL) {
                use_flags |= WF_NOCOMPAFT;
              } else {
                use_flags |= WF_NOCOMPBEF;
              }
            }

            // Store the modified word.
            if (store_word(spin, newword, use_flags,
                           spin->si_region, use_pfxlist,
                           need_affix) == FAIL) {
              retval = FAIL;
            }

            // When added a prefix or a first suffix and the affix
            // has flags may add a(nother) suffix.  RECURSIVE!
            if ((condit & CONDIT_SUF) && ae->ae_flags != NULL) {
              if (store_aff_word(spin, newword, ae->ae_flags,
                                 affile, &affile->af_suff, xht,
                                 use_condit & (xht == NULL
                                               ? ~0 : ~CONDIT_SUF),
                                 use_flags, use_pfxlist, pfxlen) == FAIL) {
                retval = FAIL;
              }
            }

            // When added a suffix and combining is allowed also
            // try adding a prefix additionally.  Both for the
            // word flags and for the affix flags.  RECURSIVE!
            if (xht != NULL && ah->ah_combine) {
              if (store_aff_word(spin, newword,
                                 afflist, affile,
                                 xht, NULL, use_condit,
                                 use_flags, use_pfxlist,
                                 pfxlen) == FAIL
                  || (ae->ae_flags != NULL
                      && store_aff_word(spin, newword,
                                        ae->ae_flags, affile,
                                        xht, NULL, use_condit,
                                        use_flags, use_pfxlist,
                                        pfxlen) == FAIL)) {
                retval = FAIL;
              }
            }
          }
        }
      }
    }
  }

  return retval;
}

// Read a file with a list of words.
// Uses Rust (rs_parse_wordfile_line) to parse each line.
static int spell_read_wordfile(spellinfo_T *spin, char *fname)
{
  linenr_T lnum = 0;
  char rline[MAXLINELEN];
  char *pc = NULL;
  int retval = OK;
  bool did_word = false;
  int non_ascii = 0;

  // Open the file.
  FILE *fd = os_fopen(fname, "r");
  if (fd == NULL) {
    semsg(_(e_notopen), fname);
    return FAIL;
  }

  vim_snprintf(IObuff, IOSIZE, _("Reading word file %s..."), fname);
  spell_message(spin, IObuff);

  // Read all the lines in the file one by one.
  while (!vim_fgets(rline, MAXLINELEN, fd) && !got_int) {
    line_breakcheck();
    lnum++;

    // Convert encoding when needed (before parsing).
    xfree(pc);
    if (spin->si_conv.vc_type != CONV_NONE) {
      pc = string_convert(&spin->si_conv, rline, NULL);
      if (pc == NULL) {
        smsg(0, _("Conversion failure for word in %s line %" PRIdLINENR ": %s"),
             fname, lnum, rline);
        continue;
      }
    } else {
      pc = NULL;
    }
    const char *line = (pc != NULL) ? pc : rline;
    size_t line_len = strlen(line);

    // Use Rust to classify and parse this line.
    RsWordfileLineResult rs_res;
    int parse_ret = rs_parse_wordfile_line((const uint8_t *)line, line_len,
                                           spin->si_region_count, &rs_res);

    if (parse_ret == 2) {
      // Skip: comment or empty line.
      continue;
    }

    if (parse_ret == 1) {
      // Directive line.
      if (rs_res.directive[0] == 'e') {
        // /encoding= directive
        if (spin->si_conv.vc_type != CONV_NONE) {
          smsg(0, _("Duplicate /encoding= line ignored in %s line %" PRIdLINENR ": %s"),
               fname, lnum, line);
        } else if (did_word) {
          smsg(0, _("/encoding= line after word ignored in %s line %" PRIdLINENR ": %s"),
               fname, lnum, line);
        } else {
          // Value starts at word_end_offset in line buffer.
          char *enc_val = (char *)(line + rs_res.word_end_offset);
          char *enc = enc_canonize(enc_val);
          if (!spin->si_ascii
              && convert_setup(&spin->si_conv, enc, p_enc) == FAIL) {
            smsg(0, _("Conversion in %s not supported: from %s to %s"),
                 fname, enc_val, p_enc);
          }
          xfree(enc);
          spin->si_conv.vc_fail = true;
        }
      } else if (rs_res.directive[0] == 'r') {
        // /regions= directive
        if (spin->si_region_count > 1) {
          smsg(0, _("Duplicate /regions= line ignored in %s line %" PRIdLINENR ": %s"),
               fname, lnum, line + 1);
        } else {
          const char *reg_val = line + rs_res.word_end_offset;
          if (rs_res.word_len > MAXREGIONS * 2) {
            smsg(0, _("Too many regions in %s line %" PRIdLINENR ": %s"),
                 fname, lnum, reg_val);
          } else {
            spin->si_region_count = rs_res.region_count;
            STRCPY(spin->si_region_name, reg_val);
            // Adjust the mask for a word valid in all regions.
            spin->si_region = (1 << spin->si_region_count) - 1;
          }
        }
      } else {
        // Unknown directive: Rust already returned 2 for these; shouldn't reach.
        smsg(0, _("/ line ignored in %s line %" PRIdLINENR ": %s"),
             fname, lnum, line);
      }
      continue;
    }

    if (parse_ret == 3) {
      // Invalid region number.
      smsg(0, _("Invalid region nr in %s line %" PRIdLINENR ": %s"),
           fname, lnum, line);
      continue;
    }

    if (parse_ret == 4) {
      // Unrecognized flag.
      smsg(0, _("Unrecognized flags in %s line %" PRIdLINENR ": %s"),
           fname, lnum, line);
      continue;
    }

    if (parse_ret != 0) {
      // Other error: skip.
      continue;
    }

    // Ordinary word line.
    // NUL-terminate the word at word_end_offset.
    char word_copy[MAXLINELEN];
    size_t wlen = rs_res.word_len;
    if (wlen >= sizeof(word_copy)) {
      wlen = sizeof(word_copy) - 1;
    }
    memcpy(word_copy, line, wlen);
    word_copy[wlen] = NUL;

    int flags = rs_res.flags;
    int regionmask = (flags & WF_REGION) ? rs_res.regionmask : spin->si_region;

    // Skip non-ASCII words when "spin->si_ascii" is true.
    if (spin->si_ascii && has_non_ascii(word_copy)) {
      non_ascii++;
      continue;
    }

    // Normal word: store it.
    if (store_word(spin, word_copy, flags, regionmask, NULL, false) == FAIL) {
      retval = FAIL;
      break;
    }
    did_word = true;
  }

  xfree(pc);
  fclose(fd);

  if (spin->si_ascii && non_ascii > 0) {
    vim_snprintf(IObuff, IOSIZE,
                 _("Ignored %d words with non-ASCII characters"), non_ascii);
    spell_message(spin, IObuff);
  }

  return retval;
}

/// Get part of an sblock_T, "len" bytes long.
/// This avoids calling free() for every little struct we use (and keeping
/// track of them).
/// The memory is cleared to all zeros.
///
/// @param len Length needed (<= SBLOCKSIZE).
/// @param align Align for pointer.
/// @return Pointer into block data.
void *getroom(spellinfo_T *spin, size_t len, bool align)
  FUNC_ATTR_NONNULL_RET
{
  sblock_T *bl = spin->si_blocks;

  assert(len <= SBLOCKSIZE);

  if (align && bl != NULL) {
    // Round size up for alignment.  On some systems structures need to be
    // aligned to the size of a pointer (e.g., SPARC).
    bl->sb_used = (int)(((size_t)bl->sb_used + sizeof(char *) - 1) & ~(sizeof(char *) - 1));
  }

  if (bl == NULL || (size_t)bl->sb_used + len > SBLOCKSIZE) {
    // Allocate a block of memory. It is not freed until much later.
    bl = xcalloc(1, offsetof(sblock_T, sb_data) + SBLOCKSIZE + 1);
    bl->sb_next = spin->si_blocks;
    spin->si_blocks = bl;
    bl->sb_used = 0;
    spin->si_blocks_cnt++;
  }

  char *p = bl->sb_data + bl->sb_used;
  bl->sb_used += (int)len;

  return p;
}

/// Make a copy of a string into memory allocated with getroom().
///
/// @return  NULL when out of memory.
static char *getroom_save(spellinfo_T *spin, char *s)
{
  const size_t s_size = strlen(s) + 1;
  return memcpy(getroom(spin, s_size, false), s, s_size);
}

// Free the list of allocated sblock_T.
void free_blocks(sblock_T *bl)
{
  while (bl != NULL) {
    sblock_T *next = bl->sb_next;
    xfree(bl);
    bl = next;
  }
}

// Thin wrapper so Rust can call the static get_wordnode.
wordnode_T *nvim_get_wordnode(spellinfo_T *spin) { return get_wordnode(spin); }

/// Write the Vim .spl file "fname".
///
/// @return  OK/FAIL.
static int write_vim_spell(spellinfo_T *spin, char *fname)
{
  int retval = OK;
  int regionmask;

  FILE *fd = os_fopen(fname, "w");
  if (fd == NULL) {
    semsg(_(e_notopen), fname);
    return FAIL;
  }

  // <HEADER>: <fileID> <versionnr>
  size_t fwv = fwrite(VIMSPELLMAGIC, VIMSPELLMAGICL, 1, fd);
  if (fwv != 1) {
    goto theend;
  }
  putc(VIMSPELLVERSION, fd);

  // Compute regionmask (needed for tree writing below).
  regionmask = spin->si_region_count > 1
               ? (1 << spin->si_region_count) - 1
               : 0;

  // <SECTIONS>: <section> ... <sectionend>

  // SN_CHARFLAGS: must stay in C (uses spelltab + utf_char2bytes).
  if (!spin->si_ascii && !spin->si_add) {
    char folchars[128 * 8];

    putc(SN_CHARFLAGS, fd);
    putc(SNF_REQUIRED, fd);

    size_t l = 0;
    for (size_t i = 128; i < 256; i++) {
      l += (size_t)utf_char2bytes(spelltab.st_fold[i], folchars + l);
    }
    put_bytes(fd, 1 + 128 + 2 + l, 4);

    fputc(128, fd);
    for (size_t i = 128; i < 256; i++) {
      int flags = 0;
      if (spelltab.st_isw[i]) {
        flags |= CF_WORD;
      }
      if (spelltab.st_isu[i]) {
        flags |= CF_UPPER;
      }
      fputc(flags, fd);
    }

    put_bytes(fd, l, 2);
    fwv &= fwrite(folchars, l, 1, fd);
  }

  // All other sections are written by Rust via rs_write_spell_sections.
  {
    // Sort REP and REPSAL before passing pointers to Rust.
    if (!GA_EMPTY(&spin->si_rep)) {
      qsort(spin->si_rep.ga_data, (size_t)spin->si_rep.ga_len,
            sizeof(fromto_T), rep_compare);
    }
    if (!GA_EMPTY(&spin->si_repsal)) {
      qsort(spin->si_repsal.ga_data, (size_t)spin->si_repsal.ga_len,
            sizeof(fromto_T), rep_compare);
    }

    // Set si_sugtime now (needed for SN_SUGFILE section).
    int64_t sugtime = 0;
    if (!spin->si_nosugfile
        && (!GA_EMPTY(&spin->si_sal)
            || (spin->si_sofofr != NULL && spin->si_sofoto != NULL))) {
      spin->si_sugtime = time(NULL);
      sugtime = (int64_t)spin->si_sugtime;
    }

    // Build flat pointer arrays for REP / SAL / REPSAL from-to pairs.
    int rep_count = spin->si_rep.ga_len;
    const uint8_t **rep_from = rep_count > 0
        ? xmalloc((size_t)rep_count * sizeof(uint8_t *)) : NULL;
    const uint8_t **rep_to = rep_count > 0
        ? xmalloc((size_t)rep_count * sizeof(uint8_t *)) : NULL;
    for (int i = 0; i < rep_count; i++) {
      fromto_T *ftp = &((fromto_T *)spin->si_rep.ga_data)[i];
      rep_from[i] = (const uint8_t *)ftp->ft_from;
      rep_to[i]   = (const uint8_t *)ftp->ft_to;
    }

    bool use_sal = !(spin->si_sofofr != NULL && spin->si_sofoto != NULL);
    int sal_count = use_sal ? spin->si_sal.ga_len : 0;
    const uint8_t **sal_from = sal_count > 0
        ? xmalloc((size_t)sal_count * sizeof(uint8_t *)) : NULL;
    const uint8_t **sal_to = sal_count > 0
        ? xmalloc((size_t)sal_count * sizeof(uint8_t *)) : NULL;
    for (int i = 0; i < sal_count; i++) {
      fromto_T *ftp = &((fromto_T *)spin->si_sal.ga_data)[i];
      sal_from[i] = (const uint8_t *)ftp->ft_from;
      sal_to[i]   = (const uint8_t *)ftp->ft_to;
    }
    uint8_t sal_flags = 0;
    if (spin->si_followup) { sal_flags |= (uint8_t)SAL_F0LLOWUP; }
    if (spin->si_collapse) { sal_flags |= (uint8_t)SAL_COLLAPSE; }
    if (spin->si_rem_accents) { sal_flags |= (uint8_t)SAL_REM_ACCENTS; }

    int repsal_count = spin->si_repsal.ga_len;
    const uint8_t **repsal_from = repsal_count > 0
        ? xmalloc((size_t)repsal_count * sizeof(uint8_t *)) : NULL;
    const uint8_t **repsal_to = repsal_count > 0
        ? xmalloc((size_t)repsal_count * sizeof(uint8_t *)) : NULL;
    for (int i = 0; i < repsal_count; i++) {
      fromto_T *ftp = &((fromto_T *)spin->si_repsal.ga_data)[i];
      repsal_from[i] = (const uint8_t *)ftp->ft_from;
      repsal_to[i]   = (const uint8_t *)ftp->ft_to;
    }

    // Build flat pointer arrays for prefcond and comppat strings.
    int prefcond_count = spin->si_prefcond.ga_len;
    const uint8_t **prefcond_strs = prefcond_count > 0
        ? xmalloc((size_t)prefcond_count * sizeof(uint8_t *)) : NULL;
    for (int i = 0; i < prefcond_count; i++) {
      char *p = ((char **)spin->si_prefcond.ga_data)[i];
      prefcond_strs[i] = (const uint8_t *)(p != NULL ? p : "");
    }

    int comppat_count = spin->si_comppat.ga_len;
    const uint8_t **comppat_strs = comppat_count > 0
        ? xmalloc((size_t)comppat_count * sizeof(uint8_t *)) : NULL;
    for (int i = 0; i < comppat_count; i++) {
      char *p = ((char **)spin->si_comppat.ga_data)[i];
      comppat_strs[i] = (const uint8_t *)(p != NULL ? p : "");
    }

    SpellSectionParams params = {
      .si_info            = spin->si_info,
      .si_region_count    = spin->si_region_count,
      .si_region_name     = (const uint8_t *)spin->si_region_name,
      .si_skip_charflags  = false,
      .si_midword         = spin->si_midword,
      .si_prefcond_strs   = prefcond_strs,
      .si_prefcond_count  = prefcond_count,
      .si_rep_from        = rep_from,
      .si_rep_to          = rep_to,
      .si_rep_count       = rep_count,
      .si_use_sal         = use_sal,
      .si_sal_from        = sal_from,
      .si_sal_to          = sal_to,
      .si_sal_count       = sal_count,
      .si_sal_flags       = sal_flags,
      .si_repsal_from     = repsal_from,
      .si_repsal_to       = repsal_to,
      .si_repsal_count    = repsal_count,
      .si_sofofr          = spin->si_sofofr,
      .si_sofoto          = spin->si_sofoto,
      .si_map_data        = (const uint8_t *)spin->si_map.ga_data,
      .si_map_len         = spin->si_map.ga_len,
      .si_sugtime         = sugtime,
      .si_nosplitsugs     = spin->si_nosplitsugs,
      .si_nocompoundsugs  = spin->si_nocompoundsugs,
      .si_compflags       = spin->si_compflags,
      .si_compmax         = spin->si_compmax,
      .si_compminlen      = spin->si_compminlen,
      .si_compsylmax      = spin->si_compsylmax,
      .si_compoptions     = spin->si_compoptions,
      .si_comppat_strs    = comppat_strs,
      .si_comppat_count   = comppat_count,
      .si_nobreak         = spin->si_nobreak,
      .si_syllable        = spin->si_syllable,
    };

    // Allocate output buffer; 256 KB should be more than enough for all sections.
    size_t sec_buf_len = 256 * 1024;
    uint8_t *sec_buf = xmalloc(sec_buf_len);
    size_t written = 0;

    int rs_ret = rs_write_spell_sections(&params, sec_buf, sec_buf_len, &written);
    if (rs_ret != 0 || (written > 0 && fwrite(sec_buf, written, 1, fd) != 1)) {
      retval = FAIL;
    }

    xfree(sec_buf);
    xfree(rep_from);
    xfree(rep_to);
    xfree(sal_from);
    xfree(sal_to);
    xfree(repsal_from);
    xfree(repsal_to);
    xfree(prefcond_strs);
    xfree(comppat_strs);

    if (retval == FAIL) {
      goto theend;
    }
  }

  // SN_WORDS: <word> ...
  // Stays in C: requires hashtable iteration with HASHITEM_EMPTY macro.
  if (spin->si_commonwords.ht_used > 0) {
    putc(SN_WORDS, fd);
    putc(0, fd);

    // round 1: count the bytes; round 2: write the bytes
    for (unsigned round = 1; round <= 2; round++) {
      size_t todo = spin->si_commonwords.ht_used;
      size_t len = 0;
      for (hashitem_T *hi = spin->si_commonwords.ht_array; todo > 0; hi++) {
        if (!HASHITEM_EMPTY(hi)) {
          size_t l = strlen(hi->hi_key) + 1;
          len += l;
          if (round == 2) {
            fwv &= fwrite(hi->hi_key, l, 1, fd);
          }
          todo--;
        }
      }
      if (round == 1) {
        put_bytes(fd, len, 4);
      }
    }
  }

  // end of <SECTIONS>
  putc(SN_END, fd);

  // <LWORDTREE>  <KWORDTREE>  <PREFIXTREE>
  spin->si_memtot = 0;
  for (unsigned round = 1; round <= 3; round++) {
    wordnode_T *tree;
    if (round == 1) {
      tree = spin->si_foldroot->wn_sibling;
    } else if (round == 2) {
      tree = spin->si_keeproot->wn_sibling;
    } else {
      tree = spin->si_prefroot->wn_sibling;
    }
    bool prefixtree = round == 3;

    // Count pass: sets wn_u1.index / wn_u2.wnode, returns nodecount.
    rs_clear_node(tree);
    size_t dummy_written = 0;
    int nodecount = rs_put_node(tree, NULL, 0, 0, regionmask, prefixtree,
                                &dummy_written);
    if (nodecount < 0) {
      retval = FAIL;
      goto theend;
    }

    put_bytes(fd, (uintmax_t)nodecount, 4);
    assert((size_t)nodecount + (size_t)nodecount * sizeof(int) < INT_MAX);
    spin->si_memtot += nodecount + (int)(nodecount * sizeof(int));

    // Write pass: allocate buffer, fill it, write to file.
    size_t tree_buf_len = (size_t)nodecount * 8 + 1024;
    uint8_t *tree_buf = xmalloc(tree_buf_len);
    rs_clear_node(tree);
    size_t tree_written = 0;
    int nodecount2 = rs_put_node(tree, tree_buf, tree_buf_len, 0, regionmask,
                                 prefixtree, &tree_written);
    if (nodecount2 < 0 || (tree_written > 0 && fwrite(tree_buf, tree_written, 1, fd) != 1)) {
      xfree(tree_buf);
      retval = FAIL;
      goto theend;
    }
    xfree(tree_buf);
  }

  // Write another byte to check for errors (file system full).
  if (putc(0, fd) == EOF) {
    retval = FAIL;
  }
theend:
  if (fclose(fd) == EOF) {
    retval = FAIL;
  }

  if (fwv != 1) {
    retval = FAIL;
  }
  if (retval == FAIL) {
    emsg(_(e_write));
  }

  return retval;
}


// ":mkspell [-ascii] outfile  infile ..."
// ":mkspell [-ascii] addfile"
void ex_mkspell(exarg_T *eap)
{
  int fcount;
  char **fnames;
  char *arg = eap->arg;
  bool ascii = false;

  if (strncmp(arg, "-ascii", 6) == 0) {
    ascii = true;
    arg = skipwhite(arg + 6);
  }

  // Expand all the remaining arguments (e.g., $VIMRUNTIME).
  if (get_arglist_exp(arg, &fcount, &fnames, false) != OK) {
    return;
  }

  mkspell(fcount, fnames, ascii, eap->forceit, false);
  FreeWild(fcount, fnames);
}

// Create the .sug file.
// Uses the soundfold info in "spin".
// Writes the file with the name "wfname", with ".spl" changed to ".sug".
// spell_make_sugfile and helpers are now implemented in Rust (spell crate).
extern void spell_make_sugfile(spellinfo_T *spin, char *wfname);

/// Create a Vim spell file from one or more word lists.
/// "fnames[0]" is the output file name.
/// "fnames[fcount - 1]" is the last input file name.
/// Exception: when "fnames[0]" ends in ".add" it's used as the input file name
/// and ".spl" is appended to make the output file name.
///
/// @param ascii  -ascii argument given
/// @param over_write  overwrite existing output file
/// @param added_word  invoked through "zg"
static void mkspell(int fcount, char **fnames, bool ascii, bool over_write, bool added_word)
{
  char *fname = NULL;
  afffile_T *(afile[MAXREGIONS]);
  bool error = false;
  spellinfo_T spin;

  CLEAR_FIELD(spin);
  spin.si_verbose = !added_word;
  spin.si_ascii = ascii;
  spin.si_followup = true;
  spin.si_rem_accents = true;
  ga_init(&spin.si_rep, (int)sizeof(fromto_T), 20);
  ga_init(&spin.si_repsal, (int)sizeof(fromto_T), 20);
  ga_init(&spin.si_sal, (int)sizeof(fromto_T), 20);
  ga_init(&spin.si_map, (int)sizeof(char), 100);
  ga_init(&spin.si_comppat, (int)sizeof(char *), 20);
  ga_init(&spin.si_prefcond, (int)sizeof(char *), 50);
  hash_init(&spin.si_commonwords);
  spin.si_newcompID = 127;      // start compound ID at first maximum

  // default: fnames[0] is output file, following are input files
  // When "fcount" is 1 there is only one file.
  char **innames = &fnames[fcount == 1 ? 0 : 1];
  int incount = fcount - 1;

  char *wfname = xmalloc(MAXPATHL);

  // Use Rust to compute the output .spl filename and detect .add/.ascii flags.
  if (fcount >= 1) {
    const char *enc_str = spin.si_ascii ? "ascii" : spell_enc();
    RsMkspellFnameResult fname_res;
    int fret = rs_mkspell_output_fname((const uint8_t *const *)fnames, fcount,
                                       (const uint8_t *)enc_str, &fname_res);
    if (fret == 0 && fname_res.fname_len > 0) {
      xstrlcpy(wfname, (const char *)fname_res.fname, MAXPATHL);
      if (fname_res.is_ascii) {
        spin.si_ascii = true;
      }
      if (fname_res.is_add) {
        spin.si_add = true;
      }
      // For single .add input file, incount must be 1.
      if (fcount == 1) {
        incount = 1;
      }
    }
  }

  if (incount <= 0) {
    emsg(_(e_invarg));          // need at least output and input names
  } else if (vim_strchr(path_tail(wfname), '_') != NULL) {
    emsg(_("E751: Output file name must not have region name"));
  } else if (incount > MAXREGIONS) {
    semsg(_("E754: Only up to %d regions supported"), MAXREGIONS);
  } else {
    // Check for overwriting before doing things that may take a lot of
    // time.
    if (!over_write && os_path_exists(wfname)) {
      emsg(_(e_exists));
      goto theend;
    }
    if (os_isdir(wfname)) {
      semsg(_(e_isadir2), wfname);
      goto theend;
    }

    fname = xmalloc(MAXPATHL);

    // Use Rust to validate input filenames and extract region names for
    // multi-region builds.
    if (incount > 1) {
      int vret = rs_mkspell_validate_args((const uint8_t *const *)innames,
                                          incount,
                                          (uint8_t *)spin.si_region_name);
      if (vret == 1) {
        // Find the offending filename and report it.
        for (int i = 0; i < incount; i++) {
          int len = (int)strlen(innames[i]);
          if (strlen(path_tail(innames[i])) < 5 || innames[i][len - 3] != '_') {
            semsg(_("E755: Invalid region in %s"), innames[i]);
            goto theend;
          }
        }
        goto theend;
      }
    }

    // Init the aff and dic pointers.
    for (int i = 0; i < incount; i++) {
      afile[i] = NULL;
    }
    spin.si_region_count = incount;

    spin.si_foldroot = wordtree_alloc(&spin);
    spin.si_keeproot = wordtree_alloc(&spin);
    spin.si_prefroot = wordtree_alloc(&spin);

    // When not producing a .add.spl file clear the character table when
    // we encounter one in the .aff file.  This means we dump the current
    // one in the .spl file if the .aff file doesn't define one.  That's
    // better than guessing the contents, the table will match a
    // previously loaded spell file.
    if (!spin.si_add) {
      spin.si_clear_chartab = true;
    }

    // Read all the .aff and .dic files.
    // Text is converted to 'encoding'.
    // Words are stored in the case-folded and keep-case trees.
    for (int i = 0; i < incount && !error; i++) {
      spin.si_conv.vc_type = CONV_NONE;
      spin.si_region = 1 << i;

      vim_snprintf(fname, MAXPATHL, "%s.aff", innames[i]);
      if (os_path_exists(fname)) {
        // Read the .aff file.  Will init "spin->si_conv" based on the
        // "SET" line.
        afile[i] = spell_read_aff(&spin, fname);
        if (afile[i] == NULL) {
          error = true;
        } else {
          // Read the .dic file and store the words in the trees.
          vim_snprintf(fname, MAXPATHL, "%s.dic", innames[i]);
          if (spell_read_dic(&spin, fname, afile[i]) == FAIL) {
            error = true;
          }
        }
      } else {
        // No .aff file, try reading the file as a word list.  Store
        // the words in the trees.
        if (spell_read_wordfile(&spin, innames[i]) == FAIL) {
          error = true;
        }
      }

      // Free any conversion stuff.
      convert_setup(&spin.si_conv, NULL, NULL);
    }

    if (spin.si_compflags != NULL && spin.si_nobreak) {
      msg(_("Warning: both compounding and NOBREAK specified"), 0);
    }

    if (!error && !got_int) {
      // Combine tails in the tree.
      spell_message(&spin, _(msg_compressing));
      wordtree_compress(&spin, spin.si_foldroot, "case-folded");
      wordtree_compress(&spin, spin.si_keeproot, "keep-case");
      wordtree_compress(&spin, spin.si_prefroot, "prefixes");
    }

    if (!error && !got_int) {
      // Write the info in the spell file.
      vim_snprintf(IObuff, IOSIZE,
                   _("Writing spell file %s..."), wfname);
      spell_message(&spin, IObuff);

      error = write_vim_spell(&spin, wfname) == FAIL;

      spell_message(&spin, _("Done!"));
      vim_snprintf(IObuff, IOSIZE,
                   _("Estimated runtime memory use: %d bytes"), spin.si_memtot);
      spell_message(&spin, IObuff);

      // If the file is loaded need to reload it.
      if (!error) {
        spell_reload_one(wfname, added_word);
      }
    }

    // Free the allocated memory.
    ga_clear(&spin.si_rep);
    ga_clear(&spin.si_repsal);
    ga_clear(&spin.si_sal);
    ga_clear(&spin.si_map);
    ga_clear(&spin.si_comppat);
    ga_clear(&spin.si_prefcond);
    hash_clear_all(&spin.si_commonwords, 0);

    // Free the .aff file structures.
    for (int i = 0; i < incount; i++) {
      if (afile[i] != NULL) {
        spell_free_aff(afile[i]);
      }
    }

    // Free all the bits and pieces at once.
    free_blocks(spin.si_blocks);

    // If there is soundfolding info and no NOSUGFILE item create the
    // .sug file with the soundfolded word trie.
    if (spin.si_sugtime != 0 && !error && !got_int) {
      spell_make_sugfile(&spin, wfname);
    }
  }

theend:
  xfree(fname);
  xfree(wfname);
}

// ":[count]spellgood  {word}"
// ":[count]spellwrong {word}"
// ":[count]spellundo  {word}"
// ":[count]spellrare  {word}"
void ex_spell(exarg_T *eap)
{
  spell_add_word(eap->arg, (int)strlen(eap->arg),
                 eap->cmdidx == CMD_spellwrong
                 ? SPELL_ADD_BAD
                 : eap->cmdidx == CMD_spellrare ? SPELL_ADD_RARE : SPELL_ADD_GOOD,
                 eap->forceit ? 0 : (int)eap->line2,
                 eap->cmdidx == CMD_spellundo);
}

/// Add "word[len]" to 'spellfile' as a good or bad word.
///
/// @param what  SPELL_ADD_ values
/// @param idx  "zG" and "zW": zero, otherwise index in 'spellfile'
/// @param bool  // true for "zug", "zuG", "zuw" and "zuW"
void spell_add_word(char *word, int len, SpellAddType what, int idx, bool undo)
{
  FILE *fd = NULL;
  buf_T *buf = NULL;
  bool new_spf = false;
  bool file_written = false;  // true when the .add file was modified
  char *fname;
  char *fnamebuf = NULL;
  char *spf;

  if (!valid_spell_word(word, word + len)) {
    emsg(_(e_illegal_character_in_word));
    return;
  }

  if (idx == 0) {           // use internal wordlist
    if (int_wordlist == NULL) {
      int_wordlist = vim_tempname();
      if (int_wordlist == NULL) {
        return;
      }
    }
    fname = int_wordlist;
  } else {
    int i;
    // If 'spellfile' isn't set figure out a good default value.
    if (*curwin->w_s->b_p_spf == NUL) {
      init_spellfile();
      new_spf = true;
    }

    if (*curwin->w_s->b_p_spf == NUL) {
      semsg(_(e_notset), "spellfile");
      return;
    }
    fnamebuf = xmalloc(MAXPATHL);

    for (spf = curwin->w_s->b_p_spf, i = 1; *spf != NUL; i++) {
      copy_option_part(&spf, fnamebuf, MAXPATHL, ",");
      if (i == idx) {
        break;
      }
      if (*spf == NUL) {
        semsg(_("E765: 'spellfile' does not have %" PRId64 " entries"), (int64_t)idx);
        xfree(fnamebuf);
        return;
      }
    }

    // Check that the user isn't editing the .add file somewhere.
    buf = buflist_findname_exp(fnamebuf);
    if (buf != NULL && buf->b_ml.ml_mfp == NULL) {
      buf = NULL;
    }
    if (buf != NULL && bufIsChanged(buf)) {
      emsg(_(e_bufloaded));
      xfree(fnamebuf);
      return;
    }

    fname = fnamebuf;
  }

  if (what == SPELL_ADD_BAD || undo) {
    // When the word appears as good word we need to remove that one,
    // since its flags sort before the one with WF_BANNED.
    // Read the whole file into a buffer so Rust can find duplicates.
    fd = os_fopen(fname, "r");
    if (fd != NULL) {
      // Measure file size.
      fseek(fd, 0, SEEK_END);
      long fsize = ftell(fd);
      rewind(fd);
      if (fsize > 0) {
        uint8_t *fbuf = xmalloc((size_t)fsize);
        size_t nread = fread(fbuf, 1, (size_t)fsize, fd);
        fclose(fd);
        fd = NULL;

        // Scan for all matching lines and comment them out.
        size_t scan_offset = 0;
        while (scan_offset < nread) {
          size_t match_offset = 0;
          if (!rs_spell_find_duplicate_word(fbuf + scan_offset,
                                            nread - scan_offset,
                                            (const uint8_t *)word, (size_t)len,
                                            &match_offset)) {
            break;
          }
          size_t abs_offset = scan_offset + match_offset;

          // Comment out the line by writing '#' at abs_offset.
          fd = os_fopen(fname, "r+");
          if (fd != NULL) {
            if (fseek(fd, (long)abs_offset, SEEK_SET) == 0) {
              fputc('#', fd);
              file_written = true;
              if (undo) {
                home_replace(NULL, fname, NameBuff, MAXPATHL, true);
                smsg(0, _("Word '%.*s' removed from %s"), len, word, NameBuff);
              }
            }
            fclose(fd);
            fd = NULL;
          }

          // Advance past this line.
          size_t next = abs_offset;
          while (next < nread && fbuf[next] != '\n') {
            next++;
          }
          scan_offset = next + 1;
        }
        xfree(fbuf);
      } else {
        fclose(fd);
        fd = NULL;
      }
    }
  }

  if (!undo) {
    fd = os_fopen(fname, "a");
    if (fd == NULL && new_spf) {
      char *p;

      // We just initialized the 'spellfile' option and can't open the
      // file.  We may need to create the "spell" directory first.  We
      // already checked the runtime directory is writable in
      // init_spellfile().
      if (!dir_of_file_exists(fname)
          && (p = path_tail_with_sep(fname)) != fname) {
        char c = *p;

        // The directory doesn't exist.  Try creating it and opening
        // the file again.
        *p = NUL;
        os_mkdir(fname, 0755);
        *p = c;
        fd = os_fopen(fname, "a");
      }
    }

    if (fd == NULL) {
      semsg(_(e_notopen), fname);
    } else {
      // Use Rust to format the line to append.
      uint8_t append_buf[MAXWLEN * 2 + 4];
      int append_len = rs_spell_add_word_format((const uint8_t *)word, (size_t)len,
                                                (int)what,
                                                append_buf, sizeof(append_buf));
      if (append_len > 0) {
        fwrite(append_buf, 1, (size_t)append_len, fd);
        file_written = true;
      }
      fclose(fd);
      fd = NULL;

      home_replace(NULL, fname, NameBuff, MAXPATHL, true);
      smsg(0, _("Word '%.*s' added to %s"), len, word, NameBuff);
    }
  }

  if (file_written) {
    // Update the .add.spl file.
    mkspell(1, &fname, false, true, true);

    // If the .add file is edited somewhere, reload it.
    if (buf != NULL) {
      buf_reload(buf, buf->b_orig_mode, false);
    }

    redraw_all_later(UPD_SOME_VALID);
  }
  xfree(fnamebuf);
}

// Initialize 'spellfile' for the current buffer.
//
// If the location does not exist, create it. Defaults to
// stdpath("data") + "/site/spell/{spelllang}.{encoding}.add".
static void init_spellfile(void)
{
  char *lend;
  bool aspath = false;
  char *lstart = curbuf->b_s.b_p_spl;

  if (*curwin->w_s->b_p_spl == NUL || GA_EMPTY(&curwin->w_s->b_langp)) {
    return;
  }

  // Find the end of the language name.  Exclude the region.  If there
  // is a path separator remember the start of the tail.
  for (lend = curwin->w_s->b_p_spl; *lend != NUL
       && vim_strchr(",._", (uint8_t)(*lend)) == NULL; lend++) {
    if (vim_ispathsep(*lend)) {
      aspath = true;
      lstart = lend + 1;
    }
  }

  char *buf = xmalloc(MAXPATHL);
  size_t buf_len = MAXPATHL;

  if (!aspath) {
    char *xdg_path = get_xdg_home(kXDGDataHome);
    xstrlcpy(buf, xdg_path, buf_len);
    xfree(xdg_path);

    xstrlcat(buf, "/site/spell", buf_len);

    char *failed_dir;
    if (os_mkdir_recurse(buf, 0755, &failed_dir, NULL) != 0) {
      xfree(buf);
      xfree(failed_dir);
      return;
    }
  } else {
    if ((size_t)(lend - curbuf->b_s.b_p_spl) >= buf_len) {
      xfree(buf);
      return;
    }
    xmemcpyz(buf, curbuf->b_s.b_p_spl, (size_t)(lend - curbuf->b_s.b_p_spl));
  }

  // Append spelllang
  vim_snprintf(buf + strlen(buf), buf_len - strlen(buf), "/%.*s", (int)(lend - lstart), lstart);

  // Append ".ascii.add" or ".{enc}.add"
  char *fname = LANGP_ENTRY(curwin->w_s->b_langp, 0)->lp_slang->sl_fname;
  const char *enc_suffix =
    (fname != NULL && strstr(path_tail(fname), ".ascii.") != NULL) ? "ascii" : spell_enc();
  vim_snprintf(buf + strlen(buf), buf_len - strlen(buf), ".%s.add", enc_suffix);

  set_option_value_give_err(kOptSpellfile, CSTR_AS_OPTVAL(buf), OPT_LOCAL);
  xfree(buf);
}

/// Set the spell character tables from strings in the .spl file.
///
/// @param cnt  length of "flags"

