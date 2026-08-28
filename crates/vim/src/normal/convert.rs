use collections::HashMap;
use editor::{SelectionEffects, display_map::ToDisplayPoint};
use gpui::{Context, Window};
use language::{Bias, Point, SelectionGoal};
use multi_buffer::MultiBufferRow;

use crate::{
    Vim,
    motion::Motion,
    normal::{ChangeCase, ConvertToLowerCase, ConvertToRot13, ConvertToRot47, ConvertToUpperCase},
    object::Object,
    state::Mode,
};

pub enum ConvertTarget {
    LowerCase,
    UpperCase,
    OppositeCase,
    Rot13,
    Rot47,
}

impl Vim {
    pub fn convert_motion(
        &mut self,
        motion: Motion,
        times: Option<usize>,
        forced_motion: bool,
        mode: ConvertTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop_recording(cx);
        self.update_editor(cx, |_, editor, cx| {
            editor.set_clip_at_line_ends(false, cx);
            let text_layout_details = editor.text_layout_details(window);
            editor.transact(window, cx, |editor, window, cx| {
                let mut selection_starts: HashMap<_, _> = Default::default();
                editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let anchor = map.display_point_to_anchor(selection.head(), Bias::Left);
                        selection_starts.insert(selection.id, anchor);
                        motion.expand_selection(map, selection, times, &text_layout_details, forced_motion);
                    });
                });
                match mode {
                    ConvertTarget::LowerCase => editor.convert_to_lower_case(&Default::default(), window, cx),
                    ConvertTarget::UpperCase => editor.convert_to_upper_case(&Default::default(), window, cx),
                    ConvertTarget::OppositeCase => editor.convert_to_opposite_case(&Default::default(), window, cx),
                    ConvertTarget::Rot13 => editor.convert_to_rot13(&Default::default(), window, cx),
                    ConvertTarget::Rot47 => editor.convert_to_rot47(&Default::default(), window, cx),
                }
                editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let anchor = selection_starts.remove(&selection.id).unwrap();
                        selection.collapse_to(anchor.to_display_point(map), SelectionGoal::None);
                    });
                });
            });
            editor.set_clip_at_line_ends(true, cx);
        });
    }

    pub fn convert_object(
        &mut self,
        object: Object,
        around: bool,
        mode: ConvertTarget,
        times: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop_recording(cx);
        self.update_editor(cx, |_, editor, cx| {
            editor.transact(window, cx, |editor, window, cx| {
                editor.set_clip_at_line_ends(false, cx);
                let mut original_positions: HashMap<_, _> = Default::default();
                editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        object.expand_selection(map, selection, around, times);
                        original_positions
                            .insert(selection.id, map.display_point_to_anchor(selection.start, Bias::Left));
                    });
                });
                match mode {
                    ConvertTarget::LowerCase => editor.convert_to_lower_case(&Default::default(), window, cx),
                    ConvertTarget::UpperCase => editor.convert_to_upper_case(&Default::default(), window, cx),
                    ConvertTarget::OppositeCase => editor.convert_to_opposite_case(&Default::default(), window, cx),
                    ConvertTarget::Rot13 => editor.convert_to_rot13(&Default::default(), window, cx),
                    ConvertTarget::Rot47 => editor.convert_to_rot47(&Default::default(), window, cx),
                }
                editor.change_selections(SelectionEffects::no_scroll(), window, cx, |s| {
                    s.move_with(|map, selection| {
                        let anchor = original_positions.remove(&selection.id).unwrap();
                        selection.collapse_to(anchor.to_display_point(map), SelectionGoal::None);
                    });
                });
                editor.set_clip_at_line_ends(true, cx);
            });
        });
    }

    pub fn change_case(&mut self, _: &ChangeCase, window: &mut Window, cx: &mut Context<Self>) {
        self.manipulate_text(window, cx, |c| {
            if c.is_lowercase() {
                c.to_uppercase().collect::<Vec<char>>()
            } else {
                c.to_lowercase().collect::<Vec<char>>()
            }
        })
    }

    pub fn convert_to_upper_case(&mut self, _: &ConvertToUpperCase, window: &mut Window, cx: &mut Context<Self>) {
        self.manipulate_text(window, cx, |c| c.to_uppercase().collect::<Vec<char>>())
    }

    pub fn convert_to_lower_case(&mut self, _: &ConvertToLowerCase, window: &mut Window, cx: &mut Context<Self>) {
        self.manipulate_text(window, cx, |c| c.to_lowercase().collect::<Vec<char>>())
    }

    pub fn convert_to_rot13(&mut self, _: &ConvertToRot13, window: &mut Window, cx: &mut Context<Self>) {
        self.manipulate_text(window, cx, |c| {
            vec![match c {
                'A'..='M' | 'a'..='m' => ((c as u8) + 13) as char,
                'N'..='Z' | 'n'..='z' => ((c as u8) - 13) as char,
                _ => c,
            }]
        })
    }

    pub fn convert_to_rot47(&mut self, _: &ConvertToRot47, window: &mut Window, cx: &mut Context<Self>) {
        self.manipulate_text(window, cx, |c| {
            let code_point = c as u32;
            if code_point >= 33 && code_point <= 126 {
                return vec![char::from_u32(33 + ((code_point + 14) % 94)).unwrap()];
            }
            vec![c]
        })
    }

    fn manipulate_text<F>(&mut self, window: &mut Window, cx: &mut Context<Self>, transform: F)
    where
        F: Fn(char) -> Vec<char> + Copy,
    {
        self.record_current_action(cx);
        self.store_visual_marks(window, cx);
        let count = Vim::take_count(cx).unwrap_or(1) as u32;
        Vim::take_forced_motion(cx);

        self.update_editor(cx, |vim, editor, cx| {
            let mut ranges = Vec::new();
            let mut cursor_positions = Vec::new();
            let snapshot = editor.buffer().read(cx).snapshot(cx);
            for selection in editor.selections.all_adjusted(&editor.display_snapshot(cx)) {
                match vim.mode {
                    Mode::Visual | Mode::VisualLine => {
                        ranges.push(selection.start..selection.end);
                        cursor_positions.push(selection.start..selection.start);
                    }
                    Mode::VisualBlock => {
                        ranges.push(selection.start..selection.end);
                        if cursor_positions.is_empty() {
                            cursor_positions.push(selection.start..selection.start);
                        }
                    }

                    Mode::HelixNormal | Mode::HelixSelect => {
                        if selection.is_empty() {
                            // Handle empty selection by operating on single character
                            let start = selection.start;
                            let end = snapshot.clip_point(start + Point::new(0, 1), Bias::Right);
                            ranges.push(start..end);
                            cursor_positions.push(selection.start..selection.start);
                        } else {
                            ranges.push(selection.start..selection.end);
                            cursor_positions.push(selection.start..selection.end);
                        }
                    }
                    Mode::Insert | Mode::Normal | Mode::Replace => {
                        let start = selection.start;
                        let mut end = start;
                        for _ in 0..count {
                            end = snapshot.clip_point(end + Point::new(0, 1), Bias::Right);
                        }
                        ranges.push(start..end);

                        if end.column == snapshot.line_len(MultiBufferRow(end.row)) && end.column > 0 {
                            end = snapshot.clip_point(end - Point::new(0, 1), Bias::Left);
                        }
                        cursor_positions.push(end..end)
                    }
                }
            }
            editor.transact(window, cx, |editor, window, cx| {
                for range in ranges.into_iter().rev() {
                    let snapshot = editor.buffer().read(cx).snapshot(cx);
                    let text = snapshot
                        .text_for_range(range.start..range.end)
                        .flat_map(|s| s.chars())
                        .flat_map(transform)
                        .collect::<String>();
                    editor.edit([(range, text)], cx)
                }
                editor.change_selections(Default::default(), window, cx, |s| s.select_ranges(cursor_positions))
            });
        });
        if self.mode != Mode::HelixNormal {
            self.switch_mode(Mode::Normal, true, window, cx)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test::VimTestContext;

    use crate::state::Mode;

    #[gpui::test]
    async fn test_change_case_helix_mode(cx: &mut gpui::TestAppContext) {
        let mut cx = VimTestContext::new(cx, true).await;

        // Explicit selection
        cx.set_state("«hello worldˇ»", Mode::HelixNormal);
        cx.simulate_keystrokes("~");
        cx.assert_state("«HELLO WORLDˇ»", Mode::HelixNormal);

        // Cursor-only (empty) selection - switch case
        cx.set_state("The ˇquick brown", Mode::HelixNormal);
        cx.simulate_keystrokes("~");
        cx.assert_state("The ˇQuick brown", Mode::HelixNormal);
        cx.simulate_keystrokes("~");
        cx.assert_state("The ˇquick brown", Mode::HelixNormal);

        // Cursor-only (empty) selection - switch to uppercase and lowercase explicitly
        cx.set_state("The ˇquick brown", Mode::HelixNormal);
        cx.simulate_keystrokes("alt-`");
        cx.assert_state("The ˇQuick brown", Mode::HelixNormal);
        cx.simulate_keystrokes("`");
        cx.assert_state("The ˇquick brown", Mode::HelixNormal);

        // With `e` motion (which extends selection to end of word in Helix)
        cx.set_state("The ˇquick brown fox", Mode::HelixNormal);
        cx.simulate_keystrokes("e");
        cx.simulate_keystrokes("~");
        cx.assert_state("The «QUICKˇ» brown fox", Mode::HelixNormal);

        // Cursor-only
    }
}
