//! Deciding what to call the swap file, and what to do when
//! that name is taken.
//!
//! `findswapname` walks `'directory'` for a name nothing else is using. A name
//! that *is* using it means either a crash to recover from or another Nvim with
//! the file open, which is what `attention_message` and the `SwapExists`
//! autocommand (`do_swapexists`) exist to sort out.
//!
//! `recover_names` walks the same option in the other direction: which swap
//! files already exist for a given file, for `:recover`, `swapfilelist()` and
//! the ATTENTION message.

#![forbid(unsafe_code)]

use crate::autocmd::fire_autocmds_for;
use crate::buffer::BufFlags;
use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::guard::{Lock, Suppress};
use crate::mbyte::cluster_len;
use crate::memory::ThinCString;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::option::split_option_part;
use crate::option::vars::P_DIR;
use crate::os::env::home_replace_in;
use crate::os::fs::{dir_exists, file_info, link_exists, mkdir_recurse};
use crate::path::ExpandFlags;
use crate::semsg;
use crate::types::TypVal;
use ::libc::{EINVAL, ENOENT};
use core::ffi::{CStr, c_int, c_uint};

use super::*;
use crate::types::{CmdModFlags, Failed, IOSIZE, MAXPATHL, NUL, ShmFlag, Vv};

/// One entry of `'directory'` after another: the entry, and whether it is
/// the last.
fn directory_entries(dir: &[u8]) -> impl Iterator<Item = (Vec<u8>, bool)> + '_ {
    let mut rest = dir;
    core::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        // The rest of the option is the longest one entry can be, so it is
        // its own bound.
        let (part, after) = split_option_part(rest, rest.len() + 1, b",");
        rest = after;
        Some((part, rest.is_empty()))
    })
}

/// Rename the swap file after the buffer's file name changed.
///
/// The name is what identifies the swap file to the next `:recover`, so it
/// has to follow the file. Failing that, the swap file is at least reopened
/// under its old name — losing it entirely is worse than a stale name.
pub fn ml_setname(mut buffer: Buf) {
    if buffer.b_ml.memfile().mf_fd < 0 {
        // There is no swap file yet: with `'updatecount'` zero and
        // `'noswapfile'` there never was one. Help files get one now.
        if p_uc() != 0 && !cmdmod_has(CmdModFlags::NOSWAPFILE) {
            ml_open_file(buffer);
        }
        return;
    }

    // Try every directory in 'directory'.
    let mut success = false;
    // A copy: the walk outlives a projection's borrow, and the option may
    // change under the autocommands it runs.
    let dir = P_DIR.get();
    let mut found_existing_dir = false;
    for (dir_name, last) in directory_entries(&dir) {
        // A copy of the name the swap file has now: finding a new one can
        // run autocommands.
        let old = buffer.b_ml.memfile().fname().map(CStr::to_owned);
        let found = findswapname(
            buffer,
            &dir_name,
            last,
            old.as_deref(),
            &mut found_existing_dir,
        );
        let Some(fname) = found else {
            continue; // no name found for this directory
        };
        let old = old.unwrap_or_default();

        // Already called that: nothing to do.
        if path_fnamecmp(&fname, old.to_bytes()) == 0 {
            success = true;
            break;
        }
        // The swap file has to be closed before it can be renamed.
        buffer.b_ml.memfile().close_fd();
        if rename_file(&old, fname.as_cstr()) {
            success = true;
            let memfile = buffer.b_ml.memfile();
            memfile.free_fnames();
            memfile.set_fnames(fname.as_cstr().to_owned());
            ml_upd_block0(buffer, UB_SAME_DIR);
            break;
        }
        // This name did not work, try another.
    }

    if buffer.b_ml.memfile().mf_fd == -1 && !buffer.b_ml.memfile().reopen() {
        // Could not reopen the swap file. Nothing can be done.
        complain(c"E301: Oops, lost the swap file!!!");
        return;
    }
    if !success {
        complain(c"E302: Could not rename swap file");
    }
}

