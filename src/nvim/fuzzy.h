#pragma once

#include <limits.h>
#include <stdint.h>  // IWYU pragma: keep

#include "nvim/buffer_defs.h"  // IWYU pragma: keep
#include "nvim/eval/typval_defs.h"  // IWYU pragma: keep
#include "nvim/garray_defs.h"  // IWYU pragma: keep
#include "nvim/pos_defs.h"  // IWYU pragma: keep
#include "nvim/types_defs.h"  // IWYU pragma: keep

enum { FUZZY_MATCH_MAX_LEN = 1024, };  ///< max characters that can be matched
enum { FUZZY_SCORE_NONE = INT_MIN, };  ///< invalid fuzzy score

/// Fuzzy matched string list item. Used for fuzzy match completion. Items are
/// usually sorted by "score". The "idx" member is used for stable-sort.
typedef struct {
  int idx;
  char *str;
  int score;
} fuzmatch_str_T;

// Layout assertions for Rust repr(C) structs (validated at build time)
#include <assert.h>
#include <stddef.h>
_Static_assert(sizeof(pos_T) == 12, "pos_T size must match Rust PosT");
_Static_assert(sizeof(garray_T) == 24, "garray_T size must match Rust GArray");
_Static_assert(sizeof(fuzmatch_str_T) == 24, "fuzmatch_str_T size must match Rust FuzmatchStr");
_Static_assert(offsetof(fuzmatch_str_T, idx) == 0, "fuzmatch_str_T.idx offset");
_Static_assert(offsetof(fuzmatch_str_T, str) == 8, "fuzmatch_str_T.str offset");
_Static_assert(offsetof(fuzmatch_str_T, score) == 16, "fuzmatch_str_T.score offset");

// The following functions are implemented in Rust and exported under their original names.
extern bool fuzzy_match(char *str, const char *pat, bool matchseq, int *outScore,
                        uint32_t *matches, int maxMatches);
extern int fuzzy_match_str(char *str, const char *pat);
extern bool fuzzy_match_str_in_line(char **ptr, char *pat, int *len, pos_T *current_pos,
                                    int *score);
extern bool search_for_fuzzy_match(buf_T *buf, pos_T *pos, char *pattern, int dir,
                                   pos_T *start_pos, int *len, char **ptr, int *score);
extern void fuzmatch_str_free(fuzmatch_str_T *fuzmatch, int count);
extern void fuzzymatches_to_strmatches(fuzmatch_str_T *fuzmatch, char ***matches, int count,
                                       bool funcsort);
extern garray_T *fuzzy_match_str_with_pos(const char *str, const char *pat);

// VimL builtin entry points: implemented in Rust (src/nvim-rs/fuzzy/src/lib.rs).
extern void f_matchfuzzy(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
extern void f_matchfuzzypos(typval_T *argvars, typval_T *rettv, EvalFuncData fptr);
