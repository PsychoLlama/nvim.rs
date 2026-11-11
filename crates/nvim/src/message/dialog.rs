//! `confirm()`, and the console dialog it falls back to.
//!
//! [`do_dialog`] renders the message plus the button list, works out the
//! hotkey letters ([`render_buttons`]) and reads a keystroke until one of
//! them matches.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::guard::{Allow, Suppress};
use crate::input::prompt_for_key;
use crate::keycodes::ModMask;
use crate::mbyte::Codepoint;
use crate::mbyte::{char_at, cluster_len};
use crate::types::{Key, KeyKind};
use core::ffi::CStr;
use core::num::NonZeroUsize;

/// How many buttons can carry a hotkey. Buttons past this share the default.
const HAS_HOTKEY_LEN: usize = 30;

/// The kind of a dialog: the icon a GUI would show, which `confirm()` names
/// by its first letter. The console dialog shows none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogKind {
    Generic,
    Error,
    Warning,
    Info,
    Question,
}

/// One of a dialog's buttons, by its 1-based place in the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Button(NonZeroUsize);

impl Button {
    /// The first button, every dialog's usual default.
    pub(crate) const FIRST: Button = Button(NonZeroUsize::MIN);

    /// The `number`th button, counting from 1; none for 0.
    pub(crate) fn nth(number: usize) -> Option<Button> {
        NonZeroUsize::new(number).map(Button)
    }

    /// The button's 1-based place in the list.
    pub(crate) fn number(self) -> usize {
        self.0.get()
    }
}

/// How a dialog was dismissed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogAnswer {
    /// The user took the default (Enter), or there was nobody to ask: no
    /// UI, or silent mode.
    Default,
    /// The user pressed a button's hotkey.
    Pressed(Button),
    /// The user cancelled (CTRL-C or `<Esc>`).
    Cancelled,
}

impl DialogAnswer {
    /// The button chosen, reading [`Default`](Self::Default) as `default`.
    pub(crate) fn button(self, default: Option<Button>) -> Option<Button> {
        match self {
            DialogAnswer::Default => default,
            DialogAnswer::Pressed(button) => Some(button),
            DialogAnswer::Cancelled => None,
        }
    }
}

/// [`do_dialog`] with no title, text field or Ex command: `confirm()`.
pub(crate) fn confirm_dialog(
    kind: DialogKind,
    message: &CStr,
    buttons: &CStr,
    default: Option<Button>,
) -> DialogAnswer {
    do_dialog(kind, None, message, buttons, default, None, false)
}

