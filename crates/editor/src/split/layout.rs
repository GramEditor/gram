//! Layout coordination for the two editors.

use super::*;

fn render_padding(height: u32) -> crate::display_map::RenderBlock {
    Arc::new(move |cx| {
        let mut color = cx.editor_style.text.color;
        color.a *= 0.15;
        div()
            .w_full()
            // The editor measures the rendered element and resizes its block.
            // An Empty element collapses even a multi-line block to one row.
            .h(cx.line_height * height as f32)
            .bg(gpui::pattern_slash(color, 1.0, 5.0))
            .into_any_element()
    })
}

type Checkpoint = ([GapKey; 2], [u32; 2]);

/// Row offsets from an excerpt's first text row, excluding alignment padding.
/// Stores the first pair and each change in row delta. Moving the excerpt does
/// not invalidate the cache.  Folds, inlays and other custom blocks bypass it.
pub(super) struct CachedGeometry {
    model: ModelKey,
    settings: [GeometrySettings; 2],
    checkpoints: Vec<Checkpoint>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct GeometrySettings {
    wrap_revision: u64,
    tab_size: u32,
    max_tab_column: u32,
}

impl SplittableEditor {
    fn sync_folds(&mut self, source: DiffSide, snapshots: &[DisplaySnapshot; 2], cx: &mut Context<Self>) {
        let source_index = source.index();
        let target_index = 1 - source_index;
        let secondary = self.secondary.as_ref().unwrap();
        let source_snapshot = &snapshots[source_index];
        let target_snapshot = &snapshots[target_index];
        let map_anchor = |anchor: Anchor| -> Option<Anchor> {
            let new_id = if source_index == 0 {
                anchor.excerpt_id
            } else {
                *secondary.secondary_to_primary.get(&anchor.excerpt_id)?
            };
            let (key, pairs) = self.alignment.models.get(&new_id)?;
            let target_id = if target_index == 0 { new_id } else { key.old_excerpt };
            let source_buffer = source_snapshot
                .buffer_snapshot()
                .buffer_for_excerpt(anchor.excerpt_id)?;
            let target_buffer = target_snapshot.buffer_snapshot().buffer_for_excerpt(target_id)?;
            let point = anchor.text_anchor.to_point(source_buffer);
            let index = pairs
                .partition_point(|pair| pair[source_index] < point.row)
                .min(pairs.len().checked_sub(1)?);
            let row = pairs[index][target_index].min(target_buffer.max_point().row);
            let point = target_buffer.clip_point(
                Point::new(row, point.column.min(target_buffer.line_len(row))),
                Bias::Left,
            );
            Some(Anchor::in_buffer(target_id, target_buffer.anchor_before(point)))
        };
        let creases = source_snapshot
            .folds_in_range(Anchor::min()..Anchor::max())
            .filter_map(|fold| {
                let range = map_anchor(fold.range.start)?..map_anchor(fold.range.end)?;
                Some(crate::display_map::Crease::simple(range, fold.placeholder.clone()))
            })
            .collect();
        let target = if target_index == 0 {
            self.primary_editor.clone()
        } else {
            secondary.editor.clone()
        };
        target.update(cx, |editor, cx| {
            editor.display_map.update(cx, |map, cx| {
                map.unfold_intersecting([Anchor::min()..Anchor::max()], true, cx);
                map.fold(creases, cx);
            });
            cx.notify();
        });
    }

    pub(super) fn clear_alignment(&mut self, cx: &mut Context<Self>) {
        let editors = [
            Some(self.primary_editor.clone()),
            self.secondary.as_ref().map(|s| s.editor.clone()),
        ];
        for (side, editor) in editors.into_iter().enumerate() {
            let ids = self.alignment.blocks[side]
                .drain()
                .map(|(_, (id, _))| id)
                .collect::<HashSet<_>>();
            if let Some(editor) = editor
                && !ids.is_empty()
            {
                editor.update(cx, |editor, cx| editor.remove_blocks(ids, None, cx));
            }
        }
        self.alignment = AlignmentState::default();
    }

