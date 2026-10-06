" Differential: list and dict ownership, references, cursors and the collector.
" Run headless; every line goes to the file named by $DIFFOUT.
" Nothing here may depend on the build, only on observable behaviour.

let s:out = []
func! S(...) abort
  call add(s:out, join(map(copy(a:000), 'type(v:val) == v:t_string ? v:val : string(v:val)'), ' '))
endfunc
func! T(what, expr) abort
  try
    call S(a:what, string(eval(a:expr)))
  catch
    call S(a:what, 'threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
  endtry
endfunc

" ------------------------------------------------------------ 1. sharing
" A list is a reference: two names, one object, and the copy takes a
" reference rather than duplicating.
call S('=== 1. sharing ===')
let a = [1, 2, 3]
let b = a
let b[0] = 99
call S('aliased', string(a), string(b), a is b)
let c = copy(a)
let c[1] = 77
call S('copy is shallow', string(a), string(c), a is c)
let nest = [[1], [2]]
let sh = copy(nest)
let sh[0][0] = 55
call S('shallow shares children', string(nest), string(sh), nest[0] is sh[0])
let dp = deepcopy(nest)
let dp[1][0] = 66
call S('deepcopy is deep', string(nest), string(dp), nest[1] is dp[1])
" deepcopy's memo: one object referenced twice stays one object.
let inner = [1, 2]
let twice = [inner, inner]
let dd = deepcopy(twice)
call S('deepcopy memo', dd[0] is dd[1], string(dd))
let dd[0][0] = 42
call S('deepcopy memo write', string(dd))
" deepcopy of a cycle
let cyc = [1]
call add(cyc, cyc)
let dc = deepcopy(cyc)
call S('deepcopy cycle', dc[1] is dc, len(dc), string(dc[0]))
" copy of a cycle
let cc = copy(cyc)
call S('copy cycle', cc[1] is cyc, len(cc))
unlet cyc cc dc

" A list handed to a function and kept
func! Keep(l) abort
  let s:kept = a:l
  return a:l
endfunc
let orig = [1, 2]
call Keep(orig)
let s:kept[0] = 5
call S('kept by callee', string(orig), orig is s:kept)
unlet s:kept

" a:000 is a real list, and outlives the call only if copied
func! Args(...) abort
  return [len(a:000), string(a:000), copy(a:000)]
endfunc
call S('a:000', string(Args(1, [2], {'a': 3})))
func! ArgsEscape(...) abort
  return a:000
endfunc
let esc = ArgsEscape(1, 2, 3)
call S('a:000 escapes', string(esc))
call garbagecollect(1)
call S('a:000 after gc', string(esc))
unlet esc

" ------------------------------------------------------- 2. the :for cursor
call S('=== 2. for cursors ===')
let l = [0, 1, 2, 3, 4, 5]
let seen = []
for x in l
  call add(seen, x)
  if x == 2 && len(l) > 3
    call remove(l, 2)
  endif
endfor
call S('remove current', string(seen), string(l))

let l = [0, 1, 2, 3, 4, 5]
let seen = []
for x in l
  call add(seen, x)
  if x == 1
    call remove(l, 1, 3)
  endif
endfor
call S('remove run', string(seen), string(l))

let l = [0, 1, 2]
let seen = []
let n = 0
for x in l
  call add(seen, x)
  let n += 1
  if n > 20 | break | endif
  call add(l, 100 + n)
endfor
call S('append while walking', string(seen), len(l))

let l = [0, 1, 2, 3]
let seen = []
let once = 0
for x in l
  call add(seen, x)
  if x == 0 && !once
    let once = 1
    call insert(l, -1, 0)
  endif
endfor
call S('insert at front', string(seen), string(l))

let l = [0, 1, 2, 3, 4]
let seen = []
let once = 0
for x in l
  call add(seen, x)
  if x == 1 && !once
    let once = 1
    call reverse(l)
  endif
endfor
call S('reverse while walking', string(seen), string(l))

let l = [5, 3, 1, 4, 2]
let seen = []
let once = 0
for x in l
  call add(seen, x)
  if x == 3 && !once
    let once = 1
    call sort(l)
  endif
endfor
call S('sort while walking', string(seen), string(l))

" nested loops over the same list
let l = [1, 2, 3]
let pairs = []
for x in l
  for y in l
    call add(pairs, x * 10 + y)
  endfor
endfor
call S('nested for', string(pairs))

" the loop holds a reference: the list survives its name going away
let l = [1, 2, 3]
let seen = []
for x in l
  call add(seen, x)
  if x == 1
    unlet l
    call garbagecollect(1)
  endif
endfor
call S('unlet while walking', string(seen))

" :for over a nested list unpacks
for [p, q] in [[1, 2], [3, 4]]
  call S('unpack', p, q)
endfor

" ------------------------------------------------------- 3. edits and moves
call S('=== 3. edits ===')
let l = [1, 2, 3]
call extend(l, l)
call S('extend with self', string(l))
let l = [1, 2, 3, 4]
call extend(l, l, 2)
call S('extend with self at 2', string(l))
let l = [1, 2, 3]
call extend(l, l, 0)
call S('extend with self at 0', string(l))
let l = [1, 2, 3]
call S('extendnew', string(extendnew(l, [4, 5])), string(l))
let l = [1, 2, 3]
call S('insert', string(insert(l, 0)), string(insert(l, 9, 2)))
let l = [1, 2, 3, 4, 5]
call S('remove range', string(remove(l, 1, 3)), string(l))
let l = [1, 2, 3, 4, 5]
call S('remove neg', string(remove(l, -2)), string(l))
let l = [[1, [2, [3]]], 4]
call S('flatten', string(flatten(deepcopy(l))), string(flatten(deepcopy(l), 1)))
call S('flattennew', string(flattennew([[1], [[2]]])), string(flattennew([[1], [[2]]], 1)))
let l = [1]
call add(l, l)
call S('self add', len(l), l[1] is l)
let l = [3, 1, 2]
call S('sort/uniq/reverse', string(sort(copy(l))), string(reverse(copy(l))), string(uniq(sort([1,1,2,2,3]))))
call S('sort numeric', string(sort([10, 9, 2], 'n')), string(sort(['b', 'A', 'a'], 'i')))
func! Cmp(a, b) abort
  return a:a == a:b ? 0 : (a:a > a:b ? 1 : -1)
endfunc
call S('sort funcref', string(sort([3, 1, 2], function('Cmp'))))
call S('repeat', string(repeat([1, 2], 3)), string(repeat([], 4)))
let l = [1, 2, 3, 4, 5]
call S('slices', string(l[1:3]), string(l[:2]), string(l[3:]), string(l[-2:]), string(l[9:]))
let l = [1, 2, 3, 4, 5]
let l[1:3] = [9, 9, 9]
call S('slice assign', string(l))
let l = [1, 2, 3]
let l += [4]
call S('+= list', string(l))
let l = [1, 2, 3, 4]
let l[0:1] = l[2:3]
call S('self slice assign', string(l))
call S('add returns the list', string(add([1], 2)))
call S('index/indexof', index([1,2,3], 2), string(indexof([1,2,3], 'v:val == 3')))
call S('count/max/min', count([1,1,2], 1), max([3,1,2]), min([3,1,2]))
call S('join/split', join([1,2,3], '-'), string(split('a,b,c', ',')))
call S('reduce', reduce([1,2,3], {a, b -> a + b}, 0))

" filter/map/mapnew, including a callback that edits the list it walks
call S('=== 3b. filter/map ===')
let l = [1, 2, 3, 4, 5]
call S('filter', string(filter(copy(l), 'v:val % 2')))
call S('map', string(map(copy(l), 'v:val * 2')))
call S('mapnew', string(mapnew(l, 'v:val * 3')), string(l))
let l = [1, 2, 3, 4]
call S('map to lists', string(map(copy(l), '[v:val]')))
let l = [1, 2, 3, 4, 5]
func! Bomb(i, v) abort
  if a:v == 2
    call remove(s:bl, 4)
  endif
  return a:v
endfunc
let s:bl = copy(l)
call S('map removing', string(map(s:bl, function('Bomb'))))
let s:bl = copy(l)
func! Grow(i, v) abort
  if a:v == 1
    call add(s:bl, 99)
  endif
  return a:v
endfunc
call S('map growing', string(map(s:bl, function('Grow'))))
unlet s:bl

" ------------------------------------------------------------ 4. lockvar
call S('=== 4. lockvar ===')
let g:lk = [1, [2, 3]]
lockvar g:lk
call T('locked add', 'add(g:lk, 4)')
call T('locked inner add', 'add(g:lk[1], 4)')
unlockvar g:lk
lockvar! g:lk
call T('deep locked inner', 'add(g:lk[1], 4)')
unlockvar! g:lk
call S('unlocked again', string(add(g:lk, 4)))
let g:lk = [1, 2]
lockvar 1 g:lk
call T('depth 1 write item', 'extend(g:lk, [3])')
call T('depth 1 write inner', 'add(g:lk, 3)')
unlockvar 1 g:lk
" locking through a shared reference
let g:lx = [1]
let g:ly = g:lx
lockvar g:lx
call T('shared lock', 'add(g:ly, 2)')
unlockvar g:lx
call S('islocked', islocked('g:lx'), islocked('g:lk'))
lockvar g:lk
call S('islocked locked', islocked('g:lk'))
unlockvar g:lk
" a function argument is not locked by lockvar inside
func! LockArg(l) abort
  lockvar a:l
  return islocked('a:l')
endfunc
let g:free = [1]
call S('lock arg', LockArg(g:free))
call T('lock arg after', 'add(g:free, 2)')
unlet g:lk g:lx g:ly g:free

" ----------------------------------------------------- 5. gc and lifetimes
call S('=== 5. gc ===')
" cycles through lists, dicts, partials and funcrefs
func! MakeCycles(n) abort
  let keep = []
  for i in range(a:n)
    let l = [i]
    let d = {'l': l}
    call add(l, d)
    let P = function("MakeCycles", [l])
    let d.p = P
    call add(l, P)
    if i % 7 == 0
      call add(keep, l)
    endif
  endfor
  return keep
endfunc
let kept = MakeCycles(200)
call S('cycles kept', len(kept), len(kept[0]))
call garbagecollect(1)
call S('after gc', len(kept), len(kept[0]), string(kept[0][0]))
unlet kept
call garbagecollect(1)
call S('after second gc')
" a closure keeps its captures alive
func! Closure() abort
  let hidden = [1, 2, 3]
  return {-> hidden}
endfunc
let Cl = Closure()
call garbagecollect(1)
call S('closure capture', string(Cl()))
unlet Cl
" self-referencing list survives a collection while named
let sl = [1]
call add(sl, sl)
call garbagecollect(1)
call S('named cycle', len(sl), sl[1] is sl)
unlet sl
call garbagecollect(1)
call S('unnamed cycle collected')
" a list held only by a dict value, and vice versa
let holder = {'items': [1, 2, 3]}
let items = holder.items
unlet holder
call garbagecollect(1)
call S('value outlives dict', string(items))
unlet items
" remove() hands ownership over
let src = [[1], [2]]
let taken = remove(src, 0)
unlet src
call garbagecollect(1)
call S('removed item owned', string(taken))
unlet taken
" many lists at once
let many = []
for i in range(500)
  call add(many, [i, [i], {'k': i}])
endfor
call S('many', len(many), string(many[499]))
let many = 0
call garbagecollect(1)
call S('many freed')
" garbagecollect() at the top level and in a function
func! GcInside() abort
  let l = [1, 2, 3]
  call garbagecollect(1)
  return l
endfunc
call S('gc inside', string(GcInside()))

" ------------------------------------------------------ 6. null and empty
call S('=== 6. null / empty ===')
call S('null list', string(v:_null_list), len(v:_null_list), empty(v:_null_list))
call T('null list add', 'add(v:_null_list, 1)')
call S('null copies', string(copy(v:_null_list)), string(deepcopy(v:_null_list)))
call S('null join', string(join(v:_null_list, ',')))
call S('null for', string(map(copy(v:_null_list), 'v:val')))
let nn = 0
for x in v:_null_list
  let nn += 1
endfor
call S('null for count', nn)
call S('null index', index(v:_null_list, 1))
call S('empty list', string([]), string(add([], 1)))
call S('null is null', v:_null_list is v:_null_list, [] is [])
call S('null compare', v:_null_list == [], [] == [])

" --------------------------------------------------------- 7. id and print
call S('=== 7. identity ===')
let p = [1]
let q = p
call S('id equal', id(p) == id(q), id(p) == id([1]))
call S('id null', id(v:_null_list) == id(v:_null_list))
call S('string', string([1, 'a', [2], {'k': 3}, 1.5, v:true, v:null]))
call S('string empty', string([]), string([[]]))
let selfl = [1]
call add(selfl, selfl)
call T('string self', 'string(selfl)')
unlet selfl
call S('type', type([]), type({}))
call S('deep nest string', string(range(3)))

" ---------------------------------------------------------- 8. encodings
call S('=== 8. encodings ===')
call S('json list', json_encode([1, 'a', [2], {'k': 3}]))
call S('json decode', string(json_decode('[1,"a",[2],{"k":3}]')))
call S('json null list', json_encode(v:_null_list))
call S('msgpack', string(msgpackdump([[1, 2, 3]])))
call S('msgpack roundtrip', string(msgpackparse(msgpackdump([[1, [2], {'a': 3}]]))))
call S('msgpack big int', string(msgpackparse(msgpackdump([18446744073709551615]))))
let big = msgpackparse(msgpackdump([18446744073709551615]))
call S('msgpack big shape', string(big))
call S('str2list/list2str', string(str2list('abc')), list2str([97, 98, 99]))
call S('flatten json', json_encode(flattennew([[1], [2, [3]]])))
echo [1, 2, 3]
call S('echo done')

" ----------------------------------------------------- 9. submatch lists
call S('=== 9. submatch ===')
call S('substitute expr', substitute('abc', '\(a\)\(b\)', '\=submatch(0) . "-" . string(submatch(1))', ''))
call S('submatch list', substitute('abc', '\(a\)\(b\)\(c\)', '\=string(submatch(0, 1))', ''))
let s:kept_sub = []
func! KeepSub() abort
  let s:kept_sub = submatch(0, 1)
  return 'x'
endfunc
call S('sub kept', substitute('abc', 'a\(b\)', '\=KeepSub()', ''))
call garbagecollect(1)
call S('sub kept after gc', string(s:kept_sub))

" --------------------------------------------------- 10. the api surface
call S('=== 10. api ===')
call S('nvim_eval list', string(nvim_eval('[1, [2], {"a": 3}]')))
call nvim_set_var('apil', [1, [2], 3])
call S('api var', string(nvim_get_var('apil')))
call nvim_del_var('apil')
call S('api call', string(nvim_call_function('reverse', [[1, 2, 3]])))
call S('api list_bufs', type(nvim_list_bufs()))
call S('lua roundtrip', string(luaeval('{1, 2, 3}')))
call S('lua empty', string(luaeval('{}')))
call S('lua nested', string(luaeval('{1, {2, 3}, {a = 4}}')))
lua vim.g.lual = {1, 2, 3}
call S('lua g var', string(g:lual))
unlet g:lual
call S('lua fn', string(luaeval('vim.fn.sort({3, 1, 2})')))
call S('lua tbl held', string(luaeval('(function() local t = {1,2}; return t end)()')))
lua _G.held = vim.fn.range(3)
call S('lua holds a list', string(luaeval('#_G.held')))
call garbagecollect(1)
call S('lua holds after gc', string(luaeval('_G.held')))
lua _G.held = nil
call S('getqflist', type(getqflist()), string(getqflist()))
call setqflist([{'filename': 'x', 'lnum': 1, 'text': 'hi'}])
call S('setqflist', len(getqflist()), getqflist()[0].text)
call setqflist([])
call S('getreg list', string(getreg('"', 1, 1)))
call setreg('a', ['x', 'y'], 'l')
call S('setreg list', string(getreg('a', 1, 1)))
call S('getline range', string(getline(1, '$')))
call setline(1, ['aa', 'bb'])
call S('setline list', string(getline(1, '$')))
call S('matchfuzzy', string(matchfuzzy(['foo', 'bar', 'foobar'], 'fo')))
call S('matchstrlist', string(matchstrlist(['ab', 'cd'], '\a')))
call S('getcompletion', type(getcompletion('ec', 'command')))
call S('gettagstack', string(sort(keys(gettagstack()))))
call T('ctxget', 'type(function("ctxget"))')

" ------------------------------------------------- 11. misc entry points
call S('=== 11. misc ===')
call S('range', string(range(5)), string(range(2, 8, 3)), string(range(0)))
call S('readfile-ish', type(split("a\nb", "\n")))
call S('sort stability', string(sort([[1,'a'],[1,'b'],[0,'c']], {a, b -> a[0] - b[0]})))
call S('uniq func', string(uniq([1, 1, 2], {a, b -> a - b})))
call S('list2dict', string(items({'a': 1})))
call S('function args list', string(call('add', [[1], 2])))
call S('partial args', string(function('add', [[1]])(2)))
let Pf = function('add', [[1]])
call S('partial held', string(Pf(9)))
call garbagecollect(1)
call S('partial after gc', string(Pf(8)))
unlet Pf
call S('nested containers', string([[[[[1]]]]]))
call S('assert', assert_equal([1], [1]), string(v:errors))
let v:errors = []
call S('deep compare', [1, [2, {'a': [3]}]] == [1, [2, {'a': [3]}]])
call S('big list', len(range(10000)), range(10000)[9999])
let big = range(20000)
call filter(big, 'v:val % 2 == 0')
call S('filtered big', len(big), big[-1])
unlet big
call garbagecollect(1)
call S('done')

" ------------------------------------------- 12. the :for cursor, widened
call S('=== 12. for cursors, widened ===')
func! Walk(l, Body) abort
  let seen = []
  let s:wl = a:l
  try
    for x in s:wl
      call add(seen, x)
      if len(seen) > 30 | break | endif
      call a:Body(x)
    endfor
  catch
    call add(seen, 'threw ' . substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
  endtry
  return string(seen) . ' ' . string(a:l)
endfunc
call S('remove current (first)', Walk([0, 1, 2, 3], {x -> x == 0 ? remove(s:wl, 0) : 0}))
call S('remove current (last)', Walk([0, 1, 2, 3], {x -> x == 3 ? remove(s:wl, 3) : 0}))
call S('remove next', Walk([0, 1, 2, 3, 4], {x -> x == 1 ? remove(s:wl, 2) : 0}))
call S('remove previous', Walk([0, 1, 2, 3, 4], {x -> x == 2 ? remove(s:wl, 1) : 0}))
call S('remove all remaining', Walk([0, 1, 2, 3, 4], {x -> x == 1 ? remove(s:wl, 2, -1) : 0}))
call S('remove current and rest', Walk([0, 1, 2, 3, 4], {x -> x == 1 ? remove(s:wl, 1, -1) : 0}))
call S('remove everything', Walk([0, 1, 2, 3], {x -> x == 1 ? remove(s:wl, 0, -1) : 0}))
call S('filter to empty', Walk([0, 1, 2, 3], {x -> x == 1 ? filter(s:wl, 0) : 0}))
call S('insert before current', Walk([0, 1, 2, 3], {x -> x == 2 && index(s:wl, 'n') < 0 ? insert(s:wl, 'n', 2) : 0}))
call S('insert at front each', Walk([0, 1, 2], {x -> type(x) == v:t_number && x < 10 ? insert(s:wl, x + 10) : 0}))
call S('insert after current', Walk([0, 1, 2], {x -> x == 1 ? insert(s:wl, 'a', 2) : 0}))
call S('remove then re-add', Walk([0, 1, 2, 3], {x -> x == 1 ? add(s:wl, remove(s:wl, 2)) : 0}))
call S('reverse at end', Walk([0, 1, 2, 3], {x -> x == 3 ? reverse(s:wl) : 0}))
call S('sort moves current', Walk([3, 0, 2, 1], {x -> x == 0 ? sort(s:wl) : 0}))
call S('uniq removes next', Walk([1, 2, 2, 2, 3], {x -> x == 1 ? uniq(s:wl) : 0}))
call S('slice assign over current', Walk([0, 1, 2, 3], {x -> x == 1 ? execute('let s:wl[1:2] = ["a", "b"]') : 0}))
call S('extend at current', Walk([0, 1, 2], {x -> x == 1 ? extend(s:wl, ['e', 'f'], 1) : 0}))

" nested :for over the same list, removing in the inner loop
let l = [1, 2, 3, 4, 5]
let pairs = []
for x in l
  for y in l
    call add(pairs, x . ':' . y)
    if x == 1 && y == 2
      call remove(l, index(l, 2))
    endif
  endfor
endfor
call S('nested remove inner current', string(pairs), string(l))
let l = [1, 2, 3, 4, 5]
let pairs = []
for x in l
  for y in l
    call add(pairs, x . ':' . y)
    if x == 2 && y == 4
      call remove(l, index(l, 2))
    endif
  endfor
endfor
call S('nested remove outer current', string(pairs), string(l))
let l = [1, 2, 3, 4, 5]
let pairs = []
for x in l
  for y in l
    call add(pairs, x . ':' . y)
    if x == 1 && y == 1
      call remove(l, 1, -1)
    endif
  endfor
endfor
call S('nested remove all but first', string(pairs), string(l))
let l = [1, 2, 3]
let pairs = []
for x in l
  for y in l
    call add(pairs, x . ':' . y)
    if len(pairs) > 40 | break | endif
    if x == 2 && y == 2
      call insert(l, 0, 1)
    endif
  endfor
endfor
call S('nested insert', string(pairs), string(l))

" :for over a list whose variable goes away or changes
let l = [1, 2, 3]
let seen = []
for x in l
  call add(seen, x)
  if x == 1
    unlet l
  endif
endfor
call S('unlet without gc', string(seen), exists('l'))
let l = [1, 2, 3]
let seen = []
for x in l
  call add(seen, x)
  if x == 1
    let l = [7, 8, 9]
  endif
endfor
call S('rebind while walking', string(seen), string(l))
func! UnletLocal() abort
  let l = [1, 2, 3]
  let seen = []
  for x in l
    call add(seen, x)
    if x == 2
      unlet l
      call garbagecollect(1)
    endif
  endfor
  return seen
endfunc
call S('unlet local while walking', string(UnletLocal()))
let g:gl = [1, 2, 3]
let seen = []
for x in g:gl
  call add(seen, x)
  if x == 1
    unlet g:gl
    call garbagecollect(1)
  endif
endfor
call S('unlet global while walking', string(seen))

" filter()/map() whose callback edits the list being walked
func! FMEdit(kind, Edit, l) abort
  let s:fl = a:l
  let v:errmsg = ''
  try
    if a:kind ==# 'filter'
      let r = filter(s:fl, a:Edit)
    else
      let r = map(s:fl, a:Edit)
    endif
    return 'ok ' . string(r) . ' ' . string(a:l) . ' errmsg=' . v:errmsg
  catch
    return 'threw ' . substitute(v:exception, '\v^Vim\(\a+\)?:?', '', '') . ' ' . string(a:l)
  endtry
endfunc
call S('filter removes first', FMEdit('filter', {i, v -> v == 2 ? len(remove(s:fl, 0)) * 0 + 1 : 1}, [1, 2, 3, 4]))
call S('filter removes current', FMEdit('filter', {i, v -> v == 2 ? remove(s:fl, i) * 0 + 1 : 1}, [1, 2, 3, 4]))
call S('filter removes next', FMEdit('filter', {i, v -> v == 2 ? remove(s:fl, i + 1) * 0 + 1 : 1}, [1, 2, 3, 4]))
call S('filter removes rest', FMEdit('filter', {i, v -> v == 1 ? len(remove(s:fl, 1, -1)) * 0 + 1 : 1}, [1, 2, 3, 4]))
call S('filter drops and removes', FMEdit('filter', {i, v -> v == 2 ? remove(s:fl, -1) * 0 : 1}, [1, 2, 3, 4]))
call S('filter appends', FMEdit('filter', {i, v -> v == 1 ? len(add(s:fl, 9)) * 0 + 1 : 1}, [1, 2, 3]))
call S('map removes first', FMEdit('map', {i, v -> v == 2 ? remove(s:fl, 0) * 0 + v * 10 : v * 10}, [1, 2, 3, 4]))
call S('map removes rest', FMEdit('map', {i, v -> v == 1 ? len(remove(s:fl, 1, -1)) * 0 + 10 : v * 10}, [1, 2, 3, 4]))
call S('map clears', FMEdit('map', {i, v -> v == 2 ? len(filter(s:fl, 0)) : v}, [1, 2, 3, 4]))
call S('filter string removes', FMEdit('filter', 'v:val == 2 ? remove(s:fl, 0) * 0 + 1 : 1', [1, 2, 3, 4]))
call S('map string removes', FMEdit('map', 'v:val == 2 ? remove(s:fl, -1) : v:val', [1, 2, 3, 4]))
call S('filter locked inside', FMEdit('filter', {i, v -> islocked('s:fl')}, [1, 2]))
unlet s:fl

" --------------------------------------- 13. sort()/uniq() comparators
call S('=== 13. sort/uniq comparators ===')
func! SortTry(what, l, ...) abort
  let s:sl = a:l
  let v:errmsg = ''
  try
    let r = call(a:what, [s:sl] + a:000)
    return 'ok ' . string(r) . ' ' . string(a:l) . ' errmsg=' . v:errmsg
  catch
    return 'threw ' . substitute(v:exception, '\v^Vim\(\a+\)?:?', '', '') . ' ' . string(a:l)
  endtry
endfunc
let s:adds = 0
func! CmpAdd(a, b) abort
  let s:adds += 1
  if s:adds <= 3 | call add(s:sl, 100 + s:adds) | endif
  return a:a == a:b ? 0 : a:a > a:b ? 1 : -1
endfunc
func! CmpRemove(a, b) abort
  if len(s:sl) > 0 | call remove(s:sl, 0) | endif
  return a:a == a:b ? 0 : a:a > a:b ? 1 : -1
endfunc
func! CmpSeen(a, b) abort
  call add(s:cmplens, len(s:sl))
  return a:a == a:b ? 0 : a:a > a:b ? 1 : -1
endfunc
func! CmpThrow(a, b) abort
  throw 'cmp threw'
endfunc
func! CmpString(a, b) abort
  return 'x'
endfunc
func! CmpList(a, b) abort
  return [1]
endfunc
func! CmpFloat(a, b) abort
  return a:a > a:b ? 0.5 : -0.5
endfunc
func! CmpSelf(a, b) dict abort
  let self.calls += 1
  if self.mutate && self.calls <= 3 | call add(s:sl, 50 + self.calls) | endif
  return a:a == a:b ? 0 : a:a > a:b ? self.dir : -self.dir
endfunc
let s:adds = 0
call S('sort funcref adds', SortTry('sort', [3, 1, 2], function('CmpAdd')))
call S('sort funcref removes', SortTry('sort', [3, 1, 2, 5, 4], function('CmpRemove')))
let s:cmplens = []
call S('sort sees the list', SortTry('sort', [3, 1, 2], function('CmpSeen')), string(uniq(sort(s:cmplens))))
call S('sort lambda adds', SortTry('sort', [3, 1, 2], {a, b -> len(add(s:sl, 0)) * 0 + a - b}))
call S('sort lambda call remove', SortTry('sort', [3, 1, 2], {a, b -> len(s:sl) ? remove(s:sl, 0) * 0 + a - b : a - b}))
let cd = {'calls': 0, 'mutate': 0, 'dir': -1}
call S('sort partial self', SortTry('sort', [3, 1, 2], function('CmpSelf', [], cd)), cd.calls > 0)
let cd = {'calls': 0, 'mutate': 1, 'dir': 1}
call S('sort partial self mutates', SortTry('sort', [3, 1, 2], function('CmpSelf', [], cd)), cd.calls > 0)
let cd = {'calls': 0, 'mutate': 0, 'dir': 1}
call S('sort dict arg', SortTry('sort', [3, 1, 2], 'CmpSelf', cd), cd.calls > 0)
call S('sort throws', SortTry('sort', [3, 1, 2], function('CmpThrow')))
call S('sort returns string', SortTry('sort', [3, 1, 2], function('CmpString')))
call S('sort returns list', SortTry('sort', [3, 1, 2], function('CmpList')))
call S('sort returns float', SortTry('sort', [3, 1, 2], function('CmpFloat')))
call S('sort lambda returns string', SortTry('sort', [3, 1, 2], {a, b -> 'a'}))
call S('sort missing func', SortTry('sort', [3, 1, 2], 'NoSuchCmp'))
call S('sort bad number', SortTry('sort', [3, 1, 2], 2))
let s:adds = 0
call S('uniq funcref adds', SortTry('uniq', [1, 1, 2, 2], function('CmpAdd')))
call S('uniq funcref removes', SortTry('uniq', [1, 1, 2, 2, 3], function('CmpRemove')))
call S('uniq lambda removes', SortTry('uniq', [1, 1, 2, 2, 3], {a, b -> len(s:sl) > 2 ? remove(s:sl, -1) * 0 + a - b : a - b}))
let cd = {'calls': 0, 'mutate': 1, 'dir': 1}
call S('uniq partial self mutates', SortTry('uniq', [1, 1, 2, 2], function('CmpSelf', [], cd)), cd.calls > 0)
call S('uniq throws', SortTry('uniq', [1, 1, 2], function('CmpThrow')))
call S('uniq returns string', SortTry('uniq', [1, 1, 2], function('CmpString')))
call S('uniq returns float', SortTry('uniq', [1, 1, 2], function('CmpFloat')))
" the built-in orderings over mixed types
let mixed = [10, '9', 2.5, 'b', 'A', 'a', -1, '10', 1.0, 'B', v:true, '', 0, '-3', [1], {'k': 1}, v:null]
let flat = [10, '9', 2.5, 'b', 'A', 'a', -1, '10', 1.0, 'B', '', 0, '-3']
for how in ['', 'N', 'n', 'f', 'l', 'i', 1, 0]
  call S('sort how=' . string(how), SortTry('sort', copy(mixed), how))
  call S('sort flat how=' . string(how), SortTry('sort', copy(flat), how))
  call S('uniq how=' . string(how), SortTry('uniq', ['a', 'A', 'A', 1, '1', 1.0, 1, 'b', 'B', 2, 2.0, '2'], how))
endfor
call S('sort f on floats', SortTry('sort', [3.5, -1.25, 0.0, 2, 1.0e10, -1.0e-3], 'f'))
call S('sort N on strings', SortTry('sort', ['10', '9', '0x1F', '-2', 'abc', '3e2'], 'N'))
call S('sort n on strings', SortTry('sort', ['10', '9', '0x1F', '-2', 'abc', '3e2'], 'n'))
call S('sort i stability', SortTry('sort', ['b', 'A', 'a', 'B', 'a', 'A'], 'i'))
call S('sort 1 stability', SortTry('sort', ['b', 'A', 'a', 'B', 'a', 'A'], 1))
let lk = [3, 1, 2]
lockvar 1 lk
call S('sort locked', SortTry('sort', lk))
call S('uniq locked', SortTry('uniq', lk))
unlockvar 1 lk

" ------------------------------------------------------------ 14. partials
call S('=== 14. partials ===')
func! Str(x) abort
  try
    return string(a:x)
  catch
    return 'threw ' . substitute(v:exception, '\v^Vim\(\a+\)?:?', '', '')
  endtry
endfunc
func! PF(...) dict abort
  return [a:000, get(self, 'tag', '-')]
endfunc
func! PG(...) abort
  return a:000
endfunc
let pd = {'tag': 'pd', 'F': function('PF')}
let od = {'tag': 'od'}
let P = function('PF', [1, 2], pd)
call S('partial string', string(P))
call S('partial name', string(get(P, 'name')), 'args', string(get(P, 'args')), 'dict', string(get(P, 'dict')))
call S('partial func', string(get(P, 'func')))
call S('partial call', string(P(3)))
call S('auto-bound d.F', string(pd.F), string(pd.F(9)))
let AB = pd.F
call S('auto-bound held', string(AB), string(get(AB, 'dict')), string(AB(8)))
let FD = function(pd.F)
call S('function(d.F)', string(FD), string(FD(7)))
let P2 = function(P)
call S('function(P)', string(P2), P2 == P, P2 is P)
let P3 = function(P, [3])
call S('function(P, [3])', string(P3), string(P3(4)))
let P4 = function(P, od)
call S('function(P, {})', string(P4), string(P4(5)))
let P5 = function(P, [6], od)
call S('function(P, [6], od)', string(P5), string(P5()))
let R = funcref('PF', [1], pd)
call S('funcref', string(R), string(R(2)))
let R2 = funcref(P)
call S('funcref(P)', string(R2), string(R2()))
call S('call(P, [x])', string(call(P, ['x'])))
call S('call(P, [x], od)', string(call(P, ['x'], od)))
call S('call(PG partial, [x], od)', string(call(function('PG', [0]), ['x'], od)))
call S('== between partials', P == function('PF', [1, 2], pd), P == function('PF', [1, 2], od), P == function('PF', [1], pd))
call S('is between partials', P is P, P is function('PF', [1, 2], pd), P is P2)
call S('== partial vs funcref', P == function('PF'), function('PF') == function('PF'))
call S('== with equal dicts', function('PF', [], {'tag': 1}) == function('PF', [], {'tag': 1}))
let CP = copy(P)
let DP = deepcopy(P)
call S('copy partial', string(CP), CP == P, CP is P)
call S('deepcopy partial', string(DP), DP == P, DP is P, get(DP, 'dict') is pd)
let held = [P, {'p': P}]
let hc = copy(held)
let hd = deepcopy(held)
call S('copy list of partials', hc[0] is P, hc[1] is held[1])
call S('deepcopy list of partials', string(hd), hd[0] is P, hd[1].p is P, hd[1] is held[1], get(hd[0], 'dict') is pd)
let dh = {'a': P, 'b': P}
let dhc = deepcopy(dh)
call S('deepcopy dict of partials', dhc.a is dhc.b, dhc.a == P)
" a partial whose bound dict holds the partial
let cy = {'tag': 'cy'}
let cy.P = function('PF', [1], cy)
call S('partial cycle string', Str(cy))
call S('partial cycle call', string(cy.P(2)))
call S('partial cycle get dict', get(cy.P, 'dict') is cy)
call garbagecollect(1)
call S('partial cycle after gc', Str(cy.P(3)), Str(cy))
let Cyp = cy.P
unlet cy
call garbagecollect(1)
call S('partial cycle held by partial', Str(Cyp(4)), Str(get(Cyp, 'dict')))
let cyd = deepcopy(get(Cyp, 'dict'))
call S('deepcopy of partial cycle', Str(cyd), get(cyd.P, 'dict') is cyd)
unlet Cyp cyd
call garbagecollect(1)
call S('partial cycle collected')
" a partial whose argv holds the partial's own dict and list
let al = [1]
let ad = {'tag': 'ad', 'l': al}
let AP = function('PF', [al, ad], ad)
call add(al, AP)
call S('argv cycle string', Str(AP))
call garbagecollect(1)
call S('argv cycle after gc', Str(AP('z')))
let APc = deepcopy(AP)
call S('argv cycle deepcopy', Str(APc), get(APc, 'args')[0] is al)
unlet al ad AP APc
call garbagecollect(1)
call S('argv cycle collected')
" lambdas and partials of lambdas
let g:Lm = {x -> [x, self]}
call T('lambda in dict', 'call(g:Lm, [1], {"s": 1})')
unlet g:Lm
let LP = function({... -> a:000}, [1, 2])
call S('partial of lambda', string(LP(3)), string(get(LP, 'args')))
unlet P P2 P3 P4 P5 R R2 CP DP held hc hd dh dhc AB FD LP

" ----------------------------------------------- 15. deepcopy with cycles
call S('=== 15. deepcopy cycles ===')
let ll = [1]
call add(ll, ll)
let llc = deepcopy(ll)
call S('list in itself', llc[1] is llc, llc[1] isnot ll, ll[1] is ll, string(ll[0]))
let dd = {'n': 1}
let dd.me = dd
let ddc = deepcopy(dd)
call S('dict in itself', ddc.me is ddc, ddc.me isnot dd, dd.me is dd)
let xl = [1]
let xd = {'l': xl}
call add(xl, xd)
let xlc = deepcopy(xl)
call S('list<->dict', xlc[1].l is xlc, xlc[1] isnot xd, xl[1] is xd, xd.l is xl)
let xdc = deepcopy(xd)
call S('dict<->list', xdc.l[1] is xdc, xdc.l isnot xl)
let pdc = {'n': 1}
let pdc.P = function('PF', [pdc], pdc)
let pdcc = deepcopy(pdc)
call S('through partial dict', get(pdcc.P, 'dict') is pdc, get(pdcc.P, 'args')[0] is pdc, Str(pdcc.P()))
let pdcc.n = 2
call S('original untouched', pdc.n, string(keys(pdc)))
let deep = [[[1]]]
call add(deep[0][0], deep)
let deepc = deepcopy(deep)
call S('deep cycle', deepc[0][0][1] is deepc, deep[0][0][1] is deep)
let two = [1]
let tw = [two, two, [two]]
let twc = deepcopy(tw)
call S('shared keeps sharing', twc[0] is twc[1], twc[2][0] is twc[0])
let twn = deepcopy(tw, 1)
call S('noref unshares', twn[0] is twn[1], twn[2][0] is twn[0], string(twn))
let shd = {'x': two, 'y': two}
let shdn = deepcopy(shd, 1)
call S('noref dict unshares', shdn.x is shdn.y, string(shdn))
let g:cl = ll
let g:cd = dd
call T('noref list cycle', 'deepcopy(g:cl, 1)')
call T('noref dict cycle', 'deepcopy(g:cd, 1)')
unlet g:cl g:cd
call S('original intact', len(ll), ll[1] is ll, len(dd), dd.me is dd)
unlet ll llc dd ddc xl xd xlc xdc pdc pdcc deep deepc two tw twc twn shd shdn
call garbagecollect(1)
call S('deepcopy cycles collected')

" ----------------------------------------------- 16. self-extension
call S('=== 16. self-extension ===')
let l = [1, 2, 3]
call extend(l, l, 1)
call S('extend(l, l, 1)', string(l))
let l = [1, 2, 3]
call extend(l, l, -1)
call S('extend(l, l, -1)', string(l))
let g:el = [1, 2, 3]
call T('extend(l, l, 9)', 'extend(g:el, g:el, 9)')
unlet g:el
let l = [1, 2, 3]
let l += l
call S('l += l', string(l))
let l = [1, 2, 3]
let l[0:1] += l[0:1]
call S('l[0:1] += l[0:1]', string(l))
let l = [[1], 2]
call extend(l, l)
call S('extend self shares items', l[0] is l[2], string(l))
let l = []
call extend(l, l)
call S('extend empty self', string(l))
let d = {'a': 1, 'b': 2}
call extend(d, d, 'keep')
call S('extend dict self keep', string(d))
let g:ed = d
call T('extend dict self error', 'extend(g:ed, g:ed, "error")')
unlet g:ed
let l = [1, 2]
let m = extendnew(l, l)
call S('extendnew self', string(m), string(l))

call writefile(s:out, $DIFFOUT)
qall!
