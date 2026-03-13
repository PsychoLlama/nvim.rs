#pragma once

#include <stdbool.h>

#include "nvim/ex_cmds_defs.h"  // IWYU pragma: keep
#include "nvim/highlight_defs.h"  // IWYU pragma: keep
#include "nvim/spell_defs.h"  // IWYU pragma: keep
#include "nvim/vim_defs.h"  // IWYU pragma: keep

/// First language that is loaded, start of the linked list of loaded languages.
extern slang_T *first_lang;

/// file used for "zG" and "zW"
extern char *int_wordlist;

extern spelltab_T spelltab;
extern bool did_set_spelltab;

extern char *e_format;

// Remember what "z?" replaced.
extern char *repl_from;
extern char *repl_to;

/// Values for behaviour in spell_move_to
typedef enum {
  SMT_ALL = 0,  ///< Move to "all" words
  SMT_BAD,      ///< Move to "bad" words only
  SMT_RARE,     ///< Move to "rare" words only
} smt_T;

// Functions implemented in Rust (nvim-spell crate) and exported via #[export_name].
size_t spell_check(win_T *wp, char *ptr, hlf_T *attrp, int *capcol, bool docount);
bool spell_valid_case(int wordflags, int treeflags);
bool byte_in_str(const uint8_t *str, int n);
void clear_spell_chartab(spelltab_T *sp);
bool valid_spelllang(const char *val);
bool valid_spellfile(const char *val);
bool spell_iswordp(const char *p, const win_T *wp);
bool spell_iswordp_nmw(const char *p, win_T *wp);
int captype(const char *word, const char *end);
void onecap_copy(const char *word, char *wcopy, bool upper);
void allcap_copy(const char *word, char *wcopy);
int nofold_len(char *fword, int flen, char *word);
void make_case_word(char *fword, char *cword, int flags);
void spell_cat_line(char *buf, char *line, int maxlen);
void spell_soundfold(slang_T *slang, char *inword, bool folded, char *res);
char *eval_soundfold(const char *word);
void init_spell_chartab(void);
bool spell_check_window(win_T *wp);
bool no_spell_checking(win_T *wp);
char *spell_enc(void);
int spell_casefold(const win_T *wp, const char *str, int len, char *buf, int buflen);
bool match_checkcompoundpattern(char *ptr, int wlen, garray_T *gap);
bool can_compound(slang_T *slang, const char *word, const uint8_t *flags);
bool match_compoundrule(slang_T *slang, const uint8_t *compflags);
int valid_word_prefix(int totprefcnt, int arridx, int flags, char *word, slang_T *slang, bool cond_req);
char *spell_to_word_end(char *start, win_T *win);
int spell_word_start(int startcol);
void spell_expand_check_cap(colnr_T col);
int expand_spelling(linenr_T lnum, char *pat, char ***matchp);
const char *did_set_spell_option(void);
const char *compile_cap_prog(synblock_T *synblock);

#include "spell.h.generated.h"
