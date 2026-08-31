local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local eq = t.eq
local eval = n.eval
local api = n.api
local clear = n.clear
local command = n.command

before_each(clear)

describe('extend()', function()
  it('succeeds to extend list with itself', function()
    api.nvim_set_var('l', { 1, {} })
    eq({ 1, {}, 1, {} }, eval('extend(l, l)'))
    eq({ 1, {}, 1, {} }, api.nvim_get_var('l'))

    api.nvim_set_var('l', { 1, {} })
    eq({ 1, {}, 1, {} }, eval('extend(l, l, 0)'))
    eq({ 1, {}, 1, {} }, api.nvim_get_var('l'))

    api.nvim_set_var('l', { 1, {} })
    eq({ 1, 1, {}, {} }, eval('extend(l, l, 1)'))
    eq({ 1, 1, {}, {} }, api.nvim_get_var('l'))
  end)
end)

-- A Dictionary is a hash table and `keys()` walks its slots in index order, so
-- the order is a function of the hash and the probe sequence, not of the
-- insertion order. Every listing that iterates a Dictionary (`keys()`,
-- `values()`, `items()`, `:echo` of a dict, `string()`) inherits it, which
-- makes the placement observable behaviour rather than an internal detail.
--
-- The expectations below are hand-computed from the hash fold
-- (`hash = hash * 101 + byte`, seeded with the first byte) and the perturbed
-- probe sequence (`idx = idx * 5 + perturb + 1`, `perturb >>= 5`, masked to
-- the table size), so they pin the placement itself and not merely "some
-- stable order".
describe('Dictionary iteration order', function()
  it('is slot order, not insertion order', function()
    -- A Dictionary starts on a 16-slot table. "a" (97), "q" (113), "A" (65)
    -- and "Q" (81) all mask to slot 1, so the last three walk on to 7, 6 and
    -- 15; "b" (98) and "r" (114) both mask to 2, so "r" walks on to 13.
    command('let g:d = {}')
    for _, key in ipairs({ 'a', 'q', 'A', 'Q', 'b', 'r' }) do
      command(('let g:d[%q] = 1'):format(key))
    end
    eq({ 'a', 'b', 'A', 'q', 'r', 'Q' }, eval('keys(g:d)'))
  end)

  it('reuses the slot a removed key left behind', function()
    -- "a" at 1, "A" at 6, "q" at 7. Removing "q" leaves a tombstone there,
    -- and "Q" -- whose probe sequence is 1, 7, 6, 15 -- stops at the first
    -- tombstone it crosses rather than at the empty slot that ends the walk,
    -- so it takes slot 7 instead of 15.
    command('let g:d = {}')
    for _, key in ipairs({ 'a', 'q', 'A' }) do
      command(('let g:d[%q] = 1'):format(key))
    end
    command('unlet g:d["q"]')
    command('let g:d["Q"] = 1')
    eq({ 'a', 'A', 'Q' }, eval('keys(g:d)'))
  end)

  it('survives the rehash that growing off the initial table forces', function()
    -- The 16th key trips the load factor and the table quadruples to 64
    -- slots, rehashing every key by its stored hash. "k18" and "k19" mask
    -- ahead of "k0" on the bigger table, which is exactly the reordering a
    -- caller sees across a growth.
    command('let g:d = {}')
    for i = 0, 19 do
      command(('let g:d["k%d"] = %d'):format(i, i))
    end
    local expected = { 'k18', 'k19' }
    for i = 0, 17 do
      expected[#expected + 1] = ('k%d'):format(i)
    end
    eq(expected, eval('keys(g:d)'))
  end)
end)
