//! Turning `'spelllang'` into a list of loaded languages.
//!
//! `'spelllang'` is a comma-separated list of names — `en`, `en_us`,
//! `de`, `cjk`, or a path ending in `.spl`. [`parse_spelllang`] resolves
//! each of them to a [`SpellLang`], loading it from `'runtimepath'` if it is
//! not already in the global chain, and leaves the result in the window's
//! `b_langp` as a list of [`LangP`]: a language plus the region mask
//! selected for it.
//!
//! `'spellfile'` entries are appended to the same list, along with the
//! internal word list `zg` writes to.
//!
//! # Regions
//!
//! A name like `en_us` asks for one region of a language whose `.spl` file
//! covers several. The region becomes a bit in `lp_region`, checked against
//! each word's own region mask at lookup time. A region named in
//! `'spelllang'` is also used for `'spellfile'` entries, but only while
//! every name agrees on it.
//!
//! # Borrowing between languages
//!
//! Sound folding and REP items are expensive to define and often missing
//! from a derived language. After the list is built, each entry that lacks
//! them borrows from the first entry whose name starts with the same two
//! letters — so `en-math` uses `en`'s.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::smsg;
use crate::types::AutoEvent;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

use crate::autocmd::fire_autocmds_for;
use crate::buffer::BufRef;
use crate::charset::vim_is_fname_char;
use crate::drawscreen::{UPD_NOT_VALID, redraw_later};
use crate::ex_docmd::do_cmdline_cmd;
use crate::global_cell::GlobalCell;
use crate::mbyte::{c_cluster_len, char_at};
use crate::message::e_invarg;
use crate::option::vars::{P_ENC, p_enc};
use crate::option::{split_option_part, valid_name};
use crate::os::fs::os_remove;
use crate::path::{path_fnamecmp, path_full_compare, tail_index};
use crate::regexp::{RE_MAGIC, vim_regcomp};
use crate::spellfile::spell_load_file;
use crate::startup::starting;
use crate::strings::concat;
use crate::types::{Failed, LangId, LangP, MAXPATHL, OptError, SpellLang, SynBlock};
use crate::window::win_valid_any_tab;

use super::chartab::init_spell_chartab;
use super::slang::{free_langs, register_lang, with_lang};
use super::{
    MAXWLEN, REGION_ALL, SpellLoad, int_wordlist, kEqualFiles, languages, last_replacement,
    loaded_langs,
};
use crate::optionstr::{OptString, OptStringRef};
use crate::runtime::RuntimeOpts;
use crate::winlayer::{Buf, buffers, windows};

