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

-- filter()/map()/foreach() lock what they walk, but the callback can
-- `:unlockvar` it and then edit it. Upstream then wrote into (or stepped
-- from) a freed item; here the walk carries on from wherever the edit left
-- it, and an item the callback removed is neither replaced nor removed again.
describe('a callback that unlocks the container being walked', function()
  local exec = n.exec

  it('may empty a List under map()/filter()/foreach()/mapnew()', function()
    exec([[
      func Empty(i, v)
        unlockvar g:l
        call remove(g:l, 0, -1)
        return 9
      endfunc
    ]])
    for _, fn in ipairs({ 'map', 'filter', 'foreach' }) do
      command('let g:l = [1, 2, 3, 4]')
      command('call ' .. fn .. "(g:l, function('Empty'))")
      eq({}, eval('g:l'), fn)
    end
    command('let g:l = [1, 2, 3, 4]')
    eq({ 9 }, eval("mapnew(g:l, function('Empty'))"))
  end)

  it('maps the item where the edit moved it, and skips one it removed', function()
    exec([[
      func DropFirst(i, v)
        if a:i == 1
          unlockvar g:l
          call remove(g:l, 0)
        endif
        return a:v * 10
      endfunc
      func PushFront(i, v)
        if a:i == 0
          unlockvar g:l
          call insert(g:l, 0)
        endif
        return a:v * 10
      endfunc
      func DropSelf(i, v)
        if a:v == 2
          unlockvar g:l
          call remove(g:l, index(g:l, 2))
        endif
        return 0
      endfunc
    ]])
    command('let g:l = [1, 2, 3, 4]')
    eq({ 20, 30, 40 }, eval("map(g:l, function('DropFirst'))"))
    command('let g:l = [1, 2, 3]')
    eq({ 0, 10, 20, 30 }, eval("map(g:l, function('PushFront'))"))
    command('let g:l = [1, 2, 3]')
    eq({}, eval("filter(g:l, function('DropSelf'))"))
  end)

  it('may empty a Dictionary under map()/filter()/foreach()/mapnew()', function()
    exec([[
      func Empty(k, v)
        unlockvar g:d
        for k in keys(g:d) | call remove(g:d, k) | endfor
        return 0
      endfunc
      func Refill(k, v)
        unlockvar g:d
        for k in keys(g:d) | call remove(g:d, k) | endfor
        let g:d.zz = 5
        return 7
      endfunc
    ]])
    for _, fn in ipairs({ 'map', 'filter', 'foreach' }) do
      command("let g:d = {'a': 1, 'b': 2, 'c': 3}")
      command('call ' .. fn .. "(g:d, function('Empty'))")
      eq({}, eval('g:d'), fn)
    end
    command("let g:d = {'a': 1, 'b': 2, 'c': 3}")
    eq(1, eval("len(mapnew(g:d, function('Empty')))"))
    -- A key added in a slot the walk has yet to reach is visited.
    command("let g:d = {'a': 1, 'b': 2, 'c': 3}")
    eq({ zz = 7 }, eval("map(g:d, function('Refill'))"))
  end)

  it('may empty a Blob under map()/filter()/mapnew()', function()
    exec([[
      func Empty(i, v)
        unlockvar g:b
        call remove(g:b, 0, -1)
        return 9
      endfunc
    ]])
    for _, fn in ipairs({ 'map', 'filter' }) do
      command('let g:b = 0z01020304')
      command('call ' .. fn .. "(g:b, function('Empty'))")
      eq('0z', eval('string(g:b)'), fn)
    end
    command('let g:b = 0z01020304')
    eq('0z09020304', eval("string(mapnew(g:b, function('Empty')))"))
  end)

  it('indexof() and uniq() survive a callback that shortens the list', function()
    command('let g:b = 0z01020304')
    eq(-1, eval('indexof(g:b, {i, v -> remove(g:b, 0, -1) is 0})'))
    command('let g:l = [1, 1, 1, 1]')
    eq({}, eval('uniq(g:l, {a, b -> len(remove(g:l, 0, -1)) * 0})'))
  end)

  it('matchfuzzy() visits only the items the list had', function()
    command("let g:l = [{'t': 'aa'}, {'t': 'ab'}]")
    eq(
      2,
      eval("len(matchfuzzy(g:l, 'a', {'text_cb': {v -> add(g:l, {'t': 'ax'}) is 0 ? 'a' : 'a'}}))")
    )
    eq(4, eval('len(g:l)'))
  end)
end)
