" Differential: dictionary order, ownership and the collector.
" Run headless; every line goes to the file named by $DIFFOUT.
" Nothing here may depend on the build, only on observable behaviour.

let s:out = []
func! S(...) abort
  call add(s:out, join(map(copy(a:000), 'type(v:val) == v:t_string ? v:val : string(v:val)'), ' '))
endfunc

" ---------------------------------------------------------------- 1. order
" keys()/values()/items() are slot order, which is user-visible.  Walk the
" growth boundaries: the table starts at 16 slots and quadruples.
call S('=== 1. slot order over sizes ===')
for n in [0, 1, 2, 5, 10, 14, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 200, 255, 256, 257, 400, 1000, 1001, 1024]
  let d = {}
  for i in range(n)
    let d['k' . i] = i
  endfor
  call S('n=' . n, 'len', len(d))
  call S('  keys', join(keys(d), ','))
  call S('  values', join(map(values(d), 'string(v:val)'), ','))
  call S('  items', join(map(items(d), 'v:val[0] . "=" . string(v:val[1])'), ','))
endfor

" Different key shapes: same length, colliding prefixes, empty-ish keys,
" numeric keys, unicode, long keys.
call S('=== 1b. key shapes ===')
let shapes = {}
let shapes['ascii-short'] = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t']
let shapes['ascii-long'] = map(range(20), '"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" . v:val')
let shapes['numeric'] = map(range(40), 'string(v:val)')
let shapes['unicode'] = map(range(20), '"kéyé" . v:val . "你好"')
let shapes['sharedprefix'] = map(range(50), '"prefix_common_" . v:val')
let shapes['revnum'] = map(reverse(range(40)), 'string(v:val)')
let shapes['punct'] = ['!','@','#','$','%','^','&','*','(',')','-','_','=','+','[',']','{','}',';',':']
for name in sort(keys(shapes))
  let d = {}
  for k in shapes[name]
    let d[k] = len(k)
  endfor
  call S('shape', name, 'keys', join(keys(d), '|'))
endfor

" ------------------------------------------------------ 2. remove / re-add
call S('=== 2. remove then re-add ===')
let d = {}
for i in range(40) | let d['k' . i] = i | endfor
call S('base', join(keys(d), ','))
for i in range(0, 39, 3)
  call remove(d, 'k' . i)
endfor
call S('after removes', len(d), join(keys(d), ','))
for i in range(0, 39, 3)
  let d['k' . i] = i * 100
endfor
call S('after re-add', len(d), join(keys(d), ','))
call S('values', join(map(values(d), 'string(v:val)'), ','))
" Tombstone compaction: remove nearly everything, then refill.
let d2 = {}
for i in range(300) | let d2['x' . i] = i | endfor
for i in range(295) | call remove(d2, 'x' . i) | endfor
call S('shrunk', len(d2), join(keys(d2), ','))
for i in range(60) | let d2['y' . i] = i | endfor
call S('refilled', len(d2), join(keys(d2), ','))
" remove() answers the value, and the dict loses its reference.
let d3 = {'a': [1,2,3], 'b': {'c': 1}}
let got = remove(d3, 'a')
call S('removed value', string(got), 'left', string(keys(d3)))
let got[0] = 99
call S('removed value is ours', string(got))
call S('remove missing', string(has_key(d3, 'zz')))