/// `ASCII_ISALPHA`: an unaccented Latin letter.
fn ascii_isalpha(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// The encoding spell files are named after: `'encoding'`, except that
/// `latin9` uses `latin1`'s files, and anything implausibly long falls back
/// to `latin1`.
pub fn spell_enc() -> XString {
    let usable = p_enc(|enc| enc.count_bytes() < 60 && enc != c"iso-8859-15");
    if usable {
        P_ENC.get()
    } else {
        XString::from_cstr(c"latin1")
    }
}

/// `parts` run together, cut to `size - 1` bytes as a `snprintf` into a
/// buffer of `size` would leave them.
fn joined(parts: &[&[u8]], size: usize) -> Vec<u8> {
    let mut out = parts.concat();
    out.truncate(size - 1);
    out
}

/// The `.spl` file name for the internal word list.
fn int_wordlist_spl() -> Vec<u8> {
    let enc = spell_enc();
    int_wordlist.with(|list| {
        // Only asked for while there is a list; upstream's `%s` of a null
        // pointer would print "(null)".
        let list = list.as_ref().map_or(&b"(null)"[..], |list| list);
        joined(&[list, b".", &enc, b".spl"], MAXPATHL as usize)
    })
}

/// Load every spell file for language `lang` (a name without a region)
/// found in `'runtimepath'`, then every `.add.spl` alongside them.
///
/// When nothing is found, a `SpellFileMissing` autocommand gets one chance
/// to produce the file, and the whole search is retried. Failing that, at
/// startup an autocommand is queued to offer downloading it, and otherwise
/// a warning is printed.
fn spell_load_lang(lang: &[u8]) {
    // The name is passed to spell_load_cb() as a cookie, and cleared there
    // when an error is found.
    let mut sl = SpellLoad {
        lang: lang[..lang.len().min(MAXWLEN)].to_vec(),
        last: None,
        nobreak: false,
    };

    // Autocommands could otherwise delete the buffer and free "lang".
    Buf::current().b_locked += 1;

    // Upstream's name buffer is 85 bytes, and names were written into 80 of
    // them.
    let room = 85 - 5;
    let mut fname_enc = Vec::new();
    let mut r = Err(Failed);
    for round in 1..=2 {
        let enc = spell_enc();
        fname_enc = joined(&[b"spell/", lang, b".", &enc, b".spl"], room);
        r = do_in_runtimepath_cb(&fname_enc, RuntimeOpts::NONE, &mut sl);

        if r.is_err() && !sl.lang.is_empty() {
            // Fall back on the ASCII version.
            fname_enc = joined(&[b"spell/", lang, b".ascii.spl"], room);
            r = do_in_runtimepath_cb(&fname_enc, RuntimeOpts::NONE, &mut sl);

            if r.is_err() && !sl.lang.is_empty() && round == 1 && {
                // Re-read each round: the handler may have changed buffers.
                let buffer = Buf::current();
                let pattern = cstr::owned(lang);
                let afile = buffer.name.shown().map(CStr::to_owned);
                let event = AutoEvent::SpellFileMissing;
                fire_autocmds_for(
                    event,
                    Some(&pattern),
                    afile.as_deref(),
                    false,
                    Some(buffer),
                    None,
                )
            } {
                continue;
            }
        }
        break;
    }

    if r.is_err() {
        if starting.get() != 0 {
            // Plugins are not loaded yet, so nvim/spellfile.lua cannot
            // offer the download itself. #3027
            let cmd = joined(
                &[
                    b"autocmd VimEnter * call v:lua.require'nvim.spellfile'.get('",
                    lang,
                    b"')|set spell",
                ],
                512,
            );
            let _ = do_cmdline_cmd(&cstr::owned(&cmd));
        } else {
            let encoding = spell_enc();
            let (lang, enc) = (msg_bytes(lang), msg_bytes(&encoding));
            smsg!(
                0,
                "Warning: Cannot find word list \"{lang}.{enc}.spl\" or \"{lang}.ascii.spl\""
            );
        }
    } else if sl.last.is_some() {
        // At least one file loaded; now take all the additions.
        fname_enc.truncate(fname_enc.len() - 3);
        fname_enc.extend_from_slice(b"add.spl");
        let _ = do_in_runtimepath_cb(&fname_enc, RuntimeOpts::ALL, &mut sl);
    }

    Buf::current().b_locked -= 1;
}

/// `do_in_runtimepath_visit` with [`spell_load_cb`] as the visit.
fn do_in_runtimepath_cb(name: &[u8], flags: RuntimeOpts, sl: &mut SpellLoad) -> Result<(), Failed> {
    let name = cstr::owned(name);
    crate::runtime::do_in_runtimepath_visit(&name, flags, &mut |fnames, all| {
        spell_load_cb(fnames, all, sl)
    })
}

/// Load the spell files the search found, keeping the last one in `sl`.
///
/// NOBREAK is sticky in both directions: a `.add` file inherits it from the
/// base language, and a base language that declares it passes it on.
fn spell_load_cb(fnames: &[&CStr], all: bool, sl: &mut SpellLoad) -> bool {
    for &fname in fnames {
        let Some(mut slang) = spell_load_file(fname, Some(&sl.lang), false) else {
            // The name is cleared, so the caller stops trying it.
            sl.lang.clear();
            continue;
        };

        if sl.nobreak && slang.sl_add {
            slang.sl_nobreak = true;
        } else if slang.sl_nobreak {
            sl.nobreak = true;
        }

        sl.last = Some(register_lang(slang));

        if !all {
            break;
        }
    }

    !fnames.is_empty()
}

/// Guard against re-entering [`parse_spelllang`]: a `SpellFileMissing`
/// autocommand can open a new buffer with `'spell'` set.
static recursive: GlobalCell<bool> = GlobalCell::new(false);

/// Where the region a `'spelllang'` name asked for is read from.
///
/// Upstream keeps a *pointer* to it, into the name buffer or the region
/// copy, and reads through it after later names have been copied over that
/// buffer -- so a later name changes what an earlier region says. Kept as a
/// place rather than a value so that the answer stays upstream's.
#[derive(Copy, Clone)]
enum RegionAt {
    /// That many bytes into the name buffer.
    Name(usize),
    /// The copy a file name's region was taken into.
    Copy,
}

/// The region string at `at`: up to its NUL, as upstream reads it.
fn region_text<'a>(at: RegionAt, name: &'a [u8], copy: &'a [u8]) -> &'a [u8] {
    match at {
        RegionAt::Name(offset) => cstr::until_nul(&name[offset..]),
        RegionAt::Copy => cstr::until_nul(copy),
    }
}

/// Copy `text` into the C string buffer `buf`, NUL terminated, cut to fit;
/// what lay past the terminator stays, as it did in upstream's buffer.
fn put_name(buf: &mut [u8], text: &[u8]) {
    let n = text.len().min(buf.len() - 1);
    buf[..n].copy_from_slice(&text[..n]);
    buf[n] = 0;
}

/// Whether `slang` is the language `name` names: by file for a file name,
/// else by language name, ignoring ASCII case.
fn names_language(slang: &SpellLang, name: &[u8], filename: bool) -> bool {
    if filename {
        path_full_compare(&cstr::owned(name), slang.sl_fname.as_cstr(), false, true) == kEqualFiles
    } else {
        name.eq_ignore_ascii_case(&slang.sl_name)
    }
}

/// The first two bytes of a language name as `strncmp(…, 2)` compares them:
/// a byte past the end of the name is its terminator.
fn name_head(slang: &SpellLang) -> [u8; 2] {
    let name = &slang.sl_name;
    match name.first() {
        None => [0, 0],
        Some(&first) => [first, name.get(1).copied().unwrap_or(0)],
    }
}

/// Parse `'spelllang'` and fill `window.w_s->b_langp`.
///
/// Returns null on success, or an untranslated error message.
pub fn parse_spelllang(mut window: Win) -> Result<(), OptError> {
    if recursive.get() {
        return Ok(());
    }
    recursive.set(true);

    let mut region_cp = [0u8; 3];
    let mut lang = [0u8; MAXWLEN + 1];
    let mut use_region: Option<RegionAt> = None;
    let mut dont_use_region = false;
    let mut nobreak = false;
    let mut ret_msg: Result<(), OptError> = Ok(());

    let bufref = BufRef::of_opt(window.buffer_or_none());

    let mut ga: Vec<LangP> = Vec::new();
    clear_midword(window);

    // The SpellFileMissing autocommands may change 'spelllang' underfoot.
    let spl_copy = window.syn_block().b_p_spl.bytes().to_vec();

    window.syn_block_mut().b_cjk = 0;

    let mut splp: &[u8] = &spl_copy;
    'names: while !splp.is_empty() {
        let (part, rest) = split_option_part(splp, MAXWLEN, b",");
        splp = rest;
        put_name(&mut lang, &part);
        let len = part.len();
        let mut region: Option<RegionAt> = None;

        if !valid_spelllang(cstr::in_bytes(&lang)) {
            continue;
        }

        if cstr::until_nul(&lang) == b"cjk" {
            window.syn_block_mut().b_cjk = 1;
            continue;
        }

        let filename = len > 4 && path_fnamecmp(&lang[len - 4..], b".spl") == 0;
        if filename {
            // The name is a file name; a region in it is pulled out.
            let tail = tail_index(cstr::until_nul(&lang));
            let at = lang[tail..len].iter().position(|&b| b == b'_');
            match at.map(|at| tail + at) {
                Some(p)
                    if ascii_isalpha(lang[p + 1])
                        && ascii_isalpha(lang[p + 2])
                        && !ascii_isalpha(lang[p + 3]) =>
                {
                    put_name(&mut region_cp, &lang[p + 1..p + 3]);
                    // Close the gap, terminator included.
                    lang.copy_within(p + 3..len + 1, p);
                    region = Some(RegionAt::Copy);
                }
                _ => dont_use_region = true,
            }
        } else if len > 3 && lang[len - 3] == b'_' {
            region = Some(RegionAt::Name(len - 2));
            lang[len - 3] = 0;
        } else {
            dont_use_region = true;
        }

        let name = cstr::until_nul(&lang).to_vec();
        let loaded = loaded_langs.with(|langs| {
            langs
                .iter()
                .any(|(_, slang)| names_language(slang, &name, filename))
        });

        if let Some(region) = region {
            // A region that disagrees with an earlier one disqualifies
            // regions for 'spellfile'.
            if let Some(used) = use_region
                && region_text(region, &lang, &region_cp) != region_text(used, &lang, &region_cp)
            {
                dont_use_region = true;
            }
            use_region = Some(region);
        }

        // Not loaded yet: load it now.
        if !loaded {
            if filename {
                match spell_load_file(&cstr::owned(&name), Some(&name), false) {
                    Some(slang) => {
                        register_lang(slang);
                    }
                    None => lang[0] = 0,
                }
            } else {
                spell_load_lang(&name);
                // The autocommands may have destroyed the buffer being
                // used, or closed the window.
                if !bufref.valid() || !win_valid_any_tab(window.id()) {
                    ret_msg = Err(c"E797: SpellFileMissing autocommand deleted buffer".into());
                    break 'names;
                }
            }
        }

        // There can be several files for one language.
        let name = cstr::until_nul(&lang).to_vec();
        let region = region.map(|at| region_text(at, &lang, &region_cp).to_vec());
        for id in languages() {
            // The mask, whether the language is NOBREAK, and whether its
            // region is one it does not have.
            let entry = loaded_langs.with(|langs| {
                let slang = langs.get(id)?;
                if !names_language(slang, &name, filename) {
                    return None;
                }
                let mut region_mask = REGION_ALL;
                let mut unsupported = false;
                if !filename && let Some(region) = &region {
                    let c = find_region(&slang.sl_regions, region);
                    if c == REGION_ALL {
                        if slang.sl_add {
                            if slang.sl_regions[0] != 0 {
                                // This addition file covers other regions.
                                region_mask = 0;
                            }
                        } else {
                            unsupported = true;
                        }
                    } else {
                        region_mask = 1 << c;
                    }
                }
                Some((region_mask, slang.sl_nobreak, unsupported))
            });
            let Some((region_mask, lang_nobreak, unsupported)) = entry else {
                continue;
            };
            if unsupported && let Some(region) = &region {
                let region = msg_bytes(region);
                smsg!(0, "Warning: region {region} not supported");
            }

            if region_mask != 0 {
                ga.push(LangP {
                    slang: id,
                    sallang: None,
                    replang: None,
                    region: region_mask,
                });
                use_midword(id, window);
                if lang_nobreak {
                    nobreak = true;
                }
            }
        }
    }

    if ret_msg.is_ok() {
        // Round 0 is the internal word list; each round after that is one
        // entry of 'spellfile'.
        let spf_copy = Win::current().syn_block().b_p_spf.bytes().to_vec();
        let mut spf: &[u8] = &spf_copy;
        let mut round = 0;
        while round == 0 || !spf.is_empty() {
            let spf_name = if round == 0 {
                if int_wordlist.with(Option::is_none) {
                    round += 1;
                    continue;
                }
                int_wordlist_spl()
            } else {
                let (part, rest) = split_option_part(spf, MAXPATHL as usize - 4, b",");
                spf = rest;
                let spf_name = joined(&[&part, b".spl"], MAXPATHL as usize);

                // Skip it if the loop above already took it.
                let name = cstr::owned(&spf_name);
                let taken = ga.iter().any(|lp| {
                    loaded_langs.with(|langs| {
                        langs.get(lp.slang).is_some_and(|slang| {
                            path_full_compare(&name, slang.sl_fname.as_cstr(), false, true)
                                == kEqualFiles
                        })
                    })
                });
                if taken {
                    round += 1;
                    continue;
                }
                spf_name
            };

            let name = cstr::owned(&spf_name);
            let mut slang = loaded_langs.with(|langs| {
                langs
                    .iter()
                    .find(|(_, slang)| {
                        path_full_compare(&name, slang.sl_fname.as_cstr(), false, true)
                            == kEqualFiles
                    })
                    .map(|(id, _)| id)
            });

            if slang.is_none() {
                // The language name includes the region; the region is
                // ignored for these. The internal word list gets an
                // arbitrary name.
                if round == 0 {
                    put_name(&mut lang, b"internal wordlist");
                } else {
                    let tail = &spf_name[tail_index(&spf_name)..];
                    put_name(&mut lang, tail);
                    // Truncate at ".encoding.add".
                    if let Some(dot) = cstr::until_nul(&lang).iter().position(|&b| b == b'.') {
                        lang[dot] = 0;
                    }
                }
                match spell_load_file(&name, Some(cstr::until_nul(&lang)), true) {
                    Some(mut loaded) => {
                        // If any language has NOBREAK assume the additions
                        // do too.
                        if nobreak {
                            loaded.sl_nobreak = true;
                        }
                        slang = Some(register_lang(loaded));
                    }
                    None => lang[0] = 0,
                }
            }

            if let Some(id) = slang {
                let mut region_mask = REGION_ALL;
                if let Some(used) = use_region
                    && !dont_use_region
                {
                    let used = region_text(used, &lang, &region_cp);
                    let regions = with_lang(id, |slang| slang.sl_regions).unwrap_or_default();
                    let c = find_region(&regions, used);
                    if c != REGION_ALL {
                        region_mask = 1 << c;
                    } else if regions[0] != 0 {
                        // This spell file covers other regions.
                        region_mask = 0;
                    }
                }

                if region_mask != 0 {
                    ga.push(LangP {
                        slang: id,
                        sallang: None,
                        replang: None,
                        region: region_mask,
                    });
                    use_midword(id, window);
                }
            }
            round += 1;
        }

        lend_tables(&mut ga);

        // Everything worked; publish the new list.
        window.syn_block_mut().b_langp = ga;
        redraw_later(window, UPD_NOT_VALID);
    }

    recursive.set(false);
    ret_msg
}
/// Fill in where each entry's sound folding and REP items come from: its own
/// language when that has them, else the first entry's whose name starts
/// with the same two letters, so that "en-math" uses "en"'s.
fn lend_tables(ga: &mut [LangP]) {
    loaded_langs.with(|langs| {
        // What each entry's language offers: the first two bytes of its
        // name (a region suffix past them does not have to match), whether
        // it sound-folds, and whether it has REP items.
        let offers: Vec<Option<([u8; 2], bool, bool)>> = ga
            .iter()
            .map(|lp| {
                let slang = langs.get(lp.slang)?;
                Some((
                    name_head(slang),
                    slang.has_soundfold(),
                    !slang.sl_rep.is_empty(),
                ))
            })
            .collect();
        let ids: Vec<LangId> = ga.iter().map(|lp| lp.slang).collect();
        let first = |head: [u8; 2], pick: fn(bool, bool) -> bool| {
            offers
                .iter()
                .position(|offer| offer.is_some_and(|(h, s, r)| h == head && pick(s, r)))
                .map(|j| ids[j])
        };
        for (lp, offer) in ga.iter_mut().zip(&offers) {
            let Some((head, soundfolds, has_rep)) = *offer else {
                continue;
            };
            if soundfolds {
                lp.sallang = Some(lp.slang);
            } else if let Some(lender) = first(head, |s, _| s) {
                lp.sallang = Some(lender);
            }
            if has_rep {
                lp.replang = Some(lp.slang);
            } else if let Some(lender) = first(head, |_, r| r) {
                lp.replang = Some(lender);
            }
        }
    });
}