/// Append `name`'s full path to `dir` (a `'directory'` entry without its
/// extra separator), with every path separator turned into a `%`, so that an
/// entry ending in `//` can hold the swap files of files from everywhere
/// without their names colliding.
///
/// `None` is the unnamed buffer's `""`, i.e. `<currentdir>/""`.
pub(crate) fn percent_swname(dir: &[u8], name: Option<&CStr>) -> XString {
    let f = fix_fname(name.unwrap_or(c""));

    // Every path separator becomes a `%`. Only the lead byte of a character
    // is tested, so the walk steps a whole character at a time.
    let mut mangled: Vec<u8> = f.to_vec();
    let mut i = 0;
    while i < mangled.len() {
        if vim_ispathsep(c_int::from(mangled[i])) {
            mangled[i] = b'%';
        }
        i += cluster_len(&mangled[i..]);
    }
    concat_fnames(dir, &mangled, true)
}

/// Resolve a symbolic link in the *last* component of a file name, and
/// answer the full name it resolves to.
///
/// `resolve()` in Vimscript does this for every part of the path; this does
/// not. `None` when the caller should keep the name it already has: not a
/// link, a link that could not be read, or a resolved name that could not be
/// made full.
pub fn resolve_symlink(fname: &CStr) -> Option<Vec<u8>> {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};

    // The result so far, starting with the original name.
    let room = MAXPATHL as usize;
    let mut tmp: Vec<u8> = fname.to_bytes()[..fname.count_bytes().min(room - 1)].to_vec();

    let mut depth = 0;
    loop {
        // Bound the depth, to catch a loop of links pointing at each
        // other.
        depth += 1;
        if depth == 100 {
            semsg!("E773: Symlink loop for \"{}\"", msg_cstr(fname));
            return None;
        }

        let path = std::path::Path::new(std::ffi::OsStr::from_bytes(&tmp));
        let target = match std::fs::read_link(path) {
            Ok(target) => target.into_os_string().into_vec(),
            Err(err) => {
                let errno = err.raw_os_error().unwrap_or(0);
                if errno != EINVAL && errno != ENOENT {
                    // Some other error reading the link: keep the original.
                    return None;
                }
                // Not a symlink, or it does not exist, so `tmp` is as
                // resolved as it gets. At the first level that means
                // nothing was resolved at all, and the caller keeps its
                // own name.
                if depth == 1 {
                    return None;
                }
                break;
            }
        };
        // What a `MAXPATHL - 1` buffer would have held of it.
        let target = &target[..target.len().min(room - 1)];
        if target.is_empty() {
            if depth == 1 {
                return None;
            }
            break;
        }
        let target = cstr::until_nul(target);

        // A relative link is relative to the directory of the name it
        // was found in, so it replaces only the tail of `tmp`.
        if path_is_absolute(&cstr::owned(target)) {
            tmp = target.to_vec();
        } else {
            let tail = tail_index(&tmp);
            if tmp.len() - tail + target.len() >= room {
                return None;
            }
            tmp.truncate(tail);
            tmp.extend_from_slice(target);
        }
    }

    // Resolve the full name too, so that opening the same relative
    // symlink from two working directories still picks one swap file.
    vim_full_name(&cstr::owned(&tmp), room, true).ok()
}

/// Whether a `'directory'` entry ends in *two* path separators, which is
/// what asks for the swap file's name to encode the file's full path.
///
/// The caller has already established with `after_pathsep` that the last
/// byte is a separator and not a character's trailing byte.
fn ends_with_double_sep(dir: &[u8]) -> bool {
    dir.len() > 1 && dir[dir.len() - 1] == dir[dir.len() - 2]
}

/// The swap file name for `fname` under the `'directory'` entry `dir_name`.
///
/// `None` is an unnamed buffer.
pub(crate) fn makeswapname(fname: Option<&CStr>, dir_name: &[u8]) -> Option<XString> {
    // Expand a symlink, so that the swap file goes with the actual file
    // rather than with the link.
    let resolved = fname
        .and_then(resolve_symlink)
        .map(|name| cstr::owned(&name));
    let fname_res = resolved.as_deref().or(fname);

    if after_pathsep(dir_name, dir_name.len()) && ends_with_double_sep(dir_name) {
        // Ends with "//": the swap file's name encodes the full path.
        let dir = &dir_name[..dir_name.len() - 1];
        let s = percent_swname(dir, fname_res);
        return modname(Some(s.as_cstr()), b".swp", false);
    }

    // A swap file in the file's own directory gets a leading '.'.
    let r = modname(fname_res, b".swp", dir_name == b".")?;
    Some(file_in_dir(&r, dir_name))
}

