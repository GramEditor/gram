#[cfg(test)]
mod alignment;

use std::{ops::Range, sync::Arc};

use buffer_diff::BufferDiff;
use collections::HashMap;
use gpui::{Action, AppContext as _, Entity, EventEmitter, Focusable, NoAction, Subscription, WeakEntity};
use language::{Buffer, Capability};
use multi_buffer::{Anchor, ExcerptId, ExcerptRange, ExpandExcerptDirection, MultiBuffer, PathKey};
use project::Project;
use rope::Point;
use text::{Bias, OffsetRangeExt as _};
use ui::{
    App, Context, InteractiveElement as _, IntoElement as _, ParentElement as _, Render, Styled as _, Window, div,
};
use workspace::{ActivePaneDecorator, Item, ItemHandle, Pane, PaneGroup, SplitDirection, Workspace};

use crate::{Editor, EditorEvent};

#[derive(Clone, Copy, PartialEq, Eq, Action, Default)]
#[action(namespace = editor)]
struct SplitDiff;

#[derive(Clone, Copy, PartialEq, Eq, Action, Default)]
#[action(namespace = editor)]
struct UnsplitDiff;

pub struct SplittableEditor {
    project: Entity<Project>,
    primary_multibuffer: Entity<MultiBuffer>,
    primary_editor: Entity<Editor>,
    secondary: Option<SecondaryEditor>,
    panes: PaneGroup,
    workspace: WeakEntity<Workspace>,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, PartialEq, Eq)]
struct ExcerptSyncKey {
    diff_revision: Option<u64>,
    excerpts: Vec<(ExcerptId, ExcerptRange<text::Anchor>)>,
    new_version: clock::Global,
    old_version: clock::Global,
    base_id: text::BufferId,
    diff_id: Option<gpui::EntityId>,
}

struct SecondaryEditor {
    multibuffer: Entity<MultiBuffer>,
    editor: Entity<Editor>,
    pane: Entity<Pane>,
    has_latest_selection: bool,
    primary_to_secondary: HashMap<ExcerptId, ExcerptId>,
    secondary_to_primary: HashMap<ExcerptId, ExcerptId>,
    synced_paths: HashMap<PathKey, ExcerptSyncKey>,
    _subscriptions: Vec<Subscription>,
}

impl SplittableEditor {
    pub fn active_new_buffer(&self, cx: &App) -> Option<Entity<Buffer>> {
        let editor = self.last_selected_editor().read(cx);
        let (mut id, _, _) = editor.active_excerpt(cx)?;
        if let Some(secondary) = &self.secondary
            && secondary.has_latest_selection
        {
            id = *secondary.secondary_to_primary.get(&id)?;
        }
        let snapshot = self.primary_multibuffer.read(cx).snapshot(cx);
        let buffer = snapshot.buffer_for_excerpt(id)?;
        self.primary_multibuffer.read(cx).buffer(buffer.remote_id())
    }

    pub fn set_split_diff_enabled(&mut self, enabled: bool, window: &mut Window, cx: &mut Context<Self>) {
        if enabled {
            self.split(&SplitDiff, window, cx);
        } else {
            self.unsplit(&UnsplitDiff, window, cx);
        }
    }

    pub fn is_split(&self) -> bool {
        self.secondary.is_some()
    }

    pub fn primary_editor(&self) -> &Entity<Editor> {
        &self.primary_editor
    }

    pub fn last_selected_editor(&self) -> &Entity<Editor> {
        if let Some(secondary) = &self.secondary
            && secondary.has_latest_selection
        {
            &secondary.editor
        } else {
            &self.primary_editor
        }
    }

    pub fn new_unsplit(
        primary_multibuffer: Entity<MultiBuffer>,
        project: Entity<Project>,
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let primary_editor = cx.new(|cx| {
            let mut editor = Editor::for_multibuffer(primary_multibuffer.clone(), Some(project.clone()), window, cx);
            editor.set_expand_all_diff_hunks(cx);
            editor
        });
        Self::from_editor(primary_editor, project, workspace, window, cx)
    }

