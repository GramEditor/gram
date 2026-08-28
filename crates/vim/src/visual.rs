use std::sync::Arc;

use collections::HashMap;
use editor::{
    Bias, DisplayPoint, Editor, MultiBufferOffset, SelectionEffects,
    display_map::{DisplaySnapshot, ToDisplayPoint},
    movement,
};
use gpui::{Context, Window, actions};
use language::{Point, Selection, SelectionGoal};
use multi_buffer::MultiBufferRow;
use search::BufferSearchBar;
use util::ResultExt;
use workspace::searchable::Direction;

use crate::{
    Vim,
    motion::{Motion, MotionKind, first_non_whitespace, next_line_end, start_of_line},
    object::Object,
    state::{Mark, Mode, Operator},
};

actions!(
    vim,
    [
        /// Toggles visual mode.
        ToggleVisual,
        /// Toggles visual line mode.
        ToggleVisualLine,
        /// Toggles visual block mode.
        ToggleVisualBlock,
        /// Deletes the visual selection.
        VisualDelete,
        /// Deletes entire lines in visual selection.
        VisualDeleteLine,
        /// Yanks (copies) the visual selection.
        VisualYank,
        /// Yanks entire lines in visual selection.
        VisualYankLine,
        /// Moves cursor to the other end of the selection.
        OtherEnd,
        /// Moves cursor to the other end of the selection (row-aware).
        OtherEndRowAware,
        /// Selects the next occurrence of the current selection.
        SelectNext,
        /// Selects the previous occurrence of the current selection.
        SelectPrevious,
        /// Selects the next match of the current selection.
        SelectNextMatch,
        /// Selects the previous match of the current selection.
        SelectPreviousMatch,
        /// Selects the next smaller syntax node.
        SelectSmallerSyntaxNode,
        /// Selects the next larger syntax node.
        SelectLargerSyntaxNode,
        /// Selects the next syntax node sibling.
        SelectNextSyntaxNode,
        /// Selects the previous syntax node sibling.
        SelectPreviousSyntaxNode,
        /// Restores the previous visual selection.
        RestoreVisualSelection,
        /// Inserts at the end of each line in visual selection.
        VisualInsertEndOfLine,
        /// Inserts at the first non-whitespace character of each line.
        VisualInsertFirstNonWhiteSpace,
    ]
);