/// Place `fname` in the directory `dir` names, for a swap or backup file.
///
/// - `dir` is `"."`: beside the file, i.e. `fname` itself.
/// - `dir` starts with `"./"`: relative to the file's own directory, with
///   the rest of `dir` spliced in before the tail.
/// - otherwise: in `dir`, under `fname`'s tail.
pub(crate) fn file_in_dir(fname: &[u8], dir: &[u8]) -> XString {
    let tail_at = tail_index(fname);
    let tail = &fname[tail_at..];
    let relative =
        dir.first() == Some(&b'.') && dir.get(1).is_some_and(|&c| vim_ispathsep(c.into()));
    if dir == b"." {
        XString::from_bytes(fname)
    } else if relative {
        let rest = &dir[2..];
        if tail_at == 0 {
            // No path in front of the file name.
            concat_fnames(rest, tail, true)
        } else {
            let t = concat_fnames(&fname[..tail_at], rest, true);
            concat_fnames(&t, tail, true)
        }
    } else {
        concat_fnames(dir, tail, true)
    }
}

/// Build the ATTENTION message: what is known about the swap file that is in
/// the way, and what the user can do about it.
///
/// `fhname` is `fname` with the home directory replaced by `~`; `shown` is
/// the buffer's name.
fn attention_message(fname: &CStr, fhname: &[u8], shown: &CStr, msg: &mut Vec<u8>) {
    complain(c"E325: ATTENTION");
    push_tr(msg, c"Found a swap file by the name \"");
    msg.extend_from_slice(fhname);
    msg.extend_from_slice(b"\"\n");
    let swap_mtime = swapfile_info(fname, msg);
    push_tr(msg, c"While opening file \"");
    msg.extend_from_slice(shown.to_bytes());
    msg.extend_from_slice(b"\"\n");

    match file_info(shown) {
        None => push_tr(msg, c"      CANNOT BE FOUND"),
        Some(info) => {
            push_tr(msg, c"             dated: ");
            let x = info.stat.st_mtim.tv_sec as time_t;
            msg.extend_from_slice(&ctime_text(x, 50, true));
            if swap_mtime != 0 && x > swap_mtime {
                push_tr(msg, c"      NEWER than swap file!\n");
            }
        }
    }

    // Some of these are long, to leave room for translation.
    push_tr(
        msg,
        c"\n(1) Another program may be editing the same file.  If this is the case,\n    be careful not to end up with two different instances of the same\n    file when making changes.  Quit, or continue with caution.\n",
    );
    push_tr(msg, c"(2) An edit session for this file crashed.\n");
    push_tr(
        msg,
        c"    If this is the case, use \":recover\" or \"nvim -r ",
    );
    msg.extend_from_slice(shown.to_bytes());
    push_tr(
        msg,
        c"\"\n    to recover the changes (see \":help recovery\").\n",
    );
    push_tr(msg, c"    If you did this already, delete the swap file \"");
    msg.extend_from_slice(fname.to_bytes());
    push_tr(msg, c"\"\n    to avoid this message.\n");
}

/// Fire the `SwapExists` autocommands and read the choice they left in
/// `v:swapchoice`.
fn do_swapexists(buffer: Buf, fname: &CStr) -> SwapExistsChoice {
    set_vim_var_string(Vv::Swapname, Some(fname.to_bytes()));
    set_vim_var_string(Vv::Swapchoice, None);

    // `<afile>` is the file being edited. Changing directory is not
    // allowed from here.
    let locked = Lock::all_buffers();
    let name = buffer.name.shown().map(CStr::to_owned);
    fire_autocmds_for(
        AutoEvent::SwapExists,
        name.as_deref(),
        None,
        false,
        None,
        None,
    );
    drop(locked);

    set_vim_var_string(Vv::Swapname, None);

    match with_vim_var_str(Vv::Swapchoice, cstr::first) {
        b'o' => SEA_CHOICE_READONLY,
        b'e' => SEA_CHOICE_EDIT,
        b'r' => SEA_CHOICE_RECOVER,
        b'd' => SEA_CHOICE_DELETE,
        b'q' => SEA_CHOICE_QUIT,
        b'a' => SEA_CHOICE_ABORT,
        _ => SEA_CHOICE_NONE,
    }
}

