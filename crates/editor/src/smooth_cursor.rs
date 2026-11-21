use crate::{DisplayPoint, editor_settings::SmoothCursorSettings};
use collections::HashMap;
use gpui::{Bounds, Size, smooth::SmoothAnimation};
use std::time::{Duration, Instant};
use ui::{Pixels, px};

const EPSILON: f64 = 0.001;

#[derive(Default)]
pub(crate) struct SmoothCursorStates {
    states: HashMap<usize, SmoothCursorState>,
    newest_selection_id: Option<usize>,
    newest_state: Option<SmoothCursorState>,
}

#[derive(Clone, Default)]
struct SmoothCursorState {
    position_animation: Option<SmoothAnimation>,
    size_animation: Option<SmoothAnimation>,
    target_bounds: Option<Bounds<f64>>,
    last_cursor_position: Option<DisplayPoint>,
}

impl SmoothCursorStates {
    pub(crate) fn reconcile_newest_selection(&mut self, newest_selection_id: Option<usize>) {
        if let Some(newest_selection_id) = newest_selection_id
            && self.newest_selection_id != Some(newest_selection_id)
            && !self.states.contains_key(&newest_selection_id)
        {
            let inherited_state = self
                .newest_selection_id
                .and_then(|selection_id| self.states.get(&selection_id).cloned())
                .or_else(|| self.newest_state.clone());
            if let Some(state) = inherited_state {
                self.states.insert(newest_selection_id, state);
            }
        }

        self.newest_selection_id = newest_selection_id;
    }

    pub(crate) fn capture_newest_state(&mut self) {
        if let Some(newest_selection_id) = self.newest_selection_id
            && let Some(state) = self.states.get(&newest_selection_id)
        {
            self.newest_state = Some(state.clone());
        }
    }

    pub(crate) fn remove(&mut self, selection_id: usize) {
        self.states.remove(&selection_id);
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(usize) -> bool) {
        self.states.retain(|selection_id, _| keep(*selection_id));
    }

    pub(crate) fn clear(&mut self) {
        self.states.clear();
        self.newest_selection_id = None;
        self.newest_state = None;
    }

    pub(crate) fn update(
        &mut self,
        selection_id: usize,
        cursor_position: DisplayPoint,
        target_bounds: Bounds<Pixels>,
        settings: SmoothCursorSettings,
        now: Instant,
    ) -> Option<Bounds<Pixels>> {
        self.states
            .entry(selection_id)
            .or_default()
            .update(cursor_position, target_bounds, settings, now)
    }

    pub(crate) fn active(&self) -> bool {
        self.states.iter().any(|state| state.1.active())
    }
}

impl SmoothCursorState {
    pub(crate) fn update(
        &mut self,
        cursor_position: DisplayPoint,
        target_bounds: Bounds<Pixels>,
        settings: SmoothCursorSettings,
        now: Instant,
    ) -> Option<Bounds<Pixels>> {
        let target_bounds = target_bounds.map(|v| v.to_f64());

        if !settings.enabled {
            self.reset(target_bounds);
            return None;
        }

        let Some(previous_bounds) = self.target_bounds else {
            self.reset(target_bounds);
            return None;
        };

        let almost_equal = target_bounds.origin.almost_equal(previous_bounds.origin, EPSILON);

        if let Some(last_cursor_position) = self.last_cursor_position
            && cursor_position == last_cursor_position
            && !almost_equal
        {
            // Cursor did not move but target bounds changed
            self.reset(target_bounds);
            return None;
        }

        self.last_cursor_position = Some(cursor_position);

        let duration = Duration::from_millis(settings.duration_ms);

        let (current_position, pos_done) = if let Some(position_animation) = self.position_animation.as_mut() {
            if almost_equal {
                position_animation.restart(target_bounds.origin);
            }
            position_animation.update(now, duration)
        } else {
            self.position_animation = Some(SmoothAnimation::new(previous_bounds.origin, target_bounds.origin, now));
            (previous_bounds.origin, false)
        };

        let (current_size, _) = if let Some(size_animation) = self.size_animation.as_mut() {
            size_animation.update(now, duration)
        } else {
            self.size_animation = Some(SmoothAnimation::new(
                previous_bounds.size.into(),
                target_bounds.size.into(),
                now,
            ));
            (previous_bounds.size.into(), false)
        };

        if pos_done {
            self.reset(target_bounds);
            return None;
        }

        self.target_bounds = Some(target_bounds);

        Some(
            Bounds {
                origin: current_position,
                size: Size {
                    width: current_size.x,
                    height: current_size.y,
                },
            }
            .map(|v| px(v as f32)),
        )
    }

    fn reset(&mut self, target_bounds: Bounds<f64>) {
        self.position_animation = None;
        self.size_animation = None;
        self.target_bounds = Some(target_bounds);
    }

    fn active(&self) -> bool {
        self.position_animation.is_some()
    }
}
