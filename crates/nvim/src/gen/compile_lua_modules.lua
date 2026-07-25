-- Compile the builtin `vim.*` Lua modules to LuaJIT bytecode for embedding.
--
-- Rust's build.rs runs this (with the deps-prefix luajit) in place of the
-- upstream CMake + src/gen/gen_char_blob.lua pipeline that produced the
-- bytecode c2rust originally transpiled as array literals in
-- src/nvim/lua/executor.rs. The compile step is byte-for-byte the same as
-- gen_char_blob.lua's `-c` mode: an unstripped string.dump() of the source
-- loaded under a chunkname of `@<mod/path>` (the `@` omitted for the
-- ignorelist below, as upstream does), plus the trailing 0 sentinel the C
-- arrays carried (the loader passes `size - 1` to luaL_loadbuffer).
--
-- NB: the dump is not deterministic across processes. LuaJIT's randomized
-- string hashing changes the order table-constructor template tables
-- serialize in, so rebuilding produces different (semantically identical)
-- bytes. Upstream's build has the same property.
--
-- Usage: luajit compile_lua_modules.lua <outdir> <source> <modname> ...
-- Writes <outdir>/<modname with '.' -> '_dot_'>_module.bin per module.

local outdir = table.remove(arg, 1) or error('Need an output directory')
assert(#arg >= 2 and #arg % 2 == 0, 'Need <source> <modname> pairs')

-- Modules upstream compiles without the `@` chunkname prefix, so their
-- chunks print in error messages as source text rather than file names.
local ignorelist = {
  ['vim._core.editor'] = true,
  ['vim._core.options'] = true,
  ['vim.keymap'] = true,
}

for argi = 1, #arg, 2 do
  local source_file, modname = arg[argi], arg[argi + 1]

  local f = assert(io.open(source_file, 'r'), source_file .. " doesn't exist")
  local source = f:read('*a')
  f:close()

  local prefix = ignorelist[modname] and '' or '@'
  local relpath = modname:gsub('%.', '/')
  local bytecode = string.dump(assert(loadstring(source, prefix .. relpath)), false)

  local out_path = outdir .. '/' .. modname:gsub('%.', '_dot_') .. '_module.bin'
  local out = assert(io.open(out_path, 'wb'))
  out:write(bytecode)
  out:write(string.char(0))
  out:close()
end
