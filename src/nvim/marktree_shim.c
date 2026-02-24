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

// Phase 1 (pass 2) migrations
extern void rs_marktree_revise_meta(MarkTree *b, MarkTreeIter *itr, MTKey old_key);
extern void rs_marktree_move(MarkTree *b, MarkTreeIter *itr, int row, int col);
extern void rs_marktree_restore_pair(MarkTree *b, MTKey key);
extern void rs_marktree_del_pair_test(MarkTree *b, uint32_t ns, uint32_t id);
extern void rs_marktree_move_region(MarkTree *b, int start_row, colnr_T start_col,
                                    int extent_row, colnr_T extent_col,
                                    int new_row, colnr_T new_col);

// Phase 2 (pass 2): intersection set operations
extern void rs_merge_node_intersect(MarkTree *b, MTNode *x, int x_old_n, MTNode *y, int y_n);
extern void rs_pivot_right_intersect(MarkTree *b, MTNode *x, MTNode *y, int y_n);
extern void rs_pivot_left_intersect(MarkTree *b, MTNode *x, int x_n, MTNode *y);
extern bool rs_intersect_mov_test(const uint64_t *x, size_t nx, const uint64_t *y, size_t ny,
                                  const uint64_t *win, size_t nwin, uint64_t *wout, size_t *nwout,
                                  uint64_t *dout, size_t *ndout);

#define T MT_BRANCH_FACTOR
#define ILEN (sizeof(MTNode) + sizeof(struct mtnode_inner_s))

#define ID_INCR (((uint64_t)1) << 2)

#define rawkey(itr) ((itr)->x->key[(itr)->i])

// Used by `marktree_splice`. Need to keep track of marks which moved
// in order to repair intersections.
typedef struct {
  uint64_t id;
  MTNode *old, *new;
  int old_i, new_i;
} Damage;
typedef kvec_withinit_t(Damage, 8) DamageList;

#include "marktree_shim.c.generated.h"

static inline void refkey(MarkTree *b, MTNode *x, int i)
{
  pmap_put(uint64_t)(b->id2node, mt_lookup_key(x->key[i]), x);
}

static MTNode *id2node(MarkTree *b, uint64_t id)
{
  return pmap_get(uint64_t)(b->id2node, id);
}

#define ptr s->i_ptr
#define meta s->i_meta
// put functions

// x must be an internal node, which is not full
// x->ptr[i] should be a full node, i e x->ptr[i]->n == 2*T-1
static inline void split_node(MarkTree *b, MTNode *x, const int i, MTKey next)
{
  rs_split_node(b, x, i, next);
}

// x must not be a full node (even if there might be internal space)
static inline void marktree_putp_aux(MarkTree *b, MTNode *x, MTKey k, uint32_t *meta_inc)
{
  rs_marktree_putp_aux(b, x, k, meta_inc);
}

void marktree_put(MarkTree *b, MTKey key, int end_row, int end_col, bool end_right)
{
  rs_marktree_put(b, key, end_row, end_col, end_right);
}


/// @param itr mutated
/// @param end_itr not mutated
void marktree_intersect_pair(MarkTree *b, uint64_t id, MarkTreeIter *itr, MarkTreeIter *end_itr,
                             bool delete)
{
  rs_marktree_intersect_pair(b, id, itr, end_itr, delete);
}

static MTNode *marktree_alloc_node(MarkTree *b, bool internal)
{
  MTNode *x = xcalloc(1, internal ? ILEN : sizeof(MTNode));
  kvi_init(x->intersect);
  b->n_nodes++;
  return x;
}


void marktree_put_key(MarkTree *b, MTKey k)
{
  rs_marktree_put_key(b, k);
}