/// A swap file already has the name we wanted. Decide what that means and
/// act on it: delete a useless one, let a `SwapExists` autocommand choose, or
/// put the ATTENTION message in front of the user.
///
/// Returns true when the swap file is gone afterwards, so its name is free.
fn resolve_swapfile_clash(mut buffer: Buf, fname: &CStr, buf_fname: &CStr) -> bool {
    // Only worth a word if the swap file belongs to *this* file, the
    // buffer was not already recovered, and 'shortmess' allows it.
    if swapfile_is_for_other_file(buffer, fname)
        || Buf::current().b_flags.has(BufFlags::RECOVERED)
        || P_SHM.has_byte(ShmFlag::ATTENTION.byte())
    {
        return false;
    }

    let mut choice = SEA_CHOICE_NONE;

    // Deleting it is safe when the file exists and the swap file records
    // no changes and looks intact.
    if path_exists(buf_fname) && swapfile_unchanged(fname) {
        choice = SEA_CHOICE_DELETE;
        if p_verbose() > 0 {
            verb_msg(gettext(
                c"Found a swap file that is not useful, deleting it",
            ));
        }
    }

    // A SwapExists autocommand gets the next word, if the caller is in a
    // position to honour what it says. It may still decline (0) and leave
    // the question to the user.
    if choice == SEA_CHOICE_NONE
        && swap_exists_action.get() != SEA_NONE
        && has_autocmd(AutoEvent::SwapExists, buf_fname, Some(buffer))
    {
        choice = do_swapexists(buffer, fname);
    }
    if choice == SEA_CHOICE_NONE && swap_exists_action.get() == SEA_READONLY {
        choice = SEA_CHOICE_READONLY;
    }

    // Set by attention_message -> swapfile_info, below.
    proc_running.set(0);
    if choice == SEA_CHOICE_NONE {
        choice = ask_about_swapfile(buffer, fname);
    }

    match choice {
        SEA_CHOICE_READONLY => buffer.b_p_ro = 1,
        SEA_CHOICE_RECOVER => swap_exists_action.set(SEA_RECOVER),
        SEA_CHOICE_DELETE => {
            os_remove(fname);
        }
        SEA_CHOICE_QUIT => swap_exists_action.set(SEA_QUIT),
        SEA_CHOICE_ABORT => {
            swap_exists_action.set(SEA_QUIT);
            got_int.set(true);
        }
        SEA_CHOICE_NONE => {
            msg_str(c"\n");
            if msg_silent.get() == 0 {
                need_wait_return.set(true); // call wait_return() later
            }
        }
        // SEA_CHOICE_EDIT: use the file as it is.
        _ => {}
    }

    // If the swap file was deleted, this name can be used after all.
    choice != SEA_CHOICE_NONE && !path_exists(fname)
}