    pub fn from_editor(
        primary_editor: Entity<Editor>,
        project: Entity<Project>,
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let primary_multibuffer = primary_editor.read(cx).buffer().clone();
        let pane = cx.new(|cx| {
            let mut pane = Pane::new(
                workspace.downgrade(),
                project.clone(),
                Default::default(),
                None,
                NoAction.boxed_clone(),
                true,
                window,
                cx,
            );
            pane.set_should_display_tab_bar(|_, _| false);
            pane.add_item(primary_editor.boxed_clone(), true, true, None, window, cx);
            pane
        });
        let panes = PaneGroup::new(pane);
        let subscriptions = vec![
            cx.subscribe(&primary_editor, |this, _, event: &EditorEvent, cx| match event {
                EditorEvent::ExpandExcerptsRequested {
                    excerpt_ids,
                    lines,
                    direction,
                } => {
                    this.expand_excerpts(excerpt_ids.iter().copied(), *lines, *direction, cx);
                }
                EditorEvent::SelectionsChanged { local: true } | EditorEvent::FocusedIn => {
                    if let Some(secondary) = &mut this.secondary {
                        secondary.has_latest_selection = false;
                    }
                    cx.emit(event.clone());
                }
                _ => cx.emit(event.clone()),
            }),
        ];

        window.defer(cx, {
            let workspace = workspace.downgrade();
            let primary_editor = primary_editor.downgrade();
            move |window, cx| {
                workspace
                    .update(cx, |workspace, cx| {
                        primary_editor.update(cx, |editor, cx| {
                            editor.added_to_workspace(workspace, window, cx);
                        })
                    })
                    .ok();
            }
        });
        Self {
            project,
            primary_editor,
            primary_multibuffer,
            secondary: None,
            panes,
            workspace: workspace.downgrade(),
            _subscriptions: subscriptions,
        }
    }

    fn split(&mut self, _: &SplitDiff, window: &mut Window, cx: &mut Context<Self>) {
        if self.secondary.is_some() {
            return;
        }
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let project = self.project.clone();

        let singleton = self.primary_multibuffer.read(cx).is_singleton();
        let singleton_source = singleton.then(|| {
            let buffer = self.primary_multibuffer.read(cx).as_singleton().unwrap();
            let diff = self.primary_multibuffer.read(cx).diff_for(buffer.read(cx).remote_id());
            (buffer, diff)
        });
        let secondary_multibuffer = cx.new(|cx| {
            let mut multibuffer = if let Some((buffer, diff)) = &singleton_source {
                let base = diff
                    .as_ref()
                    .map(|diff| diff.read(cx).base_text_buffer())
                    .unwrap_or_else(|| buffer.clone());
                let mut multibuffer = MultiBuffer::singleton(base, cx);
                if let Some(diff) = diff {
                    multibuffer.add_inverted_diff(diff.clone(), buffer.clone(), cx);
                }
                multibuffer
            } else {
                MultiBuffer::new(Capability::ReadOnly)
            };
            multibuffer.set_all_diff_hunks_expanded(cx);
            multibuffer
        });
        let secondary_editor = cx.new(|cx| {
            let mut editor = Editor::for_multibuffer(secondary_multibuffer.clone(), Some(project.clone()), window, cx);
            editor.number_deleted_lines = true;
            editor.set_read_only(true);
            editor.set_delegate_expand_excerpts(true);
            editor.disable_diagnostics(cx);
            editor.set_render_diff_hunk_controls(Arc::new(|_, _, _, _, _, _, _, _| gpui::Empty.into_any_element()), cx);
            editor
        });
        window.defer(cx, {
            let workspace = self.workspace.clone();
            let editor = secondary_editor.downgrade();
            move |window, cx| {
                workspace
                    .update(cx, |workspace, cx| {
                        editor
                            .update(cx, |editor, cx| editor.added_to_workspace(workspace, window, cx))
                            .ok();
                    })
                    .ok();
            }
        });
        let secondary_pane = cx.new(|cx| {
            let mut pane = Pane::new(
                workspace.downgrade(),
                project.clone(),
                Default::default(),
                None,
                NoAction.boxed_clone(),
                true,
                window,
                cx,
            );
            pane.set_should_display_tab_bar(|_, _| false);
            pane.add_item(
                ItemHandle::boxed_clone(&secondary_editor),
                false,
                false,
                None,
                window,
                cx,
            );
            pane
        });

        let subscriptions = vec![
            cx.subscribe(&secondary_editor, |this, _, event: &EditorEvent, cx| match event {
                EditorEvent::ExpandExcerptsRequested {
                    excerpt_ids,
                    lines,
                    direction,
                } => {
                    if let Some(secondary) = &this.secondary {
                        let primary_ids: Vec<_> = excerpt_ids
                            .iter()
                            .filter_map(|id| secondary.secondary_to_primary.get(id).copied())
                            .collect();
                        this.expand_excerpts(primary_ids.into_iter(), *lines, *direction, cx);
                    }
                }
                EditorEvent::SelectionsChanged { local: true } | EditorEvent::FocusedIn => {
                    if let Some(secondary) = &mut this.secondary {
                        secondary.has_latest_selection = true;
                    }
                    cx.emit(event.clone());
                }
                _ => cx.emit(event.clone()),
            }),
        ];
        let mut secondary = SecondaryEditor {
            editor: secondary_editor,
            multibuffer: secondary_multibuffer,
            pane: secondary_pane.clone(),
            has_latest_selection: false,
            primary_to_secondary: HashMap::default(),
            secondary_to_primary: HashMap::default(),
            synced_paths: HashMap::default(),
            _subscriptions: subscriptions,
        };
        self.primary_editor.update(cx, |editor, cx| {
            editor.set_delegate_expand_excerpts(true);
            editor.buffer().update(cx, |primary_multibuffer, cx| {
                primary_multibuffer.set_show_deleted_hunks(false, cx);
                let paths = primary_multibuffer.paths().cloned().collect::<Vec<_>>();
                for path in paths {
                    let Some(excerpt_id) = primary_multibuffer.excerpts_for_path(&path).next() else {
                        continue;
                    };
                    let snapshot = primary_multibuffer.snapshot(cx);
                    let buffer = snapshot.buffer_for_excerpt(excerpt_id).unwrap();
                    let diff = primary_multibuffer.diff_for(buffer.remote_id());
                    secondary.sync_path_excerpts(path.clone(), primary_multibuffer, diff, cx);
                }
            })
        });
        if singleton {
            let primary_id = self
                .primary_multibuffer
                .read(cx)
                .snapshot(cx)
                .excerpts()
                .next()
                .unwrap()
                .0;
            let secondary_id = secondary.multibuffer.read(cx).snapshot(cx).excerpts().next().unwrap().0;
            secondary.primary_to_secondary.insert(primary_id, secondary_id);
            secondary.secondary_to_primary.insert(secondary_id, primary_id);
        }
        self.secondary = Some(secondary);

        let primary_pane = self.panes.first_pane();
        self.panes
            .split(&primary_pane, &secondary_pane, SplitDirection::Left)
            .unwrap();
        cx.notify();
    }

