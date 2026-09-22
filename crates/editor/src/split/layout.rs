//! Layout coordination for the two editors.

use super::*;

impl SplittableEditor {
    pub(super) fn clear_alignment(&mut self, _cx: &mut Context<Self>) {
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
}
