use std::time::{Duration, Instant};

use crate::Point;

/// Exponential smoothing point to point animation.
/// See https://lisyarus.github.io/blog/posts/exponential-smoothing.html
#[derive(Clone, Copy, Debug)]
pub struct SmoothAnimation {
    /// Current position
    pub current: Point<f64>,
    /// Target position which we are chasing
    pub target: Point<f64>,
    /// Last updated at this time
    pub updated_at: Instant,
}

impl SmoothAnimation {
    /// Create a new animation state.
    pub fn new(current: Point<f64>, target: Point<f64>, now: Instant) -> Self {
        Self {
            current,
            target,
            updated_at: now,
        }
    }

    /// Create a new animation state from a point.
    pub fn from_point(point: Point<f64>, now: Instant) -> Self {
        Self {
            current: point,
            target: point,
            updated_at: now,
        }
    }

    /// Resets the animation target position.
    pub fn restart(&mut self, target: Point<f64>) {
        self.target = target;
    }

    /// Returns whether the target was reached.
    pub fn update(&mut self, now: Instant, duration: Duration) -> (Point<f64>, bool) {
        let duration = duration.as_secs_f64();
        let current = self.current;
        let target = self.target;

        const EPSILON: f64 = 0.001;
        let delta_x = target.x - current.x;
        let delta_y = target.y - current.y;
        if delta_x.abs() < EPSILON && delta_y.abs() < EPSILON {
            self.current = self.target;
            return (self.current, true);
        }

        let dt = now.duration_since(self.updated_at).as_secs_f64();
        let speed = 3.0 / duration;
        let decay = if dt > 0.0 { 1.0 - (-speed * dt).exp() } else { 1.0 };
        self.updated_at = now;
        self.current.x += delta_x * decay;
        self.current.y += delta_y * decay;
        (self.current, false)
    }
}
