use std::hash::{DefaultHasher, Hash, Hasher};

use crate::prelude::*;

use gpui::{Hsla, ImageSource, IntoElement, Styled, img};

pub struct AvatarStyle {
    pub accent_foreground: Option<Hsla>,
    pub accent_background: Option<Hsla>,
    pub image: Option<ImageSource>,
}

impl AvatarStyle {
    /// Generate a "unique" avatar based on an identity string (usually an email address).
    pub fn new(identity: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        identity.hash(&mut hasher);
        let id = hasher.finish();
        Self {
            accent_foreground: Some(Hsla {
                h: ((id % 32) as f32) / 32.0,
                s: 0.66,
                l: 0.66,
                a: 1.0,
            }),
            accent_background: None,
            image: None,
        }
    }

    pub fn foreground(&self, fallback: Color) -> Color {
        self.accent_foreground
            .map(|hsla| Color::Custom(hsla))
            .unwrap_or(fallback)
    }

    pub fn background(&self, fallback: Hsla) -> Hsla {
        self.accent_background.unwrap_or(fallback)
    }
}

pub fn render_avatar(style: &AvatarStyle, size: Rems, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let border_width = px(0.);
    let container_size = size.to_pixels(window.rem_size()) + border_width * 2.;
    let image = style.image.clone().map(|image| img(image)).unwrap_or_else(|| img(""));

    let background = style.background(cx.theme().colors().element_disabled);
    let foreground = style.foreground(Color::Muted);
    div()
        .size(container_size)
        .rounded_full()
        .child(image.size(size).rounded_full().bg(background).with_fallback(move || {
            h_flex()
                .size_full()
                .justify_center()
                .child(Icon::new(IconName::Person).color(foreground).size(IconSize::Small))
                .into_any_element()
        }))
}