    fn unsplit(&mut self, _: &UnsplitDiff, window: &mut Window, cx: &mut Context<Self>) {
        let Some(secondary) = self.secondary.take() else {
            return;
        };
        let restore_focus = secondary.editor.focus_handle(cx).contains_focused(window, cx);
        self.panes.remove(&secondary.pane).unwrap();
        if restore_focus {
            self.primary_editor.focus_handle(cx).focus(window, cx);
        }
        self.primary_editor.update(cx, |primary, cx| {
            primary.set_delegate_expand_excerpts(false);
            primary.buffer().update(cx, |buffer, cx| {
                buffer.set_show_deleted_hunks(true, cx);
            });
        });
        cx.notify();
    }

    pub fn added_to_workspace(&mut self, workspace: &mut Workspace, window: &mut Window, cx: &mut Context<Self>) {
        self.workspace = workspace.weak_handle();
        self.project = workspace.project().clone();
        self.primary_editor.update(cx, |primary_editor, cx| {
            primary_editor.added_to_workspace(workspace, window, cx);
        });
        if let Some(secondary) = &self.secondary {
            secondary.editor.update(cx, |secondary_editor, cx| {
                secondary_editor.added_to_workspace(workspace, window, cx);
            });
        }
    }

    pub fn set_excerpts_for_path(
        &mut self,
        path: PathKey,
        buffer: Entity<Buffer>,
        ranges: impl IntoIterator<Item = Range<Point>> + Clone,
        context_line_count: u32,
        diff: Entity<BufferDiff>,
        cx: &mut Context<Self>,
    ) -> (Vec<Range<Anchor>>, bool) {
        self.primary_multibuffer.update(cx, |primary_multibuffer, cx| {
            let (anchors, added_a_new_excerpt) =
                primary_multibuffer.set_excerpts_for_path(path.clone(), buffer.clone(), ranges, context_line_count, cx);
            if !anchors.is_empty()
                && primary_multibuffer
                    .diff_for(buffer.read(cx).remote_id())
                    .is_none_or(|old_diff| old_diff.entity_id() != diff.entity_id())
            {
                primary_multibuffer.add_diff(diff.clone(), cx);
            }
            if let Some(secondary) = &mut self.secondary {
                secondary.sync_path_excerpts(path, primary_multibuffer, Some(diff), cx);
            }
            (anchors, added_a_new_excerpt)
        })
    }

