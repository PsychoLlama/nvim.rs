// Tree data structure for storing marks at (row, col) positions and updating
// them to arbitrary text changes. Derivative work of kbtree in klib, whose
// copyright notice is reproduced below. Also inspired by the design of the
// marker tree data structure of the Atom editor, regarding efficient updates
// to text changes.
//
// Marks are inserted using marktree_put. Text changes are processed using
// marktree_splice. All read and delete operations use the iterator.
// use marktree_itr_get to put an iterator at a given position or
// marktree_lookup to lookup a mark by its id (iterator optional in this case).
// Use marktree_itr_current and marktree_itr_next/prev to read marks in a loop.
// marktree_del_itr deletes the current mark of the iterator and implicitly
// moves the iterator to the next mark.

// Copyright notice for kbtree (included in heavily modified form):
//
// Copyright 1997-1999, 2001, John-Mark Gurney.
//           2008-2009, Attractive Chaos <attractor@live.co.uk>
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.
//
// Changes done by by the neovim project follow the Apache v2 license available
// at the repo root.

#include <assert.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <uv.h>

#include "klib/kvec.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/marktree.h"
#include "nvim/memory.h"
#include "nvim/pos_defs.h"
// only for debug functions
#include "nvim/api/private/defs.h"
#include "nvim/api/private/helpers.h"
#include "nvim/garray.h"
#include "nvim/garray_defs.h"

// Rust marktree FFI declarations
// Position comparison and manipulation functions
extern bool rs_pos_leq(MTPos a, MTPos b);
extern bool rs_pos_less(MTPos a, MTPos b);
extern void rs_relative(MTPos base, MTPos *val);
extern void rs_unrelative(MTPos base, MTPos *val);
extern void rs_compose(MTPos *base, MTPos val);

// Mark key property functions
extern bool rs_mt_paired(MTKey key);
extern bool rs_mt_end(MTKey key);
extern bool rs_mt_start(MTKey key);
extern bool rs_mt_right(MTKey key);
extern bool rs_mt_no_undo(MTKey key);
extern bool rs_mt_invalidate(MTKey key);
extern bool rs_mt_invalid(MTKey key);
extern bool rs_mt_decor_any(MTKey key);
extern bool rs_mt_decor_sign(MTKey key);
extern bool rs_mt_conceal_lines(MTKey key);

// Lookup ID functions
extern uint64_t rs_mt_lookup_id(uint32_t ns, uint32_t id, bool end);
extern uint64_t rs_mt_lookup_key_side(MTKey key, bool end);
extern uint64_t rs_mt_lookup_key(MTKey key);

// Key comparison and flag functions
extern int rs_key_cmp(MTKey a, MTKey b);
extern uint16_t rs_mt_flags(bool right_gravity, bool no_undo, bool invalidate, bool decor_ext);

// Iterator functions
extern bool rs_marktree_itr_valid(MarkTreeIter *itr);
extern MTPos rs_marktree_itr_pos(MarkTreeIter *itr);
extern MTKey rs_marktree_itr_current(MarkTreeIter *itr);
extern bool rs_marktree_itr_node_done(MarkTreeIter *itr);
extern bool rs_marktree_itr_next(MarkTree *b, MarkTreeIter *itr);
extern bool rs_marktree_itr_prev(MarkTree *b, MarkTreeIter *itr);
extern bool rs_marktree_itr_first(MarkTree *b, MarkTreeIter *itr);
extern bool rs_marktree_itr_last(MarkTree *b, MarkTreeIter *itr);
extern bool rs_marktree_itr_get(MarkTree *b, int32_t row, int col, MarkTreeIter *itr);

// Lookup and pair functions
extern MTKey rs_marktree_lookup(MarkTree *b, uint64_t id, MarkTreeIter *itr);
extern MTKey rs_marktree_lookup_ns(MarkTree *b, uint32_t ns, uint32_t id, bool end,
                                   MarkTreeIter *itr);
extern MTKey rs_marktree_get_alt(MarkTree *b, MTKey mark, MarkTreeIter *itr);
extern MTPos rs_marktree_get_altpos(MarkTree *b, MTKey mark, MarkTreeIter *itr);
extern MTKey rs_marktree_itr_set_node(MarkTree *b, MarkTreeIter *itr, MTNode *n, int i);
extern void rs_marktree_itr_fix_pos(MarkTree *b, MarkTreeIter *itr);