/// Show the ATTENTION message, as a dialog if the caller can act on an
/// answer and as a warning otherwise.
fn ask_about_swapfile(buffer: Buf, fname: &CStr) -> SwapExistsChoice {
    let mut choice = SEA_CHOICE_NONE;
    let no_prompt = Suppress::wait_return();

    // Upstream's `kv_resize(msg, IOSIZE)`: a screenful before the first
    // reallocation. Only a size hint -- nothing reads the capacity.
    let mut msg: Vec<u8> = Vec::with_capacity(IOSIZE as usize);

    // Room for the name, a `~/` and the terminator, as `home_replace_save`
    // sized it.
    let fhname = home_replace_in(None, fname, fname.count_bytes() + 3, true);
    let shown = buffer.name.shown().unwrap_or(c"").to_owned();
    attention_message(fname, &fhname, &shown, &mut msg);

    // A 'q' typed at the more-prompt must not interrupt loading the
    // file, and a "simalt ~x" in the vimrc must not answer the prompt
    // below.
    got_int.set(false);
    flush_buffers(FLUSH_TYPEAHEAD);

    if swap_exists_action.get() != SEA_NONE {
        push_tr(&mut msg, c"Swap file \"");
        msg.extend_from_slice(&fhname);
        push_tr(&mut msg, c"\" already exists!");
        // "Delete it" is not offered while the owning process is alive.
        let run_but = gettext(c"&Open Read-Only\n&Edit anyway\n&Recover\n&Quit\n&Abort");
        let but = gettext(c"&Open Read-Only\n&Edit anyway\n&Recover\n&Delete it\n&Quit\n&Abort");
        let running = proc_running.get() != 0;
        let buttons = if running { run_but } else { but };
        let (title, warn) = (gettext(c"VIM - ATTENTION"), VIM_WARNING as c_int);
        // `do_dialog` reads the message as a C string, which ends at the
        // first NUL, as the `vsnprintf` behind `kv_printf` left it.
        let end = msg
            .iter()
            .position(|&byte| byte == NUL as u8)
            .unwrap_or(msg.len());
        let text = XString::from_bytes(&msg[..end]);
        let text = text.as_cstr();
        choice = do_dialog(warn, Some(title), text, buttons, 1, None, false) as SwapExistsChoice;
        // Compensate for the missing "Delete it" button.
        choice = choice.wrapping_add(SwapExistsChoice::from(running && choice >= 4));
        // Pretend the screen did not scroll; it needs a redraw anyway.
        msg_reset_scroll();
    } else {
        let mut need_clear = false;
        msg_ext_set_kind(c"wmsg");
        msg_multiline(&msg, 0, false, false, &mut need_clear);
    }

    drop(no_prompt);
    choice
}

/// Find a name for `buffer`'s swap file under the `'directory'` entry
/// `dir_name`.
///
/// Names are tried in turn until one is free: `".swp"`, then `".swo"`,
/// `".swn"` and so on down to `".saa"`. The last directory in the option
/// (`last_dir`) is created if it does not exist.
///
/// May trigger the `SwapExists` autocommand, so whatever the caller holds
/// about the buffer may change.
///
/// `old_fname`, when given, is a name this buffer already owns and may keep.
/// `found_existing_dir` starts out false and is only ever set — once any
/// directory in the list exists, no new one is created.
pub(crate) fn findswapname(
    buffer: Buf,
    dir_name: &[u8],
    last_dir: bool,
    old_fname: Option<&CStr>,
    found_existing_dir: &mut bool,
) -> Option<XString> {
    // A copy: the autocommands below may rename the buffer.
    let buf_fname = buffer.name.shown().map(CStr::to_owned);

    let mut fname = makeswapname(buf_fname.as_deref(), dir_name);
    while let Some(name) = fname.as_mut() {
        if name.is_empty() {
            // Safety check.
            fname = None;
            break;
        }

        // Is the name taken? A swap file that is a symbolic link is most
        // likely a symlink attack, so the link itself counts as taken.
        if !link_exists(name.as_cstr()) {
            break;
        }
        // A name this buffer already owns is free for it to keep.
        if old_fname.is_some_and(|old| path_fnamecmp(name, old.to_bytes()) == 0) {
            break;
        }

        // The last two bytes of the name are the extension being permuted
        // below. Every name `makeswapname` answers ends in ".swp", so there
        // are always two.
        let n = name.len();
        let Some(ext) = name.get(n.saturating_sub(2)..).filter(|ext| ext.len() == 2) else {
            break;
        };
        let ext = [ext[0], ext[1]];

        // The name is taken. On the first try — the plain ".swp" — that
        // means a real swap file, and it is worth telling the user about
        // unless we are recovering, have no file name, are in a help file
        // or in a dummy buffer.
        if &ext == b"wp"
            && !recoverymode.get()
            && let Some(buf_fname) = buf_fname.as_deref()
            && !buffer.b_help
            && !buffer.b_flags.has(BufFlags::DUMMY)
        {
            let swap = name.as_cstr().to_owned();
            if resolve_swapfile_clash(buffer, &swap, buf_fname) {
                break;
            }
        }

        // Permute the extension to find a name nothing is using: first
        // count the last character down (".swo", ".swn", …), and when
        // that runs out the one before it (".svz", ".suz", …). Both can
        // happen with many Nvims editing one file, including "No Name"
        // buffers.
        let [mut first, mut second] = ext;
        if second == b'a' {
            if first == b'a' {
                // ".saa": tried enough, give up.
                complain(c"E326: Too many swap files found");
                fname = None;
                break;
            }
            first -= 1;
            second = b'z' + 1;
        }
        second -= 1;
        name.truncate(n - 2);
        name.push_bytes(&[first, second]);
    }

    let dir = cstr::owned(dir_name);
    if dir_exists(&dir) {
        *found_existing_dir = true;
    } else if !*found_existing_dir && last_dir {
        // The last entry in 'directory' is created on demand.
        if let Err(failure) = mkdir_recurse(&dir, 0o755) {
            let shown = failure.dir.as_deref().unwrap_or(c"");
            semsg!(
                "E303: Unable to create directory \"{}\" for swap file, recovery impossible: {}",
                msg_cstr(shown),
                msg_cstr(failure.why)
            );
        }
    }

    fname
}