    fn expand_excerpts(
        &mut self,
        excerpt_ids: impl Iterator<Item = ExcerptId> + Clone,
        lines: u32,
        direction: ExpandExcerptDirection,
        cx: &mut Context<Self>,
    ) {
        let mut corresponding_paths = HashMap::default();
        self.primary_multibuffer.update(cx, |multibuffer, cx| {
            let snapshot = multibuffer.snapshot(cx);
            if self.secondary.is_some() {
                corresponding_paths = excerpt_ids
                    .clone()
                    .map(|excerpt_id| {
                        let path = multibuffer.path_for_excerpt(excerpt_id).unwrap();
                        let buffer = snapshot.buffer_for_excerpt(excerpt_id).unwrap();
                        let diff = multibuffer.diff_for(buffer.remote_id());
                        (path, diff)
                    })
                    .collect::<HashMap<_, _>>();
            }
            multibuffer.expand_excerpts(excerpt_ids.clone(), lines, direction, cx);
        });

        if let Some(secondary) = &mut self.secondary {
            self.primary_multibuffer.update(cx, |multibuffer, cx| {
                for (path, diff) in corresponding_paths {
                    secondary.sync_path_excerpts(path, multibuffer, diff, cx);
                }
            })
        }
    }

    pub fn remove_excerpts_for_path(&mut self, path: PathKey, cx: &mut Context<Self>) {
        self.primary_multibuffer
            .update(cx, |buffer, cx| buffer.remove_excerpts_for_path(path.clone(), cx));
        if let Some(secondary) = &mut self.secondary {
            secondary.remove_mappings_for_path(&path, cx);
            secondary
                .multibuffer
                .update(cx, |buffer, cx| buffer.remove_excerpts_for_path(path, cx))
        }
    }
}

impl EventEmitter<EditorEvent> for SplittableEditor {}
impl Focusable for SplittableEditor {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.last_selected_editor().read(cx).focus_handle(cx)
    }
}

impl Render for SplittableEditor {
    fn render(&mut self, window: &mut ui::Window, cx: &mut ui::Context<Self>) -> impl ui::IntoElement {
        let inner = if self.secondary.is_none() {
            self.primary_editor.clone().into_any_element()
        } else if let Some(active) = self.panes.panes().into_iter().next() {
            self.panes
                .render(None, &ActivePaneDecorator::new(active, &self.workspace), window, cx)
                .into_any_element()
        } else {
            div().into_any_element()
        };
        div()
            .id("splittable-editor")
            .on_action(cx.listener(Self::split))
            .on_action(cx.listener(Self::unsplit))
            .size_full()
            .child(inner)
    }
}