// Extended iterator functions
extern bool rs_marktree_itr_get_ext_full(MarkTree *b, MTPos p, MarkTreeIter *itr, bool last,
                                         bool gravity, MTPos *oldbase,
                                         MetaFilter meta_filter);
extern bool rs_marktree_itr_next_skip(MarkTree *b, MarkTreeIter *itr, bool skip, bool preload,
                                      MTPos *oldbase, MetaFilter meta_filter);
extern bool rs_marktree_itr_check_filter(MarkTree *b, MarkTreeIter *itr, int stop_row,
                                         int stop_col, MetaFilter meta_filter);

// Filter functions
extern bool rs_marktree_itr_get_filter(MarkTree *b, int32_t row, int col, int stop_row,
                                       int stop_col, MetaFilter meta_filter, MarkTreeIter *itr);
extern bool rs_marktree_itr_next_filter(MarkTree *b, MarkTreeIter *itr, int stop_row,
                                        int stop_col, MetaFilter meta_filter);
extern bool rs_marktree_itr_step_out_filter(MarkTree *b, MarkTreeIter *itr,
                                            MetaFilter meta_filter);

// Overlap functions
extern bool rs_marktree_itr_get_overlap(MarkTree *b, int row, int col, MarkTreeIter *itr);
extern bool rs_marktree_itr_step_overlap(MarkTree *b, MarkTreeIter *itr, MTPair *pair);

// Tree operations
extern void rs_marktree_clear(MarkTree *b);
extern void rs_marktree_check(MarkTree *b);
extern bool rs_marktree_check_intersections(MarkTree *b);

// Memory management operations
extern void rs_marktree_free_node(MarkTree *b, MTNode *x);
extern void rs_marktree_free_subtree(MarkTree *b, MTNode *x);

// Intersection operations
extern void rs_intersect_node(MarkTree *b, MTNode *x, uint64_t id);
extern void rs_unintersect_node(MarkTree *b, MTNode *x, uint64_t id, bool strict);
extern bool rs_intersection_has(MTNode *x, uint64_t id);
extern void rs_bubble_up(MTNode *x);

// Meta description and pseudo-index operations
extern void rs_meta_describe_key(MTKey k, uint32_t *meta_out);
extern void rs_meta_describe_key_inc(uint32_t *meta_inc, MTKey *k);
extern void rs_meta_describe_node(uint32_t *meta_out, MTNode *x);
extern uint64_t rs_pseudo_index(MTNode *x, int i);
extern uint64_t rs_pseudo_index_for_id(MarkTree *b, uint64_t id, bool sloppy);

// Binary search
extern int rs_marktree_getp_aux(const MTNode *x, MTKey k, bool *match);

// Intersection pair
extern void rs_marktree_intersect_pair(MarkTree *b, uint64_t id, MarkTreeIter *itr,
                                       MarkTreeIter *end_itr, bool delete);

// Insertion operations
extern void rs_split_node(MarkTree *b, MTNode *x, int i, MTKey next);
extern void rs_marktree_putp_aux(MarkTree *b, MTNode *x, MTKey k, uint32_t *meta_inc);
extern void rs_marktree_put_key(MarkTree *b, MTKey k);
extern void rs_marktree_put(MarkTree *b, MTKey key, int end_row, int end_col, bool end_right);

// Test helper operations
extern void rs_marktree_put_test(MarkTree *b, uint32_t ns, uint32_t id, int row, int col,
                                  bool right_gravity, int end_row, int end_col, bool end_right,
                                  bool meta_inline);
extern bool rs_mt_right_test(MTKey key);

// Phase 4 (pass 4): splice
extern bool rs_marktree_splice(MarkTree *b, int32_t start_line, int start_col,
                               int old_extent_line, int old_extent_col,
                               int new_extent_line, int new_extent_col);

