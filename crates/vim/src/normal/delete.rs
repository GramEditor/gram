use crate::{
    Vim,
    motion::{Motion, MotionKind},
    object::Object,
    state::Mode,
};
use collections::{HashMap, HashSet};
use editor::{
    Bias, DisplayPoint,
    display_map::{DisplaySnapshot, ToDisplayPoint},
};
use gpui::{Context, Window};
use language::{Point, Selection};
use multi_buffer::MultiBufferRow;

impl Vim {
    pub fn delete_motion(
        &mut self,
        motion: Motion,
        times: Option<usize>,
        forced_motion: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop_recording(cx);
        self.update_editor(cx, |vim, editor, cx| {
            let text_layout_details = editor.text_layout_details(window);
            editor.transact(window, cx, |editor, window, cx| {
                editor.set_clip_at_line_ends(false, cx);
                let mut original_columns: HashMap<_, _> = Default::default();
                let mut motion_kind = None;
                let mut ranges_to_copy = Vec::new();
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let original_head = selection.head();
                        original_columns.insert(selection.id, original_head.column());
                        let kind = motion.expand_selection(map, selection, times, &text_layout_details, forced_motion);
                        ranges_to_copy.push(selection.start.to_point(map)..selection.end.to_point(map));

                        // When deleting line-wise, we always want to delete a newline.
                        // If there is one after the current line, it goes; otherwise we
                        // pick the one before.
                        if kind == Some(MotionKind::Linewise) {
                            let start = selection.start.to_point(map);
                            let end = selection.end.to_point(map);
                            if end.row < map.buffer_snapshot().max_point().row {
                                selection.end = Point::new(end.row + 1, 0).to_display_point(map)
                            } else if start.row > 0 {
                                selection.start = Point::new(
                                    start.row - 1,
                                    map.buffer_snapshot().line_len(MultiBufferRow(start.row - 1)),
                                )
                                .to_display_point(map)
                            }
                        }
                        if let Some(kind) = kind {
                            motion_kind.get_or_insert(kind);
                        }
                    });
                });
                let Some(kind) = motion_kind else { return };
                vim.copy_ranges(editor, kind, false, ranges_to_copy, window, cx);
                editor.insert("", window, cx);

                // Fixup cursor position after the deletion
                editor.set_clip_at_line_ends(true, cx);
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let mut cursor = selection.head();
                        if kind.linewise()
                            && let Some(column) = original_columns.get(&selection.id)
                        {
                            *cursor.column_mut() = *column
                        }
                        cursor = map.clip_point(cursor, Bias::Left);
                        selection.collapse_to(cursor, selection.goal)
                    });
                });
            });
        });
    }

    pub fn delete_object(
        &mut self,
        object: Object,
        around: bool,
        times: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop_recording(cx);
        self.update_editor(cx, |vim, editor, cx| {
            editor.transact(window, cx, |editor, window, cx| {
                editor.set_clip_at_line_ends(false, cx);
                // Emulates behavior in vim where if we expanded backwards to include a newline
                // the cursor gets set back to the start of the line
                let mut should_move_to_start: HashSet<_> = Default::default();

                // Emulates behavior in vim where after deletion the cursor should try to move
                // to the same column it was before deletion if the line is not empty or only
                // contains whitespace
                let mut column_before_move: HashMap<_, _> = Default::default();
                let target_mode = object.target_visual_mode(vim.mode, around);

                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let cursor_point = selection.head().to_point(map);
                        if target_mode == Mode::VisualLine {
                            column_before_move.insert(selection.id, cursor_point.column);
                        }

                        object.expand_selection(map, selection, around, times);
                        let offset_range = selection.map(|p| p.to_offset(map, Bias::Left)).range();
                        let mut move_selection_start_to_previous_line =
                            |map: &DisplaySnapshot, selection: &mut Selection<DisplayPoint>| {
                                let start = selection.start.to_offset(map, Bias::Left);
                                if selection.start.row().0 > 0 {
                                    should_move_to_start.insert(selection.id);
                                    selection.start = (start - '\n'.len_utf8()).to_display_point(map);
                                }
                            };
                        let range =
                            selection.start.to_offset(map, Bias::Left)..selection.end.to_offset(map, Bias::Right);
                        let contains_only_newlines = map
                            .buffer_chars_at(range.start)
                            .take_while(|(_, p)| p < &range.end)
                            .all(|(char, _)| char == '\n')
                            && !offset_range.is_empty();
                        let end_at_newline = map
                            .buffer_chars_at(range.end)
                            .next()
                            .map(|(c, _)| c == '\n')
                            .unwrap_or(false);

                        // If expanded range contains only newlines and
                        // the object is around or sentence, expand to include a newline
                        // at the end or start
                        if (around || object == Object::Sentence) && contains_only_newlines {
                            if end_at_newline {
                                move_selection_end_to_next_line(map, selection);
                            } else {
                                move_selection_start_to_previous_line(map, selection);
                            }
                        }

                        // Does post-processing for the trailing newline and EOF
                        // when not cancelled.
                        let cancelled = around && selection.start == selection.end;
                        if object == Object::Paragraph && !cancelled {
                            // EOF check should be done before including a trailing newline.
                            if ends_at_eof(map, selection) {
                                move_selection_start_to_previous_line(map, selection);
                            }

                            if end_at_newline {
                                move_selection_end_to_next_line(map, selection);
                            }
                        }
                    });
                });
                vim.copy_selections_content(editor, MotionKind::Exclusive, window, cx);
                editor.insert("", window, cx);

                // Fixup cursor position after the deletion
                editor.set_clip_at_line_ends(true, cx);
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let mut cursor = selection.head();
                        if should_move_to_start.contains(&selection.id) {
                            *cursor.column_mut() = 0;
                        } else if let Some(column) = column_before_move.get(&selection.id)
                            && *column > 0
                        {
                            let mut cursor_point = cursor.to_point(map);
                            cursor_point.column = *column;
                            cursor = map
                                .buffer_snapshot()
                                .clip_point(cursor_point, Bias::Left)
                                .to_display_point(map);
                        }
                        cursor = map.clip_point(cursor, Bias::Left);
                        selection.collapse_to(cursor, selection.goal)
                    });
                });
            });
        });
    }
}

