use editor::{Bias, Direction, Editor, display_map::ToDisplayPoint, movement};
use gpui::{Context, Window, actions};

use crate::{Vim, state::Mode};

actions!(
    vim,
    [
        /// Navigates to an older position in the change list.
        ChangeListOlder,
        /// Navigates to a newer position in the change list.
        ChangeListNewer
    ]
);

pub(crate) fn register(editor: &mut Editor, cx: &mut Context<Vim>) {
    Vim::action(editor, cx, |vim, _: &ChangeListOlder, window, cx| {
        vim.move_to_change(Direction::Prev, window, cx);
    });
    Vim::action(editor, cx, |vim, _: &ChangeListNewer, window, cx| {
        vim.move_to_change(Direction::Next, window, cx);
    });
}

impl Vim {
    fn move_to_change(&mut self, direction: Direction, window: &mut Window, cx: &mut Context<Self>) {
        let count = Vim::take_count(cx).unwrap_or(1);
        Vim::take_forced_motion(cx);
        self.update_editor(cx, |_, editor, cx| {
            if let Some(selections) = editor.change_list.next_change(count, direction).map(|s| s.to_vec()) {
                editor.change_selections(Default::default(), window, cx, |s| {
                    let map = s.display_snapshot();
                    s.select_display_ranges(selections.iter().map(|a| {
                        let point = a.to_display_point(&map);
                        point..point
                    }))
                })
            };
        });
    }

    pub(crate) fn push_to_change_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((new_positions, buffer)) = self.update_editor(cx, |vim, editor, cx| {
            let display_map = editor.display_snapshot(cx);
            let selections = editor.selections.all_adjusted_display(&display_map);
            let buffer = editor.buffer().clone();

            let pop_state = editor
                .change_list
                .last()
                .map(|previous| {
                    previous.len() == selections.len()
                        && previous
                            .iter()
                            .enumerate()
                            .all(|(ix, p)| p.to_display_point(&display_map).row() == selections[ix].head().row())
                })
                .unwrap_or(false);

            let new_positions = selections
                .into_iter()
                .map(|s| {
                    let point = if vim.mode == Mode::Insert {
                        movement::saturating_left(&display_map, s.head())
                    } else {
                        s.head()
                    };
                    display_map.display_point_to_anchor(point, Bias::Left)
                })
                .collect::<Vec<_>>();

            editor.change_list.push_to_change_list(pop_state, new_positions.clone());

            (new_positions, buffer)
        }) else {
            return;
        };

        self.set_mark(".".to_string(), new_positions, &buffer, window, cx)
    }
}