pub fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, |vim, _: &ToggleVisual, window, cx| {
        vim.toggle_mode(Mode::Visual, window, cx)
    });
    Vim::action(editor, cx, |vim, _: &ToggleVisualLine, window, cx| {
        vim.toggle_mode(Mode::VisualLine, window, cx)
    });
    Vim::action(editor, cx, |vim, _: &ToggleVisualBlock, window, cx| {
        vim.toggle_mode(Mode::VisualBlock, window, cx)
    });
    Vim::action(editor, cx, Vim::other_end);
    Vim::action(editor, cx, Vim::other_end_row_aware);
    Vim::action(editor, cx, Vim::visual_insert_end_of_line);
    Vim::action(editor, cx, Vim::visual_insert_first_non_white_space);
    Vim::action(editor, cx, |vim, _: &VisualDelete, window, cx| {
        vim.record_current_action(cx);
        vim.visual_delete(false, window, cx);
    });
    Vim::action(editor, cx, |vim, _: &VisualDeleteLine, window, cx| {
        vim.record_current_action(cx);
        vim.visual_delete(true, window, cx);
    });
    Vim::action(editor, cx, |vim, _: &VisualYank, window, cx| {
        vim.visual_yank(false, window, cx)
    });
    Vim::action(editor, cx, |vim, _: &VisualYankLine, window, cx| {
        vim.visual_yank(true, window, cx)
    });

    Vim::action(editor, cx, Vim::select_next);
    Vim::action(editor, cx, Vim::select_previous);
    Vim::action(editor, cx, |vim, _: &SelectNextMatch, window, cx| {
        vim.select_match(Direction::Next, window, cx);
    });
    Vim::action(editor, cx, |vim, _: &SelectPreviousMatch, window, cx| {
        vim.select_match(Direction::Prev, window, cx);
    });

    Vim::action(editor, cx, |vim, _: &SelectLargerSyntaxNode, window, cx| {
        let count = Vim::take_count(cx).unwrap_or(1);
        Vim::take_forced_motion(cx);
        for _ in 0..count {
            vim.update_editor(cx, |_, editor, cx| {
                editor.select_larger_syntax_node(&Default::default(), window, cx);
            });
        }
    });

    Vim::action(editor, cx, |vim, _: &SelectNextSyntaxNode, window, cx| {
        let count = Vim::take_count(cx).unwrap_or(1);
        Vim::take_forced_motion(cx);
        for _ in 0..count {
            vim.update_editor(cx, |_, editor, cx| {
                editor.select_next_syntax_node(&Default::default(), window, cx);
            });
        }
    });

    Vim::action(editor, cx, |vim, _: &SelectPreviousSyntaxNode, window, cx| {
        let count = Vim::take_count(cx).unwrap_or(1);
        Vim::take_forced_motion(cx);
        for _ in 0..count {
            vim.update_editor(cx, |_, editor, cx| {
                editor.select_prev_syntax_node(&Default::default(), window, cx);
            });
        }
    });

    Vim::action(editor, cx, |vim, _: &SelectSmallerSyntaxNode, window, cx| {
        let count = Vim::take_count(cx).unwrap_or(1);
        Vim::take_forced_motion(cx);
        for _ in 0..count {
            vim.update_editor(cx, |_, editor, cx| {
                editor.select_smaller_syntax_node(&Default::default(), window, cx);
            });
        }
    });

    Vim::action(editor, cx, |vim, _: &RestoreVisualSelection, window, cx| {
        let Some((stored_mode, reversed)) = vim.stored_visual_mode.take() else {
            return;
        };
        let marks = vim
            .update_editor(cx, |vim, editor, cx| {
                vim.get_mark("<", editor, window, cx)
                    .zip(vim.get_mark(">", editor, window, cx))
            })
            .flatten();
        let Some((Mark::Local(start), Mark::Local(end))) = marks else {
            return;
        };
        let ranges = start
            .iter()
            .zip(end)
            .zip(reversed)
            .map(|((start, end), reversed)| (*start, end, reversed))
            .collect::<Vec<_>>();

        if vim.mode.is_visual() {
            vim.create_visual_marks(vim.mode, window, cx);
        }

        vim.update_editor(cx, |_, editor, cx| {
            editor.set_clip_at_line_ends(false, cx);
            editor.change_selections(Default::default(), window, cx, |s| {
                let map = s.display_snapshot();
                let ranges = ranges
                    .into_iter()
                    .map(|(start, end, reversed)| {
                        let mut new_end = movement::saturating_right(&map, end.to_display_point(&map));
                        let mut new_start = start.to_display_point(&map);
                        if new_start >= new_end {
                            if new_end.column() == 0 {
                                new_end = movement::right(&map, new_end)
                            } else {
                                new_start = movement::saturating_left(&map, new_end);
                            }
                        }
                        Selection {
                            id: s.new_selection_id(),
                            start: new_start.to_point(&map),
                            end: new_end.to_point(&map),
                            reversed,
                            goal: SelectionGoal::None,
                        }
                    })
                    .collect();
                s.select(ranges);
            })
        });
        vim.switch_mode(stored_mode, true, window, cx)
    });
}

