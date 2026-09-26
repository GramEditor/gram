use crate::{ContentGroup, prelude::*};
use gpui::{AnimationElement, AnimationExt, Styled};
use std::time::Duration;

use gpui::ease_out_quint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationDuration {
    Instant = 50,
    Fast = 150,
    Slow = 300,
}

impl AnimationDuration {
    pub fn duration(&self) -> Duration {
        Duration::from_millis(*self as u64)
    }
}

impl From<AnimationDuration> for std::time::Duration {
    fn from(val: AnimationDuration) -> Self {
        val.duration()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationDirection {
    Bottom,
    Left,
    Right,
    Top,
}

pub trait DefaultAnimations: Styled + Sized + Element {
    fn animate_in(self, animation_type: AnimationDirection, fade_in: bool) -> AnimationElement<Self> {
        let animation_name = match animation_type {
            AnimationDirection::Bottom => "animate_from_bottom",
            AnimationDirection::Left => "animate_from_left",
            AnimationDirection::Right => "animate_from_right",
            AnimationDirection::Top => "animate_from_top",
        };

        let animation_id = self
            .id()
            .map_or_else(|| ElementId::from(animation_name), |id| (id, animation_name).into());

        self.with_animation(
            animation_id,
            gpui::Animation::new(AnimationDuration::Fast.into()).with_easing(ease_out_quint),
            move |mut this, delta| {
                let start_opacity = 0.4;
                let start_pos = 0.0;
                let end_pos = 40.0;

                if fade_in {
                    this = this.opacity(start_opacity + delta * (1.0 - start_opacity));
                }

                match animation_type {
                    AnimationDirection::Bottom => this.bottom(px(start_pos + delta * (end_pos - start_pos))),
                    AnimationDirection::Left => this.left(px(start_pos + delta * (end_pos - start_pos))),
                    AnimationDirection::Right => this.right(px(start_pos + delta * (end_pos - start_pos))),
                    AnimationDirection::Top => this.top(px(start_pos + delta * (end_pos - start_pos))),
                }
            },
        )
    }

    fn animate_in_from_bottom(self, fade: bool) -> AnimationElement<Self> {
        self.animate_in(AnimationDirection::Bottom, fade)
    }

    fn animate_in_from_left(self, fade: bool) -> AnimationElement<Self> {
        self.animate_in(AnimationDirection::Left, fade)
    }

    fn animate_in_from_right(self, fade: bool) -> AnimationElement<Self> {
        self.animate_in(AnimationDirection::Right, fade)
    }

    fn animate_in_from_top(self, fade: bool) -> AnimationElement<Self> {
        self.animate_in(AnimationDirection::Top, fade)
    }
}

impl<E: Styled + Element> DefaultAnimations for E {}

// Don't use this directly, it only exists to show animation previews
#[derive(RegisterComponent)]
struct Animation {}

impl Component for Animation {
    fn scope() -> ComponentScope {
        ComponentScope::Utilities
    }

    fn description() -> Option<&'static str> {
        Some("Demonstrates various animation patterns and transitions available in the UI system.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let container_size = 128.0;
        let element_size = 32.0;
        let offset = container_size / 2.0 - element_size / 2.0;
        Some(
            v_flex()
                .gap_6()
                .children(vec![
                    example_group_with_title(
                        "Animate In",
                        vec![
                            single_example(
                                "From Bottom",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("animate-in-from-bottom")
                                            .absolute()
                                            .size(px(element_size))
                                            .left(px(offset))
                                            .rounded_md()
                                            .bg(gpui::red())
                                            .animate_in_from_bottom(false),
                                    )
                                    .into_any_element(),
                            ),
                            single_example(
                                "From Top",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("animate-in-from-top")
                                            .absolute()
                                            .size(px(element_size))
                                            .left(px(offset))
                                            .rounded_md()
                                            .bg(gpui::blue())
                                            .animate_in_from_top(false),
                                    )
                                    .into_any_element(),
                            ),
                            single_example(
                                "From Left",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("animate-in-from-left")
                                            .absolute()
                                            .size(px(element_size))
                                            .top(px(offset))
                                            .rounded_md()
                                            .bg(gpui::green())
                                            .animate_in_from_left(false),
                                    )
                                    .into_any_element(),
                            ),
                            single_example(
                                "From Right",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("animate-in-from-right")
                                            .absolute()
                                            .size(px(element_size))
                                            .top(px(offset))
                                            .rounded_md()
                                            .bg(gpui::yellow())
                                            .animate_in_from_right(false),
                                    )
                                    .into_any_element(),
                            ),
                        ],
                    )
                    .grow(),
                    example_group_with_title(
                        "Fade and Animate In",
                        vec![
                            single_example(
                                "From Bottom",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("fade-animate-in-from-bottom")
                                            .absolute()
                                            .size(px(element_size))
                                            .left(px(offset))
                                            .rounded_md()
                                            .bg(gpui::red())
                                            .animate_in_from_bottom(true),
                                    )
                                    .into_any_element(),
                            ),
                            single_example(
                                "From Top",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("fade-animate-in-from-top")
                                            .absolute()
                                            .size(px(element_size))
                                            .left(px(offset))
                                            .rounded_md()
                                            .bg(gpui::blue())
                                            .animate_in_from_top(true),
                                    )
                                    .into_any_element(),
                            ),
                            single_example(
                                "From Left",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("fade-animate-in-from-left")
                                            .absolute()
                                            .size(px(element_size))
                                            .top(px(offset))
                                            .rounded_md()
                                            .bg(gpui::green())
                                            .animate_in_from_left(true),
                                    )
                                    .into_any_element(),
                            ),
                            single_example(
                                "From Right",
                                ContentGroup::new()
                                    .relative()
                                    .items_center()
                                    .justify_center()
                                    .size(px(container_size))
                                    .child(
                                        div()
                                            .id("fade-animate-in-from-right")
                                            .absolute()
                                            .size(px(element_size))
                                            .top(px(offset))
                                            .rounded_md()
                                            .bg(gpui::yellow())
                                            .animate_in_from_right(true),
                                    )
                                    .into_any_element(),
                            ),
                        ],
                    )
                    .grow(),
                ])
                .into_any_element(),
        )
    }
}