fn move_selection_end_to_next_line(map: &DisplaySnapshot, selection: &mut Selection<DisplayPoint>) {
    let end = selection.end.to_offset(map, Bias::Left);
    selection.end = (end + '\n'.len_utf8()).to_display_point(map);
}

fn ends_at_eof(map: &DisplaySnapshot, selection: &mut Selection<DisplayPoint>) -> bool {
    selection.end.to_point(map) == map.buffer_snapshot().max_point()
}

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{state::Mode, test::VimTestContext};

    #[gpui::test]
    async fn test_cancel_delete_operator(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        cx.set_state(
            indoc! {"
                The quick brown
                fox juˇmps over
                the lazy dog"},
            Mode::Normal,
        );

        // Canceling operator twice reverts to normal mode with no active operator
        cx.simulate_keystrokes("d escape k");
        assert_eq!(cx.active_operator(), None);
        assert_eq!(cx.mode(), Mode::Normal);
        cx.assert_editor_state(indoc! {"
            The quˇick brown
            fox jumps over
            the lazy dog"});
    }

    #[gpui::test]
    async fn test_unbound_command_cancels_pending_operator(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;
        cx.set_state(
            indoc! {"
                The quick brown
                fox juˇmps over
                the lazy dog"},
            Mode::Normal,
        );

        // Canceling operator twice reverts to normal mode with no active operator
        cx.simulate_keystrokes("d y");
        assert_eq!(cx.active_operator(), None);
        assert_eq!(cx.mode(), Mode::Normal);
    }
}
