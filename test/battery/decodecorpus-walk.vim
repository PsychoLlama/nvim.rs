" Walker differential: the encoders' shared container walk, the decoders'
" truncation behaviour, and the collector, each driven past what evalsweep's
" per-value matrix reaches.  Every section prints one line per case.
"
"   enc   string()/:echo/json_encode()/msgpackdump() over cycles, deep nests,
"         partials, special dicts and floats, with the exact error text.
"   pre   json_decode() of every byte-prefix of a set of valid documents:
"         the truncation corpus, which finds a slice bound where C read a NUL.
"   mp    msgpackparse() of every byte-prefix of a set of dumped values.
"   gc    deepcopy() and the collector over cycles through partials, dict
"         watchers, :for cursors and v: variables.
function! Try(expr) abort
  try
    let V = eval(a:expr)
    " The :echo form, which marks a cycle rather than throwing on it.
    return strtrans(execute('echon V'))
  catch
    return 'THROW ' . v:exception
  endtry
endfunction

function! Show(label, expr) abort
  let v:errmsg = ''
  let r = Try(a:expr)
  echo a:label . "\t" . r . (v:errmsg ==# '' ? '' : "\tERR " . v:errmsg)
endfunction

" ---- enc: containers that reach themselves --------------------------------
let g:l = [1, 2]
call add(g:l, g:l)
let g:d = {'a': 1}
let g:d.self = g:d
let g:d.list = [g:d, g:l]
let g:ll = [[[]]]
call add(g:ll[0][0], g:ll[0])
let g:dd = {'x': {'y': {}}}
let g:dd.x.y.z = g:dd.x
let g:shared = [1]
let g:twice = [g:shared, g:shared, {'k': g:shared}]
for name in ['g:l', 'g:d', 'g:ll', 'g:dd', 'g:twice']
  call Show('enc string ' . name, 'string(' . name . ')')
  call Show('enc json ' . name, 'json_encode(' . name . ')')
  call Show('enc mp ' . name, 'msgpackdump([' . name . '])')
  let v:errmsg = ''
  redir => g:echoed
  silent execute 'echo ' . name
  redir END
  echo 'enc echo ' . name . "\t" . strtrans(g:echoed)
endfor

" Deep nests, below and past the walk's inline frame budget.
for depth in [1, 7, 8, 9, 30, 100]
  let deep = []
  let cur = deep
  for i in range(depth)
    let next = [i]
    call add(cur, next)
    let cur = next
  endfor
  let deepd = {}
  let curd = deepd
  for i in range(depth)
    let curd['k' . i] = {}
    let curd = curd['k' . i]
  endfor
  echo 'enc deep ' . depth . "\t" . len(string(deep)) . ' ' . len(json_encode(deep))
        \ . ' ' . len(msgpackdump([deep])[0]) . ' ' . len(string(deepd))
        \ . ' ' . len(json_encode(deepd)) . ' ' . string(msgpackdump([deepd], 'B'))[:40]
  call add(cur, deep)
  let curd.back = deepd
  call Show('enc deepcyc string ' . depth, 'string(deep)[-60:]')
  call Show('enc deepcyc mp ' . depth, 'msgpackdump([deep])')
  call Show('enc deepcycd string ' . depth, 'string(deepd)[-60:]')
  call Show('enc deepcycd mp ' . depth, 'msgpackdump([deepd])')
endfor

" Partials: arguments, self dictionaries, cycles through both.
function! g:Fn(...) dict
  return a:000
endfunction
let g:pd = {'n': 1}
let g:P1 = function('g:Fn', [1, 'two', [3]])
let g:P2 = function('g:Fn', [g:pd], g:pd)
let g:pd.p = g:P2
let g:P3 = function('g:Fn', {})
let g:P4 = function('g:Fn', [[{'a': [function('tr')]}]])
let g:Lam = {x -> x + 1}
let g:P5 = function(g:Lam, [1])
let g:P6 = function('g:Fn', [g:l], g:d)
for name in ['g:P1', 'g:P2', 'g:P3', 'g:P4', 'g:P5', 'g:P6', 'g:pd',
      \ 'function("tr")', 'function("g:Fn")', '[function("tr"), g:P1]',
      \ '{"f": {"g": [1, function("tr")]}}']
  call Show('enc string ' . name, 'string(' . name . ')')
  call Show('enc json ' . name, 'json_encode(' . name . ')')
  call Show('enc mp ' . name, 'msgpackdump([' . name . '])')
  call Show('enc mpdeep ' . name, 'msgpackdump([1, [[' . name . ']]])')
endfor

" Floats.
for f in ['0.0', '-0.0', '1.0', '-1.5', '1.0e300', '-1.0e300', '1.0e-300',
      \ '5.0e-324', '1.7976931348623157e308', '0.1', '123456789.0', '1.0e15',
      \ '1.0e16', 'str2float("nan")', 'str2float("inf")', 'str2float("-inf")',
      \ '[str2float("nan")]', '{"a": str2float("inf")}']
  call Show('enc string ' . f, 'string(' . f . ')')
  call Show('enc json ' . f, 'json_encode(' . f . ')')
  call Show('enc mp ' . f, 'msgpackdump([' . f . '], "B")')
endfor

" Strings JSON has to escape or refuse.
for s in ['"\x01\x1f\x7f"', '"a\"b\\c/"', '"é€"', '"\U0001F600"',
      \ '"\xff"', '"a\xc3"', '"\xc3a"', '"\xed\xa0\x80"', '"\xed\xbf\xbf"',
      \ '"\xf4\x90\x80\x80"', '"\xe2\x80\xa8"', '"﻿"', '"￿"',
      \ '"\t\n\r\b\f"', '""', '0z', '0z00FF10', '0zDEADBEEF.01020304.05']
  call Show('enc string ' . s, 'string(' . s . ')')
  call Show('enc json ' . s, 'json_encode(' . s . ')')
  call Show('enc json key ' . s, 'json_encode({' . s . ': 1})')
  call Show('enc mp ' . s, 'msgpackdump([' . s . '], "B")')
endfor

" Special dictionaries, well-formed and not.
let s:mt = v:msgpack_types
let s:specials = [
      \ ['nil', {'_TYPE': s:mt.nil, '_VAL': 0}],
      \ ['bool1', {'_TYPE': s:mt.boolean, '_VAL': 1}],
      \ ['bool0', {'_TYPE': s:mt.boolean, '_VAL': 0}],
      \ ['boolbad', {'_TYPE': s:mt.boolean, '_VAL': 'x'}],
      \ ['intpos', {'_TYPE': s:mt.integer, '_VAL': [1, 3, 0x7fffffff, 0x7fffffff]}],
      \ ['intneg', {'_TYPE': s:mt.integer, '_VAL': [-1, 2, 0, 1]}],
      \ ['intzero', {'_TYPE': s:mt.integer, '_VAL': [0, 0, 0, 1]}],
      \ ['intshort', {'_TYPE': s:mt.integer, '_VAL': [1, 0, 1]}],
      \ ['intneg2', {'_TYPE': s:mt.integer, '_VAL': [1, -1, 0, 1]}],
      \ ['intstr', {'_TYPE': s:mt.integer, '_VAL': [1, 0, '1', 1]}],
      \ ['float', {'_TYPE': s:mt.float, '_VAL': 1.5}],
      \ ['floatnum', {'_TYPE': s:mt.float, '_VAL': 1}],
      \ ['floatnan', {'_TYPE': s:mt.float, '_VAL': str2float('nan')}],
      \ ['str', {'_TYPE': s:mt.string, '_VAL': ['a', 'b']}],
      \ ['strnul', {'_TYPE': s:mt.string, '_VAL': ["a\nb"]}],
      \ ['strempty', {'_TYPE': s:mt.string, '_VAL': []}],
      \ ['strnum', {'_TYPE': s:mt.string, '_VAL': [1]}],
      \ ['strbadutf', {'_TYPE': s:mt.string, '_VAL': ["\xff"]}],
      \ ['arr', {'_TYPE': s:mt.array, '_VAL': [1, [2]]}],
      \ ['arrempty', {'_TYPE': s:mt.array, '_VAL': []}],
      \ ['arrbad', {'_TYPE': s:mt.array, '_VAL': 1}],
      \ ['map', {'_TYPE': s:mt.map, '_VAL': [['a', 1], ['b', [2]]]}],
      \ ['mapkeys', {'_TYPE': s:mt.map, '_VAL': [[1, 'one'], [[], 2], [{}, 3]]}],
      \ ['mapempty', {'_TYPE': s:mt.map, '_VAL': []}],
      \ ['mapbad', {'_TYPE': s:mt.map, '_VAL': [['a']]}],
      \ ['mapbad2', {'_TYPE': s:mt.map, '_VAL': [1]}],
      \ ['mapstrkey', {'_TYPE': s:mt.map, '_VAL': [[{'_TYPE': s:mt.string, '_VAL': ["a\nb"]}, 1]]}],
      \ ['mapstrkeybad', {'_TYPE': s:mt.map, '_VAL': [[{'_TYPE': s:mt.string, '_VAL': [1]}, 1]]}],
      \ ['ext', {'_TYPE': s:mt.ext, '_VAL': [5, ['ab', 'c']]}],
      \ ['extneg', {'_TYPE': s:mt.ext, '_VAL': [-128, []]}],
      \ ['extbig', {'_TYPE': s:mt.ext, '_VAL': [128, ['x']]}],
      \ ['extbad', {'_TYPE': s:mt.ext, '_VAL': [1, [1]]}],
      \ ['extshort', {'_TYPE': s:mt.ext, '_VAL': [1]}],
      \ ['notype', {'_TYPE': [], '_VAL': 1}],
      \ ['three', {'_TYPE': s:mt.nil, '_VAL': 0, 'x': 1}],
      \ ['typestr', {'_TYPE': 'nil', '_VAL': 0}],
      \ ['noval', {'_TYPE': s:mt.nil, '_XAL': 0}],
      \ ]
for [name, value] in s:specials
  let g:sp = value
  call Show('enc sp string ' . name, 'string(g:sp)')
  call Show('enc sp json ' . name, 'json_encode(g:sp)')
  call Show('enc sp jsonnest ' . name, 'json_encode([{"k": g:sp}])')
  call Show('enc sp mp ' . name, 'msgpackdump([g:sp], "B")')
  call Show('enc sp mprt ' . name, 'msgpackparse(msgpackdump([g:sp]))')
endfor
" A special map's _VAL that reaches itself, and one that reaches the map.
let g:spself = {'_TYPE': s:mt.map, '_VAL': [['k', 1]]}
call add(g:spself._VAL[0], 0)
call remove(g:spself._VAL[0], 2)
let g:spself._VAL[0][1] = g:spself
call Show('enc spself json', 'json_encode(g:spself)')
call Show('enc spself mp', 'msgpackdump([g:spself])')
call Show('enc spself string', 'string(g:spself)')
let g:spval = {'_TYPE': s:mt.map, '_VAL': []}
call add(g:spval._VAL, ['a', g:spval._VAL])
call Show('enc spval json', 'json_encode(g:spval)')
call Show('enc spval mp', 'msgpackdump([g:spval])')
let g:sparr = {'_TYPE': s:mt.array, '_VAL': [1]}
call add(g:sparr._VAL, g:sparr._VAL)
call Show('enc sparr json', 'json_encode(g:sparr)')
call Show('enc sparr mp', 'msgpackdump([g:sparr])')
let g:sparr2 = {'_TYPE': s:mt.array, '_VAL': [1]}
call add(g:sparr2._VAL, g:sparr2)
call Show('enc sparr2 json', 'json_encode(g:sparr2)')
call Show('enc sparr2 mp', 'msgpackdump([g:sparr2])')
call Show('enc mp errpath', 'msgpackdump([{"a": [1, {"b": {"_TYPE": v:msgpack_types.map, "_VAL": [["k", function("tr")]]}}]}])')
call Show('enc mp errpath2', 'msgpackdump([1, 2, {"x": g:P1}])')
call Show('enc json errpath', 'json_encode({"a": [1, {"b": [function("tr")]}]})')
call Show('enc json partialarg', 'json_encode([g:P1])')
call Show('enc mp partialself', 'msgpackdump([[g:P2]])')
call Show('enc v:null etc', 'msgpackdump([v:null, v:true, v:false, [v:null]], "B")')
call Show('enc json v:null etc', 'json_encode([v:null, v:true, v:false, {}, [], ""])')
call Show('enc string v:null etc', 'string([v:null, v:true, v:false, {}, [], "", 0z])')
call Show('enc mp list', 'msgpackdump(["a\nb", 0z00, "x"])')
call Show('enc mp nested empty', 'msgpackdump([[[], {}], {"": []}], "B")')

" ---- pre: every prefix of a valid JSON document ----------------------------
let s:docs = [
      \ '{"a": [1, 2.5, -3e2, true, false, null], "b": {"c": "dé\\n"}}',
      \ '[[], {}, "", [[[]]], {"x": {"y": {"z": []}}}]',
      \ '"😀 é \"q\" \\ \/ \b\f\n\r\t"',
      \ '{"a": 1, "a": 2, "": 3, "b": [{"c": 1, "c": 2}]}',
      \ '-12.5e-3',
      \ '[1e400, -1e400, 9223372036854775808, -9223372036854775809]',
      \ "[\"\xc3\xa9\", \"\xe2\x82\xac\", \"\xf0\x9f\x98\x80\"]",
      \ '  {  "k"  :  [  1  ,  2  ]  }  ',
      \ ]
for doc in s:docs
  for n in range(len(doc) + 1)
    let g:pre = strpart(doc, 0, n)
    call Show('pre ' . strtrans(g:pre), 'json_decode(g:pre)')
  endfor
  " The spaced document's joined-list error tail differs at the pinned
  " reference (its message read past the joined buffer), so it is not here.
  if doc !~# '^  '
    let g:doc = doc
    call Show('prelist ' . strtrans(doc), 'json_decode(split(g:doc, ","))')
  endif
endfor

" ---- mp: every prefix of a dumped value ------------------------------------
" Blob prefixes stop short of one shape: an array whose element is a
" str/bin/ext/map cut off mid-way.  The old decoder appended that element
" empty and cleared the half-built list, which is an internal error (E685)
" upstream and a debug assertion here; it is a lib test, not a row.
let s:blobvalues = [
      \ [1, -1, 300, -300, 70000, -70000, 5000000000, -5000000000, 1.5, v:null, v:true, [2, [3]]],
      \ {'a': 'str', 'b': {'c': 0z0102, 'd': [1, 2]}, 'e': {'_TYPE': v:msgpack_types.ext, '_VAL': [3, ['abc']]}},
      \ {'_TYPE': v:msgpack_types.map, '_VAL': [[1, 2], ['', 3]]},
      \ repeat('y', 40),
      \ {'_TYPE': v:msgpack_types.ext, '_VAL': [-3, ['a', 'b']]},
      \ ]
for value in s:blobvalues
  let bytes = msgpackdump([value], 'B')
  for n in range(len(bytes) + 1)
    let g:pre = bytes[: n - 1]
    if n == 0
      let g:pre = 0z
    endif
    call Show('mp ' . n . ' ' . string(g:pre), 'msgpackparse(g:pre)')
  endfor
endfor
let s:values = s:blobvalues + [
      \ {"k\nx": "a\nb", 'big': repeat('y', 300), 'x': {"\n": "\n\n"}},
      \ ]
for value in s:values
  let lines = msgpackdump([value, value])
  for n in range(len(lines) + 1)
    let g:prel = lines[: n - 1]
    if n == 0
      let g:prel = []
    endif
    call Show('mpl ' . n, 'msgpackparse(g:prel)')
  endfor
endfor
call Show('mpl notstr', 'msgpackparse([1])')
call Show('mpl notstr2', 'msgpackparse(["\x91", 1])')
call Show('mpl split', 'msgpackparse(["\x92\xa1", "\x01"])')

" ---- gc: cycles through partials, watchers, :for cursors, v: vars ----------
let v:testing = 1
function! g:Mk() abort
  let d = {}
  let d.f = function('g:Fn', [d], d)
  let d.l = [d]
  return d
endfunction
let g:keep = g:Mk()
call g:Mk()
let g:Lamcount = 0
function! g:MkLam() abort
  let d = {}
  let d.lam = {-> d}
  return string(d.lam)
endfunction
let g:lname = g:MkLam()
let g:lname2 = g:MkLam()
let g:keeplam = g:MkLam()
let g:watched = {}
let g:wlog = []
function! g:Watcher(d, k, z) abort
  call add(g:wlog, [a:k, a:z])
endfunction
call dictwatcheradd(g:watched, '*', function('g:Watcher', [], {'cyc': g:watched}))
let g:watched.x = 1
let g:copied = deepcopy(g:keep)
let g:copied2 = deepcopy([g:l, g:d, g:P2, g:keep])
call Show('gc deep', '[g:copied.l[0] is g:copied, g:copied2[0][2] is g:copied2[0], get(g:copied2[1], "self") is g:copied2[1]]')
call test_garbagecollect_now()
call Show('gc 1', '[g:keep, g:copied2[3]]')
let g:watched.y = 2
call Show('gc wlog', 'g:wlog')
let g:forlog = []
let g:forlist = [[1], {'a': 1}, function('g:Fn', [{}]), 4]
for Item in g:forlist
  if len(g:forlist) < 8
    call add(g:forlist, [Item])
  endif
  call test_garbagecollect_now()
  call add(g:forlog, string(Item))
endfor
call Show('gc for', 'g:forlog')
let v:errors = [g:l, {'k': g:d}]
call test_garbagecollect_now()
call Show('gc v:', 'v:errors')
let v:errors = []
call garbagecollect()
call garbagecollect(1)
call test_garbagecollect_now()
call Show('gc 2', '[g:keep.f, g:copied2]')
call Show('gc lam', '[g:lname, g:lname2, g:keeplam]')
let g:cyc = {}
let g:cyc.me = g:cyc
let g:cyc.p = function('g:Fn', [g:cyc])
unlet g:cyc
call test_garbagecollect_now()
call Show('gc after', '[g:keep, g:pd, g:P2, g:watched]')