/// Forget the midword characters recorded for `window`.
fn clear_midword(mut window: Win) {
    let syn = window.syn_block_mut();
    syn.b_spell_ismw = [false; 256];
    syn.b_spell_ismw_mb = Vec::new();
}

/// The index of region `region[..2]` in `regions` (which is `sl_regions`,
/// two characters per region, NUL terminated), or `REGION_ALL` when it is
/// not there.
fn find_region(regions: &[u8], region: &[u8]) -> c_int {
    let at = |i: usize| cstr::byte_at(region, i);
    for (i, pair) in regions.chunks(2).enumerate() {
        if pair[0] == 0 {
            break;
        }
        if pair[0] == at(0) && pair.get(1).copied().unwrap_or(0) == at(1) {
            return i as c_int;
        }
    }
    REGION_ALL
}

/// Delete the internal word list and its compiled `.spl`.
pub fn spell_delete_wordlist() {
    let Some(list) = int_wordlist.with(Clone::clone) else {
        return;
    };

    os_remove(list.as_cstr());
    os_remove(&cstr::owned(&int_wordlist_spl()));
    int_wordlist.set(None);
}

/// Free every loaded language and everything derived from them.
pub fn spell_free_all() {
    for buf in buffers() {
        buf.syntax().b_langp = Vec::new();
    }

    free_langs();

    spell_delete_wordlist();

    last_replacement.set(None);
}