/// INITIATING DELETION PROTOCOL:
///
/// 1. Construct a valid iterator to the node to delete (argument)
/// 2. If an "internal" key. Iterate one step to the left or right,
///     which gives an internal key "auxiliary key".
/// 3. Now delete this internal key (intended or auxiliary).
///    The leaf node X might become undersized.
/// 4. If step two was done: now replace the key that _should_ be
///    deleted with the auxiliary key. Adjust relative
/// 5. Now "repair" the tree as needed. We always start at a leaf node X.
///     - if the node is big enough, terminate
///     - if we can steal from the left, steal
///     - if we can steal from the right, steal
///     - otherwise merge this node with a neighbour. This might make our
///       parent undersized. So repeat 5 for the parent.
/// 6. If 4 went all the way to the root node. The root node
///    might have ended up with size 0. Delete it then.
///
/// The iterator remains valid, and now points at the key _after_ the deleted
/// one.
///
/// @param rev should be true if we plan to iterate _backwards_ and delete
///            stuff before this key. Most of the time this is false (the
///            recommended strategy is to always iterate forward)
uint64_t marktree_del_itr(MarkTree *b, MarkTreeIter *itr, bool rev)
{
  int adjustment = 0;

  MTNode *cur = itr->x;
  int curi = itr->i;
  uint64_t id = mt_lookup_key(cur->key[curi]);

  MTKey raw = rawkey(itr);
  uint64_t other = 0;
  if (mt_paired(raw) && !(raw.flags & MT_FLAG_ORPHANED)) {
    other = mt_lookup_key_side(raw, !mt_end(raw));

    MarkTreeIter other_itr[1];
    rs_marktree_lookup(b, other, other_itr);
    rawkey(other_itr).flags |= MT_FLAG_ORPHANED;
    // Remove intersect markers. NB: must match exactly!
    if (mt_start(raw)) {
      MarkTreeIter this_itr[1] = { *itr };  // mutated copy
      marktree_intersect_pair(b, id, this_itr, other_itr, true);
    } else {
      marktree_intersect_pair(b, other, other_itr, itr, true);
    }
  }

  // 2.
  if (itr->x->level) {
    if (rev) {
      abort();
    } else {
      // steal previous node
      rs_marktree_itr_prev(b, itr);
      adjustment = -1;
    }
  }

  // 3.
  MTNode *x = itr->x;
  assert(x->level == 0);
  MTKey intkey = x->key[itr->i];

  uint32_t meta_inc[kMTMetaCount];
  rs_meta_describe_key(intkey, meta_inc);
  if (x->n > itr->i + 1) {
    memmove(&x->key[itr->i], &x->key[itr->i + 1],
            sizeof(MTKey) * (size_t)(x->n - itr->i - 1));
  }
  x->n--;

  b->n_keys--;
  pmap_del(uint64_t)(b->id2node, id, NULL);

  // 4.
  // if (adjustment == 1) {
  //   abort();
  // }
  if (adjustment == -1) {
    int ilvl = itr->lvl - 1;
    MTNode *lnode = x;
    uint64_t start_id = 0;
    bool did_bubble = false;
    if (mt_end(intkey)) {
      start_id = mt_lookup_key_side(intkey, false);
    }
    do {
      MTNode *p = lnode->parent;
      if (ilvl < 0) {
        abort();
      }
      int i = itr->s[ilvl].i;
      assert(p->ptr[i] == lnode);
      if (i > 0) {
        rs_unrelative(p->key[i - 1].pos, &intkey.pos);
      }

      if (p != cur && start_id) {
        if (rs_intersection_has(p->ptr[0], start_id)) {
          // if not the first time, we need to undo the addition in the
          // previous step (`rs_intersect_node` just below)
          int last = (lnode != x) ? 1 : 0;
          for (int k = 0; k < p->n + last; k++) {  // one less as p->ptr[n] is the last
            rs_unintersect_node(b, p->ptr[k], start_id, true);
          }
          rs_intersect_node(b, p, start_id);
          did_bubble = true;
        }
      }

      for (int m = 0; m < kMTMetaCount; m++) {
        p->meta[lnode->p_idx][m] -= meta_inc[m];
      }

      lnode = p;
      ilvl--;
    } while (lnode != cur);

    MTKey deleted = cur->key[curi];
    rs_meta_describe_key(deleted, meta_inc);
    cur->key[curi] = intkey;
    refkey(b, cur, curi);
    // if `did_bubble` then we already added `start_id` to some parent
    if (mt_end(cur->key[curi]) && !did_bubble) {
      uint64_t pi = rs_pseudo_index(x, 0);  // note: sloppy pseudo-index
      uint64_t pi_start = rs_pseudo_index_for_id(b, start_id, true);
      if (pi_start > 0 && pi_start < pi) {
        rs_intersect_node(b, x, start_id);
      }
    }

    rs_relative(intkey.pos, &deleted.pos);
    MTNode *y = cur->ptr[curi + 1];
    if (deleted.pos.row || deleted.pos.col) {
      while (y) {
        for (int k = 0; k < y->n; k++) {
          rs_unrelative(deleted.pos, &y->key[k].pos);
        }
        y = y->level ? y->ptr[0] : NULL;
      }
    }
    itr->i--;
  }

  MTNode *lnode = cur;
  while (lnode->parent) {
    uint32_t *meta_p = lnode->parent->meta[lnode->p_idx];
    for (int m = 0; m < kMTMetaCount; m++) {
      meta_p[m] -= meta_inc[m];
    }

    lnode = lnode->parent;
  }
  for (int m = 0; m < kMTMetaCount; m++) {
    assert(b->meta_root[m] >= meta_inc[m]);
    b->meta_root[m] -= meta_inc[m];
  }

  // 5.
  bool itr_dirty = false;
  int rlvl = itr->lvl - 1;
  int *lasti = &itr->i;
  MTPos ppos = itr->pos;
  while (x != b->root) {
    assert(rlvl >= 0);
    MTNode *p = x->parent;
    if (x->n >= T - 1) {
      // we are done, if this node is fine the rest of the tree will be
      break;
    }
    int pi = itr->s[rlvl].i;
    assert(p->ptr[pi] == x);
    if (pi > 0) {
      ppos.row -= p->key[pi - 1].pos.row;
      ppos.col = itr->s[rlvl].oldcol;
    }
    // ppos is now the pos of p

    if (pi > 0 && p->ptr[pi - 1]->n > T - 1) {
      *lasti += 1;
      itr_dirty = true;
      // steal one key from the left neighbour
      pivot_right(b, ppos, p, pi - 1);
      break;
    } else if (pi < p->n && p->ptr[pi + 1]->n > T - 1) {
      // steal one key from right neighbour
      pivot_left(b, ppos, p, pi);
      break;
    } else if (pi > 0) {
      assert(p->ptr[pi - 1]->n == T - 1);
      // merge with left neighbour
      *lasti += T;
      x = merge_node(b, p, pi - 1);
      if (lasti == &itr->i) {
        // TRICKY: we merged the node the iterator was on
        itr->x = x;
      }
      itr->s[rlvl].i--;
      itr_dirty = true;
    } else {
      assert(pi < p->n && p->ptr[pi + 1]->n == T - 1);
      merge_node(b, p, pi);
      // no iter adjustment needed
    }
    lasti = &itr->s[rlvl].i;
    rlvl--;
    x = p;
  }

  // 6.
  if (b->root->n == 0) {
    if (itr->lvl > 0) {
      memmove(itr->s, itr->s + 1, (size_t)(itr->lvl - 1) * sizeof(*itr->s));
      itr->lvl--;
    }
    if (b->root->level) {
      MTNode *oldroot = b->root;
      b->root = b->root->ptr[0];
      for (int m = 0; m < kMTMetaCount; m++) {
        assert(b->meta_root[m] == oldroot->meta[0][m]);
      }

      b->root->parent = NULL;
      marktree_free_node(b, oldroot);
    } else {
      // no items, nothing for iterator to point to
      // not strictly needed, should handle delete right-most mark anyway
      itr->x = NULL;
    }
  }

  if (itr->x && itr_dirty) {
    rs_marktree_itr_fix_pos(b, itr);
  }

  // BONUS STEP: fix the iterator, so that it points to the key afterwards
  // TODO(bfredl): with "rev" should point before
  // if (adjustment == 1) {
  //   abort();
  // }
  if (adjustment == -1) {
    // tricky: we stand at the deleted space in the previous leaf node.
    // But the inner key is now the previous key we stole, so we need
    // to skip that one as well.
    rs_marktree_itr_next(b, itr);
    rs_marktree_itr_next(b, itr);
  } else {
    if (itr->x && itr->i >= itr->x->n) {
      // we deleted the last key of a leaf node
      // go to the inner key after that.
      assert(itr->x->level == 0);
      rs_marktree_itr_next(b, itr);
    }
  }

  return other;
}