/// Ask the user to pick one of `buttons`.
///
/// Answers [`DialogAnswer::Default`] as well when there is no UI to ask
/// through -- without one Nvim would wait for input forever. `default` is
/// the button Enter picks, shown in `[]`; none shows no default.
///
/// `buttons` is `"Button1\nButton2\n..."`, with `&` marking a hotkey letter.
/// `ex_cmd` allows `:` to dismiss the dialog and start an Ex command. The
/// kind, title and text field are a GUI's; the console dialog shows none.
pub(crate) fn do_dialog(
    _kind: DialogKind,
    _title: Option<&CStr>,
    message: &CStr,
    buttons: &CStr,
    default: Option<Button>,
    _textfield: Option<&CStr>,
    ex_cmd: bool,
) -> DialogAnswer {
    if SILENT_MODE.get() {
        return DialogAnswer::Default;
    }

    let old_state = STATE.get();
    // If the dialog prompts for input, the user needs to see it.
    let loud = Allow::messages();
    // We wait for a keypress, so don't make the user press RETURN as well.
    let no_prompt = Suppress::wait_return();

    let hotkeys = msg_show_console_dialog(message.to_bytes(), buttons.to_bytes(), default);
    let retval;
    loop {
        // Without a UI Nvim waits for input forever.
        if ui_active() == 0 && input_available() == 0 {
            retval = DialogAnswer::Default;
            break;
        }

        // Get a typed character directly from the user. The prompt is a
        // copy: a dialog the typing reaches renders its own buttons.
        let prompt = CONFIRM_BUTTONS.with(Clone::clone).unwrap_or_default();
        let key = prompt_for_key(prompt.as_cstr(), HLF_M);
        match key.kind() {
            KeyKind::Char(Key::CAR | Key::NUL) => {
                // User accepts the default option.
                retval = DialogAnswer::Default;
                break;
            }
            KeyKind::Char(Key::CTRL_C | Key::ESC) => {
                // User aborts/cancels.
                retval = DialogAnswer::Cancelled;
                break;
            }
            KeyKind::Char(c) if c.is(':') && ex_cmd => {
                retval = DialogAnswer::Default;
                ins_char_typebuf(c, ModMask::NONE, false);
                break;
            }
            KeyKind::Char(c) => {
                // Could be a hotkey. Lowercase it, as the ones in
                // "hotkeys" are, and count how many buttons precede it. A
                // NUL hotkey (a button list ending in a separator) ends the
                // list, as the terminator of upstream's string did.
                let c = c.code().to_lower();
                let listed = hotkeys.iter().take_while(|hotkey| !hotkey.is_nul());
                if let Some(button) = listed
                    .clone()
                    .position(|&hotkey| hotkey == c)
                    .and_then(|at| Button::nth(at + 1))
                {
                    retval = DialogAnswer::Pressed(button);
                    break;
                }
                // No hotkey match, so keep waiting.
                MSG_DIDANY.set(false);
                MSG_DIDOUT.set(false);
            }
            _ => {
                // Special keys are ignored here.
                MSG_DIDANY.set(false);
                MSG_DIDOUT.set(false);
            }
        }
    }

    CONFIRM_MSG.set(None);

    drop(loud);
    STATE.set(old_state);
    setmouse();
    drop(no_prompt);
    msg_end_prompt();

    retval
}

/// Which buttons name their own hotkey, for the first [`HAS_HOTKEY_LEN`].
fn buttons_with_hotkeys(buttons: &[u8]) -> [bool; HAS_HOTKEY_LEN] {
    let mut has_hotkey = [false; HAS_HOTKEY_LEN];
    let mut idx = 0;
    let mut at = 0;
    while at < buttons.len() {
        if u32::from(buttons[at]) == DLG_BUTTON_SEP {
            if idx < HAS_HOTKEY_LEN - 1 {
                idx += 1;
                has_hotkey[idx] = false;
            }
        } else if u32::from(buttons[at]) == DLG_HOTKEY_CHAR {
            // The character after the `&` is skipped along with it.
            at += 1;
            if idx < HAS_HOTKEY_LEN - 1 {
                has_hotkey[idx] = true;
            }
        }
        at += cluster_len(&buttons[at.min(buttons.len())..]).max(1);
    }
    has_hotkey
}

/// The character at the start of `bytes`, lowercased: a hotkey.
fn hotkey_at(bytes: &[u8]) -> Codepoint {
    char_at(bytes).to_lower()
}

