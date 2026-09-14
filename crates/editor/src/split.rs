mod alignment;
mod layout;

use std::{ops::Range, sync::Arc};

use buffer_diff::BufferDiff;
use collections::{HashMap, HashSet};
use gpui::{Action, AppContext as _, Entity, EventEmitter, Focusable, NoAction, Subscription, WeakEntity};
use language::{Buffer, Capability};
use multi_buffer::{
    Anchor, AnchorRangeExt as _, ExcerptId, ExcerptRange, ExpandExcerptDirection, MultiBuffer, PathKey,
};
use project::Project;
use rope::Point;
use text::{Bias, OffsetRangeExt as _, ToPoint as _};
use ui::{
    App, Context, InteractiveElement as _, IntoElement as _, ParentElement as _, Render, Styled as _, Window, div,
};
use workspace::{ActivePaneDecorator, Item, ItemHandle, Pane, PaneGroup, SplitDirection, Workspace};

use crate::display_map::{
    BlockId, BlockPlacement, BlockProperties, BlockStyle, CustomBlockId, DisplayRow, DisplaySnapshot, ToDisplayPoint,
};
use crate::{Editor, EditorEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffSide {
    New,
    Old,
}

impl DiffSide {
    // Arrays use [new/right, old/left], not visual column order.
    fn index(self) -> usize {
        match self {
            Self::New => 0,
            Self::Old => 1,
        }
    }
}

/// Stable insertion point in one multibuffer; `below` also covers EOF padding.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GapKey {
    anchor: Anchor,
    below: bool,
}

/// Identity of the logical row mapping. All pairs below use absolute buffer rows.
#[derive(Clone, PartialEq, Eq)]
struct ModelKey {
    diff_revision: Option<u64>,
    versions: [clock::Global; 2],
    buffers: [text::BufferId; 2],
    ranges: [Range<Point>; 2],
    old_excerpt: ExcerptId,
}