impl Vim {
    pub fn visual_motion(&mut self, motion: Motion, times: Option<usize>, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(cx, |vim, editor, cx| {
            let text_layout_details = editor.text_layout_details(window);
            if vim.mode == Mode::VisualBlock && !matches!(motion, Motion::EndOfLine { display_lines: false }) {
                let is_up_or_down = matches!(motion, Motion::Up { .. } | Motion::Down { .. });
                vim.visual_block_motion(is_up_or_down, editor, window, cx, |map, point, goal| {
                    motion.move_point(map, point, goal, times, &text_layout_details)
                })
            } else {
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let was_reversed = selection.reversed;
                        let mut current_head = selection.head();

                        // our motions assume the current character is after the cursor,
                        // but in (forward) visual mode the current character is just
                        // before the end of the selection.

                        // If the file ends with a newline (which is common) we don't do this.
                        // so that if you go to the end of such a file you can use "up" to go
                        // to the previous line and have it work somewhat as expected.
                        if !selection.reversed
                            && !selection.is_empty()
                            && !(selection.end.column() == 0 && selection.end == map.max_point())
                        {
                            current_head = movement::left(map, selection.end)
                        }

                        let Some((new_head, goal)) =
                            motion.move_point(map, current_head, selection.goal, times, &text_layout_details)
                        else {
                            return;
                        };

                        selection.set_head(new_head, goal);

                        // ensure the current character is included in the selection.
                        if !selection.reversed {
                            let next_point = if vim.mode == Mode::VisualBlock {
                                movement::saturating_right(map, selection.end)
                            } else {
                                movement::right(map, selection.end)
                            };

                            if !(next_point.column() == 0 && next_point == map.max_point()) {
                                selection.end = next_point;
                            }
                        }

                        // vim always ensures the anchor character stays selected.
                        // if our selection has reversed, we need to move the opposite end
                        // to ensure the anchor is still selected.
                        if was_reversed && !selection.reversed {
                            selection.start = movement::left(map, selection.start);
                        } else if !was_reversed && selection.reversed {
                            selection.end = movement::right(map, selection.end);
                        }
                    })
                });
            }
        });
    }

    pub fn visual_block_motion(
        &mut self,
        preserve_goal: bool,
        editor: &mut Editor,
        window: &mut Window,
        cx: &mut Context<Editor>,
        mut move_selection: impl FnMut(
            &DisplaySnapshot,
            DisplayPoint,
            SelectionGoal,
        ) -> Option<(DisplayPoint, SelectionGoal)>,
    ) {
        let text_layout_details = editor.text_layout_details(window);
        editor.change_selections(Default::default(), window, cx, |s| {
            let map = &s.display_snapshot();
            let mut head = s.newest_anchor().head().to_display_point(map);
            let mut tail = s.oldest_anchor().tail().to_display_point(map);

            let mut head_x = map.x_for_display_point(head, &text_layout_details);
            let mut tail_x = map.x_for_display_point(tail, &text_layout_details);

            let (start, end) = match s.newest_anchor().goal {
                SelectionGoal::HorizontalRange { start, end } if preserve_goal => (start, end),
                SelectionGoal::HorizontalPosition(start) if preserve_goal => (start, start),
                _ => (tail_x.into(), head_x.into()),
            };
            let mut goal = SelectionGoal::HorizontalRange { start, end };

            let was_reversed = tail_x > head_x;
            if !was_reversed && !preserve_goal {
                head = movement::saturating_left(map, head);
            }

            let reverse_aware_goal = if was_reversed {
                SelectionGoal::HorizontalRange { start: end, end: start }
            } else {
                goal
            };

            let Some((new_head, _)) = move_selection(map, head, reverse_aware_goal) else {
                return;
            };
            head = new_head;
            head_x = map.x_for_display_point(head, &text_layout_details);

            let is_reversed = tail_x > head_x;
            if was_reversed && !is_reversed {
                tail = movement::saturating_left(map, tail);
                tail_x = map.x_for_display_point(tail, &text_layout_details);
            } else if !was_reversed && is_reversed {
                tail = movement::saturating_right(map, tail);
                tail_x = map.x_for_display_point(tail, &text_layout_details);
            }
            if !is_reversed && !preserve_goal {
                head = movement::saturating_right(map, head);
                head_x = map.x_for_display_point(head, &text_layout_details);
            }

            let positions = if is_reversed { head_x..tail_x } else { tail_x..head_x };

            if !preserve_goal {
                goal = SelectionGoal::HorizontalRange {
                    start: f64::from(positions.start),
                    end: f64::from(positions.end),
                };
            }

            let mut selections = Vec::new();
            let mut row = tail.row();
            let going_up = tail.row() > head.row();
            let direction = if going_up { -1 } else { 1 };

            loop {
                let laid_out_line = map.layout_row(row, &text_layout_details);
                let start = DisplayPoint::new(row, laid_out_line.closest_index_for_x(positions.start) as u32);
                let mut end = DisplayPoint::new(row, laid_out_line.closest_index_for_x(positions.end) as u32);
                if end <= start {
                    if start.column() == map.line_len(start.row()) {
                        end = start;
                    } else {
                        end = movement::saturating_right(map, start);
                    }
                }

                if positions.start <= laid_out_line.width {
                    let selection = Selection {
                        id: s.new_selection_id(),
                        start: start.to_point(map),
                        end: end.to_point(map),
                        reversed: is_reversed &&
                                    // For neovim parity: cursor is not reversed when column is a single character
                                    end.column() - start.column() > 1,
                        goal,
                    };

                    selections.push(selection);
                }

                // When dealing with soft wrapped lines, it's possible that
                // `row` ends up being set to a value other than `head.row()` as
                // `head.row()` might be a `DisplayPoint` mapped to a soft
                // wrapped line, hence the need for `<=` and `>=` instead of
                // `==`.
                if going_up && row <= head.row() || !going_up && row >= head.row() {
                    break;
                }

                // Find the next or previous buffer row where the `row` should
                // be moved to, so that wrapped lines are skipped.
                row = map
                    .start_of_relative_buffer_row(DisplayPoint::new(row, 0), direction)
                    .row();
            }

            s.select(selections);
        })
    }

    pub fn visual_object(&mut self, object: Object, count: Option<usize>, window: &mut Window, cx: &mut Context<Vim>) {
        if let Some(Operator::Object { around }) = self.active_operator() {
            self.pop_operator(window, cx);
            let current_mode = self.mode;
            let target_mode = object.target_visual_mode(current_mode, around);
            if target_mode != current_mode {
                self.switch_mode(target_mode, true, window, cx);
            }

            self.update_editor(cx, |_, editor, cx| {
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let mut mut_selection = selection.clone();

                        // all our motions assume that the current character is
                        // after the cursor; however in the case of a visual selection
                        // the current character is before the cursor.
                        // But this will affect the judgment of the html tag
                        // so the html tag needs to skip this logic.
                        if !selection.reversed && object != Object::Tag {
                            mut_selection.set_head(movement::left(map, mut_selection.head()), mut_selection.goal);
                        }

                        let original_point = selection.tail().to_point(map);

                        if let Some(range) = object.range(map, mut_selection, around, count) {
                            if !range.is_empty() {
                                let expand_both_ways = object.always_expands_both_ways()
                                    || selection.is_empty()
                                    || movement::right(map, selection.start) == selection.end;

                                if expand_both_ways {
                                    if selection.start == range.start
                                        && selection.end == range.end
                                        && object.always_expands_both_ways()
                                    {
                                        if let Some(range) = object.range(map, selection.clone(), around, count) {
                                            selection.start = range.start;
                                            selection.end = range.end;
                                        }
                                    } else {
                                        selection.start = range.start;
                                        selection.end = range.end;
                                    }
                                } else if selection.reversed {
                                    selection.start = range.start;
                                } else {
                                    selection.end = range.end;
                                }
                            }

                            // In the visual selection result of a paragraph object, the cursor is
                            // placed at the start of the last line. And in the visual mode, the
                            // selection end is located after the end character. So, adjustment of
                            // selection end is needed.
                            //
                            // We don't do this adjustment for a one-line blank paragraph since the
                            // trailing newline is included in its selection from the beginning.
                            if object == Object::Paragraph && range.start != range.end {
                                let row_of_selection_end_line = selection.end.to_point(map).row;
                                let new_selection_end = if map
                                    .buffer_snapshot()
                                    .line_len(MultiBufferRow(row_of_selection_end_line))
                                    == 0
                                {
                                    Point::new(row_of_selection_end_line + 1, 0)
                                } else {
                                    Point::new(row_of_selection_end_line, 1)
                                };
                                selection.end = new_selection_end.to_display_point(map);
                            }

                            // To match vim, if the range starts of the same line as it originally
                            // did, we keep the tail of the selection in the same place instead of
                            // snapping it to the start of the line
                            if target_mode == Mode::VisualLine {
                                let new_start_point = selection.start.to_point(map);
                                if new_start_point.row == original_point.row {
                                    if selection.end.to_point(map).row > new_start_point.row {
                                        if original_point.column
                                            == map.buffer_snapshot().line_len(MultiBufferRow(original_point.row))
                                        {
                                            selection.start =
                                                movement::saturating_left(map, original_point.to_display_point(map))
                                        } else {
                                            selection.start = original_point.to_display_point(map)
                                        }
                                    } else {
                                        let original_display_point = original_point.to_display_point(map);
                                        if selection.end <= original_display_point {
                                            selection.end = movement::saturating_right(map, original_display_point);
                                            if original_point.column > 0 {
                                                selection.reversed = true
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
            });
        }
    }

    fn visual_insert_end_of_line(&mut self, _: &VisualInsertEndOfLine, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(cx, |_, editor, cx| {
            editor.split_selection_into_lines(&Default::default(), window, cx);
            editor.change_selections(Default::default(), window, cx, |s| {
                s.move_cursors_with(|map, cursor, _| (next_line_end(map, cursor, 1), SelectionGoal::None));
            });
        });

        self.switch_mode(Mode::Insert, false, window, cx);
    }

    fn visual_insert_first_non_white_space(
        &mut self,
        _: &VisualInsertFirstNonWhiteSpace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_editor(cx, |_, editor, cx| {
            editor.split_selection_into_lines(&Default::default(), window, cx);
            editor.change_selections(Default::default(), window, cx, |s| {
                s.move_cursors_with(|map, cursor, _| (first_non_whitespace(map, false, cursor), SelectionGoal::None));
            });
        });

        self.switch_mode(Mode::Insert, false, window, cx);
    }

    fn toggle_mode(&mut self, mode: Mode, window: &mut Window, cx: &mut Context<Self>) {
        if self.mode == mode {
            self.switch_mode(Mode::Normal, false, window, cx);
        } else {
            self.switch_mode(mode, false, window, cx);
        }
    }

    pub fn other_end(&mut self, _: &OtherEnd, window: &mut Window, cx: &mut Context<Self>) {
        self.update_editor(cx, |_, editor, cx| {
            editor.change_selections(Default::default(), window, cx, |s| {
                s.move_with(|_, selection| {
                    selection.reversed = !selection.reversed;
                });
            })
        });
    }

    pub fn other_end_row_aware(&mut self, _: &OtherEndRowAware, window: &mut Window, cx: &mut Context<Self>) {
        let mode = self.mode;
        self.update_editor(cx, |_, editor, cx| {
            editor.change_selections(Default::default(), window, cx, |s| {
                s.move_with(|_, selection| {
                    selection.reversed = !selection.reversed;
                });
                if mode == Mode::VisualBlock {
                    s.reverse_selections();
                }
            })
        });
    }

    pub fn visual_delete(&mut self, line_mode: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.store_visual_marks(window, cx);
        self.update_editor(cx, |vim, editor, cx| {
            let mut original_columns: HashMap<_, _> = Default::default();
            let line_mode = line_mode || editor.selections.line_mode();
            editor.selections.set_line_mode(false);

            editor.transact(window, cx, |editor, window, cx| {
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        if line_mode {
                            let mut position = selection.head();
                            if !selection.reversed {
                                position = movement::left(map, position);
                            }
                            original_columns.insert(selection.id, position.to_point(map).column);
                            if vim.mode == Mode::VisualBlock {
                                *selection.end.column_mut() = map.line_len(selection.end.row())
                            } else {
                                let start = selection.start.to_point(map);
                                let end = selection.end.to_point(map);
                                selection.start = map.prev_line_boundary(start).1;
                                if end.column == 0 && end > start {
                                    let row = end.row.saturating_sub(1);
                                    selection.end = Point::new(row, map.buffer_snapshot().line_len(MultiBufferRow(row)))
                                        .to_display_point(map)
                                } else {
                                    selection.end = map.next_line_boundary(end).1;
                                }
                            }
                        }
                        selection.goal = SelectionGoal::None;
                    });
                });
                let kind = if line_mode {
                    MotionKind::Linewise
                } else {
                    MotionKind::Exclusive
                };
                vim.copy_selections_content(editor, kind, window, cx);

                if line_mode && vim.mode != Mode::VisualBlock {
                    editor.change_selections(Default::default(), window, cx, |s| {
                        s.move_with(|map, selection| {
                            let end = selection.end.to_point(map);
                            let start = selection.start.to_point(map);
                            if end.row < map.buffer_snapshot().max_point().row {
                                selection.end = Point::new(end.row + 1, 0).to_display_point(map)
                            } else if start.row > 0 {
                                selection.start = Point::new(
                                    start.row - 1,
                                    map.buffer_snapshot().line_len(MultiBufferRow(start.row - 1)),
                                )
                                .to_display_point(map)
                            }
                        });
                    });
                }
                editor.insert("", window, cx);

                // Fixup cursor position after the deletion
                editor.set_clip_at_line_ends(true, cx);
                editor.change_selections(Default::default(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let mut cursor = selection.head().to_point(map);

                        if let Some(column) = original_columns.get(&selection.id) {
                            cursor.column = *column
                        }
                        let cursor = map.clip_point(cursor.to_display_point(map), Bias::Left);
                        selection.collapse_to(cursor, selection.goal)
                    });
                    if vim.mode == Mode::VisualBlock {
                        s.select_anchors(vec![s.first_anchor()])
                    }
                });
            })
        });
        self.switch_mode(Mode::Normal, true, window, cx);
    }

    pub fn visual_yank(&mut self, line_mode: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.store_visual_marks(window, cx);
        self.update_editor(cx, |vim, editor, cx| {
            let line_mode = line_mode || editor.selections.line_mode();

            // For visual line mode, adjust selections to avoid yanking the next line when on \n
            if line_mode && vim.mode != Mode::VisualBlock {
                editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let start = selection.start.to_point(map);
                        let end = selection.end.to_point(map);
                        if end.column == 0 && end > start {
                            let row = end.row.saturating_sub(1);
                            selection.end = Point::new(row, map.buffer_snapshot().line_len(MultiBufferRow(row)))
                                .to_display_point(map);
                        }
                    });
                });
            }

            editor.selections.set_line_mode(line_mode);
            let kind = if line_mode {
                MotionKind::Linewise
            } else {
                MotionKind::Exclusive
            };
            vim.yank_selections_content(editor, kind, window, cx);
            editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                s.move_with(|map, selection| {
                    if line_mode {
                        selection.start = start_of_line(map, false, selection.start);
                    };
                    selection.collapse_to(selection.start, SelectionGoal::None)
                });
                if vim.mode == Mode::VisualBlock {
                    s.select_anchors(vec![s.first_anchor()])
                }
            });
        });
        self.switch_mode(Mode::Normal, true, window, cx);
    }

    pub(crate) fn visual_replace(&mut self, text: Arc<str>, window: &mut Window, cx: &mut Context<Self>) {
        self.stop_recording(cx);
        self.update_editor(cx, |_, editor, cx| {
            editor.transact(window, cx, |editor, window, cx| {
                let display_map = editor.display_snapshot(cx);
                let selections = editor.selections.all_adjusted_display(&display_map);

                // Selections are biased right at the start. So we need to store
                // anchors that are biased left so that we can restore the selections
                // after the change
                let stable_anchors = editor
                    .selections
                    .disjoint_anchors_arc()
                    .iter()
                    .map(|selection| {
                        let start = selection.start.bias_left(&display_map.buffer_snapshot());
                        start..start
                    })
                    .collect::<Vec<_>>();

                let mut edits = Vec::new();
                for selection in selections.iter() {
                    let selection = selection.clone();
                    for row_range in movement::split_display_range_by_lines(&display_map, selection.range()) {
                        let range = row_range.start.to_offset(&display_map, Bias::Right)
                            ..row_range.end.to_offset(&display_map, Bias::Right);
                        let text = text.repeat(range.end - range.start);
                        edits.push((range, text));
                    }
                }

                editor.edit(edits, cx);
                editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                    s.select_ranges(stable_anchors)
                });
            });
        });
        self.switch_mode(Mode::Normal, false, window, cx);
    }

    pub fn select_next(&mut self, _: &SelectNext, window: &mut Window, cx: &mut Context<Self>) {
        Vim::take_forced_motion(cx);
        let count = Vim::take_count(cx).unwrap_or_else(|| if self.mode.is_visual() { 1 } else { 2 });
        self.update_editor(cx, |_, editor, cx| {
            editor.set_clip_at_line_ends(false, cx);
            for _ in 0..count {
                if editor.select_next(&Default::default(), window, cx).log_err().is_none() {
                    break;
                }
            }
        });
    }

    pub fn select_previous(&mut self, _: &SelectPrevious, window: &mut Window, cx: &mut Context<Self>) {
        Vim::take_forced_motion(cx);
        let count = Vim::take_count(cx).unwrap_or_else(|| if self.mode.is_visual() { 1 } else { 2 });
        self.update_editor(cx, |_, editor, cx| {
            for _ in 0..count {
                if editor
                    .select_previous(&Default::default(), window, cx)
                    .log_err()
                    .is_none()
                {
                    break;
                }
            }
        });
    }

    pub fn select_match(&mut self, direction: Direction, window: &mut Window, cx: &mut Context<Self>) {
        Vim::take_forced_motion(cx);
        let count = Vim::take_count(cx).unwrap_or(1);
        let Some(pane) = self.pane(window, cx) else {
            return;
        };
        let vim_is_normal = self.mode == Mode::Normal;
        let mut start_selection = MultiBufferOffset(0);
        let mut end_selection = MultiBufferOffset(0);

        self.update_editor(cx, |_, editor, _| {
            editor.set_collapse_matches(false);
        });
        if vim_is_normal {
            pane.update(cx, |pane, cx| {
                if let Some(search_bar) = pane.toolbar().read(cx).item_of_type::<BufferSearchBar>() {
                    search_bar.update(cx, |search_bar, cx| {
                        if !search_bar.has_active_match() || !search_bar.show(window, cx) {
                            return;
                        }
                        // without update_match_index there is a bug when the cursor is before the first match
                        search_bar.update_match_index(window, cx);
                        search_bar.select_match(direction.opposite(), 1, window, cx);
                    });
                }
            });
        }
        self.update_editor(cx, |_, editor, cx| {
            let latest = editor
                .selections
                .newest::<MultiBufferOffset>(&editor.display_snapshot(cx));
            start_selection = latest.start;
            end_selection = latest.end;
        });

        let mut match_exists = false;
        pane.update(cx, |pane, cx| {
            if let Some(search_bar) = pane.toolbar().read(cx).item_of_type::<BufferSearchBar>() {
                search_bar.update(cx, |search_bar, cx| {
                    search_bar.update_match_index(window, cx);
                    search_bar.select_match(direction, count, window, cx);
                    match_exists = search_bar.match_exists(window, cx);
                });
            }
        });
        if !match_exists {
            self.clear_operator(window, cx);
            self.stop_replaying(cx);
            return;
        }
        self.update_editor(cx, |_, editor, cx| {
            let latest = editor
                .selections
                .newest::<MultiBufferOffset>(&editor.display_snapshot(cx));
            if vim_is_normal {
                start_selection = latest.start;
                end_selection = latest.end;
            } else {
                start_selection = start_selection.min(latest.start);
                end_selection = end_selection.max(latest.end);
            }
            if direction == Direction::Prev {
                std::mem::swap(&mut start_selection, &mut end_selection);
            }
            editor.change_selections(Default::default(), window, cx, |s| {
                s.select_ranges([start_selection..end_selection]);
            });
            editor.set_collapse_matches(true);
        });

        match self.maybe_pop_operator() {
            Some(Operator::Change) => self.substitute(None, false, window, cx),
            Some(Operator::Delete) => {
                self.stop_recording(cx);
                self.visual_delete(false, window, cx)
            }
            Some(Operator::Yank) => self.visual_yank(false, window, cx),
            _ => {} // Ignoring other operators
        }
    }
}
#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::{state::Mode, test::VimTestContext};

    #[gpui::test]
    async fn test_visual_insert_first_non_whitespace(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state(
            indoc! {
                "«The quick brown
                fox jumps over
                the lazy dogˇ»"
            },
            Mode::Visual,
        );
        cx.simulate_keystrokes("g shift-i");
        cx.assert_state(
            indoc! {
                "ˇThe quick brown
                ˇfox jumps over
                ˇthe lazy dog"
            },
            Mode::Insert,
        );
    }

    #[gpui::test]
    async fn test_visual_insert_end_of_line(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state(
            indoc! {
                "«The quick brown
                fox jumps over
                the lazy dogˇ»"
            },
            Mode::Visual,
        );
        cx.simulate_keystrokes("g shift-a");
        cx.assert_state(
            indoc! {
                "The quick brownˇ
                fox jumps overˇ
                the lazy dogˇ"
            },
            Mode::Insert,
        );
    }

    #[gpui::test]
    async fn test_mode_across_command(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state("aˇbc", Mode::Normal);
        cx.simulate_keystrokes("ctrl-v");
        assert_eq!(cx.mode(), Mode::VisualBlock);
        cx.simulate_keystrokes("cmd-shift-p escape");
        assert_eq!(cx.mode(), Mode::VisualBlock);
    }

    #[gpui::test]
    async fn test_gl(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state("aaˇ aa\naa", Mode::Normal);
        cx.simulate_keystrokes("g l");
        cx.assert_state("«aaˇ» «aaˇ»\naa", Mode::Visual);
        cx.simulate_keystrokes("g >");
        cx.assert_state("«aaˇ» aa\n«aaˇ»", Mode::Visual);
    }

    #[gpui::test]
    async fn test_visual_syntax_sibling_selection(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        cx.set_state(
            indoc! {"
                fn test() {
                    let ˇa = 1;
                    let b = 2;
                    let c = 3;
                }
            "},
            Mode::Normal,
        );

        // Enter visual mode and select the statement
        cx.simulate_keystrokes("v w w w");
        cx.assert_state(
            indoc! {"
                fn test() {
                    let «a = 1;ˇ»
                    let b = 2;
                    let c = 3;
                }
            "},
            Mode::Visual,
        );

        // The specific behavior of syntax sibling selection in vim mode
        // would depend on the key bindings configured, but the actions
        // are now available for use
    }
}
