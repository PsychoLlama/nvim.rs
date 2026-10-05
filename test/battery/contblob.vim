" Differential: blob ownership, references, ranges, conversions and errors.
" Run headless; every line goes to the file named by $DIFFOUT.
" Nothing here may depend on the build, only on observable behaviour.
"
" Every variable is `g:`-scoped: the probes below evaluate their expression
" inside a function, where an unscoped name would be function-local.

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
" As T, but for a statement rather than an expression.
func! X(what, cmd) abort
  try
    execute a:cmd
    call S(a:what, 'ok')
  catch
    call S(a:what, 'threw', substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
  endtry
endfunc

" ------------------------------------------------------------ 1. literals
call S('=== 1. literals ===')
call T('empty', '0z')
call T('bytes', '0zDEADBEEF')
call T('lower', '0zdeadbeef')
call T('dots', '0zDE.AD.BE.EF')
call T('odd digits', '0zDEA')
call T('bad digit', '0zDEADBEEG')
call T('type', 'type(0zDEAD)')

" ------------------------------------------------------------ 2. sharing
call S('=== 2. sharing ===')
let g:one = 0z0011
let g:two = g:one
let g:two[0] = 0xFF
call S('aliased', string(g:one), string(g:two), g:one is g:two)
let g:shallow = copy(g:one)
let g:shallow[1] = 0x22
call S('copy is a new blob', string(g:one), string(g:shallow), g:one is g:shallow)
let g:deep = deepcopy(g:one)
let g:deep[0] = 0x01
call S('deepcopy', string(g:one), string(g:deep), g:one is g:deep)
call S('is against equal', 0z00 is 0z00, 0z00 == 0z00)

" ------------------------------------------------------------ 3. indexing
call S('=== 3. indexing ===')
let g:four = 0z00112233
call T('index', 'g:four[0]')
call T('index last', 'g:four[3]')
call T('negative index', 'g:four[-1]')
call T('out of range', 'g:four[4]')
call T('out of range negative', 'g:four[-5]')
call T('slice', 'g:four[1:2]')
call T('slice open end', 'g:four[1:]')
call T('slice open start', 'g:four[:1]')
call T('slice all', 'g:four[:]')
call T('slice reversed', 'g:four[2:1]')
call T('slice past end', 'g:four[2:99]')
call T('slice negative', 'g:four[-2:-1]')
call T('slice is a new blob', 'g:four[0:1] is g:four[0:1]')

" ------------------------------------------------------------ 4. assignment
call S('=== 4. assignment ===')
let g:set = 0z00112233
call X('set', 'let g:set[1] = 0xAA')
call S('  ->', string(g:set))
let g:setneg = 0z00112233
call X('set negative', 'let g:setneg[-1] = 0xBB')
call S('  ->', string(g:setneg))
let g:grow = 0z0011
call X('append past end', 'let g:grow[2] = 0xCC')
call S('  ->', string(g:grow))
let g:far = 0z0011
call X('set far past end', 'let g:far[5] = 0xDD')
call S('  ->', string(g:far))
let g:range = 0z00112233
call X('range assign', 'let g:range[1:2] = 0zAABB')
call S('  ->', string(g:range))
let g:short = 0z00112233
call X('range assign short', 'let g:short[1:2] = 0zAA')
call S('  ->', string(g:short))
let g:long = 0z00112233
call X('range assign long', 'let g:long[1:2] = 0zAABBCC')
call S('  ->', string(g:long))
" Self-assignment: the source and the destination are one object.
let g:self = 0z00112233
call X('range assign from self', 'let g:self[0:1] = g:self[2:3]')
call S('  ->', string(g:self))
let g:overlap = 0z00112233
call X('overlapping range assign', 'let g:overlap[1:2] = g:overlap[0:1]')
call S('  ->', string(g:overlap))

" ------------------------------------------------------------ 5. concatenation
call S('=== 5. concatenation ===')
call T('concat', '0z0011 + 0z2233')
call T('concat empty left', '0z + 0z2233')
call T('concat empty right', '0z0011 + 0z')
let g:acc = 0z0011
let g:alias = g:acc
let g:acc += 0z2233
call S('add-assign rebinds', string(g:acc), string(g:alias), g:acc is g:alias)
call T('concat string', '0z0011 + "x"')
call T('concat number', '0z0011 + 1')
call T('concat list', '0z0011 + [1]')

" ------------------------------------------------------------ 6. builtins
call S('=== 6. builtins ===')
call T('len', 'len(0z00112233)')
call T('len empty', 'len(0z)')
call T('empty()', 'empty(0z)')
call T('empty() nonempty', 'empty(0z00)')
call T('add', 'add(0z0011, 0x22)')
call T('add out of range', 'add(0z0011, 256)')
call T('add negative', 'add(0z0011, -1)')
call T('add string', 'add(0z0011, "x")')
call T('remove one', 'remove(0z00112233, 1)')
call T('remove range', 'remove(0z00112233, 1, 2)')
call T('remove negative', 'remove(0z00112233, -1)')
call T('remove bad range', 'remove(0z00112233, 2, 1)')
call T('remove out of range', 'remove(0z0011, 5)')
call T('insert', 'insert(0z1122, 0x00)')
call T('insert at', 'insert(0z1122, 0xFF, 1)')
call T('insert past end', 'insert(0z1122, 0xFF, 9)')
call T('index()', 'index(0z00112233, 0x22)')
call T('index() missing', 'index(0z00112233, 0x99)')
call T('index() from', 'index(0z00110011, 0x11, 2)')
call T('count()', 'count(0z00110011, 0x11)')
call T('reverse', 'reverse(0z00112233)')
call T('repeat', 'repeat(0z0011, 3)')
call T('repeat zero', 'repeat(0z0011, 0)')
call T('repeat negative', 'repeat(0z0011, -1)')
call T('string()', 'string(0z00112233)')
call T('blob2list', 'blob2list(0z00112233)')
call T('blob2list empty', 'blob2list(0z)')
call T('list2blob', 'list2blob([0, 17, 34])')
call T('list2blob out of range', 'list2blob([256])')
call T('list2blob bad', 'list2blob(["x"])')
call T('list2blob empty', 'list2blob([])')

" ------------------------------------------------------------ 7. coercion
call S('=== 7. coercion ===')
call T('to number', '0z0011 + 0')
call T('to string', '"" . 0z0011')
call T('as bool', '0z ? 1 : 2')
call T('as bool nonempty', '0z00 ? 1 : 2')
call T('compare blobs equal', '0z0011 == 0z0011')
call T('compare blobs unequal', '0z0011 == 0z1100')
call T('compare with string', '0z0011 == "0z0011"')
call T('compare with number', '0z0011 == 1')
call T('sort of blobs', 'sort([0z11, 0z00])')
call T('json_encode', 'json_encode(0z0011)')
call T('printf %s', 'printf("%s", 0z0011)')
call T('eval(string())', 'eval(string(0z00112233))')

" ------------------------------------------------------------ 8. containers
call S('=== 8. containers ===')
let g:dict = {'b': 0z0011}
let g:held = g:dict.b
let g:held[0] = 0xFF
call S('dict value is a reference', string(g:dict), string(g:held), g:dict.b is g:held)
let g:list = [0z0011]
let g:elem = g:list[0]
let g:elem[0] = 0xEE
call S('list element is a reference', string(g:list), string(g:elem), g:list[0] is g:elem)
let g:copied = deepcopy({'b': 0z0011})
let g:copied.b[0] = 0xDD
call S('deepcopy of a container', string(g:copied))
call T('nested string()', 'string({"b": 0z00, "l": [0z11]})')

" ------------------------------------------------------------ 9. locking
call S('=== 9. locking ===')
let g:locked = 0z0011
lockvar g:locked
call X('locked set', 'let g:locked[0] = 0xFF')
call X('locked add', 'call add(g:locked, 0x22)')
call S('  ->', string(g:locked))
unlockvar g:locked
call X('unlocked set', 'let g:locked[0] = 0x77')
call S('  ->', string(g:locked))
call T('islocked', 'islocked("g:locked")')

" ------------------------------------------------------------ 10. for and functions
call S('=== 10. for and functions ===')
let g:seen = []
for g:byte in 0z00112233
  call add(g:seen, g:byte)
endfor
call S('for over a blob', string(g:seen))
" Mutating the blob the loop walks.
let g:walked = 0z00112233
let g:partial = []
try
  for g:byte in g:walked
    call add(g:partial, g:byte)
    if len(g:walked) > 2
      call remove(g:walked, -1)
    endif
  endfor
  call S('for over a shrinking blob', string(g:partial), string(g:walked))
catch
  call S('for over a shrinking blob threw', string(g:partial), substitute(v:exception, '\v^Vim\(\a+\)?:?', '', ''))
endtry
func! Ret() abort
  return 0z0011
endfunc
call T('returned blob', 'Ret()')
call T('map over a blob', 'map(copy(0z00112233), "v:val + 1")')
call T('filter over a blob', 'filter(copy(0z00112233), "v:val > 17")')
call T('reduce over a blob', 'reduce(0z00112233, {a, b -> a + b}, 0)')
call T('call with a blob', 'call("len", [0z00112233])')
call T('function result identity', 'Ret() is Ret()')

" ------------------------------------------------------------ 11. null and errors
call S('=== 11. null and errors ===')
call T('null blob compare', 'v:_null_blob == 0z')
call T('null blob len', 'len(v:_null_blob)')
call T('null blob string', 'string(v:_null_blob)')
call T('null blob index', 'v:_null_blob[0]')
call T('null blob slice', 'v:_null_blob[0:1]')
call T('null blob add', 'add(v:_null_blob, 1)')
call T('null blob concat', 'v:_null_blob + 0z00')
call T('null blob blob2list', 'blob2list(v:_null_blob)')
call T('null blob copy', 'copy(v:_null_blob)')
call T('null blob deepcopy', 'deepcopy(v:_null_blob)')
call T('null blob remove', 'remove(v:_null_blob, 0)')
call T('null blob empty()', 'empty(v:_null_blob)')
call T('null blob type', 'type(v:_null_blob)')
call X('null blob for', 'for g:b in v:_null_blob | endfor')
call X('assign into a null blob', 'let g:nb = v:_null_blob | let g:nb[0] = 1')
call T('index a number', '(1)[0]')
call T('blob to float', 'str2float(string(0z00))')

call writefile(s:out, $DIFFOUT)
qa!
