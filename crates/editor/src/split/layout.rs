//! Layout coordination for the two editors.

use super::*;

#[cfg(test)]
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

impl SplittableEditor {
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

    pub(super) fn update_split_layout(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.secondary.is_none() {
            return;
        }
        if std::mem::take(&mut self.alignment.refresh_excerpts) {
            self.sync_excerpts(cx);
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

    #[cfg(test)]
    pub(super) fn apply_padding(
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
}