// Phase 1 (pass 2) migrations
extern void rs_marktree_revise_meta(MarkTree *b, MarkTreeIter *itr, MTKey old_key);
extern void rs_marktree_move(MarkTree *b, MarkTreeIter *itr, int row, int col);
extern void rs_marktree_restore_pair(MarkTree *b, MTKey key);
extern void rs_marktree_del_pair_test(MarkTree *b, uint32_t ns, uint32_t id);
extern void rs_marktree_move_region(MarkTree *b, int start_row, colnr_T start_col,
                                    int extent_row, colnr_T extent_col,
                                    int new_row, colnr_T new_col);

// Phase 5 (pass 5): deletion rebalancing
extern uint64_t rs_marktree_del_itr(MarkTree *b, MarkTreeIter *itr, bool rev);

#define ILEN (sizeof(MTNode) + sizeof(struct mtnode_inner_s))

#define rawkey(itr) ((itr)->x->key[(itr)->i])

#include "marktree_shim.c.generated.h"

#define ptr s->i_ptr
#define meta s->i_meta
// put functions




// TODO(bfredl): kv_print
#define GA_PUT(x) ga_concat(ga, (char *)(x))
#define GA_PRINT(fmt, ...) snprintf(buf, sizeof(buf), fmt, __VA_ARGS__); \
  GA_PUT(buf);

String mt_inspect(MarkTree *b, bool keys, bool dot)
{
  garray_T ga[1];
  ga_init(ga, (int)sizeof(char), 80);
  MTPos p = { 0, 0 };
  if (b->root) {
    if (dot) {
      GA_PUT("digraph D {\n\n");
      mt_inspect_dotfile_node(b, ga, b->root, p, NULL);
      GA_PUT("\n}");
    } else {
      mt_inspect_node(b, ga, keys, b->root, p);
    }
  }
  return ga_take_string(ga);
}

static inline uint64_t mt_dbg_id(uint64_t id)
{
  return (id >> 1) & 0xffffffff;
}

static void mt_inspect_node(MarkTree *b, garray_T *ga, bool keys, MTNode *n, MTPos off)
{
  static char buf[1024];
  GA_PUT("[");
  if (keys && kv_size(n->intersect)) {
    for (size_t i = 0; i < kv_size(n->intersect); i++) {
      GA_PUT(i == 0 ? "{" : ";");
      // GA_PRINT("%"PRIu64, kv_A(n->intersect, i));
      GA_PRINT("%" PRIu64, mt_dbg_id(kv_A(n->intersect, i)));
    }
    GA_PUT("},");
  }
  if (n->level) {
    mt_inspect_node(b, ga, keys, n->ptr[0], off);
  }
  for (int i = 0; i < n->n; i++) {
    MTPos p = n->key[i].pos;
    rs_unrelative(off, &p);
    GA_PRINT("%d/%d", p.row, p.col);
    if (keys) {
      MTKey key = n->key[i];
      GA_PUT(":");
      if (mt_start(key)) {
        GA_PUT("<");
      }
      // GA_PRINT("%"PRIu64, mt_lookup_id(key.ns, key.id, false));
      GA_PRINT("%" PRIu32, key.id);
      if (mt_end(key)) {
        GA_PUT(">");
      }
    }
    if (n->level) {
      mt_inspect_node(b, ga, keys, n->ptr[i + 1], p);
    } else {
      ga_concat(ga, ",");
    }
  }
  ga_concat(ga, "]");
}

static void mt_inspect_dotfile_node(MarkTree *b, garray_T *ga, MTNode *n, MTPos off, char *parent)
{
  static char buf[1024];
  char namebuf[64];
  if (parent != NULL) {
    snprintf(namebuf, sizeof namebuf, "%s_%c%d", parent, 'a' + n->level, n->p_idx);
  } else {
    snprintf(namebuf, sizeof namebuf, "MTNode");
  }

  GA_PRINT("  %s[shape=plaintext, label=<\n", namebuf);
  GA_PUT("    <table border='0' cellborder='1' cellspacing='0'>\n");
  if (kv_size(n->intersect)) {
    GA_PUT("    <tr><td>");
    for (size_t i = 0; i < kv_size(n->intersect); i++) {
      if (i > 0) {
        GA_PUT(", ");
      }
      GA_PRINT("%" PRIu64, mt_dbg_id(kv_A(n->intersect, i)));
    }
    GA_PUT("</td></tr>\n");
  }

  GA_PUT("    <tr><td>");
  for (int i = 0; i < n->n; i++) {
    MTKey k = n->key[i];
    if (i > 0) {
      GA_PUT(", ");
    }
    GA_PRINT("%d", k.id);
    if (mt_paired(k)) {
      GA_PUT(mt_end(k) ? "e" : "s");
    }
  }
  GA_PUT("</td></tr>\n");
  GA_PUT("    </table>\n");
  GA_PUT(">];\n");
  if (parent) {
    GA_PRINT("  %s -> %s\n", parent, namebuf);
  }
  if (n->level) {
    mt_inspect_dotfile_node(b, ga, n->ptr[0], off, namebuf);
  }
  for (int i = 0; i < n->n; i++) {
    MTPos p = n->key[i].pos;
    rs_unrelative(off, &p);
    if (n->level) {
      mt_inspect_dotfile_node(b, ga, n->ptr[i + 1], p, namebuf);
    }
  }
}