/// Drop every spelling table and load them again, after `'encoding'`
/// changed or `:mkspell` ran.
pub fn spell_reload() {
    init_spell_chartab();
    spell_free_all();

    // Only load word lists where 'spelllang' is set and some window on
    // the buffer has 'spell' on. The walk is over the current tab, which
    // always starts at `firstwin`.
    for wp in windows() {
        if wp.syn_block().b_p_spl.first_byte() != 0 && wp.w_onebuf_opt.wo_spell != 0 {
            let _ = parse_spelllang(wp);
            break;
        }
    }
}

/// Whether `val` is a usable `'spelllang'` value.
pub fn valid_spelllang(val: &CStr) -> bool {
    valid_name(val, b".-_,@")
}

/// Whether `value` is a usable `'spellfile'` value: a comma-separated list
/// of file names, each ending in `.add` and made of file-name characters.
///
/// Each part is measured as upstream copies it, into a `MAXPATHL` buffer:
/// a longer one is cut there, and then fails the length test.
pub fn valid_spellfile(value: &[u8]) -> bool {
    let mut rest = value;
    while !rest.is_empty() {
        let (name, after) = split_option_part(rest, MAXPATHL as usize, b",");
        rest = after;
        let l = name.len();
        if l >= MAXPATHL as usize - 4 || l < 4 || !name.ends_with(b".add") {
            return false;
        }
        if !name.iter().all(|&c| vim_is_fname_char(c_int::from(c))) {
            return false;
        }
    }
    true
}

