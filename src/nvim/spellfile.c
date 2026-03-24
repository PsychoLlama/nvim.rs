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

// Dictionary and wordfile line parsers

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

// mkspell argument parsing helpers

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

// Rust utility replacements
extern int rs_set_spell_charflags(const uint8_t *flags_in, int cnt, const char *fol);
extern int *rs_mb_str2wide(const char *s);
extern void rs_tree_count_words(const uint8_t *byts, int *idxs, int len);
extern void rs_set_sal_first(slang_T *slang);
extern void rs_set_map_str(slang_T *slang, const char *map);
extern int rs_set_sofo(slang_T *slang, const char *from, const char *to);
extern int rs_read_compound(const uint8_t *buf, size_t len, slang_T *slang);
extern int rs_read_sal_section(const uint8_t *buf, size_t len, slang_T *slang);
extern int rs_read_prefcond_section(FILE *fd, slang_T *lp);
extern int rs_read_rep_section(FILE *fd, garray_T *gap, int16_t *first);

// SpellSectionParams is filled from spellinfo_T and passed to rs_write_spell_sections().
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

// rs_put_node writes tree to buffer and returns the nodecount;
// rs_clear_node resets index/wnode fields.
extern int rs_put_node(wordnode_T *node, uint8_t *buf, size_t buf_len,
                       int idx, int regionmask, bool prefixtree, size_t *written_out);
extern void rs_clear_node(wordnode_T *node);

// rs_node_compress compresses a sibling list (first sibling of root->wn_sibling).
// Returns compressed node count; sets *tot_out to total nodes visited.
extern int rs_node_compress(spellinfo_T *spin, wordnode_T *node, int *tot_out);

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

// Phase 1 (type-independent functions): small pure helpers -- now in Rust.
extern unsigned rs_get_affitem(int flagtype, char **pp);
extern unsigned rs_affitem2flag(int flagtype, char *item, const char *fname, int lnum);
extern bool rs_flag_in_afflist(int flagtype, char *afflist, unsigned flag);
extern void rs_aff_check_number(int spinval, int affval, const char *name);
extern void rs_aff_check_string(const char *spinval, const char *affval, const char *name);
extern void rs_check_renumber(spellinfo_T *spin);
#define get_affitem(ft, pp)          rs_get_affitem(ft, pp)
#define affitem2flag(ft, it, fn, ln) rs_affitem2flag(ft, it, fn, ln)
#define flag_in_afflist(ft, al, f)   rs_flag_in_afflist(ft, al, f)
#define aff_check_number(sv, av, n)  rs_aff_check_number(sv, av, n)
#define aff_check_string(sv, av, n)  rs_aff_check_string(sv, av, n)
#define check_renumber(s)            rs_check_renumber(s)

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

// Phase 1 (afffile_T/affentry_T-dependent functions): now in Rust.
extern int rs_get_affix_flags(afffile_T *affile, char *afflist);
extern int rs_get_pfxlist(afffile_T *affile, char *afflist, char *store_afflist);
extern void rs_get_compflags(afffile_T *affile, char *afflist, char *store_afflist);
extern void rs_aff_process_flags(afffile_T *affile, affentry_T *entry);
#define get_affix_flags(af, al)      rs_get_affix_flags(af, al)
#define get_pfxlist(af, al, sa)      rs_get_pfxlist(af, al, sa)
#define get_compflags(af, al, sa)    rs_get_compflags(af, al, sa)
#define aff_process_flags(af, e)     rs_aff_process_flags(af, e)

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

// Phase 2: Arena allocator and memory management -- now in Rust.
extern void *rs_getroom(spellinfo_T *spin, size_t len, bool align);
extern char *rs_getroom_save(spellinfo_T *spin, char *s);
extern void rs_free_blocks(sblock_T *bl);
extern void rs_add_fromto(spellinfo_T *spin, garray_T *gap, const char *from, const char *to);
#define getroom_save(sp, s)          rs_getroom_save(sp, s)
#define add_fromto(sp, g, f, t)      rs_add_fromto(sp, g, f, t)

// Phase 3: process_compflags, spell_free_aff, store_aff_word -- now in Rust.
extern void rs_process_compflags(spellinfo_T *spin, afffile_T *aff, char *compflags);
extern void rs_spell_free_aff(afffile_T *aff);
extern int rs_store_aff_word(spellinfo_T *spin, char *word, char *afflist, afffile_T *affile,
                             hashtab_T *ht, hashtab_T *xht, int condit, int flags,
                             char *pfxlist, int pfxlen);
#define process_compflags(sp, a, c)  rs_process_compflags(sp, a, c)
#define spell_free_aff(a)            rs_spell_free_aff(a)
#define store_aff_word(sp, w, al, af, ht, xht, c, f, pl, plen) \
    rs_store_aff_word(sp, w, al, af, ht, xht, c, f, pl, plen)

// Phase 4: spell_read_dic, spell_read_wordfile, spell_read_aff -- now in Rust.
extern int rs_spell_read_dic(spellinfo_T *spin, char *fname, afffile_T *affile);
extern int rs_spell_read_wordfile(spellinfo_T *spin, char *fname);
extern afffile_T *rs_spell_read_aff(spellinfo_T *spin, char *fname);
#define spell_read_dic(sp, f, af)    rs_spell_read_dic(sp, f, af)
#define spell_read_wordfile(sp, f)   rs_spell_read_wordfile(sp, f)
#define spell_read_aff(sp, f)        rs_spell_read_aff(sp, f)

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

// spell_read_aff: migrated to Rust (rs_spell_read_aff). Redirected via #define above.

// aff_process_flags, affitem2flag, get_affitem: migrated to Rust (rs_aff_process_flags,
// rs_affitem2flag, rs_get_affitem). Redirected via #define above.

// process_compflags: migrated to Rust (rs_process_compflags). Redirected via #define above.
// check_renumber, flag_in_afflist, aff_check_number, aff_check_string:
// migrated to Rust. Redirected via #define above.
// add_fromto: migrated to Rust (rs_add_fromto). Redirected via #define above.

// spell_free_aff: migrated to Rust (rs_spell_free_aff). Redirected via #define above.

// spell_read_dic: migrated to Rust (rs_spell_read_dic). Redirected via #define above.

// get_affix_flags, get_pfxlist, get_compflags: migrated to Rust.
// Redirected via #define above.

// store_aff_word: migrated to Rust (rs_store_aff_word). Redirected via #define above.

// spell_read_wordfile: migrated to Rust (rs_spell_read_wordfile). Redirected via #define above.

/// Thin wrapper: arena allocator delegated to Rust rs_getroom.
void *getroom(spellinfo_T *spin, size_t len, bool align)
  FUNC_ATTR_NONNULL_RET
{
  return rs_getroom(spin, len, align);
}

// getroom_save: migrated to Rust (rs_getroom_save). Redirected via #define above.

/// Thin wrapper: free sblock_T list delegated to Rust rs_free_blocks.
void free_blocks(sblock_T *bl)
{
  rs_free_blocks(bl);
}

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

/// C wrapper for SPELL_TOUPPER macro (used by Rust via FFI).
int nvim_spell_toupper(int c)
{
  return SPELL_TOUPPER(c);
}

/// Set the spell character tables from strings in the .spl file.
///
/// @param cnt  length of "flags"