// ============================================================================
// Rust FFI Accessor Functions
// ============================================================================

int nvim_mtnode_get_n(MTNode *x) { return x->n; }
int nvim_mtnode_get_level(MTNode *x) { return x->level; }
MTKey nvim_mtnode_get_key(MTNode *x, int idx) { return x->key[idx]; }
MTNode *nvim_mtnode_get_ptr(MTNode *x, int idx) { return x->ptr[idx]; }
MTNode *nvim_marktree_get_root(MarkTree *b) { return b->root; }
size_t nvim_marktree_get_n_keys(MarkTree *b) { return b->n_keys; }
int nvim_marktree_get_root_level(MarkTree *b) { return b->root ? b->root->level : 0; }
MTNode *nvim_mtnode_get_parent(MTNode *x) { return x->parent; }
int nvim_mtnode_get_p_idx(MTNode *x) { return x->p_idx; }

// ============================================================================
// Iterator Accessor Functions
// ============================================================================

MTNode *nvim_mtitr_get_x(MarkTreeIter *itr) { return itr->x; }
int nvim_mtitr_get_i(MarkTreeIter *itr) { return itr->i; }
int nvim_mtitr_get_lvl(MarkTreeIter *itr) { return itr->lvl; }
MTPos nvim_mtitr_get_pos(MarkTreeIter *itr) { return itr->pos; }
void nvim_mtitr_set_x(MarkTreeIter *itr, MTNode *x) { itr->x = x; }
void nvim_mtitr_set_i(MarkTreeIter *itr, int i) { itr->i = i; }
void nvim_mtitr_set_lvl(MarkTreeIter *itr, int lvl) { itr->lvl = lvl; }
void nvim_mtitr_set_pos(MarkTreeIter *itr, MTPos pos) { itr->pos = pos; }
int nvim_mtitr_get_s_i(MarkTreeIter *itr, int lvl) { return itr->s[lvl].i; }
int nvim_mtitr_get_s_oldcol(MarkTreeIter *itr, int lvl) { return itr->s[lvl].oldcol; }
void nvim_mtitr_set_s_i(MarkTreeIter *itr, int lvl, int i) { itr->s[lvl].i = i; }
void nvim_mtitr_set_s_oldcol(MarkTreeIter *itr, int lvl, int oldcol) { itr->s[lvl].oldcol = oldcol; }

// ============================================================================
// Lookup and Pair Functions (for Rust FFI)
// ============================================================================

MTKey nvim_marktree_lookup(MarkTree *b, uint64_t id, MarkTreeIter *itr) { return rs_marktree_lookup(b, id, itr); }
MTKey nvim_marktree_lookup_ns(MarkTree *b, uint32_t ns, uint32_t id, bool end, MarkTreeIter *itr) { return rs_marktree_lookup_ns(b, ns, id, end, itr); }
MTKey nvim_marktree_get_alt(MarkTree *b, MTKey mark, MarkTreeIter *itr) { return rs_marktree_get_alt(b, mark, itr); }
MTPos nvim_marktree_get_altpos(MarkTree *b, MTKey mark, MarkTreeIter *itr) { return rs_marktree_get_altpos(b, mark, itr); }

// ============================================================================
// Iterator Allocation Functions (for Rust FFI - extmark crate)
// ============================================================================

