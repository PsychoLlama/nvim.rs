/// Static assertions validating that Rust's SynBlockStruct and SynStateStruct
/// match C's synblock_T and synstate_T layouts.
///
/// These compile-time checks ensure every field accessed directly from Rust
/// has the correct offset and the overall struct sizes match.

#include <stddef.h>
#include "nvim/buffer_defs.h"
#include "nvim/syntax_defs.h"

// =============================================================================
// synblock_T (1168 bytes)
// =============================================================================

_Static_assert(sizeof(synblock_T) == 1168,
               "synblock_T size mismatch: update SynBlockStruct in synblock_struct.rs");

_Static_assert(offsetof(synblock_T, b_keywtab) == 0, "b_keywtab offset mismatch");
_Static_assert(offsetof(synblock_T, b_keywtab_ic) == 296, "b_keywtab_ic offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_error) == 592, "b_syn_error offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_slow) == 593, "b_syn_slow offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_ic) == 596, "b_syn_ic offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_foldlevel) == 600, "b_syn_foldlevel offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_spell) == 604, "b_syn_spell offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_patterns) == 608, "b_syn_patterns offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_clusters) == 632, "b_syn_clusters offset mismatch");
_Static_assert(offsetof(synblock_T, b_spell_cluster_id) == 656, "b_spell_cluster_id offset mismatch");
_Static_assert(offsetof(synblock_T, b_nospell_cluster_id) == 660, "b_nospell_cluster_id offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_containedin) == 664, "b_syn_containedin offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_sync_flags) == 668, "b_syn_sync_flags offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_sync_id) == 672, "b_syn_sync_id offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_sync_minlines) == 676, "b_syn_sync_minlines offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_sync_maxlines) == 680, "b_syn_sync_maxlines offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_sync_linebreaks) == 684, "b_syn_sync_linebreaks offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_linecont_pat) == 688, "b_syn_linecont_pat offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_linecont_prog) == 696, "b_syn_linecont_prog offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_linecont_time) == 704, "b_syn_linecont_time offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_linecont_ic) == 728, "b_syn_linecont_ic offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_topgrp) == 732, "b_syn_topgrp offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_conceal) == 736, "b_syn_conceal offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_folditems) == 740, "b_syn_folditems offset mismatch");
_Static_assert(offsetof(synblock_T, b_sst_array) == 744, "b_sst_array offset mismatch");
_Static_assert(offsetof(synblock_T, b_sst_len) == 752, "b_sst_len offset mismatch");
_Static_assert(offsetof(synblock_T, b_sst_first) == 760, "b_sst_first offset mismatch");
_Static_assert(offsetof(synblock_T, b_sst_firstfree) == 768, "b_sst_firstfree offset mismatch");
_Static_assert(offsetof(synblock_T, b_sst_freecount) == 776, "b_sst_freecount offset mismatch");
_Static_assert(offsetof(synblock_T, b_sst_check_lnum) == 780, "b_sst_check_lnum offset mismatch");
_Static_assert(offsetof(synblock_T, b_sst_lasttick) == 784, "b_sst_lasttick offset mismatch");
_Static_assert(offsetof(synblock_T, b_langp) == 792, "b_langp offset mismatch");
_Static_assert(offsetof(synblock_T, b_spell_ismw) == 816, "b_spell_ismw offset mismatch");
_Static_assert(offsetof(synblock_T, b_spell_ismw_mb) == 1072, "b_spell_ismw_mb offset mismatch");
_Static_assert(offsetof(synblock_T, b_p_spc) == 1080, "b_p_spc offset mismatch");
_Static_assert(offsetof(synblock_T, b_cap_prog) == 1088, "b_cap_prog offset mismatch");
_Static_assert(offsetof(synblock_T, b_p_spf) == 1096, "b_p_spf offset mismatch");
_Static_assert(offsetof(synblock_T, b_p_spl) == 1104, "b_p_spl offset mismatch");
_Static_assert(offsetof(synblock_T, b_p_spo) == 1112, "b_p_spo offset mismatch");
_Static_assert(offsetof(synblock_T, b_p_spo_flags) == 1120, "b_p_spo_flags offset mismatch");
_Static_assert(offsetof(synblock_T, b_cjk) == 1124, "b_cjk offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_chartab) == 1128, "b_syn_chartab offset mismatch");
_Static_assert(offsetof(synblock_T, b_syn_isk) == 1160, "b_syn_isk offset mismatch");

// =============================================================================
// synstate_T (216 bytes)
// =============================================================================

_Static_assert(sizeof(synstate_T) == 216,
               "synstate_T size mismatch: update SynStateStruct in synstate_struct.rs");

_Static_assert(offsetof(synstate_T, sst_next) == 0, "sst_next offset mismatch");
_Static_assert(offsetof(synstate_T, sst_lnum) == 8, "sst_lnum offset mismatch");
_Static_assert(offsetof(synstate_T, sst_union) == 16, "sst_union offset mismatch");
_Static_assert(offsetof(synstate_T, sst_next_flags) == 184, "sst_next_flags offset mismatch");
_Static_assert(offsetof(synstate_T, sst_stacksize) == 188, "sst_stacksize offset mismatch");
_Static_assert(offsetof(synstate_T, sst_next_list) == 192, "sst_next_list offset mismatch");
_Static_assert(offsetof(synstate_T, sst_tick) == 200, "sst_tick offset mismatch");
_Static_assert(offsetof(synstate_T, sst_change_lnum) == 208, "sst_change_lnum offset mismatch");