/// Render the button list as the command-line prompt shows it, and the
/// hotkey of each button, in order.
///
/// `&x` makes `x` the button's hotkey and shows it as `(x)`, or `[x]` on the
/// default button; `&&` is a literal `&`. A button with no `&` takes its
/// first character. Buttons are separated by `, ` and the line ends `: `.
fn render_buttons(buttons: &[u8], default: Option<Button>) -> (Vec<u8>, Vec<Codepoint>) {
    let has_hotkey = buttons_with_hotkeys(buttons);
    let mut shown = Vec::with_capacity(buttons.len() + 8);
    // The first button's default hotkey.
    let mut hotkeys = vec![hotkey_at(buttons)];
    // Counts down to 1 at the default button.
    let mut default_button_idx = default.map_or(0, Button::number);

    // Is the first char of the button a hotkey? It is when the button
    // does not name one itself.
    let mut first_hotkey = !has_hotkey[0];

    let mut idx = 0;
    let mut at = 0;
    while at < buttons.len() {
        let byte = u32::from(buttons[at]);
        if byte == DLG_BUTTON_SEP {
            shown.extend_from_slice(b", "); // '\n' -> ', '

            // Advance to the next hotkey and set the default one.
            hotkeys.push(hotkey_at(&buttons[at + 1..]));

            default_button_idx = default_button_idx.saturating_sub(1);
            // If no hotkey is specified, the first char is used. The
            // increment is inside the short circuit, as upstream's
            // `has_hotkey[++idx]` is.
            if idx < HAS_HOTKEY_LEN - 1 && {
                idx += 1;
                !has_hotkey[idx]
            } {
                first_hotkey = true;
            }
        } else if byte == DLG_HOTKEY_CHAR || first_hotkey {
            if byte == DLG_HOTKEY_CHAR {
                at += 1;
            }
            first_hotkey = false;
            let rest = &buttons[at.min(buttons.len())..];
            if rest
                .first()
                .is_some_and(|&b| u32::from(b) == DLG_HOTKEY_CHAR)
            {
                shown.push(rest[0]); // '&&a' -> '&a'
            } else {
                // '&a' -> '[a]', or '(a)' when it is not the default.
                let default = default_button_idx == 1;
                shown.push(if default { b'[' } else { b'(' });
                shown.extend_from_slice(&rest[..cluster_len(rest)]);
                shown.push(if default { b']' } else { b')' });

                // Redefine the hotkey.
                if let Some(last) = hotkeys.last_mut() {
                    *last = hotkey_at(rest);
                }
            }
        } else {
            // Everything else is copied literally.
            let rest = &buttons[at..];
            shown.extend_from_slice(&rest[..cluster_len(rest)]);
        }
        at += cluster_len(&buttons[at.min(buttons.len())..]);
    }

    shown.extend_from_slice(b": ");
    (shown, hotkeys)
}

/// Format the dialog and display it, answering each button's hotkey.
fn msg_show_console_dialog(
    message: &[u8],
    buttons: &[u8],
    default: Option<Button>,
) -> Vec<Codepoint> {
    // With `ext_messages` the UI puts the message where it likes, so the
    // blank lines that separate it from the buttons are left out.
    let mut text = Vec::with_capacity(message.len() + 2);
    let framed = !ui_has(UI_MESSAGES);
    if framed {
        text.push(b'\n');
    }
    text.extend_from_slice(message);
    if framed {
        text.push(b'\n');
    }
    CONFIRM_MSG.set(Some(XString::from_bytes(&text)));

    let (shown, hotkeys) = render_buttons(buttons, default);
    CONFIRM_BUTTONS.set(Some(XString::from_bytes(&shown)));
    display_confirm_msg();
    hotkeys
}

/// Display the `:confirm` message. Also called when the screen is resized.
pub(crate) fn display_confirm_msg() {
    // Avoid that 'q' at the more prompt truncates the message here.
    let _in_use = Suppress::counter(CONFIRM_MSG_USED);
    // A copy: showing it can redraw, and a redraw can show it again.
    if let Some(text) = CONFIRM_MSG.with(Clone::clone) {
        msg_ext_set_kind(c"confirm");
        msg_str_hl(text.as_cstr(), HLF_M, false);
    }
}

/// What a yes/no-style dialog was answered with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    Yes,
    No,
    /// "Save All".
    All,
    /// "Discard All".
    DiscardAll,
    Cancel,
}

impl Answer {
    /// The answer `button` stands for in a dialog whose buttons are
    /// `answers`, in order. No button (cancelled), or one past the list,
    /// is the list's last answer: "No" or "Cancel".
    fn of_button(answers: &[Answer], button: Option<Button>) -> Answer {
        button
            .and_then(|button| answers.get(button.number() - 1))
            .or(answers.last())
            .copied()
            .unwrap_or(Answer::Cancel)
    }

    /// The button that answers `self` in a dialog whose buttons are
    /// `answers`.
    fn button_in(self, answers: &[Answer]) -> Option<Button> {
        let at = answers.iter().position(|&answer| answer == self)?;
        Button::nth(at + 1)
    }
}

/// Ask through `buttons`, which answer `answers` in order, defaulting to
/// `default`.
fn ask(
    kind: DialogKind,
    title: &CStr,
    message: &CStr,
    buttons: &CStr,
    answers: &[Answer],
    default: Answer,
) -> Answer {
    let default = default.button_in(answers);
    let answer = do_dialog(kind, Some(title), message, buttons, default, None, false);
    Answer::of_button(answers, answer.button(default))
}