/// Re-parse `'spelllang'` for the current buffer after a spell option
/// changed.
pub fn did_set_spell_option() -> Result<(), OptError> {
    let mut errmsg = Ok(());
    for wp in windows() {
        if wp.w_buffer.is_current() && wp.w_onebuf_opt.wo_spell != 0 {
            errmsg = parse_spelllang(wp);
            break;
        }
    }
    errmsg
}

/// Compile `'spellcapcheck'` into `b_cap_prog`, anchored so that it can
/// only match at one column.
///
/// Returns an error message when the pattern does not compile, leaving the
/// previous program in place.
pub fn compile_cap_prog(synblock: &mut SynBlock) -> Result<(), OptError> {
    if synblock.b_p_spc.bytes().is_empty() {
        synblock.b_cap_prog = None;
    } else {
        let re = concat(b"^", synblock.b_p_spc.bytes());
        // A pattern that does not compile keeps the previous program.
        let Some(prog) = vim_regcomp(re.as_cstr(), RE_MAGIC as c_int) else {
            return Err(e_invarg.into());
        };
        synblock.b_cap_prog = Some(prog);
    }
    Ok(())
}

/// Record the `MIDWORD` characters of the language in slot `id` in
/// `window`, so that [`spell_iswordp`] treats them as part of a word when a
/// word character follows.
///
/// Characters below 256 that take at most two bytes go in the flat
/// `b_spell_ismw` table; anything wider is appended to the
/// `b_spell_ismw_mb` string, which is scanned instead.
///
/// [`spell_iswordp`]: super::chartab::spell_iswordp
fn use_midword(id: LangId, mut window: Win) {
    let midword = with_lang(id, |slang| slang.sl_midword.clone()).unwrap_or_default();
    let syn = window.syn_block_mut();
    let mut p = 0;
    while p < midword.len() {
        let rest = &midword[p..];
        let c = char_at(rest);
        let l = c_cluster_len(rest).max(1);
        if c < 256 && l <= 2 {
            syn.b_spell_ismw[c as usize] = true;
        } else {
            syn.b_spell_ismw_mb.extend_from_slice(&rest[..l]);
        }
        p += l;
    }
}