void marktree_revise_meta(MarkTree *b, MarkTreeIter *itr, MTKey old_key)
{
  rs_marktree_revise_meta(b, itr, old_key);
}

bool intersect_mov_test(const uint64_t *x, size_t nx, const uint64_t *y, size_t ny,
                        const uint64_t *win, size_t nwin, uint64_t *wout, size_t *nwout,
                        uint64_t *dout, size_t *ndout)
{
  return rs_intersect_mov_test(x, nx, y, ny, win, nwin, wout, nwout, dout, ndout);
}


static MTNode *merge_node(MarkTree *b, MTNode *p, int i)
{
  MTNode *x = p->ptr[i];
  MTNode *y = p->ptr[i + 1];
  int x_old_n = x->n;  // save before modification

  x->key[x->n] = p->key[i];
  refkey(b, x, x->n);
  if (i > 0) {
    rs_relative(p->key[i - 1].pos, &x->key[x->n].pos);
  }

  uint32_t meta_inc[kMTMetaCount];
  rs_meta_describe_key(x->key[x->n], meta_inc);

  memmove(&x->key[x->n + 1], y->key, (size_t)y->n * sizeof(MTKey));
  for (int k = 0; k < y->n; k++) {
    refkey(b, x, x->n + 1 + k);
    rs_unrelative(x->key[x->n].pos, &x->key[x->n + 1 + k].pos);
  }
  if (x->level) {
    // bubble down: ranges that intersected old-x but not old-y or vice versa
    // must be moved to their respective children
    memmove(&x->ptr[x->n + 1], y->ptr, ((size_t)y->n + 1) * sizeof(MTNode *));
    memmove(&x->meta[x->n + 1], y->meta, ((size_t)y->n + 1) * sizeof(y->meta[0]));
    // Fix parent/p_idx for y's children (now in x's second half)
    for (int ky = 0; ky < y->n + 1; ky++) {
      int k = x_old_n + ky + 1;
      x->ptr[k]->parent = x;
      x->ptr[k]->p_idx = (int16_t)k;
    }
    // Compute intersection merge and apply to children (Rust)
    rs_merge_node_intersect(b, x, x_old_n, y, y->n);
  }
  x->n += y->n + 1;
  for (int m = 0; m < kMTMetaCount; m++) {
    // x now contains everything of y plus old p->key[i]
    p->meta[i][m] += (p->meta[i + 1][m] + meta_inc[m]);
  }

  memmove(&p->key[i], &p->key[i + 1], (size_t)(p->n - i - 1) * sizeof(MTKey));
  memmove(&p->ptr[i + 1], &p->ptr[i + 2], (size_t)(p->n - i - 1) * sizeof(MTKey *));
  memmove(&p->meta[i + 1], &p->meta[i + 2], (size_t)(p->n - i - 1) * sizeof(p->meta[0]));
  for (int j = i + 1; j < p->n; j++) {  // note: one has been deleted
    p->ptr[j]->p_idx = (int16_t)j;
  }
  p->n--;
  marktree_free_node(b, y);

  return x;
}