MarkTreeIter *nvim_marktree_itr_alloc(void) { return xcalloc(1, sizeof(MarkTreeIter)); }
void nvim_marktree_itr_free(MarkTreeIter *itr) { xfree(itr); }
void nvim_marktree_itr_copy(MarkTreeIter *dst, MarkTreeIter *src) { *dst = *src; }
void nvim_marktree_itr_get(MarkTree *b, int row, int col, MarkTreeIter *itr) { rs_marktree_itr_get(b, row, col, itr); }
bool nvim_marktree_itr_next(MarkTree *b, MarkTreeIter *itr) { return rs_marktree_itr_next(b, itr); }
MTKey nvim_marktree_itr_current(MarkTreeIter *itr) { return rs_marktree_itr_current(itr); }
uint16_t nvim_mt_itr_rawkey_get_flags(MarkTreeIter *itr) { return rawkey(itr).flags; }
void nvim_mt_itr_rawkey_set_flags(MarkTreeIter *itr, uint16_t flags) { rawkey(itr).flags = flags; }
DecorInlineData nvim_mt_itr_rawkey_get_decor_data(MarkTreeIter *itr) { return rawkey(itr).decor_data; }
void nvim_mt_itr_rawkey_set_decor_data(MarkTreeIter *itr, DecorInlineData data) { rawkey(itr).decor_data = data; }

// ============================================================================
// Iterator Overlap Accessor Functions (for Rust FFI)
// ============================================================================

MTPos nvim_mtitr_get_intersect_pos(MarkTreeIter *itr) { return itr->intersect_pos; }
void nvim_mtitr_set_intersect_pos(MarkTreeIter *itr, MTPos pos) { itr->intersect_pos = pos; }
MTPos nvim_mtitr_get_intersect_pos_x(MarkTreeIter *itr) { return itr->intersect_pos_x; }
void nvim_mtitr_set_intersect_pos_x(MarkTreeIter *itr, MTPos pos) { itr->intersect_pos_x = pos; }
size_t nvim_mtitr_get_intersect_idx(MarkTreeIter *itr) { return itr->intersect_idx; }
void nvim_mtitr_set_intersect_idx(MarkTreeIter *itr, size_t idx) { itr->intersect_idx = idx; }

// ============================================================================
// Overlap Iteration Wrapper Functions (for Rust extmark FFI)
// ============================================================================

bool nvim_marktree_itr_get_overlap(MarkTree *b, int row, int col, MarkTreeIter *itr) { return rs_marktree_itr_get_overlap(b, row, col, itr); }
bool nvim_marktree_itr_step_overlap(MarkTree *b, MarkTreeIter *itr, MTPair *pair) { return rs_marktree_itr_step_overlap(b, itr, pair); }
void nvim_marktree_itr_get_ext_simple(MarkTree *b, int row, int col, MarkTreeIter *itr) { rs_marktree_itr_get_ext_full(b, MTPos(row, col), itr, false, false, NULL, NULL); }

// ============================================================================
// Node Intersection Accessor Functions (for Rust FFI)
// ============================================================================

size_t nvim_mtnode_get_intersect_size(MTNode *x) { return kv_size(x->intersect); }
uint64_t nvim_mtnode_get_intersect_elem(MTNode *x, size_t idx) { return kv_A(x->intersect, idx); }
uint32_t nvim_mtnode_get_meta(MTNode *x, int idx, int m) { return x->s[0].i_meta[idx][m]; }
void nvim_marktree_get_meta_root(MarkTree *b, uint32_t *meta_out)
{
  for (int m = 0; m < kMTMetaCount; m++) {
    meta_out[m] = b->meta_root[m];
  }
}

bool nvim_meta_has(const uint32_t *meta_count, const uint32_t *meta_filter)
{
  uint32_t count = 0;
  for (int m = 0; m < kMTMetaCount; m++) {
    count += meta_count[m] & meta_filter[m];
  }
  return count > 0;
}

uint64_t nvim_mtnode_intersect_id(MTNode *x, size_t idx)
{
  if (idx < kv_size(x->intersect)) {
    return kv_A(x->intersect, idx);
  }
  return 0;
}

