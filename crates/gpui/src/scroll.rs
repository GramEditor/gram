use std::time::{Duration, Instant};

use crate::{App, Axis, Global, Pixels, point, px};

/// Minimum time to switch from horizontal to vertical touchpad scroll.
pub const SCROLL_EVENT_SEPARATION: Duration = Duration::from_millis(28);

/// Settings for scrolling: Smooth scroll, sensitivity.
/// Updated by the editor crate, so other users of the gpui
/// create would need to set these or use the defaults.
pub struct GlobalScrollSettings {
    /// Configured duration of the animation (None if animation is disabled).
    pub duration: Option<Duration>,
    /// Scroll sensitivity
    pub scroll_sensitivity: f32,
    /// Scroll sensitivity (holding alt/option)
    pub fast_scroll_sensitivity: f32,
}

impl Global for GlobalScrollSettings {}

impl GlobalScrollSettings {
    /// Some(duration) if smooth scroll is configured, None if disabled.
    pub fn duration(cx: &App) -> Option<Duration> {
        cx.try_global::<Self>().and_then(|setting| setting.duration)
    }

    /// Base scroll sensitivity
    pub fn scroll_sensitivity(cx: &App) -> f32 {
        cx.try_global::<Self>()
            .map(|setting| setting.scroll_sensitivity)
            .unwrap_or(1.0)
    }

    /// Fast scroll sensitivity (when holding alt/option)
    pub fn fast_scroll_sensitivity(cx: &App) -> f32 {
        cx.try_global::<Self>()
            .map(|setting| setting.fast_scroll_sensitivity)
            .unwrap_or(4.0)
    }
}

/// Used to implement touchpad scrolling
#[derive(Clone, Copy, Debug)]
pub struct OngoingScroll {
    /// Time of last scroll event.
    pub last_event: Instant,
    /// Last scroll axis (horizontal/vertical).
    pub axis: Option<Axis>,
}

impl Default for OngoingScroll {
    fn default() -> Self {
        Self::new()
    }
}

impl OngoingScroll {
    /// Create a new OngoingScroll instance
    pub fn new() -> Self {
        Self {
            last_event: Instant::now() - SCROLL_EVENT_SEPARATION,
            axis: None,
        }
    }

    /// Filter scroll events to avoid suddenly shifting from horizontal to vertical scroll
    pub fn filter(&self, delta: &mut gpui::Point<Pixels>) -> Option<Axis> {
        const UNLOCK_PERCENT: f32 = 1.9;
        const UNLOCK_LOWER_BOUND: Pixels = px(6.);
        let mut axis = self.axis;

        let x = delta.x.abs();
        let y = delta.y.abs();
        let duration = Instant::now().duration_since(self.last_event);
        if duration > SCROLL_EVENT_SEPARATION {
            //New ongoing scroll will start, determine axis
            axis = if x <= y {
                Some(Axis::Vertical)
            } else {
                Some(Axis::Horizontal)
            };
        } else if x.max(y) >= UNLOCK_LOWER_BOUND {
            //Check if the current ongoing will need to unlock
            match axis {
                Some(Axis::Vertical) if x > y && x >= y * UNLOCK_PERCENT => {
                    axis = None;
                }

                Some(Axis::Horizontal) if y > x && y >= x * UNLOCK_PERCENT => {
                    axis = None;
                }

                _ => {}
            }
        }

        match axis {
            Some(Axis::Vertical) => {
                *delta = point(px(0.), delta.y);
            }
            Some(Axis::Horizontal) => {
                *delta = point(delta.x, px(0.));
            }
            None => {}
        }

        axis
    }

    /// Update the axis at the current time.
    pub fn update(&mut self, axis: Option<Axis>) {
        self.last_event = Instant::now();
        self.axis = axis;
    }
}