// TODO(bfredl): as a potential "micro" optimization, pivoting should balance
// the two nodes instead of stealing just one key
// x_pos is the absolute position of the key just before x (or a dummy key strictly less than any
// key inside x, if x is the first leaf)
static void pivot_right(MarkTree *b, MTPos p_pos, MTNode *p, const int i)
{
  MTNode *x = p->ptr[i];
  MTNode *y = p->ptr[i + 1];
  memmove(&y->key[1], y->key, (size_t)y->n * sizeof(MTKey));
  if (y->level) {
    memmove(&y->ptr[1], y->ptr, ((size_t)y->n + 1) * sizeof(MTNode *));
    memmove(&y->meta[1], y->meta, ((size_t)y->n + 1) * sizeof(y->meta[0]));
    for (int j = 1; j < y->n + 2; j++) {
      y->ptr[j]->p_idx = (int16_t)j;
    }
  }

  y->key[0] = p->key[i];
  refkey(b, y, 0);
  p->key[i] = x->key[x->n - 1];
  refkey(b, p, i);

  uint32_t meta_inc_y[kMTMetaCount];
  rs_meta_describe_key(y->key[0], meta_inc_y);
  uint32_t meta_inc_x[kMTMetaCount];
  rs_meta_describe_key(p->key[i], meta_inc_x);

  for (int m = 0; m < kMTMetaCount; m++) {
    p->meta[i + 1][m] += meta_inc_y[m];
    p->meta[i][m] -= meta_inc_x[m];
  }

  if (x->level) {
    y->ptr[0] = x->ptr[x->n];
    memcpy(y->meta[0], x->meta[x->n], sizeof(y->meta[0]));
    for (int m = 0; m < kMTMetaCount; m++) {
      p->meta[i + 1][m] += y->meta[0][m];
      p->meta[i][m] -= y->meta[0][m];
    }
    y->ptr[0]->parent = y;
    y->ptr[0]->p_idx = 0;
  }
  x->n--;
  y->n++;
  if (i > 0) {
    rs_unrelative(p->key[i - 1].pos, &p->key[i].pos);
  }
  rs_relative(p->key[i].pos, &y->key[0].pos);
  for (int k = 1; k < y->n; k++) {
    rs_unrelative(y->key[0].pos, &y->key[k].pos);
  }

  // repair intersections of x
  if (x->level) {
    // y->ptr[0] was moved from x to y; adjust intersections via Rust
    rs_pivot_right_intersect(b, x, y, y->n);

    rs_bubble_up(x);
  } else {
    // if the last element of x used to be an end node, check if it now covers all of x
    if (mt_end(p->key[i])) {
      uint64_t pi = rs_pseudo_index(x, 0);  // note: sloppy pseudo-index
      uint64_t start_id = mt_lookup_key_side(p->key[i], false);
      uint64_t pi_start = rs_pseudo_index_for_id(b, start_id, true);
      if (pi_start > 0 && pi_start < pi) {
        rs_intersect_node(b, x, start_id);
      }
    }

    if (mt_start(y->key[0])) {
      // no need for a check, just delet it if it was there
      rs_unintersect_node(b, y, mt_lookup_key(y->key[0]), false);
    }
  }
}

static void pivot_left(MarkTree *b, MTPos p_pos, MTNode *p, int i)
{
  MTNode *x = p->ptr[i];
  MTNode *y = p->ptr[i + 1];

  // reverse from how we "always" do it. but pivot_left
  // is just the inverse of pivot_right, so reverse it literally.
  for (int k = 1; k < y->n; k++) {
    rs_relative(y->key[0].pos, &y->key[k].pos);
  }
  rs_unrelative(p->key[i].pos, &y->key[0].pos);
  if (i > 0) {
    rs_relative(p->key[i - 1].pos, &p->key[i].pos);
  }

  x->key[x->n] = p->key[i];
  refkey(b, x, x->n);
  p->key[i] = y->key[0];
  refkey(b, p, i);

  uint32_t meta_inc_x[kMTMetaCount];
  rs_meta_describe_key(x->key[x->n], meta_inc_x);
  uint32_t meta_inc_y[kMTMetaCount];
  rs_meta_describe_key(p->key[i], meta_inc_y);
  for (int m = 0; m < kMTMetaCount; m++) {
    p->meta[i][m] += meta_inc_x[m];
    p->meta[i + 1][m] -= meta_inc_y[m];
  }

  if (x->level) {
    x->ptr[x->n + 1] = y->ptr[0];
    memcpy(x->meta[x->n + 1], y->meta[0], sizeof(y->meta[0]));
    for (int m = 0; m < kMTMetaCount; m++) {
      p->meta[i + 1][m] -= y->meta[0][m];
      p->meta[i][m] += y->meta[0][m];
    }
    x->ptr[x->n + 1]->parent = x;
    x->ptr[x->n + 1]->p_idx = (int16_t)(x->n + 1);
  }
  memmove(y->key, &y->key[1], (size_t)(y->n - 1) * sizeof(MTKey));
  if (y->level) {
    memmove(y->ptr, &y->ptr[1], (size_t)y->n * sizeof(MTNode *));
    memmove(y->meta, &y->meta[1], (size_t)y->n * sizeof(y->meta[0]));
    for (int j = 0; j < y->n; j++) {  // note: last item deleted
      y->ptr[j]->p_idx = (int16_t)j;
    }
  }
  x->n++;
  y->n--;

  // repair intersections of x,y
  if (x->level) {
    // x->ptr[x->n] was moved from y to x; adjust intersections via Rust
    rs_pivot_left_intersect(b, x, x->n, y);

    rs_bubble_up(y);
  } else {
    // if the first element of y used to be an start node, check if it now covers all of y
    if (mt_start(p->key[i])) {
      uint64_t pi = rs_pseudo_index(y, 0);  // note: sloppy pseudo-index

      uint64_t end_id = mt_lookup_key_side(p->key[i], true);
      uint64_t pi_end = rs_pseudo_index_for_id(b, end_id, true);

      if (pi_end > pi) {
        rs_intersect_node(b, y, mt_lookup_key(p->key[i]));
      }
    }

    if (mt_end(x->key[x->n - 1])) {
      // no need for a check, just delet it if it was there
      rs_unintersect_node(b, x, mt_lookup_key_side(x->key[x->n - 1], false), false);
    }
  }
}