/// A yes/no dialog: [`Answer::Yes`] or [`Answer::No`].
pub(crate) fn vim_dialog_yesno(
    kind: DialogKind,
    title: Option<&CStr>,
    message: &CStr,
    default: Answer,
) -> Answer {
    let title = title.unwrap_or_else(|| gettext(c"Question"));
    let buttons = gettext(c"&Yes\n&No");
    ask(
        kind,
        title,
        message,
        buttons,
        &[Answer::Yes, Answer::No],
        default,
    )
}

/// A yes/no/cancel dialog.
pub(crate) fn vim_dialog_yesnocancel(
    kind: DialogKind,
    title: Option<&CStr>,
    message: &CStr,
    default: Answer,
) -> Answer {
    let title = title.unwrap_or_else(|| gettext(c"Question"));
    let buttons = gettext(c"&Yes\n&No\n&Cancel");
    ask(
        kind,
        title,
        message,
        buttons,
        &[Answer::Yes, Answer::No, Answer::Cancel],
        default,
    )
}

/// A yes/no/all/discard/cancel dialog, for `:wq` over several changed buffers.
pub(crate) fn vim_dialog_yesnoallcancel(
    kind: DialogKind,
    title: Option<&CStr>,
    message: &CStr,
    default: Answer,
) -> Answer {
    // Note: unlike its two siblings, this default title is not translated.
    let title = title.unwrap_or(c"Question");
    let buttons = gettext(c"&Yes\n&No\nSave &All\n&Discard All\n&Cancel");
    let answers = [
        Answer::Yes,
        Answer::No,
        Answer::All,
        Answer::DiscardAll,
        Answer::Cancel,
    ];
    ask(kind, title, message, buttons, &answers, default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_count_from_one() {
        assert_eq!(Button::nth(0), None);
        assert_eq!(Button::nth(1), Some(Button::FIRST));
        assert_eq!(Button::nth(3).map(Button::number), Some(3));
    }

    #[test]
    fn a_dismissed_dialog_names_its_button() {
        let second = Button::nth(2);
        assert_eq!(DialogAnswer::Default.button(second), second);
        assert_eq!(DialogAnswer::Default.button(None), None);
        assert_eq!(
            DialogAnswer::Pressed(Button::FIRST).button(second),
            Some(Button::FIRST)
        );
        assert_eq!(DialogAnswer::Cancelled.button(second), None);
    }

    #[test]
    fn buttons_answer_in_order_and_anything_else_is_the_last() {
        use Answer::*;
        let yes_no = [Yes, No];
        let all = [Yes, No, All, DiscardAll, Cancel];
        assert_eq!(Answer::of_button(&yes_no, Button::nth(1)), Yes);
        assert_eq!(Answer::of_button(&yes_no, Button::nth(2)), No);
        assert_eq!(Answer::of_button(&yes_no, None), No);
        assert_eq!(Answer::of_button(&all, Button::nth(4)), DiscardAll);
        assert_eq!(Answer::of_button(&all, Button::nth(9)), Cancel);
        assert_eq!(No.button_in(&yes_no), Button::nth(2));
        assert_eq!(All.button_in(&yes_no), None);
        assert_eq!(DiscardAll.button_in(&all), Button::nth(4));
    }

    #[test]
    fn rendered_buttons_mark_hotkeys_and_the_default() {
        let (shown, hotkeys) = render_buttons(b"&Yes\n&No\nSave &All", Button::nth(2));
        assert_eq!(shown, b"(Y)es, [N]o, Save (A)ll: ");
        assert_eq!(
            hotkeys,
            [
                Key::ascii(b'y').code(),
                Key::ascii(b'n').code(),
                Key::ascii(b'a').code()
            ]
        );
        // No `&`: the first character. `&&` is a literal `&`, and still
        // counts as naming a hotkey, so none is shown -- but the first
        // character answers.
        let (shown, hotkeys) = render_buttons(b"Ok\nR&&D", None);
        assert_eq!(shown, b"(O)k, R&D: ");
        assert_eq!(hotkeys, [Key::ascii(b'o').code(), Key::ascii(b'r').code()]);
    }
}
