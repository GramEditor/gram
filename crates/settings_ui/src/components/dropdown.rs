use std::rc::Rc;

use gpui::{App, ElementId, IntoElement, RenderOnce};
use heck::ToTitleCase as _;
use ui::{ButtonSize, ContextMenu, DropdownMenu, DropdownStyle, FluentBuilder as _, IconPosition, px};

#[derive(IntoElement)]
pub struct EnumVariantDropdown {
    id: ElementId,
    current: usize,
    labels: &'static [&'static str],
    should_do_title_case: bool,
    tab_index: Option<isize>,
    on_change: Rc<dyn Fn(usize, &mut App) + 'static>,
}

impl EnumVariantDropdown {
    pub fn new(
        id: impl Into<ElementId>,
        current: usize,
        labels: &'static [&'static str],
        on_change: impl Fn(usize, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            current,
            labels,
            should_do_title_case: true,
            tab_index: None,
            on_change: Rc::new(on_change),
        }
    }

    pub fn title_case(mut self, title_case: bool) -> Self {
        self.should_do_title_case = title_case;
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = Some(tab_index);
        self
    }
}

impl RenderOnce for EnumVariantDropdown {
    fn render(self, window: &mut ui::Window, cx: &mut ui::App) -> impl gpui::IntoElement {
        let current_value_label = self.labels[self.current];

        let context_menu = window.use_keyed_state(current_value_label, cx, |window, cx| {
            ContextMenu::new(window, cx, move |mut menu, _, _| {
                for (index, &label) in self.labels.iter().enumerate() {
                    let on_change = self.on_change.clone();
                    let current = self.current;
                    menu = menu.toggleable_entry(
                        if self.should_do_title_case {
                            label.to_title_case()
                        } else {
                            label.to_string()
                        },
                        index == current,
                        IconPosition::End,
                        None,
                        move |_, cx| {
                            on_change(index, cx);
                        },
                    );
                }
                menu
            })
        });

        DropdownMenu::new(
            self.id,
            if self.should_do_title_case {
                current_value_label.to_title_case()
            } else {
                current_value_label.to_string()
            },
            context_menu,
        )
        .when_some(self.tab_index, |elem, tab_index| elem.tab_index(tab_index))
        .trigger_size(ButtonSize::Medium)
        .style(DropdownStyle::Outlined)
        .offset(gpui::Point { x: px(0.0), y: px(2.0) })
        .into_any_element()
    }
}