/// frees all mem, resets tree to valid empty state
void marktree_clear(MarkTree *b)
{
  rs_marktree_clear(b);
}

void marktree_free_subtree(MarkTree *b, MTNode *x)
{
  rs_marktree_free_subtree(b, x);
}

static void marktree_free_node(MarkTree *b, MTNode *x)
{
  rs_marktree_free_node(b, x);
}

/// @param itr iterator is invalid after call
void marktree_move(MarkTree *b, MarkTreeIter *itr, int row, int col)
{
  rs_marktree_move(b, itr, row, col);
}

void marktree_restore_pair(MarkTree *b, MTKey key)
{
  rs_marktree_restore_pair(b, key);
}

static bool itr_eq(MarkTreeIter *itr1, MarkTreeIter *itr2)
{
  return (&rawkey(itr1) == &rawkey(itr2));
}

static void swap_keys(MarkTree *b, MarkTreeIter *itr1, MarkTreeIter *itr2, DamageList *damage)
{
  if (itr1->x != itr2->x) {
    if (mt_paired(rawkey(itr1))) {
      kvi_push(*damage, ((Damage){ mt_lookup_key(rawkey(itr1)), itr1->x, itr2->x,
                                   itr1->i, itr2->i }));
    }
    if (mt_paired(rawkey(itr2))) {
      kvi_push(*damage, ((Damage){ mt_lookup_key(rawkey(itr2)), itr2->x, itr1->x,
                                   itr2->i, itr1->i }));
    }

    uint32_t meta_inc_1[kMTMetaCount];
    rs_meta_describe_key(rawkey(itr1), meta_inc_1);
    uint32_t meta_inc_2[kMTMetaCount];
    rs_meta_describe_key(rawkey(itr2), meta_inc_2);

    if (memcmp(meta_inc_1, meta_inc_2, sizeof(meta_inc_1)) != 0) {
      MTNode *x1 = itr1->x;
      MTNode *x2 = itr2->x;
      while (x1 != x2) {
        if (x1->level <= x2->level) {
          // as the root node uniquely has the highest level, x1 cannot be it
          uint32_t *meta_node = x1->parent->meta[x1->p_idx];
          for (int m = 0; m < kMTMetaCount; m++) {
            meta_node[m] += meta_inc_2[m] - meta_inc_1[m];
          }
          x1 = x1->parent;
        }
        if (x2->level < x1->level) {
          uint32_t *meta_node = x2->parent->meta[x2->p_idx];
          for (int m = 0; m < kMTMetaCount; m++) {
            meta_node[m] += meta_inc_1[m] - meta_inc_2[m];
          }
          x2 = x2->parent;
        }
      }
    }
  }

  MTKey key1 = rawkey(itr1);
  MTKey key2 = rawkey(itr2);
  rawkey(itr1) = key2;
  rawkey(itr1).pos = key1.pos;
  rawkey(itr2) = key1;
  rawkey(itr2).pos = key2.pos;
  refkey(b, itr1->x, itr1->i);
  refkey(b, itr2->x, itr2->i);
}

static int damage_cmp(const void *s1, const void *s2)
{
  Damage *d1 = (Damage *)s1;
  Damage *d2 = (Damage *)s2;
  assert(d1->id != d2->id);
  return d1->id > d2->id ? 1 : -1;
}