    pub(super) fn update_split_layout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.secondary.is_none() {
            return;
        }
        if std::mem::take(&mut self.alignment.refresh_excerpts) {
            self.sync_excerpts(cx);
        }
        let editors = [
            self.primary_editor.clone(),
            self.secondary.as_ref().unwrap().editor.clone(),
        ];
        if self.alignment.dirty {
            let Some(snapshots) = Self::ready_snapshots(&editors, cx) else {
                return;
            };
            self.alignment.dirty = false;
            let Some(desired) = self.alignment_gaps(&snapshots, cx) else {
                return;
            };
            if let Some(source) = self.alignment.fold_source.take() {
                self.sync_folds(source, &snapshots, cx);
                self.alignment.dirty = true;
                cx.notify();
                return;
            }
            self.apply_padding(&editors, desired, cx);
            self.alignment
                .scroll_source
                .get_or_insert(if self.secondary.as_ref().unwrap().has_latest_selection {
                    DiffSide::Old
                } else {
                    DiffSide::New
                });
        }
        if let Some(source) = self.alignment.scroll_source.take() {
            let source = source.index();
            let y = editors[source].update(cx, |editor, cx| editor.scroll_position(cx).y);
            let mut animation_state = editors[source].read(cx).scroll_animation();
            editors[1 - source].update(cx, |editor, cx| {
                let mut position = editor.scroll_position(cx);
                position.y = y;
                editor.set_scroll_position_internal(position, false, false, window, cx);
                if let Some(animation) = &mut animation_state {
                    animation.current.x = position.x;
                    animation.target.x = position.x;
                    editor.set_scroll_animation(*animation, false);
                };
            });
        }
    }

    fn sync_excerpts(&mut self, cx: &mut Context<Self>) {
        let secondary = self.secondary.as_mut().unwrap();
        self.primary_multibuffer.update(cx, |primary, cx| {
            let paths: HashSet<_> = primary.paths().cloned().collect();
            let removed: Vec<_> = secondary
                .multibuffer
                .read(cx)
                .paths()
                .filter(|p| !paths.contains(*p))
                .cloned()
                .collect();
            for path in removed {
                secondary.remove_mappings_for_path(&path, cx);
                secondary
                    .multibuffer
                    .update(cx, |buffer, cx| buffer.remove_excerpts_for_path(path, cx));
            }
            for path in paths {
                let Some(buffer) = primary.buffer_for_path(&path, cx) else {
                    continue;
                };
                let diff = primary.diff_for(buffer.read(cx).remote_id());
                secondary.sync_path_excerpts(path, primary, diff, cx);
            }
        });
        let primary_snapshot = self.primary_multibuffer.read(cx).snapshot(cx);
        let old_snapshot = secondary.multibuffer.read(cx).snapshot(cx);
        for (new_id, old_id) in &secondary.primary_to_secondary {
            let Some(new_buffer) = primary_snapshot.buffer_for_excerpt(*new_id) else {
                continue;
            };
            let Some(old_buffer) = old_snapshot.buffer_for_excerpt(*old_id) else {
                continue;
            };
            let primary = self.primary_editor.read(cx);
            let hide_header = primary
                .display_map
                .read(cx)
                .header_disabled_for_buffer(new_buffer.remote_id());
            let hide_guides = primary
                .buffers_with_disabled_indent_guides
                .contains(&new_buffer.remote_id());
            let folded = primary.is_buffer_folded(new_buffer.remote_id(), cx);
            secondary.editor.update(cx, |editor, cx| {
                let id = old_buffer.remote_id();
                if hide_header && !editor.display_map.read(cx).header_disabled_for_buffer(id) {
                    editor.disable_header_for_buffer(id, cx);
                }
                if hide_guides && !editor.buffers_with_disabled_indent_guides.contains(&id) {
                    editor.disable_indent_guides_for_buffer(id, cx);
                }
                if folded && !editor.is_buffer_folded(id, cx) {
                    editor.fold_buffer(id, cx);
                }
            });
        }
    }