#[derive(Default)]
struct AlignmentState {
    models: HashMap<ExcerptId, (ModelKey, Vec<[u32; 2]>)>,
    geometry: HashMap<ExcerptId, layout::CachedGeometry>,
    #[cfg(test)]
    geometry_builds: HashMap<ExcerptId, usize>,
    model_task: Option<gpui::Task<()>>,
    // Only the most recently requested background calculation may be installed.
    generation: u64,
    #[cfg(test)]
    model_builds: HashMap<ExcerptId, usize>,
    blocks: [HashMap<GapKey, (CustomBlockId, u32)>; 2],
    // GPUI defers notifications, so layout changes are reconciled from snapshots
    dirty: bool,
    refresh_excerpts: bool,
    scroll_source: Option<DiffSide>,
    fold_source: Option<DiffSide>,
}

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
    alignment: AlignmentState,
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

    /// Resolve selections against the authoritative diff, never against filler
    /// rows or the other editor's cursor. A missing/stale model disables actions.
    pub fn selected_new_ranges(&self, cx: &App) -> Option<Vec<Range<Anchor>>> {
        let editor = self.last_selected_editor().read(cx);
        let ranges = editor.selections.disjoint_anchor_ranges().collect();
        if self.secondary.as_ref().is_some_and(|s| s.has_latest_selection) {
            self.map_old_ranges(ranges, cx)
        } else {
            Some(ranges)
        }
    }

    fn map_old_ranges(&self, ranges: Vec<Range<Anchor>>, cx: &App) -> Option<Vec<Range<Anchor>>> {
        let secondary = self.secondary.as_ref()?;
        let old = secondary.multibuffer.read(cx).snapshot(cx);
        let new = self.primary_multibuffer.read(cx).snapshot(cx);
        if self.primary_editor.read(cx).read_only(cx) {
            return None;
        }
        let mut result = Vec::new();
        for (old_buffer, range, old_id) in old.ranges_to_buffer_ranges(ranges.into_iter()) {
            let new_id = *secondary.secondary_to_primary.get(&old_id)?;
            let new_buffer = new.buffer_for_excerpt(new_id)?;
            let diff = new.diff_for_buffer_id(new_buffer.remote_id())?;
            let (key, _) = self.alignment.models.get(&new_id)?;
            if key.versions != [new_buffer.version().clone(), old_buffer.version().clone()]
                || key.diff_revision != Some(diff.revision())
            {
                return None;
            }
            for hunk in diff.hunks_intersecting_base_text_range(range.start.0..range.end.0, new_buffer) {
                result.push(Anchor::range_in_buffer(new_id, hunk.buffer_range));
            }
        }
        result.sort_by(|a, b| a.start.cmp(&b.start, &new));
        result.dedup();
        Some(result)
    }

    pub(super) fn stage_old_ranges(
        &mut self,
        stage: Option<bool>,
        old_ranges: Vec<Range<Anchor>>,
        cx: &mut Context<Self>,
    ) {
        let Some(ranges) = self.map_old_ranges(old_ranges.clone(), cx) else {
            return;
        };
        if ranges.is_empty() {
            return;
        }
        let keys = ranges
            .iter()
            .filter_map(|range| {
                self.alignment
                    .models
                    .get(&range.start.excerpt_id)
                    .map(|(key, _)| (range.start.excerpt_id, key.clone()))
            })
            .collect::<Vec<_>>();
        let save = self
            .primary_editor
            .update(cx, |editor, cx| editor.save_buffers_for_ranges_if_needed(&ranges, cx));
        cx.spawn(async move |this, cx| {
            if let Err(error) = save.await {
                log::error!("saving split diff before staging: {error:#}");
                return;
            }
            this.update(cx, |this, cx| {
                if keys
                    .iter()
                    .any(|(id, key)| this.alignment.models.get(id).is_none_or(|(current, _)| current != key))
                {
                    return;
                }
                let Some(ranges) = this.map_old_ranges(old_ranges, cx) else {
                    return;
                };
                this.primary_editor.update(cx, |editor, cx| {
                    let snapshot = editor.buffer.read(cx).snapshot(cx);
                    let stage = stage.unwrap_or_else(|| editor.has_stageable_diff_hunks_in_ranges(&ranges, &snapshot));
                    let mut by_buffer = HashMap::<_, Vec<_>>::default();
                    for hunk in editor.diff_hunks_in_ranges(&ranges, &snapshot) {
                        by_buffer.entry(hunk.buffer_id).or_default().push(hunk);
                    }
                    for (buffer, hunks) in by_buffer {
                        editor.do_stage_or_unstage(stage, buffer, hunks.into_iter(), cx);
                    }
                });
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn restore_old_ranges(
        &mut self,
        ranges: Vec<Range<Anchor>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ranges) = self.map_old_ranges(ranges, cx) else {
            return;
        };
        if ranges.is_empty() {
            return;
        }
        self.primary_editor.update(cx, |editor, cx| {
            let snapshot = editor.buffer.read(cx).snapshot(cx);
            editor.restore_hunks_in_ranges(
                ranges.into_iter().map(|range| range.to_point(&snapshot)).collect(),
                window,
                cx,
            );
        });
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

    fn on_editor_event(&mut self, side: DiffSide, event: &EditorEvent, cx: &mut Context<Self>) {
        if self.secondary.is_none() {
            return;
        }
        match event {
            EditorEvent::FoldsChanged => {
                self.alignment.fold_source = Some(side);
                self.alignment.dirty = true;
                cx.notify();
            }
            EditorEvent::ScrollPositionChanged { local: true, .. } => {
                self.alignment.scroll_source = Some(side);
                cx.notify();
            }
            EditorEvent::BufferEdited
            | EditorEvent::ExcerptsEdited { .. }
            | EditorEvent::ExcerptsExpanded { .. }
            | EditorEvent::ExcerptsAdded { .. }
            | EditorEvent::ExcerptsRemoved { .. } => {
                self.alignment.refresh_excerpts = true;
                self.alignment.dirty = true;
                cx.notify();
            }
            EditorEvent::BufferFoldToggled { ids, folded } => {
                if let Some(secondary) = &self.secondary {
                    let (target, mappings) = match side {
                        DiffSide::New => (secondary.editor.clone(), &secondary.primary_to_secondary),
                        DiffSide::Old => (self.primary_editor.clone(), &secondary.secondary_to_primary),
                    };
                    let snapshot = target.read(cx).buffer().read(cx).snapshot(cx);
                    let buffers: HashSet<_> = ids
                        .iter()
                        .filter_map(|id| mappings.get(id))
                        .filter_map(|id| snapshot.buffer_for_excerpt(*id))
                        .map(|buffer| buffer.remote_id())
                        .collect();
                    target.update(cx, |editor, cx| {
                        for id in buffers {
                            if *folded {
                                editor.fold_buffer(id, cx);
                            } else {
                                editor.unfold_buffer(id, cx);
                            }
                        }
                    });
                    self.alignment.dirty = true;
                    cx.notify();
                }
            }
            _ => {}
        }
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
            cx.subscribe(&primary_editor, |this, _, event: &EditorEvent, cx| {
                this.on_editor_event(DiffSide::New, event, cx);
                match event {
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
                }
            }),
            cx.observe(&primary_editor.read(cx).display_map.clone(), |this, _, cx| {
                this.alignment.dirty = true;
                cx.notify();
            }),
            cx.observe(&primary_multibuffer, |this, _, cx| {
                this.alignment.refresh_excerpts = true;
                this.alignment.dirty = true;
                cx.notify();
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
            alignment: AlignmentState::default(),
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
        let delegate = cx.weak_entity();
        let secondary_editor = cx.new(|cx| {
            let mut editor = Editor::for_multibuffer(secondary_multibuffer.clone(), Some(project.clone()), window, cx);
            editor.number_deleted_lines = true;
            editor.set_read_only(true);
            editor.diff_action_delegate = Some(delegate);
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
            cx.subscribe(&secondary_editor, |this, _, event: &EditorEvent, cx| {
                this.on_editor_event(DiffSide::Old, event, cx);
                match event {
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
                }
            }),
            cx.observe(&secondary_editor.read(cx).display_map.clone(), |this, _, cx| {
                this.alignment.dirty = true;
                cx.notify();
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
        self.alignment.dirty = true;
        self.alignment.refresh_excerpts = true;
        self.alignment.scroll_source = Some(DiffSide::New);
        self.alignment.fold_source = Some(DiffSide::New);

        let primary_pane = self.panes.first_pane();
        self.panes
            .split(&primary_pane, &secondary_pane, SplitDirection::Left)
            .unwrap();
        cx.notify();
    }

    fn unsplit(&mut self, _: &UnsplitDiff, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_alignment(cx);
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
        self.update_split_layout(window, cx);
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

    use crate::SplittableEditor;

    fn init_test(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let store = SettingsStore::test(cx);
            cx.set_global(store);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::init(cx);
        });
    }

    fn settle_split(editor: &gpui::Entity<SplittableEditor>, cx: &mut gpui::VisualTestContext) {
        for _ in 0..8 {
            cx.run_until_parked();
            editor.update_in(cx, |editor, window, cx| editor.update_split_layout(window, cx));
        }
        cx.run_until_parked();
    }

    #[gpui::test]
    async fn test_split_padding_survives_render(cx: &mut gpui::TestAppContext) {
        use super::*;
        use gpui::{point, px, size};
        use util::rel_path::RelPath;
        init_test(cx);
        let project = Project::test(FakeFs::new(cx.executor()), [], cx).await;
        let (workspace, cx) = cx.add_window_view(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let primary = cx.new(|_| MultiBuffer::new(Capability::ReadWrite));
        let split = cx.new_window_entity(|window, cx| {
            let mut split = SplittableEditor::new_unsplit(primary, project, workspace, window, cx);
            split.split(&SplitDiff, window, cx);
            split
        });
        for index in 0..2 {
            let buffer = cx.new(|cx| Buffer::local("before\ninsert one\ninsert two\ninsert three\nafter\n", cx));
            let diff =
                cx.new(|cx| BufferDiff::new_with_base_text("before\nafter\n", &buffer.read(cx).text_snapshot(), cx));
            let path = PathKey::with_sort_prefix(index, RelPath::unix(&format!("{index}.txt")).unwrap().into());
            split.update(cx, |split, cx| {
                let end = buffer.read(cx).snapshot().max_point();
                split.set_excerpts_for_path(path, buffer, [Point::zero()..end], 0, diff, cx);
            });
        }
        for width in [1000., 500., 1000.] {
            for _ in 0..8 {
                cx.run_until_parked();
                cx.draw(point(px(0.), px(0.)), size(px(width), px(700.)), |_, _| split.clone());
            }
            cx.run_until_parked();
            split.update(cx, |split, cx| {
                let old = split.secondary.as_ref().unwrap();
                let snapshots = [split.primary_editor.clone(), old.editor.clone()]
                    .map(|editor| editor.update(cx, |editor, cx| editor.display_snapshot(cx)));
                assert!(split.alignment.blocks[1].values().any(|(_, height)| *height == 3));
                for side in 0..2 {
                    for (id, expected_height) in split.alignment.blocks[side].values() {
                        assert_eq!(
                            snapshots[side].block_for_id(BlockId::Custom(*id)).unwrap().height(),
                            *expected_height,
                            "render must preserve the requested padding height"
                        );
                    }
                }
                for (new_id, old_id) in &old.primary_to_secondary {
                    let ids = [*new_id, *old_id];
                    let headers = [0, 1].map(|side| {
                        snapshots[side]
                            .blocks_in_range(DisplayRow(0)..DisplayRow(snapshots[side].max_point().row().0 + 1))
                            .find(|(_, block)| block.id() == BlockId::ExcerptBoundary(ids[side]))
                            .unwrap()
                            .0
                    });
                    assert_eq!(
                        headers[0], headers[1],
                        "next file header must stay aligned after rendering"
                    );
                    let rows = [0, 1].map(|side| {
                        let buffer = snapshots[side].buffer_snapshot().buffer_for_excerpt(ids[side]).unwrap();
                        Anchor::in_buffer(ids[side], buffer.anchor_before(Point::new([4, 1][side], 0)))
                            .to_display_point(&snapshots[side])
                            .row()
                    });
                    assert_eq!(rows[0], rows[1], "context after the insertion must stay aligned");
                }
            });
        }
    }

    #[gpui::test]
    async fn test_split_layout_wraps_headers_and_scroll(cx: &mut gpui::TestAppContext) {
        use super::*;
        use gpui::px;
        use util::rel_path::RelPath;
        init_test(cx);
        let project = Project::test(FakeFs::new(cx.executor()), [], cx).await;
        let (workspace, cx) = cx.add_window_view(|window, cx| Workspace::test_new(project.clone(), window, cx));
        let primary = cx.new(|_| MultiBuffer::new(Capability::ReadWrite));
        let split = cx.new_window_entity(|window, cx| {
            let mut split = SplittableEditor::new_unsplit(primary.clone(), project, workspace, window, cx);
            split.split(&SplitDiff, window, cx);
            split
        });
        let mut paths = Vec::new();
        for index in 0..12 {
            let path = PathKey::with_sort_prefix(index, RelPath::unix(&format!("{index}.txt")).unwrap().into());
            let buffer = cx.new(|cx| {
                Buffer::local(
                    "unchanged long line for different wrap widths\ninsert one\ninsert two\nend\n",
                    cx,
                )
            });
            let diff = cx.new(|cx| {
                BufferDiff::new_with_base_text(
                    "unchanged long line for different wrap widths\nend\n",
                    &buffer.read(cx).text_snapshot(),
                    cx,
                )
            });
            split.update(cx, |split, cx| {
                let end = buffer.read(cx).snapshot().max_point();
                split.set_excerpts_for_path(path.clone(), buffer, [Point::zero()..end], 0, diff, cx);
            });
            paths.push(path);
        }
        split.update(cx, |split, cx| {
            split.primary_editor.update(cx, |editor, cx| {
                editor.set_wrap_width(Some(px(180.)), cx);
            });
            split.secondary.as_ref().unwrap().editor.update(cx, |editor, cx| {
                editor.set_wrap_width(Some(px(100.)), cx);
            });
        });
        settle_split(&split, cx);
        let before_builds = split.read_with(cx, |split, _| split.alignment.model_builds.clone());
        let before_geometry = split.read_with(cx, |split, _| split.alignment.geometry_builds.clone());
        split.update(cx, |split, _| split.alignment.dirty = true);
        settle_split(&split, cx);
        split.read_with(cx, |split, _| {
            assert_eq!(
                split.alignment.geometry_builds, before_geometry,
                "an unchanged snapshot must reuse geometry after deferred notifications"
            )
        });
        split.update(cx, |split, cx| {
            split.secondary.as_ref().unwrap().editor.update(cx, |editor, cx| {
                editor.set_wrap_width(Some(px(120.)), cx);
            });
        });
        settle_split(&split, cx);
        split.read_with(cx, |split, _| {
            for (id, count) in &before_geometry {
                assert!(
                    split.alignment.geometry_builds[id] > *count,
                    "changing wrap width must invalidate cached geometry"
                );
            }
        });
        split.update(cx, |split, cx| {
            split.secondary.as_ref().unwrap().editor.update(cx, |editor, cx| {
                editor.set_wrap_width(Some(px(100.)), cx);
            });
        });
        settle_split(&split, cx);
        let before_geometry = split.read_with(cx, |split, _| split.alignment.geometry_builds.clone());
        split.update(cx, |split, cx| {
            let buffer = split
                .primary_multibuffer
                .read(cx)
                .buffer_for_path(&paths[0], cx)
                .unwrap();
            buffer.update(cx, |buffer, cx| {
                buffer.edit([(0..0, "prefix ")], None, cx);
            });
            let snapshot = buffer.read(cx).text_snapshot();
            split
                .primary_multibuffer
                .read(cx)
                .diff_for(snapshot.remote_id())
                .unwrap()
                .update(cx, |diff, cx| {
                    diff.recalculate_diff_sync(&snapshot, cx);
                });
        });
        settle_split(&split, cx);
        split.read_with(cx, |split, cx| {
            for path in &paths[1..] {
                let id = split
                    .primary_multibuffer
                    .read(cx)
                    .excerpts_for_path(path)
                    .next()
                    .unwrap();
                assert_eq!(
                    split.alignment.model_builds[&id], before_builds[&id],
                    "unchanged file must keep its model"
                );
                assert_eq!(
                    split.alignment.geometry_builds[&id], before_geometry[&id],
                    "editing another file must not remeasure this excerpt"
                );
            }
        });
        let blocks = split.update(cx, |split, cx| {
            let old = split.secondary.as_ref().unwrap();
            let snapshots = [split.primary_editor.clone(), old.editor.clone()]
                .map(|e| e.update(cx, |e, cx| e.display_snapshot(cx)));
            for path in &paths {
                let new_id = split
                    .primary_multibuffer
                    .read(cx)
                    .excerpts_for_path(path)
                    .next()
                    .unwrap();
                let old_id = old.primary_to_secondary[&new_id];
                let headers = [0, 1].map(|side| {
                    let id = [new_id, old_id][side];
                    snapshots[side]
                        .blocks_in_range(DisplayRow(0)..DisplayRow(snapshots[side].max_point().row().0 + 1))
                        .find(|(_, block)| block.id() == BlockId::ExcerptBoundary(id))
                        .unwrap()
                        .0
                });
                assert_eq!(headers[0], headers[1], "file header: {path:?}");
                let new_buffer = snapshots[0].buffer_snapshot().buffer_for_excerpt(new_id).unwrap();
                let old_buffer = snapshots[1].buffer_snapshot().buffer_for_excerpt(old_id).unwrap();
                let rows = [
                    Anchor::in_buffer(new_id, new_buffer.anchor_before(Point::new(3, 0)))
                        .to_display_point(&snapshots[0])
                        .row(),
                    Anchor::in_buffer(old_id, old_buffer.anchor_before(Point::new(1, 0)))
                        .to_display_point(&snapshots[1])
                        .row(),
                ];
                assert_eq!(rows[0], rows[1], "context after insertion: {path:?}");
            }
            split.alignment.blocks.clone()
        });
        split.update_in(cx, |split, window, cx| {
            let old = split.secondary.as_ref().unwrap().editor.clone();
            let before = old.read(cx).selections.disjoint_anchors().to_vec();
            split.primary_editor.update(cx, |editor, cx| {
                let mut position = editor.scroll_position(cx);
                position.y = 37.5;
                editor.set_scroll_position(position, window, cx);
            });
            split.alignment.scroll_source = Some(DiffSide::New);
            split.update_split_layout(window, cx);
            let new_y = split.primary_editor.update(cx, |e, cx| e.scroll_position(cx).y);
            let old_y = old.update(cx, |e, cx| e.scroll_position(cx).y);
            assert_eq!(new_y, old_y);
            assert_eq!(before, old.read(cx).selections.disjoint_anchors().to_vec());
            assert!(
                split.alignment.blocks == blocks,
                "scroll must not replace alignment blocks"
            );
            split.unsplit(&UnsplitDiff, window, cx);
            assert!(split.alignment.blocks.iter().all(|blocks| blocks.is_empty()));
        });
    }

    #[gpui::test]
    async fn test_split_singleton_deletion_and_stale_action_mapping(cx: &mut gpui::TestAppContext) {
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
        settle_split(&split, cx);
        split.update(cx, |split, cx| {
            let secondary = split.secondary.as_ref().unwrap();
            assert!(secondary.editor.read(cx).read_only(cx));
            let old = secondary.multibuffer.read(cx).snapshot(cx);
            let selection = old.anchor_before(Point::new(0, 0));
            let mapped = split.map_old_ranges(vec![selection..selection], cx).unwrap();
            assert!(!mapped.is_empty(), "deleted-only hunk must be actionable");
            buffer.update(cx, |buffer, cx| {
                buffer.edit([(0..0, "new\n")], None, cx);
            });
            assert!(
                split.map_old_ranges(vec![selection..selection], cx).is_none(),
                "outdated correspondence must not stage/revert another hunk"
            );
        });
    }
}