bool marktree_splice(MarkTree *b, int32_t start_line, int start_col, int old_extent_line,
                     int old_extent_col, int new_extent_line, int new_extent_col)
{
  MTPos start = { start_line, start_col };
  MTPos old_extent = { old_extent_line, old_extent_col };
  MTPos new_extent = { new_extent_line, new_extent_col };

  bool may_delete = (old_extent.row != 0 || old_extent.col != 0);
  bool same_line = old_extent.row == 0 && new_extent.row == 0;
  rs_unrelative(start, &old_extent);
  rs_unrelative(start, &new_extent);
  MarkTreeIter itr[1] = { 0 };
  MarkTreeIter enditr[1] = { 0 };

  MTPos oldbase[MT_MAX_DEPTH] = { 0 };

  rs_marktree_itr_get_ext_full(b, start, itr, false, true, oldbase, NULL);
  if (!itr->x) {
    // den e FÄRDIG
    return false;
  }
  MTPos delta = { new_extent.row - old_extent.row,
                  new_extent.col - old_extent.col };

  if (may_delete) {
    MTPos ipos = rs_marktree_itr_pos(itr);
    if (!rs_pos_leq(old_extent, ipos)
        || (old_extent.row == ipos.row && old_extent.col == ipos.col
            && !mt_right(rawkey(itr)))) {
      rs_marktree_itr_get_ext_full(b, old_extent, enditr, true, true, NULL, NULL);
      assert(enditr->x);
      // "assert" (itr <= enditr)
    } else {
      may_delete = false;
    }
  }

  bool past_right = false;
  bool moved = false;
  DamageList damage;
  kvi_init(damage);

  // Follow the general strategy of messing things up and fix them later
  // "oldbase" carries the information needed to calculate old position of
  // children.
  if (may_delete) {
    while (itr->x && !past_right) {
      MTPos loc_start = start;
      MTPos loc_old = old_extent;
      rs_relative(itr->pos, &loc_start);

      rs_relative(oldbase[itr->lvl], &loc_old);

continue_same_node:
      // NB: strictly should be less than the right gravity of loc_old, but
      // the iter comparison below will already break on that.
      if (!rs_pos_leq(rawkey(itr).pos, loc_old)) {
        break;
      }

      if (mt_right(rawkey(itr))) {
        while (!itr_eq(itr, enditr)
               && mt_right(rawkey(enditr))) {
          rs_marktree_itr_prev(b, enditr);
        }
        if (!mt_right(rawkey(enditr))) {
          swap_keys(b, itr, enditr, &damage);
        } else {
          past_right = true;  // NOLINT
          (void)past_right;
          break;
        }
      }

      if (itr_eq(itr, enditr)) {
        // actually, will be past_right after this key
        past_right = true;
      }

      moved = true;
      if (itr->x->level) {
        oldbase[itr->lvl + 1] = rawkey(itr).pos;
        rs_unrelative(oldbase[itr->lvl], &oldbase[itr->lvl + 1]);
        rawkey(itr).pos = loc_start;
        rs_marktree_itr_next_skip(b, itr,false, false, oldbase, NULL);
      } else {
        rawkey(itr).pos = loc_start;
        if (itr->i < itr->x->n - 1) {
          itr->i++;
          if (!past_right) {
            goto continue_same_node;
          }
        } else {
          rs_marktree_itr_next(b, itr);
        }
      }
    }
    while (itr->x) {
      MTPos loc_new = new_extent;
      rs_relative(itr->pos, &loc_new);
      MTPos limit = old_extent;

      rs_relative(oldbase[itr->lvl], &limit);

past_continue_same_node:

      if (rs_pos_leq(limit, rawkey(itr).pos)) {
        break;
      }

      MTPos oldpos = rawkey(itr).pos;
      rawkey(itr).pos = loc_new;
      moved = true;
      if (itr->x->level) {
        oldbase[itr->lvl + 1] = oldpos;
        rs_unrelative(oldbase[itr->lvl], &oldbase[itr->lvl + 1]);

        rs_marktree_itr_next_skip(b, itr,false, false, oldbase, NULL);
      } else {
        if (itr->i < itr->x->n - 1) {
          itr->i++;
          goto past_continue_same_node;
        } else {
          rs_marktree_itr_next(b, itr);
        }
      }
    }
  }

  while (itr->x) {
    rs_unrelative(oldbase[itr->lvl], &rawkey(itr).pos);
    int realrow = rawkey(itr).pos.row;
    assert(realrow >= old_extent.row);
    bool done = false;
    if (realrow == old_extent.row) {
      if (delta.col) {
        rawkey(itr).pos.col += delta.col;
      }
    } else {
      if (same_line) {
        // optimization: column only adjustment can skip remaining rows
        done = true;
      }
    }
    if (delta.row) {
      rawkey(itr).pos.row += delta.row;
      moved = true;
    }
    rs_relative(itr->pos, &rawkey(itr).pos);
    if (done) {
      break;
    }
    rs_marktree_itr_next_skip(b, itr,true, false, NULL, NULL);
  }

  if (kv_size(damage)) {
    // TODO(bfredl): a full sort is not really needed. we just need a "start" node to find
    // its corresponding "end" node. Set up some dedicated hash for this later (c.f. the
    // "grow only" variant of khash_t branch)
    qsort((void *)&kv_A(damage, 0), kv_size(damage), sizeof(kv_A(damage, 0)),
          damage_cmp);

    for (size_t i = 0; i < kv_size(damage); i++) {
      Damage d = kv_A(damage, i);
      assert(i == 0 || d.id > kv_A(damage, i - 1).id);
      if (!(d.id & MARKTREE_END_FLAG)) {  // start
        if (i + 1 < kv_size(damage) && kv_A(damage, i + 1).id == (d.id | MARKTREE_END_FLAG)) {
          Damage d2 = kv_A(damage, i + 1);

          // pair
          rs_marktree_itr_set_node(b,itr, d.old, d.old_i);
          rs_marktree_itr_set_node(b,enditr, d2.old, d2.old_i);
          marktree_intersect_pair(b, d.id, itr, enditr, true);
          rs_marktree_itr_set_node(b,itr, d.new, d.new_i);
          rs_marktree_itr_set_node(b,enditr, d2.new, d2.new_i);
          marktree_intersect_pair(b, d.id, itr, enditr, false);

          i++;  // consume two items
          continue;
        }

        // d is lone start, end didn't move
        MarkTreeIter endpos[1];
        rs_marktree_lookup(b, d.id | MARKTREE_END_FLAG, endpos);
        if (endpos->x) {
          rs_marktree_itr_set_node(b,itr, d.old, d.old_i);
          *enditr = *endpos;
          marktree_intersect_pair(b, d.id, itr, enditr, true);
          rs_marktree_itr_set_node(b,itr, d.new, d.new_i);
          *enditr = *endpos;
          marktree_intersect_pair(b, d.id, itr, enditr, false);
        }
      } else {
        // d is lone end, start didn't move
        MarkTreeIter startpos[1];
        uint64_t start_id = d.id & ~MARKTREE_END_FLAG;

        rs_marktree_lookup(b, start_id, startpos);
        if (startpos->x) {
          *itr = *startpos;
          rs_marktree_itr_set_node(b,enditr, d.old, d.old_i);
          marktree_intersect_pair(b, start_id, itr, enditr, true);
          *itr = *startpos;
          rs_marktree_itr_set_node(b,enditr, d.new, d.new_i);
          marktree_intersect_pair(b, start_id, itr, enditr, false);
        }
      }
    }
  }
  kvi_destroy(damage);

  return moved;
}