impl SecondaryEditor {
    fn sync_path_excerpts(
        &mut self,
        path_key: PathKey,
        primary_multibuffer: &mut MultiBuffer,
        diff: Option<Entity<BufferDiff>>,
        cx: &mut App,
    ) {
        let Some(excerpt_id) = primary_multibuffer.excerpts_for_path(&path_key).next() else {
            self.remove_mappings_for_path(&path_key, cx);
            self.multibuffer.update(cx, |multibuffer, cx| {
                multibuffer.remove_excerpts_for_path(path_key, cx);
            });
            return;
        };

        let primary_excerpt_ids: Vec<ExcerptId> = primary_multibuffer.excerpts_for_path(&path_key).collect();

        let primary_multibuffer_snapshot = primary_multibuffer.snapshot(cx);
        let main_buffer = primary_multibuffer_snapshot.buffer_for_excerpt(excerpt_id).unwrap();
        let base_text_buffer = diff
            .as_ref()
            .map(|diff| diff.read(cx).base_text_buffer())
            .unwrap_or_else(|| primary_multibuffer.buffer(main_buffer.remote_id()).unwrap());
        let diff_snapshot = diff.as_ref().map(|diff| diff.read(cx).snapshot(cx));
        let base_text_buffer_snapshot = base_text_buffer.read(cx).snapshot();
        let excerpts = primary_multibuffer.excerpts_for_buffer(main_buffer.remote_id(), cx);
        let key = ExcerptSyncKey {
            diff_revision: diff_snapshot.as_ref().map(|d| d.revision()),
            excerpts: excerpts.clone(),
            new_version: main_buffer.version().clone(),
            old_version: base_text_buffer_snapshot.version().clone(),
            base_id: base_text_buffer_snapshot.remote_id(),
            diff_id: diff.as_ref().map(|d| d.entity_id()),
        };
        if self.synced_paths.get(&path_key) == Some(&key) {
            return;
        }
        let new = excerpts
            .into_iter()
            .map(|(_, excerpt_range)| {
                let point_range_to_base_text_point_range = |range: Range<Point>| {
                    let Some(diff_snapshot) = &diff_snapshot else {
                        return range;
                    };
                    let start_row = diff_snapshot.row_to_base_text_row(range.start.row, Bias::Left, main_buffer);
                    let end_row = diff_snapshot.row_to_base_text_row(range.end.row, Bias::Right, main_buffer);
                    let end_column = diff_snapshot.base_text().line_len(end_row);
                    Point::new(start_row, 0)..Point::new(end_row, end_column)
                };
                let primary = excerpt_range.primary.to_point(main_buffer);
                let context = excerpt_range.context.to_point(main_buffer);
                ExcerptRange {
                    primary: point_range_to_base_text_point_range(primary),
                    context: point_range_to_base_text_point_range(context),
                }
            })
            .collect();

        let main_buffer = primary_multibuffer.buffer(main_buffer.remote_id()).unwrap();

        self.remove_mappings_for_path(&path_key, cx);

        self.editor.update(cx, |editor, cx| {
            editor.buffer().update(cx, |buffer, cx| {
                let (ids, _) = buffer.update_path_excerpts(
                    path_key.clone(),
                    base_text_buffer.clone(),
                    &base_text_buffer_snapshot,
                    new,
                    cx,
                );
                if let Some(diff) = diff
                    && !ids.is_empty()
                    && buffer
                        .diff_for(base_text_buffer.read(cx).remote_id())
                        .is_none_or(|old_diff| old_diff.entity_id() != diff.entity_id())
                {
                    buffer.add_inverted_diff(diff, main_buffer, cx);
                }
            })
        });

        let secondary_excerpt_ids: Vec<ExcerptId> = self.multibuffer.read(cx).excerpts_for_path(&path_key).collect();

        for (primary_id, secondary_id) in primary_excerpt_ids.into_iter().zip(secondary_excerpt_ids) {
            self.primary_to_secondary.insert(primary_id, secondary_id);
            self.secondary_to_primary.insert(secondary_id, primary_id);
        }
        self.synced_paths.insert(path_key, key);
    }

    fn remove_mappings_for_path(&mut self, path_key: &PathKey, cx: &App) {
        self.synced_paths.remove(path_key);
        let secondary_excerpt_ids: Vec<ExcerptId> = self.multibuffer.read(cx).excerpts_for_path(path_key).collect();

        for secondary_id in secondary_excerpt_ids {
            if let Some(primary_id) = self.secondary_to_primary.remove(&secondary_id) {
                self.primary_to_secondary.remove(&primary_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use fs::FakeFs;
    use settings::SettingsStore;
    use ui::VisualContext as _;

    fn init_test(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let store = SettingsStore::test(cx);
            cx.set_global(store);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::init(cx);
        });
    }

    #[gpui::test]
    async fn test_split_singleton_shows_base_text(cx: &mut gpui::TestAppContext) {
        use super::*;
        init_test(cx);
        let project = Project::test(FakeFs::new(cx.executor()), [], cx).await;
        let (workspace, cx) = cx.add_window_view(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let buffer = cx.new(|cx| Buffer::local("", cx));
        let diff =
            cx.new(|cx| BufferDiff::new_with_base_text("removed\nlines\n", &buffer.read(cx).text_snapshot(), cx));
        let primary = cx.new(|cx| {
            let mut primary = MultiBuffer::singleton(buffer.clone(), cx);
            primary.add_diff(diff.clone(), cx);
            primary
        });
        let split = cx.new_window_entity(|window, cx| {
            let mut split = SplittableEditor::new_unsplit(primary, project, workspace, window, cx);
            split.split(&SplitDiff, window, cx);
            split
        });
        split.update(cx, |split, cx| {
            let secondary = split.secondary.as_ref().unwrap();
            assert!(secondary.editor.read(cx).read_only(cx));
            assert_eq!(secondary.multibuffer.read(cx).snapshot(cx).text(), "removed\nlines\n");
            assert_eq!(split.primary_multibuffer.read(cx).snapshot(cx).text(), "");
            assert_eq!(secondary.primary_to_secondary.len(), 1);
        });
    }
}
