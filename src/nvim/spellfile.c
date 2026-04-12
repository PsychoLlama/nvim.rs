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

// Rust FFI declarations

// Dictionary and wordfile line parsers

extern int rs_spell_add_word_format(const uint8_t *word, size_t word_len,
                                    int what, uint8_t *buf_out, size_t buf_cap);

extern bool rs_spell_find_duplicate_word(const uint8_t *file_content, size_t content_len,
                                         const uint8_t *word, size_t word_len,
                                         size_t *offset_out);


static const char *e_illegal_character_in_word = N_("E1280: Illegal character in word");

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

extern bool valid_spell_word(const char *word, const char *end);
#include "spellfile.c.generated.h"

// ex_mkspell, ex_spell, write_vim_spell, and mkspell are now implemented in
// Rust (commands.rs / spellfile.rs) and exported directly with their C names.
// spell_add_word still calls mkspell via rs_mkspell until Phase 3.
extern void rs_mkspell(int fcount, char **fnames, bool ascii, bool over_write, bool added_word);

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
    rs_mkspell(1, &fname, false, true, true);

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
int nvim_spell_toupper(int c) { return SPELL_TOUPPER(c); }

/// Set the spell character tables from strings in the .spl file.
///
/// @param cnt  length of "flags"