/// Every file matching one of `patterns`.
///
/// The flags are the same at both call sites and are what makes this a
/// lookup rather than an expansion the user sees: keep everything, files
/// only, and say nothing about a pattern that matches none.
fn expanded(patterns: &[&CStr]) -> Result<Vec<XString>, Failed> {
    let how = ExpandFlags::KEEPALL | ExpandFlags::FILE | ExpandFlags::SILENT;
    expand_wildcards(patterns, how)
}

/// [`recover_names`] of every swap file in 'directory': collected into
/// `list` -- what `swapfilelist()` answers -- or, with no list, listed on
/// the screen, which is what `nvim -r` with no file shows.
pub(crate) fn list_swap_files(list: Option<&mut List>) {
    let do_list = list.is_none();
    let _ = recover_names(None, do_list, list, 0);
}

/// Find the swap files in the current directory and in every directory of
/// the `'directory'` option.
///
/// Used to list them for `nvim -r`, to count them while recovering, to list
/// them while recovering, to fill `swapfilelist()`, and to name the n'th one.
///
/// `fname` is the file whose swap files are wanted, or `None` for all of
/// them. `do_list` lists the names; `ret_list`, when given, collects them;
/// `nr`, when non-zero, asks for the n'th name.
///
/// Answers the number of swap files found, and the n'th name when one was
/// asked for and found.
pub fn recover_names(
    fname: Option<&CStr>,
    do_list: bool,
    mut ret_list: Option<&mut List>,
    nr: c_int,
) -> (c_int, Option<XString>) {
    // Expand a symlink, because the swap file was created against the
    // actual file rather than the link.
    let resolved = fname
        .and_then(resolve_symlink)
        .map(|name| cstr::owned(&name));
    let fname_res: Option<&CStr> = resolved.as_deref().or(fname);

    msg_ext_skip_flush.set(true);
    if do_list {
        // Use msg() to start the scrolling properly.
        msg_ext_set_kind(c"list_cmd");
        tell(c"Swap files found:", 0);
        msg_putchar('\n' as c_int);
    }

    let mut file_count = 0;
    let mut wanted = None;
    // A copy: the walk outlives a projection's borrow.
    let dir_option = P_DIR.get();
    for (dir, last) in directory_entries(&dir_option) {
        let current_dir = dir == b".";
        let names: Vec<XString> = match fname_res {
            None => {
                // Every swap file, under whatever name. On unix a leading
                // dot is special, so the two forms are listed separately.
                let patterns = [c"*.sw?", c".*.sw?", c".sw?"];
                patterns
                    .iter()
                    .map(|pattern| {
                        if current_dir {
                            XString::from_cstr(pattern)
                        } else {
                            concat_fnames(&dir, pattern.to_bytes(), true)
                        }
                    })
                    .collect()
            }
            Some(fname_res) if current_dir => recov_file_names(fname_res.to_bytes(), true),
            Some(fname_res) => {
                let tail = if after_pathsep(&dir, dir.len()) && ends_with_double_sep(&dir) {
                    // Ends with "//": the swap file's name holds the full
                    // path.
                    percent_swname(&dir[..dir.len() - 1], Some(fname_res))
                } else {
                    concat_fnames(&dir, path_tail(fname_res).to_bytes(), true)
                };
                recov_file_names(&tail, false)
            }
        };

        let patterns: Vec<&CStr> = names.iter().map(XString::as_cstr).collect();
        let mut found = if patterns.is_empty() {
            Vec::new()
        } else {
            expanded(&patterns).unwrap_or_default()
        };

        // Nothing found may mean the wildcard expansion itself failed
        // (no shell to run, say). Try the plain ".swp" name.
        if last && file_count == 0 && found.is_empty() && fname.is_some() {
            let swapname = modname(fname_res, b".swp", true);
            if let Some(swapname) = swapname.filter(|name| path_exists(name.as_cstr())) {
                found.push(swapname);
            }
        }

        // The current buffer's own swap file is not interesting — except
        // to swapfilelist(), which wants everything.
        let mine = Buf::current()
            .b_ml
            .ml_mfp
            .as_ref()
            .and_then(|memfile| memfile.fname().map(CStr::to_owned));
        if let Some(mine) = mine
            && ret_list.is_none()
        {
            // Do not expand wildcards: on Windows that would try to expand
            // the "%tmp%" in "%tmp%file". Upstream's shifting loop runs `i`
            // up to the new count, so the enclosing loop ends with it: only
            // the *first* match is dropped, which is all there can be.
            let hit = found.iter().position(|name| {
                let how = path_full_compare(&mine, name.as_cstr(), true, false);
                how as c_uint & kEqualFiles as c_uint != 0
            });
            if let Some(hit) = hit {
                found.remove(hit);
            }
        }
        let num_files = c_int::try_from(found.len()).unwrap_or(c_int::MAX);

        if nr > 0 {
            file_count += num_files;
            if nr <= file_count {
                let at = (nr - 1 + num_files - file_count) as usize;
                wanted = Some(found.swap_remove(at));
                break; // stop searching
            }
        } else if do_list {
            if current_dir {
                if fname.is_none() {
                    say(c"   In current directory:\n");
                } else {
                    say(c"   Using specified name:\n");
                }
            } else {
                say(c"   In directory ");
                msg_home_replace(&cstr::owned(&dir));
                msg_str(c":\n");
            }

            if num_files == 0 {
                say(c"      -- none --\n");
            } else {
                for name in &found {
                    file_count += 1;
                    msg_outnum(file_count);
                    msg_str(c".    ");
                    msg_str(path_tail(name.as_cstr()));
                    msg_putchar('\n' as c_int);

                    // Upstream's `kv_resize(msg, IOSIZE)`: a size hint.
                    let mut msg_buf: Vec<u8> = Vec::with_capacity(IOSIZE as usize);
                    swapfile_info(name.as_cstr(), &mut msg_buf);
                    let mut need_clear = false;
                    msg_multiline(&msg_buf, 0, false, false, &mut need_clear);
                }
            }
            ui_flush();
        } else if let Some(list) = ret_list.as_deref_mut() {
            for name in &found {
                let joined = concat_fnames(&dir, name, true);
                let joined = Some(ThinCString::from_vec(joined.to_vec()));
                list.push(TypVal::string(joined));
            }
        } else {
            file_count += num_files;
        }
    }
    msg_ext_skip_flush.set(false);
    (file_count, wanted)
}

/// The wildcard patterns that match `path`'s swap files.
///
/// `prepend_dot` also asks for the hidden form, for a swap file kept in the
/// same directory as the file itself.
fn recov_file_names(path: &[u8], prepend_dot: bool) -> Vec<XString> {
    let mut names = Vec::with_capacity(2);
    if prepend_dot {
        let Some(dotted) = modname(Some(&cstr::owned(path)), b".sw?", true) else {
            return names;
        };
        names.push(dotted);
    }

    // The plain form: the name with ".sw?" appended.
    let plain = concat_fnames(path, b".sw?", false);
    // Both forms may have come out the same; keep only one. The dotted
    // form may have been expanded to a full path, so it is the *tail*
    // that is compared.
    if !names.last().is_some_and(|dotted| dotted.ends_with(&plain)) {
        names.push(plain);
    }
    names
}
