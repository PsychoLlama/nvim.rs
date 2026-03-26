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

#include <stdlib.h>
#include <string.h>

#include "klib/kvec.h"
#include "nvim/macros_defs.h"
#include "nvim/map_defs.h"
#include "nvim/marktree.h"
#include "nvim/memory.h"
#include "nvim/pos_defs.h"

// Rust marktree FFI declarations

// Extended iterator functions (used by nvim_marktree_itr_get_ext_simple)
extern bool rs_marktree_itr_get_ext_full(MarkTree *b, MTPos p, MarkTreeIter *itr, bool last,
                                         bool gravity, MTPos *oldbase,
                                         MetaFilter meta_filter);

#define ILEN (sizeof(MTNode) + sizeof(struct mtnode_inner_s))

#define rawkey(itr) ((itr)->x->key[(itr)->i])

#include "marktree_shim.c.generated.h"

// ============================================================================
// Rust FFI Accessor Functions
// ============================================================================

int nvim_mtnode_get_n(MTNode *x) { return x->n; }
int nvim_mtnode_get_level(MTNode *x) { return x->level; }
MTKey nvim_mtnode_get_key(MTNode *x, int idx) { return x->key[idx]; }
MTNode *nvim_mtnode_get_ptr(MTNode *x, int idx) { return x->s[0].i_ptr[idx]; }
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
// Iterator Allocation Functions (for Rust FFI - extmark crate)
// ============================================================================

MarkTreeIter *nvim_marktree_itr_alloc(void) { return xcalloc(1, sizeof(MarkTreeIter)); }
void nvim_marktree_itr_free(MarkTreeIter *itr) { xfree(itr); }
void nvim_marktree_itr_copy(MarkTreeIter *dst, MarkTreeIter *src) { *dst = *src; }
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
// Iterator Adapter Functions (for Rust extmark FFI)
// ============================================================================

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

// Memory management accessors
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

void nvim_marktree_del_id(MarkTree *b, uint64_t id) { pmap_del(uint64_t)(b->id2node, id, NULL); }
void nvim_marktree_dec_n_keys(MarkTree *b) { b->n_keys--; }
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
// MTKey Accessor Functions (for Rust sign crate)
// ============================================================================

int32_t nvim_mtkey_get_row(MTKey key) { return key.pos.row; }
uint32_t nvim_mtkey_get_ns(MTKey key) { return key.ns; }
uint32_t nvim_mtkey_get_id(MTKey key) { return key.id; }