MTNode *nvim_marktree_id2node(MarkTree *b, uint64_t id) { return pmap_get(uint64_t)(b->id2node, id); }
size_t nvim_marktree_id2node_count(MarkTree *b) { return b->id2node ? map_size(b->id2node) : 0; }

// ============================================================================
// Helper Functions (for Rust FFI)
// ============================================================================

MTKey nvim_marktree_itr_set_node(MarkTree *b, MarkTreeIter *itr, MTNode *n, int i) { return rs_marktree_itr_set_node(b, itr, n, i); }
void nvim_marktree_itr_fix_pos(MarkTree *b, MarkTreeIter *itr) { rs_marktree_itr_fix_pos(b, itr); }

// ============================================================================
// Node Mutation Accessor Functions (for Rust FFI)
// ============================================================================

void nvim_mtnode_set_n(MTNode *x, int n) { x->n = (int32_t)n; }
void nvim_mtnode_set_level(MTNode *x, int level) { x->level = (int16_t)level; }
void nvim_mtnode_set_key(MTNode *x, int idx, MTKey k) { x->key[idx] = k; }
void nvim_mtnode_set_ptr(MTNode *x, int idx, MTNode *child) { x->s[0].i_ptr[idx] = child; }
void nvim_mtnode_set_parent(MTNode *x, MTNode *parent) { x->parent = parent; }
void nvim_mtnode_set_p_idx(MTNode *x, int p_idx) { x->p_idx = (int16_t)p_idx; }
void nvim_mtnode_set_meta(MTNode *x, int idx, int m, uint32_t val) { x->s[0].i_meta[idx][m] = val; }
void nvim_mtnode_memmove_keys(MTNode *x, int dst, int src, int count) { memmove(&x->key[dst], &x->key[src], (size_t)count * sizeof(MTKey)); }
void nvim_mtnode_memmove_ptr(MTNode *x, int dst, int src, int count) { memmove(&x->s[0].i_ptr[dst], &x->s[0].i_ptr[src], (size_t)count * sizeof(MTNode *)); }
void nvim_mtnode_memmove_meta(MTNode *x, int dst, int src, int count) { memmove(&x->s[0].i_meta[dst], &x->s[0].i_meta[src], (size_t)count * sizeof(x->s[0].i_meta[0])); }
void nvim_mtnode_memcpy_keys(MTNode *dst, int dst_idx, MTNode *src, int src_idx, int count) { memcpy(&dst->key[dst_idx], &src->key[src_idx], (size_t)count * sizeof(MTKey)); }
void nvim_mtnode_memcpy_ptr(MTNode *dst, int dst_idx, MTNode *src, int src_idx, int count) { memcpy(&dst->s[0].i_ptr[dst_idx], &src->s[0].i_ptr[src_idx], (size_t)count * sizeof(MTNode *)); }
void nvim_mtnode_memcpy_meta(MTNode *dst, int dst_idx, MTNode *src, int src_idx, int count) { memcpy(&dst->s[0].i_meta[dst_idx], &src->s[0].i_meta[src_idx], (size_t)count * sizeof(dst->s[0].i_meta[0])); }

// ============================================================================
// Tree Mutation Functions (for Rust FFI)
// ============================================================================

MTNode *nvim_marktree_alloc_node(MarkTree *b, bool internal)
{
  MTNode *x = xcalloc(1, internal ? ILEN : sizeof(MTNode));
  kvi_init(x->intersect);
  b->n_nodes++;
  return x;
}
void nvim_marktree_refkey(MarkTree *b, MTNode *x, int i) { pmap_put(uint64_t)(b->id2node, mt_lookup_key(x->key[i]), x); }
void nvim_marktree_set_root(MarkTree *b, MTNode *root) { b->root = root; }
void nvim_marktree_inc_n_keys(MarkTree *b) { b->n_keys++; }
void nvim_marktree_add_meta_root(MarkTree *b, int m, uint32_t val) { b->meta_root[m] += val; }
void nvim_marktree_set_meta_root(MarkTree *b, int m, uint32_t val) { b->meta_root[m] = val; }