    fn ready_snapshots(editors: &[Entity<Editor>; 2], cx: &mut Context<Self>) -> Option<[DisplaySnapshot; 2]> {
        // Taking a snapshot can itself start wrapping. Check both before and after
        // so we never combine provisional rows with a completed layout.
        let is_rewrapping = |cx: &App| {
            editors
                .iter()
                .any(|editor| editor.read(cx).display_map.read(cx).is_rewrapping(cx))
        };
        if is_rewrapping(cx) {
            return None;
        }
        let snapshots = editors
            .each_ref()
            .map(|editor| editor.update(cx, |editor, cx| editor.display_snapshot(cx)));
        (!is_rewrapping(cx)).then_some(snapshots)
    }

    fn apply_padding(
        &mut self,
        editors: &[Entity<Editor>; 2],
        desired: [HashMap<GapKey, u32>; 2],
        cx: &mut Context<Self>,
    ) {
        for side in 0..2 {
            let current = &mut self.alignment.blocks[side];
            let mut removed = HashSet::default();
            current.retain(|key, (id, _)| {
                if desired[side].contains_key(key) {
                    true
                } else {
                    removed.insert(*id);
                    false
                }
            });
            let mut resized = HashMap::default();
            let mut added = Vec::new();
            for (&key, &height) in &desired[side] {
                if let Some((id, old_height)) = current.get_mut(&key) {
                    if *old_height != height {
                        resized.insert(*id, height);
                        *old_height = height;
                    }
                } else {
                    added.push((key, height));
                }
            }
            editors[side].update(cx, |editor, cx| {
                if !removed.is_empty() {
                    editor.remove_blocks(removed, None, cx);
                }
                if !resized.is_empty() {
                    editor.replace_blocks(
                        resized
                            .iter()
                            .map(|(&id, &height)| (id, render_padding(height)))
                            .collect(),
                        None,
                        cx,
                    );
                    editor.resize_blocks(resized, None, cx);
                }
                if !added.is_empty() {
                    let ids = editor.insert_blocks(
                        added.iter().map(|(key, height)| BlockProperties {
                            placement: if key.below {
                                BlockPlacement::Below(key.anchor)
                            } else {
                                BlockPlacement::Above(key.anchor)
                            },
                            height: Some(*height),
                            style: BlockStyle::Sticky,
                            render: render_padding(*height),
                            priority: usize::MAX,
                        }),
                        None,
                        cx,
                    );
                    for ((key, height), id) in added.into_iter().zip(ids) {
                        current.insert(key, (id, height));
                    }
                }
            });
        }
    }

