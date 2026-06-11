" Funcref / LuaRef marshalling regression smoke test.
"
" Guards against the class of bug where nvim_tv_set_string is called after
" nvim_tv_set_type(VAR_FUNC), causing v_type to be clobbered back to
" VAR_STRING.  Symptoms: luaeval('function()...end') returns a plain string
" instead of a Funcref; type() reports 1 (String) instead of 2 (Func).
"
" Root cause was fixed in:
"   src/nvim-rs/lua/src/api.rs  (nlua_pop_typval, LUA_TFUNCTION arm)
"   src/nvim-rs/typval/src/lib.rs (rs_tv_dict_add_func)
" by using nvim_tv_set_string_val (field-only setter) instead of
" nvim_tv_set_string (which resets v_type to VAR_STRING).
"
" IMPORTANT: Must complete within the timeout enforced by just smoke-test-funcref
" (15 seconds).

let s:errors = []
let s:pass_count = 0

function! s:ok(name)
  let s:pass_count += 1
endfunction

function! s:fail(name, msg)
  call add(s:errors, a:name .. ': ' .. a:msg)
endfunction

" ============================================================================
" Test 1: luaeval of a lambda is a Funcref (type == 2), not a String (type == 1)
" Regression: VAR_FUNC was clobbered to VAR_STRING by nvim_tv_set_string,
" causing type() to return 1 (String) instead of 2 (Funcref).
" ============================================================================

let s:fn_type = type(luaeval('function() return 1 end'))
if s:fn_type ==# 2
  call s:ok('luaeval-lambda-type-is-funcref')
else
  call s:fail('luaeval-lambda-type-is-funcref', 'expected type 2 (Funcref), got ' .. s:fn_type)
endif

" ============================================================================
" Test 2: Funcref returned from luaeval has correct string representation
" (should look like function('<lambda>N'), not plain '<lambda>N').
" ============================================================================

let s:fn_str = string(luaeval('function() return 1 end'))
if s:fn_str =~# "^function('<lambda>"
  call s:ok('luaeval-lambda-string-repr')
else
  call s:fail('luaeval-lambda-string-repr', 'expected function(''<lambda>N'') repr, got: ' .. s:fn_str)
endif

" ============================================================================
" Test 3: A dict returned by luaeval that contains a function value has
" the callback entry typed as Funcref (type == 2), not String (type == 1).
" This exercises the LUA_TFUNCTION arm inside nlua_pop_typval for nested
" table entries.
" ============================================================================

let s:d = luaeval('{cb = function() return 7 end}')
let s:cb_type = type(s:d.cb)
if s:cb_type ==# 2
  call s:ok('luaeval-dict-cb-type-is-funcref')
else
  call s:fail('luaeval-dict-cb-type-is-funcref', 'expected type 2 (Funcref), got ' .. s:cb_type)
endif

" ============================================================================
" Report results
" ============================================================================

if len(s:errors) == 0
  echomsg 'funcref-smoke: ALL ' .. s:pass_count .. ' tests passed'
else
  for s:err in s:errors
    echoerr 'FAIL: ' .. s:err
  endfor
  echoerr 'funcref-smoke: ' .. len(s:errors) .. ' FAILED, ' .. s:pass_count .. ' passed'
  cquit!
endif

qall!