" ------------------------------------------------------------- 3. extend()
call S('=== 3. extend ===')
for act in ['keep', 'force', 'error']
  let a = {'a': 1, 'b': 2, 'c': 3}
  let b = {'b': 20, 'd': 40, 'a': 10}
  try
    call extend(a, b, act)
    call S('extend', act, 'keys', join(keys(a), ','), 'vals', join(map(values(a), 'string(v:val)'), ','))
  catch
    call S('extend', act, 'threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''), 'keys', join(keys(a), ','))
  endtry
endfor
" extend with self
let a = {'p': 1, 'q': 2}
call extend(a, a)
call S('extend self', join(keys(a), ','), join(map(values(a), 'string(v:val)'), ','))
" extend across a resize boundary
let a = {}
for i in range(14) | let a['e' . i] = i | endfor
let b = {}
for i in range(14, 40) | let b['e' . i] = i | endfor
call extend(a, b)
call S('extend grow', len(a), join(keys(a), ','))
" extendnew leaves the source alone
let src = {'s': 1}
let new = extendnew(src, {'t': 2})
call S('extendnew', string(keys(src)), string(keys(new)))

" ------------------------------------------------ 4. filter / map in place
call S('=== 4. filter/map ===')
let d = {}
for i in range(50) | let d['f' . i] = i | endfor
call filter(d, 'v:val % 3 == 0')
call S('filter mod3', len(d), join(keys(d), ','))
call map(d, 'v:val * 2')
call S('map x2', join(map(items(d), 'v:val[0] . "=" . string(v:val[1])'), ','))
" filter that removes while the walk is live, on a table big enough to resize
let d = {}
for i in range(100) | let d['g' . i] = i | endfor
call filter(d, 'v:val < 5')
call S('filter down to 5', len(d), join(keys(d), ','))
" mapnew
let d = {'a': 1, 'b': 2}
let n = mapnew(d, 'v:val + 1')
call S('mapnew', string(keys(d)), string(keys(n)), string(values(n)))
" filter over a dict of containers: the removed values must be freed, not
" leaked or double-freed.
let d = {}
for i in range(30) | let d['h' . i] = [i, {'inner': i}] | endfor
call filter(d, 'v:val[0] % 2 == 0')
call S('filter containers', len(d), join(keys(d), ','))
" a filter callback that mutates the dict it walks
let d = {'a': 1, 'b': 2, 'c': 3, 'd': 4}
let s:seen = []
func! Watch(k, v) abort
  call add(s:seen, a:k)
  return a:v != 2
endfunc
call filter(d, function('Watch'))
call S('filter fn', join(s:seen, ','), join(keys(d), ','))

" ------------------------------------------------------------ 5. deepcopy
call S('=== 5. copy/deepcopy ===')
let d = {'n': 1, 'l': [1, [2, 3]], 'd': {'x': {'y': 1}}, 's': 'str'}
let sh = copy(d)
let dp = deepcopy(d)
let d.l[1][0] = 99
call S('shallow sees', string(sh.l), 'deep does not', string(dp.l))
call S('copy keys', join(keys(sh), ','), 'deepcopy keys', join(keys(dp), ','))
" self-referential
let r = {'a': 1}
let r.self = r
let rc = deepcopy(r)
call S('cycle deepcopy', string(rc.self.a), string(rc.self.self.a), string(rc.self is rc))
" deepcopy(x, 1) refuses to share
let sub = {'q': 1}
let outer = {'one': sub, 'two': sub}
let c1 = deepcopy(outer)
call S('deepcopy shares', string(c1.one is c1.two))
try
  let c2 = deepcopy(r, 1)
  call S('deepcopy noref cycle ok')
catch
  call S('deepcopy noref cycle threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
" deepcopy over a big dict keeps slot order
let big = {}
for i in range(70) | let big['b' . i] = [i] | endfor
call S('big deepcopy order', join(keys(deepcopy(big)), ',') ==# join(keys(big), ','))
call S('big copy order', join(keys(copy(big)), ',') ==# join(keys(big), ','))

" ------------------------------------------------------------- 6. lockvar
call S('=== 6. lockvar ===')
let s:d = {'a': 1, 'b': [1, 2], 'c': {'d': 3}}
lockvar s:d.a
call S('islocked a', islocked('s:d.a'), 'b', islocked('s:d.b'))
try
  let s:d.a = 2
catch
  call S('write locked item threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
unlockvar s:d.a
let s:d.a = 2
call S('after unlock', s:d.a)
lockvar 1 s:d
try
  let s:d.newkey = 1
catch
  call S('add to locked dict threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
try
  call remove(s:d, 'a')
catch
  call S('remove from locked dict threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
let s:d.b = [9]
call S('depth-1 lock lets the item through?', string(s:d.b))
unlockvar 1 s:d
lockvar s:d
try
  let s:d.b[0] = 5
catch
  call S('deep lock nested threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
unlockvar s:d
call S('islocked after unlock', islocked('s:d'))
" locking a key that does not exist
try
  call S('islocked missing', islocked('s:d.nope'))
catch
  call S('islocked missing threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
" lockvar on a variable holding a dict shared with another name
let s:x = {'k': 1}
let s:y = s:x
lockvar s:x
try
  let s:y.k = 2
catch
  call S('shared lock threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
unlockvar s:x

" ------------------------------------------------------- 7. scopes and v:
call S('=== 7. scope dicts ===')
func! ScopeProbe(a, b, ...) abort
  let l:loc = 1
  let l:other = 2
  return [sort(keys(l:)), sort(keys(a:)), a:0, string(a:000)]
endfunc
call S('scope', string(ScopeProbe(1, 2, 3, 4)))
let b:one = 1
let b:two = 2
call S('b: keys', join(sort(keys(b:)), ','))
call S('changedtick', type(b:changedtick), b:changedtick == getbufvar('%', 'changedtick'))
let tick = b:changedtick
call setline(1, 'x')
call S('changedtick moves', b:changedtick > tick)
call S('changedtick in b:', has_key(b:, 'changedtick'))
try
  let b:changedtick = 5
catch
  call S('changedtick write threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
try
  call remove(b:, 'changedtick')
catch
  call S('changedtick remove threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
call S('v: has count', has_key(v:, 'count'), 'type', type(v:))
unlet b:one
unlet b:two
let w:wv = 1
call S('w: keys', join(sort(keys(w:)), ','))
unlet w:wv
let t:tv = 1
call S('t: keys', join(sort(keys(t:)), ','))
unlet t:tv
" g: is a real dict
let g:probe_one = 1
call S('g: has', has_key(g:, 'probe_one'), get(g:, 'probe_one'))
unlet g:probe_one

" ------------------------------------------------------- 8. garbagecollect
call S('=== 8. garbagecollect ===')
func! Cyc() abort
  let d = {}
  let l = [d]
  let d.l = l
  let d.self = d
  let Pf = function('Cyc', [d])
  let d.p = Pf
  let d.f = function('Cyc')
  return 0
endfunc
for i in range(200)
  call Cyc()
endfor
call garbagecollect(1)
call S('gc after cycles ok')
" cycles that outlive a script variable
let s:keep = {}
let s:keep.me = s:keep
let s:keep.list = [s:keep]
call garbagecollect(1)
call S('rooted cycle survives', string(s:keep.me.me.list[0].me is s:keep))
" partial cycles
let s:pd = {}
let s:pd.p = function('Cyc', [s:pd])
call garbagecollect(1)
call S('partial cycle survives', string(type(s:pd.p)))
" funcref through a dict function
func! s:Meth() dict abort
  return len(self)
endfunc
let s:obj = {'m': function('s:Meth'), 'x': 1}
call garbagecollect(1)
call S('dict method', s:obj.m())
" a cycle discarded inside a loop, collected between iterations
for i in range(20)
  let tmp = {'i': i}
  let tmp.self = tmp
  call garbagecollect(1)
endfor
call S('loop gc ok')
" gc with the dict reachable only from a list held by a local
func! GcLocal() abort
  let d = {'a': [1,2,3]}
  let l = [d, d]
  let d.back = l
  call garbagecollect(1)
  return len(l[0].a)
endfunc
call S('gc local', GcLocal())
call garbagecollect(1)
call S('gc ok tail')

" ----------------------------------------------------------- 9. misc dict
call S('=== 9. misc ===')
call S('get default', get({'a':1}, 'b', 'dflt'), get({'a':1}, 'a'))
call S('has_key', has_key({'a':1}, 'a'), has_key({'a':1}, 'b'))
call S('empty', empty({}), empty({'a':1}))
call S('count', count({'a':1,'b':1,'c':2}, 1))
call S('max/min', max({'a':3,'b':9}), min({'a':3,'b':9}))
call S('string', string({'b':1,'a':2}))
call S('json_encode', json_encode({'b':1,'a':2}))
call S('json_decode', string(json_decode('{"z":1,"a":2,"m":3}')), join(keys(json_decode('{"z":1,"a":2,"m":3}')), ','))
call S('msgpack roundtrip', string(msgpackparse(msgpackdump([{'k': 1}]))))
call S('sort by key', string(sort(keys({'c':1,'a':2,'b':3}))))
call S('index of', string(index(values({'a':1,'b':2}), 2)))
call S('type', type({}), type({'a':1}))
call S('flatten of values', string(flatten(values({'a':[1],'b':[2]}))))
call S('dict in list', string([{'a':1}, {'b':2}]))
call S('nested string', string({'a': {'b': {'c': [1, {'d': 2}]}}}))
call S('eval of string', string(eval(string({'a':1,'b':[1,2]}))))
" a dict as a funcref's self
func! s:Sum() dict abort
  return self.a + self.b
endfunc
let o = {'a': 3, 'b': 4}
let o.sum = function('s:Sum')
call S('method sum', o.sum())
let Bound = function('s:Sum', o)
call S('bound sum', Bound())
" empty-string key
let d = {'': 'empty'}
call S('empty key', string(keys(d)), string(d['']))
let d[''] = 'again'
call S('empty key again', len(d), string(values(d)))
call remove(d, '')
call S('empty key removed', len(d))
" number keys coerce
let d = {}
let d[1] = 'one'
let d[2] = 'two'
call S('number keys', string(keys(d)), string(d['1']))
" very long key
let k = repeat('z', 5000)
let d = {}
let d[k] = 1
call S('long key', len(keys(d)[0]), d[k])
" many dicts alive at once
let many = []
for i in range(500)
  call add(many, {'a' . i: i, 'b': [i]})
endfor
call S('many dicts', len(many), string(many[499]))
let many = 0
call garbagecollect(1)
call S('many freed')
" self-reference printing
let sd = {'a': 1}
let sd.s = sd
try
  call S('self string', string(sd))
catch
  call S('self string threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
let sl = [1]
call add(sl, sl)
try
  call S('self list string', string(sl))
catch
  call S('self list string threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
" null dict
call S('null dict', string(v:_null_dict), len(v:_null_dict), string(keys(v:_null_dict)), empty(v:_null_dict))
try
  let nd = v:_null_dict
  let nd['a'] = 1
catch
  call S('null dict write threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
call S('null dict copy', string(copy(v:_null_dict)), string(deepcopy(v:_null_dict)))
call S('null dict extend', string(extendnew({}, v:_null_dict)))
" dict function results
call S('getbufinfo keys', join(sort(keys(getbufinfo()[0])), ','))
call S('nvim_get_mode', string(sort(keys(nvim_get_mode()))))
call S('win getwininfo', join(sort(keys(getwininfo()[0])), ','))
call S('undotree', join(sort(keys(undotree())), ','))
call S('winsaveview', join(sort(keys(winsaveview())), ','))
call S('matchadd/getmatches', string(getmatches()))

" -------------------------------------------------- 10. api dict surface
call S('=== 10. api ===')
call S('nvim_eval dict', string(nvim_eval('{"a": 1, "b": [1,2]}')))
call nvim_set_var('apid', {'z': 1, 'a': 2})
call S('api var', string(nvim_get_var('apid')), join(keys(nvim_get_var('apid')), ','))
call nvim_del_var('apid')
call S('api call', string(nvim_call_function('keys', [{'q':1,'p':2}])))
call S('api exec_lua', string(sort(luaeval('vim.fn.keys({q=1,p=2})'))))
call S('lua roundtrip', string(luaeval('{a=1,b=2}')))
call S('lua keys sorted', string(sort(keys(luaeval('{a=1,b=2,c=3}')))))
lua vim.g.luad = {x = 1, y = 2}
call S('lua g var', string(sort(keys(g:luad))))
unlet g:luad

" ------------------------------------------------------- 11. dict watchers
call S('=== 11. dict watchers ===')
func! WTry(cmd) abort
  try
    execute a:cmd
    call S('  ok', a:cmd)
  catch
    call S('  threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
  endtry
endfunc
let s:log = []
func! WLog(name, d, k, ch) abort
  let ch = map(copy(a:ch), 'type(v:val) == v:t_dict ? "<dict " . join(sort(keys(v:val)), ",") . ">" : v:val')
  call add(s:log, a:name . ' ' . a:k . ' ' . string(ch) . ' ' . string(sort(keys(a:d))))
endfunc
func! WStar(d, k, ch) abort
  call WLog('star', a:d, a:k, a:ch)
endfunc
func! WPrefix(d, k, ch) abort
  call WLog('aprefix', a:d, a:k, a:ch)
endfunc
func! WExact(d, k, ch) abort
  call WLog('exact', a:d, a:k, a:ch)
endfunc
func! WAfter(d, k, ch) abort
  call WLog('after', a:d, a:k, a:ch)
endfunc
func! WLate(d, k, ch) abort
  call WLog('late', a:d, a:k, a:ch)
endfunc
func! WBy(d, k, ch) abort
  call WLog('bystander', a:d, a:k, a:ch)
endfunc
func! WRm(d, k, ch) abort
  call WLog('rm', a:d, a:k, a:ch)
endfunc
func! WFlush(label) abort
  call S(a:label, len(s:log))
  for e in s:log
    call S('  ' . e)
  endfor
  let s:log = []
endfunc
" plain recording, every pattern shape
let wd = {}
call dictwatcheradd(wd, '*', 'WStar')
call dictwatcheradd(wd, 'a*', 'WPrefix')
call dictwatcheradd(wd, 'abc', 'WExact')
let wd.abc = 1
let wd.abd = 2
let wd.x = 3
let wd.abc = 10
let wd.ab = 4
call remove(wd, 'abc')
unlet wd.x
call extend(wd, {'abc': 5, 'y': 6})
call extend(wd, {'abc': 5}, 'force')
call filter(wd, 'v:key !=# "y"')
call map(wd, 'v:val + 1')
let wd['a'] = [1]
call add(wd.a, 2)
call WFlush('patterns')
call dictwatcherdel(wd, 'abc', 'WExact')
let wd.abc = 0
call WFlush('exact removed')
call WTry('call dictwatcherdel(g:wd, "abc", "WExact")')
call WTry('call dictwatcherdel(g:wd, "zz", "WStar")')
call WTry('call dictwatcherdel(g:wd, "*", function("WLog", ["other"]))')
call dictwatcheradd(wd, 'p', function('WLog', ['partial']))
let wd.p = 1
call WTry('call dictwatcherdel(g:wd, "p", function("WLog", ["partial"]))')
let wd.p = 2
call WFlush('missing removals')
call dictwatcherdel(wd, '*', 'WStar')
call dictwatcherdel(wd, 'a*', 'WPrefix')
let wd.abc = 99
call WFlush('all removed')

" a callback that adds a key
let wa = {}
func! WAdd(d, k, ch) abort
  call add(s:log, 'add ' . a:k . ' ' . string(a:ch))
  if a:k !=# 'added'
    let a:d.added = get(a:d, 'added', 0) + 1
  endif
endfunc
call dictwatcheradd(wa, '*', 'WAdd')
let wa.one = 1
let wa.two = 2
call WFlush('callback adds')
call S('  dict', string(wa))
call dictwatcherdel(wa, '*', 'WAdd')

" a callback that removes the watched key
let wr = {}
func! WRemove(d, k, ch) abort
  call add(s:log, 'rm ' . a:k . ' ' . string(a:ch))
  if has_key(a:d, a:k) && a:k ==# 'gone'
    call remove(a:d, a:k)
  endif
endfunc
call dictwatcheradd(wr, 'gone', 'WRemove')
let wr.gone = 1
let wr.kept = 2
call WFlush('callback removes watched key')
call S('  dict', string(wr))
let wr.gone = 2
call WFlush('again')
call S('  dict', string(wr))
call dictwatcherdel(wr, 'gone', 'WRemove')

" a callback that removes itself
let ws = {}
func! WSelf(d, k, ch) abort
  call add(s:log, 'self ' . a:k . ' ' . string(a:ch))
  call dictwatcherdel(a:d, '*', 'WSelf')
endfunc
call dictwatcheradd(ws, '*', 'WSelf')
call dictwatcheradd(ws, '*', 'WAfter')
let ws.a = 1
let ws.b = 2
call WFlush('callback removes itself')
call WTry('call dictwatcherdel(g:ws, "*", "WSelf")')
call dictwatcherdel(ws, '*', 'WAfter')
call WFlush('self gone')

" a callback that adds a second watcher
let w2 = {}
let s:added2 = 0
func! WSecond(d, k, ch) abort
  call add(s:log, 'second ' . a:k . ' ' . string(a:ch))
  if !s:added2
    let s:added2 = 1
    call dictwatcheradd(a:d, '*', 'WLate')
  endif
endfunc
call dictwatcheradd(w2, '*', 'WSecond')
let w2.a = 1
let w2.b = 2
call WFlush('callback adds a watcher')
call dictwatcherdel(w2, '*', 'WSecond')
call dictwatcherdel(w2, '*', 'WLate')

" a callback that changes the dict a second time (recursion)
let wc = {}
let s:depth = 0
func! WRecurse(d, k, ch) abort
  let s:depth += 1
  call add(s:log, 'rec depth=' . s:depth . ' ' . a:k . ' ' . string(a:ch))
  if s:depth < 4
    let a:d[a:k] = get(a:d, a:k, 0) + 100
  endif
  let s:depth -= 1
endfunc
call dictwatcheradd(wc, '*', 'WRecurse')
call dictwatcheradd(wc, '*', 'WBy')
let wc.n = 1
call WFlush('recursion')
call S('  dict', string(wc))
call dictwatcherdel(wc, '*', 'WRecurse')
call dictwatcherdel(wc, '*', 'WBy')

" a callback that throws, and a callback held by a lambda
let wt = {}
call dictwatcheradd(wt, '*', {d, k, ch -> execute('throw "watch threw"')})
call WTry('let g:wt.a = 1')
call S('  dict', string(wt))
let wl = {}
call dictwatcheradd(wl, 'k', {d, k, ch -> add(s:log, 'lambda ' . k . ' ' . string(ch))})
let wl.k = 1
let wl.j = 2
unlet wl.k
call WFlush('lambda watcher')

" remove() and friends while a watcher is active
let wm = {'a': 1, 'b': [2], 'c': {'d': 3}}
call dictwatcheradd(wm, '*', 'WRm')
call S('remove under watch', string(remove(wm, 'b')), string(remove(wm, 'c')))
call WTry('call remove(g:wm, "nope")')
call WFlush('removals logged')
call S('  dict', string(wm))
" a watched dict that is copied, deep copied, locked, and collected
let wcp = copy(wm)
let wdp = deepcopy(wm)
let wcp.z = 1
let wdp.z = 1
call WFlush('copies are unwatched')
lockvar 1 wm
call WTry('let g:wm.q = 1')
unlockvar 1 wm
call WFlush('locked')
let wm.self = wm
call garbagecollect(1)
let wm.after = 1
call WFlush('watched cycle after gc')
unlet wm.self
call dictwatcherdel(wm, '*', 'WRm')
" a watched dict dropped with its watcher still registered
let wgone = {}
call dictwatcheradd(wgone, '*', function('WLog', ['dropped']))
let wgone.a = 1
unlet wgone
call garbagecollect(1)
call WFlush('dropped with watcher')
" argument errors
call WTry('call dictwatcheradd([], "*", "WLog")')
call WTry('call dictwatcheradd({}, "", "WLog")')
call WTry('call dictwatcheradd({}, "*", 0)')
call WTry('call dictwatcherdel({}, "*", "WLog")')

call writefile(s:out, $DIFFOUT)
qall!