void marktree_move_region(MarkTree *b, int start_row, colnr_T start_col, int extent_row,
                          colnr_T extent_col, int new_row, colnr_T new_col)
{
  rs_marktree_move_region(b, start_row, start_col, extent_row, extent_col, new_row, new_col);
}


// for unit test
void marktree_put_test(MarkTree *b, uint32_t ns, uint32_t id, int row, int col, bool right_gravity,
                       int end_row, int end_col, bool end_right, bool meta_inline)
{
  rs_marktree_put_test(b, ns, id, row, col, right_gravity, end_row, end_col, end_right,
                       meta_inline);
}

// for unit test
bool mt_right_test(MTKey key)
{
  return rs_mt_right_test(key);
}

// for unit test
void marktree_del_pair_test(MarkTree *b, uint32_t ns, uint32_t id)
{
  rs_marktree_del_pair_test(b, ns, id);
}

void marktree_check(MarkTree *b)
{
#ifndef NDEBUG
  if (b->root == NULL) {
    assert(b->n_keys == 0);
    assert(b->n_nodes == 0);
    assert(b->id2node == NULL || map_size(b->id2node) == 0);
    return;
  }

  MTPos dummy;
  bool last_right = false;

  size_t nkeys = marktree_check_node(b, b->root, &dummy, &last_right, b->meta_root);
  assert(b->n_keys == nkeys);
  assert(b->n_keys == map_size(b->id2node));
#else
  // Do nothing, as assertions are required
  (void)b;
#endif
}

size_t marktree_check_node(MarkTree *b, MTNode *x, MTPos *last, bool *last_right,
                           const uint32_t *meta_node_ref)
{
  assert(x->n <= 2 * T - 1);
  // TODO(bfredl): too strict if checking "in repair" post-delete tree.
  assert(x->n >= (x != b->root ? T - 1 : 0));
  size_t n_keys = (size_t)x->n;

  for (int i = 0; i < x->n; i++) {
    if (x->level) {
      n_keys += marktree_check_node(b, x->ptr[i], last, last_right, x->meta[i]);
    } else {
      *last = (MTPos) { 0, 0 };
    }
    if (i > 0) {
      rs_unrelative(x->key[i - 1].pos, last);
    }
    assert(rs_pos_leq(*last, x->key[i].pos));
    if (last->row == x->key[i].pos.row && last->col == x->key[i].pos.col) {
      assert(!*last_right || mt_right(x->key[i]));
    }
    *last_right = mt_right(x->key[i]);
    assert(x->key[i].pos.col >= 0);
    assert(pmap_get(uint64_t)(b->id2node, mt_lookup_key(x->key[i])) == x);
  }

  if (x->level) {
    n_keys += marktree_check_node(b, x->ptr[x->n], last, last_right, x->meta[x->n]);
    rs_unrelative(x->key[x->n - 1].pos, last);

    for (int i = 0; i < x->n + 1; i++) {
      assert(x->ptr[i]->parent == x);
      assert(x->ptr[i]->p_idx == i);
      assert(x->ptr[i]->level == x->level - 1);
      // PARANOIA: check no double node ref
      for (int j = 0; j < i; j++) {
        assert(x->ptr[i] != x->ptr[j]);
      }
    }
  } else if (x->n > 0) {
    *last = x->key[x->n - 1].pos;
  }

  uint32_t meta_node[kMTMetaCount];
  rs_meta_describe_node(meta_node, x);
  for (int m = 0; m < kMTMetaCount; m++) {
    assert(meta_node_ref[m] == meta_node[m]);
  }

  return n_keys;
}

bool marktree_check_intersections(MarkTree *b)
{
  if (!b->root) {
    return true;
  }
  PMap(ptr_t) checked = MAP_INIT;

  // 1. move x->intersect to checked[x] and reinit x->intersect
  mt_recurse_nodes(b->root, &checked);

  // 2. iterate over all marks. for each START mark of a pair,
  // intersect the nodes between the pair
  MarkTreeIter itr[1];
  rs_marktree_itr_first(b, itr);
  while (true) {
    MTKey mark = rs_marktree_itr_current(itr);
    if (mark.pos.row < 0) {
      break;
    }

    if (mt_start(mark)) {
      MarkTreeIter start_itr[1];
      MarkTreeIter end_itr[1];
      uint64_t end_id = mt_lookup_id(mark.ns, mark.id, true);
      MTKey k = rs_marktree_lookup(b, end_id, end_itr);
      if (k.pos.row >= 0) {
        *start_itr = *itr;
        marktree_intersect_pair(b, mt_lookup_key(mark), start_itr, end_itr, false);
      }
    }

    rs_marktree_itr_next(b, itr);
  }

  // 3. for each node check if the recreated intersection
  // matches the old checked[x] intersection.
  bool status = mt_recurse_nodes_compare(b->root, &checked);

  uint64_t *val;
  map_foreach_value(&checked, val, {
    xfree(val);
  });
  map_destroy(ptr_t, &checked);

  return status;
}