// Memory management accessors for Phase 7
void nvim_kvi_destroy_intersect(MTNode *x) { kvi_destroy(x->intersect); }
void nvim_xfree_node(MTNode *x) { xfree(x); }
void nvim_marktree_dec_n_nodes(MarkTree *b) { b->n_nodes--; }
void nvim_marktree_set_n_keys(MarkTree *b, size_t n) { b->n_keys = n; }
void nvim_marktree_destroy_id2node(MarkTree *b) { map_destroy(uint64_t, b->id2node); }

// ============================================================================
// Intersection Operations (for Rust FFI)
// ============================================================================

void nvim_kvi_copy_intersect(MTNode *dst, MTNode *src) { kvi_copy(dst->intersect, src->intersect); }
void nvim_kvi_init_intersect(MTNode *x) { kvi_init(x->intersect); }

// Intersection list mutation accessors for Rust to implement set operations
void nvim_mtnode_intersect_clear(MTNode *x)
{
  kvi_destroy(x->intersect);
  kvi_init(x->intersect);
}

void nvim_mtnode_intersect_push(MTNode *x, uint64_t id)
{
  kvi_push(x->intersect, id);
}


// ============================================================================
// B-tree Deletion Operations (for Rust FFI)
// ============================================================================

uint64_t nvim_marktree_del_itr(MarkTree *b, MarkTreeIter *itr, bool rev) { return rs_marktree_del_itr(b, itr, rev); }
void nvim_marktree_revise_meta(MarkTree *b, MarkTreeIter *itr, MTKey old_key) { rs_marktree_revise_meta(b, itr, old_key); }
void nvim_marktree_move(MarkTree *b, MarkTreeIter *itr, int row, int col) { rs_marktree_move(b, itr, row, col); }
void nvim_marktree_restore_pair(MarkTree *b, MTKey key) { rs_marktree_restore_pair(b, key); }
void nvim_marktree_del_id(MarkTree *b, uint64_t id) { pmap_del(uint64_t)(b->id2node, id, NULL); }
void nvim_marktree_dec_n_keys(MarkTree *b) { b->n_keys--; }
MarkTreeIter *nvim_alloc_marktreeiter(void) { return xcalloc(1, sizeof(MarkTreeIter)); }
void nvim_free_marktreeiter(MarkTreeIter *itr) { xfree(itr); }
void nvim_marktree_sub_meta_root(MarkTree *b, int m, uint32_t val) { b->meta_root[m] -= val; }
MTKey nvim_rawkey(MarkTreeIter *itr) { return rawkey(itr); }
void nvim_rawkey_set_flags(MarkTreeIter *itr, uint16_t flags) { rawkey(itr).flags = flags; }
void nvim_rawkey_or_flags(MarkTreeIter *itr, uint16_t flags) { rawkey(itr).flags |= flags; }
void nvim_rawkey_clear_flags(MarkTreeIter *itr, uint16_t flags) { rawkey(itr).flags &= (uint16_t)~flags; }
void nvim_rawkey_set_pos(MarkTreeIter *itr, MTPos pos) { rawkey(itr).pos = pos; }
MTPos nvim_rawkey_get_pos(MarkTreeIter *itr) { return rawkey(itr).pos; }
void nvim_rawkey_add_pos_col(MarkTreeIter *itr, int delta) { rawkey(itr).pos.col += delta; }
void nvim_rawkey_add_pos_row(MarkTreeIter *itr, int delta) { rawkey(itr).pos.row += delta; }

// ============================================================================
// Splice Operations (for Rust FFI)
// ============================================================================

void nvim_marktree_move_region(MarkTree *b, int start_row, colnr_T start_col,
                               int extent_row, colnr_T extent_col,
                               int new_row, colnr_T new_col) { rs_marktree_move_region(b, start_row, start_col, extent_row, extent_col, new_row, new_col); }

// ============================================================================
// Debug and Validation (for Rust FFI)
// ============================================================================

void nvim_marktree_check(MarkTree *b)
{
#ifndef NDEBUG
  rs_marktree_check(b);
#else
  (void)b;
#endif
}

// ============================================================================
// MTKey Accessor Functions (for Rust sign crate)
// ============================================================================

int32_t nvim_mtkey_get_row(MTKey key) { return key.pos.row; }
uint32_t nvim_mtkey_get_ns(MTKey key) { return key.ns; }
uint32_t nvim_mtkey_get_id(MTKey key) { return key.id; }
