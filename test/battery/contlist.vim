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

call writefile(s:out, $DIFFOUT)
qall!