    fn alignment_gaps(
        &mut self,
        snapshots: &[DisplaySnapshot; 2],
        cx: &mut Context<Self>,
    ) -> Option<[HashMap<GapKey, u32>; 2]> {
        let secondary = self.secondary.as_ref().unwrap();
        let own_ids = self
            .alignment
            .blocks
            .each_ref()
            .map(|blocks| blocks.values().map(|(id, _)| *id).collect::<HashSet<_>>());
        let editors = [self.primary_editor.clone(), secondary.editor.clone()];
        let mut cacheable = snapshots
            .iter()
            .all(|snapshot| snapshot.folds_in_range(Anchor::min()..Anchor::max()).next().is_none())
            && editors
                .iter()
                .all(|editor| editor.read(cx).display_map.read(cx).current_inlays().next().is_none());
        let settings = editors
            .each_ref()
            .map(|editor| editor.read(cx).display_map.read(cx).wrap_settings_version(cx));
        let settings = [0, 1].map(|side| GeometrySettings {
            wrap_revision: settings[side],
            tab_size: snapshots[side].tab_snapshot().tab_size.get(),
            max_tab_column: snapshots[side].tab_snapshot().max_expansion_column,
        });
        let mut prefixes: [Vec<(u32, u32)>; 2] = Default::default();
        let mut headers: [HashMap<ExcerptId, u32>; 2] = Default::default();
        for side in 0..2 {
            let mut total = 0;
            for (row, block) in
                snapshots[side].blocks_in_range(DisplayRow(0)..DisplayRow(snapshots[side].max_point().row().0 + 1))
            {
                match block.id() {
                    BlockId::Custom(id) if own_ids[side].contains(&id) => {
                        total += block.height();
                        prefixes[side].push((row.0 + block.height(), total));
                    }
                    BlockId::ExcerptBoundary(id) => {
                        headers[side].insert(id, row.0);
                    }
                    BlockId::FoldedBuffer(id) => {
                        cacheable = false;
                        headers[side].insert(id, row.0);
                    }
                    BlockId::Custom(_) => cacheable = false,
                }
            }
        }
        if !cacheable {
            self.alignment.geometry.clear();
        }
        let intrinsic = |side: usize, row: u32| {
            let index = prefixes[side].partition_point(|(end, _)| *end <= row);
            row.saturating_sub(index.checked_sub(1).map_or(0, |i| prefixes[side][i].1))
        };
        let old_excerpts = snapshots[1]
            .buffer_snapshot()
            .excerpts()
            .map(|(id, b, r)| (id, (b, r)))
            .collect::<HashMap<_, _>>();
        let mut checkpoints = Vec::<Checkpoint>::new();
        let mut jobs = Vec::new();
        let mut live_ids = HashSet::default();
        let mut previous_end: Option<[GapKey; 2]> = None;
        for (new_id, new_buffer, new_range) in snapshots[0].buffer_snapshot().excerpts() {
            let Some(old_id) = secondary.primary_to_secondary.get(&new_id) else {
                continue;
            };
            let Some((old_buffer, old_range)) = old_excerpts.get(old_id) else {
                continue;
            };
            let ranges = [
                new_range.context.to_point(new_buffer),
                old_range.context.to_point(old_buffer),
            ];
            let buffers = [new_buffer, *old_buffer];
            let ids = [new_id, *old_id];
            let key_at = |side: usize, row: u32| {
                if row > ranges[side].end.row {
                    GapKey {
                        anchor: Anchor::in_buffer(ids[side], buffers[side].anchor_before(ranges[side].end)),
                        below: true,
                    }
                } else {
                    GapKey {
                        anchor: Anchor::in_buffer(ids[side], buffers[side].anchor_before(Point::new(row, 0))),
                        below: false,
                    }
                }
            };
            if let Some(keys) = previous_end {
                let rows = [0, 1].map(|side| {
                    intrinsic(
                        side,
                        headers[side].get(&ids[side]).copied().unwrap_or_else(|| {
                            key_at(side, ranges[side].start.row)
                                .anchor
                                .to_display_point(&snapshots[side])
                                .row()
                                .0
                        }),
                    )
                });
                checkpoints.push((keys, rows));
            }
            live_ids.insert(new_id);
            let key = ModelKey {
                diff_revision: snapshots[0]
                    .buffer_snapshot()
                    .diff_for_buffer_id(new_buffer.remote_id())
                    .map(|d| d.revision()),
                versions: [new_buffer.version().clone(), old_buffer.version().clone()],
                buffers: [new_buffer.remote_id(), old_buffer.remote_id()],
                ranges: ranges.clone(),
                old_excerpt: *old_id,
            };
            let Some((_, pairs)) = self.alignment.models.get(&new_id).filter(|(cached, _)| *cached == key) else {
                jobs.push((
                    new_id,
                    key,
                    new_buffer.clone(),
                    snapshots[0]
                        .buffer_snapshot()
                        .diff_for_buffer_id(new_buffer.remote_id())
                        .cloned(),
                ));
                continue;
            };
            let origins = [0, 1].map(|side| {
                intrinsic(
                    side,
                    key_at(side, ranges[side].start.row)
                        .anchor
                        .to_display_point(&snapshots[side])
                        .row()
                        .0,
                )
            });
            let cached = self
                .alignment
                .geometry
                .get(&new_id)
                .filter(|cached| cached.model == key && cached.settings == settings);
            if let Some(cached) = cached {
                checkpoints.extend(
                    cached
                        .checkpoints
                        .iter()
                        .map(|(keys, rows)| (*keys, [rows[0] + origins[0], rows[1] + origins[1]])),
                );
            } else {
                #[cfg(test)]
                {
                    *self.alignment.geometry_builds.entry(new_id).or_default() += 1;
                }
                let mut measured = Vec::new();
                for &pair in pairs {
                    let keys = [key_at(0, pair[0]), key_at(1, pair[1])];
                    let rows = [0, 1].map(|side| {
                        let key = keys[side];
                        let point = if key.below {
                            ranges[side].end
                        } else {
                            Point::new(pair[side], 0)
                        };
                        let anchor = Anchor::in_buffer(ids[side], buffers[side].anchor_before(point));
                        intrinsic(
                            side,
                            anchor.to_display_point(&snapshots[side]).row().0 + u32::from(key.below),
                        )
                    });
                    let relative = [0, 1].map(|side| rows[side] - origins[side]);
                    let delta = i64::from(relative[0]) - i64::from(relative[1]);
                    if measured.last().is_none_or(|(_, previous): &Checkpoint| {
                        i64::from(previous[0]) - i64::from(previous[1]) != delta
                    }) {
                        measured.push((keys, relative));
                    }
                }
                checkpoints.extend(
                    measured
                        .iter()
                        .map(|(keys, rows)| (*keys, [rows[0] + origins[0], rows[1] + origins[1]])),
                );
                if cacheable {
                    self.alignment.geometry.insert(
                        new_id,
                        CachedGeometry {
                            model: key,
                            settings,
                            checkpoints: measured,
                        },
                    );
                }
            }
            previous_end = Some([key_at(0, ranges[0].end.row + 1), key_at(1, ranges[1].end.row + 1)]);
        }
        self.alignment.models.retain(|id, _| live_ids.contains(id));
        self.alignment.geometry.retain(|id, _| live_ids.contains(id));
        if !jobs.is_empty() {
            self.alignment.generation += 1;
            let generation = self.alignment.generation;
            let task = cx.background_spawn(async move {
                let mut changes_by_buffer = HashMap::default();
                jobs.into_iter()
                    .map(|(id, key, buffer, diff)| {
                        let changes = changes_by_buffer.entry(buffer.remote_id()).or_insert_with(|| {
                            diff.map(|diff| {
                                diff.hunks(&buffer)
                                    .map(|hunk| {
                                        let start = hunk.diff_base_byte_range.start.to_point(diff.base_text());
                                        let end = hunk.diff_base_byte_range.end.to_point(diff.base_text());
                                        alignment::Change {
                                            new: hunk.range.start.row
                                                ..hunk.range.end.row + u32::from(hunk.range.end.column > 0),
                                            old: start.row..end.row + u32::from(end.column > 0),
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default()
                        });
                        let rows = alignment::boundaries(
                            key.ranges[0].start.row..key.ranges[0].end.row + 1,
                            key.ranges[1].start.row..key.ranges[1].end.row + 1,
                            &changes,
                        );
                        (id, key, rows)
                    })
                    .collect::<Vec<_>>()
            });
            self.alignment.model_task = Some(cx.spawn(async move |this, cx| {
                let models = task.await;
                this.update(cx, |this, cx| {
                    if this.alignment.generation != generation || this.secondary.is_none() {
                        return;
                    }
                    for (id, key, rows) in models {
                        #[cfg(test)]
                        {
                            *this.alignment.model_builds.entry(id).or_default() += 1;
                        }
                        this.alignment.models.insert(id, (key, rows));
                    }
                    this.alignment.dirty = true;
                    cx.notify();
                })
                .ok();
            }));
            return None;
        }
        let mut result: [HashMap<GapKey, u32>; 2] = Default::default();
        for ((keys, _), gaps) in checkpoints
            .iter()
            .zip(alignment::padding(checkpoints.iter().map(|(_, rows)| *rows)))
        {
            for side in 0..2 {
                if gaps[side] > 0 {
                    *result[side].entry(keys[side]).or_default() += gaps[side];
                }
            }
        }
        Some(result)
    }
}