void mt_recurse_nodes(MTNode *x, PMap(ptr_t) *checked)
{
  if (kv_size(x->intersect)) {
    kvi_push(x->intersect, (uint64_t)-1);  // sentinel
    uint64_t *val;
    if (x->intersect.items == x->intersect.init_array) {
      val = xmemdup(x->intersect.items, x->intersect.size * sizeof(*x->intersect.items));
    } else {
      val = x->intersect.items;
    }
    pmap_put(ptr_t)(checked, x, val);
    kvi_init(x->intersect);
  }

  if (x->level) {
    for (int i = 0; i < x->n + 1; i++) {
      mt_recurse_nodes(x->ptr[i], checked);
    }
  }
}

bool mt_recurse_nodes_compare(MTNode *x, PMap(ptr_t) *checked)
{
  uint64_t *ref = pmap_get(ptr_t)(checked, x);
  if (ref != NULL) {
    for (size_t i = 0;; i++) {
      if (ref[i] == (uint64_t)-1) {
        if (i != kv_size(x->intersect)) {
          return false;
        }

        break;
      } else {
        if (kv_size(x->intersect) <= i || ref[i] != kv_A(x->intersect, i)) {
          return false;
        }
      }
    }
  } else {
    if (kv_size(x->intersect)) {
      return false;
    }
  }

  if (x->level) {
    for (int i = 0; i < x->n + 1; i++) {
      if (!mt_recurse_nodes_compare(x->ptr[i], checked)) {
        return false;
      }
    }
  }

  return true;
}

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

MTNode *nvim_marktree_id2node(MarkTree *b, uint64_t id) { return id2node(b, id); }

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

MTNode *nvim_marktree_alloc_node(MarkTree *b, bool internal) { return marktree_alloc_node(b, internal); }
void nvim_marktree_refkey(MarkTree *b, MTNode *x, int i) { refkey(b, x, i); }
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

uint64_t nvim_marktree_del_itr(MarkTree *b, MarkTreeIter *itr, bool rev) { return marktree_del_itr(b, itr, rev); }
void nvim_marktree_revise_meta(MarkTree *b, MarkTreeIter *itr, MTKey old_key) { marktree_revise_meta(b, itr, old_key); }
void nvim_marktree_move(MarkTree *b, MarkTreeIter *itr, int row, int col) { marktree_move(b, itr, row, col); }
void nvim_marktree_restore_pair(MarkTree *b, MTKey key) { marktree_restore_pair(b, key); }
void nvim_pivot_right(MarkTree *b, MTPos p_pos, MTNode *p, int i) { pivot_right(b, p_pos, p, i); }
void nvim_pivot_left(MarkTree *b, MTPos p_pos, MTNode *p, int i) { pivot_left(b, p_pos, p, i); }
MTNode *nvim_merge_node(MarkTree *b, MTNode *p, int i) { return merge_node(b, p, i); }
void nvim_marktree_del_id(MarkTree *b, uint64_t id) { pmap_del(uint64_t)(b->id2node, id, NULL); }
void nvim_marktree_dec_n_keys(MarkTree *b) { b->n_keys--; }
MarkTreeIter *nvim_alloc_marktreeiter(void) { return xcalloc(1, sizeof(MarkTreeIter)); }
void nvim_free_marktreeiter(MarkTreeIter *itr) { xfree(itr); }
void nvim_marktree_sub_meta_root(MarkTree *b, int m, uint32_t val) { b->meta_root[m] -= val; }
MTKey nvim_rawkey(MarkTreeIter *itr) { return rawkey(itr); }
void nvim_rawkey_set_flags(MarkTreeIter *itr, uint16_t flags) { rawkey(itr).flags = flags; }
void nvim_rawkey_or_flags(MarkTreeIter *itr, uint16_t flags) { rawkey(itr).flags |= flags; }
void nvim_rawkey_clear_flags(MarkTreeIter *itr, uint16_t flags) { rawkey(itr).flags &= (uint16_t)~flags; }

// ============================================================================
// Splice Operations (for Rust FFI)
// ============================================================================

bool nvim_marktree_splice(MarkTree *b, int32_t start_line, int start_col,
                          int old_extent_line, int old_extent_col,
                          int new_extent_line, int new_extent_col) { return marktree_splice(b, start_line, start_col, old_extent_line, old_extent_col, new_extent_line, new_extent_col); }
void nvim_marktree_move_region(MarkTree *b, int start_row, colnr_T start_col,
                               int extent_row, colnr_T extent_col,
                               int new_row, colnr_T new_col) { marktree_move_region(b, start_row, start_col, extent_row, extent_col, new_row, new_col); }

// ============================================================================
// Debug and Validation (for Rust FFI)
// ============================================================================

void nvim_marktree_check(MarkTree *b) { marktree_check(b); }

// ============================================================================
// MTKey Accessor Functions (for Rust sign crate)
// ============================================================================

int32_t nvim_mtkey_get_row(MTKey key) { return key.pos.row; }
uint32_t nvim_mtkey_get_ns(MTKey key) { return key.ns; }
uint32_t nvim_mtkey_get_id(MTKey key) { return key.id; }
