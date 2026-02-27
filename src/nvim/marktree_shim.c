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

// Phase 2 (pass 2): intersection set operations
extern void rs_merge_node_intersect(MarkTree *b, MTNode *x, int x_old_n, MTNode *y, int y_n);
extern void rs_pivot_right_intersect(MarkTree *b, MTNode *x, MTNode *y, int y_n);
extern void rs_pivot_left_intersect(MarkTree *b, MTNode *x, int x_n, MTNode *y);

// Phase 5 (pass 5): deletion rebalancing
extern MTNode *rs_merge_node(MarkTree *b, MTNode *p, int i);
extern bool rs_intersect_mov_test(const uint64_t *x, size_t nx, const uint64_t *y, size_t ny,
                                  const uint64_t *win, size_t nwin, uint64_t *wout, size_t *nwout,
                                  uint64_t *dout, size_t *ndout);

#define T MT_BRANCH_FACTOR
#define ILEN (sizeof(MTNode) + sizeof(struct mtnode_inner_s))

#define ID_INCR (((uint64_t)1) << 2)

#define rawkey(itr) ((itr)->x->key[(itr)->i])

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


static MTNode *marktree_alloc_node(MarkTree *b, bool internal)
{
  MTNode *x = xcalloc(1, internal ? ILEN : sizeof(MTNode));
  kvi_init(x->intersect);
  b->n_nodes++;
  return x;
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
      rs_marktree_intersect_pair(b, id, this_itr, other_itr, true);
    } else {
      rs_marktree_intersect_pair(b, other, other_itr, itr, true);
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
      rs_marktree_free_node(b, oldroot);
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
  return rs_merge_node(b, p, i);
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


/// @param itr iterator is invalid after call
void marktree_move(MarkTree *b, MarkTreeIter *itr, int row, int col)
{
  rs_marktree_move(b, itr, row, col);
}

void marktree_restore_pair(MarkTree *b, MTKey key)
{
  rs_marktree_restore_pair(b, key);
}

bool marktree_splice(MarkTree *b, int32_t start_line, int start_col, int old_extent_line,
                     int old_extent_col, int new_extent_line, int new_extent_col)
{
  return rs_marktree_splice(b, start_line, start_col, old_extent_line, old_extent_col,
                            new_extent_line, new_extent_col);
}

void marktree_move_region(MarkTree *b, int start_row, colnr_T start_col, int extent_row,
                          colnr_T extent_col, int new_row, colnr_T new_col)
{
  rs_marktree_move_region(b, start_row, start_col, extent_row, extent_col, new_row, new_col);
}



// for unit test
void marktree_del_pair_test(MarkTree *b, uint32_t ns, uint32_t id)
{
  rs_marktree_del_pair_test(b, ns, id);
}

void marktree_check(MarkTree *b)
{
#ifndef NDEBUG
  rs_marktree_check(b);
#else
  (void)b;
#endif
}

bool marktree_check_intersections(MarkTree *b)
{
  return rs_marktree_check_intersections(b);
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
