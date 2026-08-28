use crate::{
    Vim,
    motion::{self, Motion, MotionKind},
    object::Object,
    state::Mode,
};
use editor::{
    Bias, DisplayPoint,
    display_map::{DisplaySnapshot, ToDisplayPoint},
    movement::TextLayoutDetails,
};
use gpui::{Context, Window};
use language::Selection;

impl Vim {
    pub fn change_motion(
        &mut self,
        motion: Motion,
        times: Option<usize>,
        forced_motion: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Some motions ignore failure when switching to normal mode
        let mut motion_kind = if matches!(
            motion,
            Motion::Left | Motion::Right | Motion::EndOfLine { .. } | Motion::WrappingLeft | Motion::StartOfLine { .. }
        ) {
            Some(MotionKind::Exclusive)
        } else {
            None
        };
        self.update_editor(cx, |vim, editor, cx| {
            let text_layout_details = editor.text_layout_details(window);
            editor.transact(window, cx, |editor, window, cx| {
                // We are swapping to insert mode anyway. Just set the line end clipping behavior now
                editor.set_clip_at_line_ends(false, cx);
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let kind = match motion {
                            Motion::NextWordStart { ignore_punctuation }
                            | Motion::NextSubwordStart { ignore_punctuation } => expand_changed_word_selection(
                                map,
                                selection,
                                times,
                                ignore_punctuation,
                                &text_layout_details,
                                motion == Motion::NextSubwordStart { ignore_punctuation },
                                !matches!(motion, Motion::NextWordStart { .. }),
                            ),
                            _ => {
                                let kind =
                                    motion.expand_selection(map, selection, times, &text_layout_details, forced_motion);
                                if matches!(motion, Motion::CurrentLine | Motion::Down { .. } | Motion::Up { .. }) {
                                    let mut start_offset = selection.start.to_offset(map, Bias::Left);
                                    let classifier =
                                        map.buffer_snapshot().char_classifier_at(selection.start.to_point(map));
                                    for (ch, offset) in map.buffer_chars_at(start_offset) {
                                        if ch == '\n' || !classifier.is_whitespace(ch) {
                                            break;
                                        }
                                        start_offset = offset + ch.len_utf8();
                                    }
                                    selection.start = start_offset.to_display_point(map);
                                }
                                kind
                            }
                        };
                        if let Some(kind) = kind {
                            motion_kind.get_or_insert(kind);
                        }
                    });
                });
                if let Some(kind) = motion_kind {
                    vim.copy_selections_content(editor, kind, window, cx);
                    editor.insert("", window, cx);
                }
            });
        });

        if motion_kind.is_some() {
            self.switch_mode(Mode::Insert, false, window, cx)
        } else {
            self.switch_mode(Mode::Normal, false, window, cx)
        }
    }

    pub fn change_object(
        &mut self,
        object: Object,
        around: bool,
        times: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut objects_found = false;
        self.update_editor(cx, |vim, editor, cx| {
            // We are swapping to insert mode anyway. Just set the line end clipping behavior now
            editor.set_clip_at_line_ends(false, cx);
            editor.transact(window, cx, |editor, window, cx| {
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        objects_found |= object.expand_selection(map, selection, around, times);
                    });
                });
                if objects_found {
                    let kind = match object.target_visual_mode(vim.mode, around) {
                        Mode::VisualLine => MotionKind::Linewise,
                        _ => MotionKind::Exclusive,
                    };
                    vim.copy_selections_content(editor, kind, window, cx);
                    editor.insert("", window, cx);
                }
            });
        });

        if objects_found {
            self.switch_mode(Mode::Insert, false, window, cx);
        } else {
            self.switch_mode(Mode::Normal, false, window, cx);
        }
    }
}

// From the docs https://vimdoc.sourceforge.net/htmldoc/motion.html
// Special case: "cw" and "cW" are treated like "ce" and "cE" if the cursor is
// on a non-blank.  This is because "cw" is interpreted as change-word, and a
// word does not include the following white space.  {Vi: "cw" when on a blank
// followed by other blanks changes only the first blank; this is probably a
// bug, because "dw" deletes all the blanks}
fn expand_changed_word_selection(
    map: &DisplaySnapshot,
    selection: &mut Selection<DisplayPoint>,
    times: Option<usize>,
    ignore_punctuation: bool,
    text_layout_details: &TextLayoutDetails,
    use_subword: bool,
    always_advance: bool,
) -> Option<MotionKind> {
    let is_in_word = || {
        let classifier = map.buffer_snapshot().char_classifier_at(selection.start.to_point(map));

        map.buffer_chars_at(selection.head().to_offset(map, Bias::Left))
            .next()
            .map(|(c, _)| !classifier.is_whitespace(c))
            .unwrap_or_default()
    };
    if (times.is_none() || times.unwrap() == 1) && is_in_word() {
        let next_char = map
            .buffer_chars_at(motion::next_char(map, selection.end, false).to_offset(map, Bias::Left))
            .next();
        match next_char {
            Some((' ', _)) => selection.end = motion::next_char(map, selection.end, false),
            _ => {
                if use_subword {
                    selection.end = motion::next_subword_end(map, selection.end, ignore_punctuation, 1, false);
                } else {
                    selection.end =
                        motion::next_word_end(map, selection.end, ignore_punctuation, 1, false, always_advance);
                }
                selection.end = motion::next_char(map, selection.end, false);
            }
        }
        Some(MotionKind::Inclusive)
    } else {
        let motion = if use_subword {
            Motion::NextSubwordStart { ignore_punctuation }
        } else {
            Motion::NextWordStart { ignore_punctuation }
        };
        motion.expand_selection(map, selection, times, text_layout_details, false)
    }
}
