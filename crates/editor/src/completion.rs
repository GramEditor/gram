use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use fuzzy::StringMatch;
use gpui::{AppContext, Entity, Task};
use language::{Buffer, CharScopeContext};
use lsp::CompletionContext;
use multi_buffer::ExcerptId;
use project::{Completion, CompletionResponse, Project};
use ui::{App, Context, Window};

use crate::Editor;

pub trait CompletionProvider {
    fn completions(
        &self,
        excerpt_id: ExcerptId,
        buffer: &Entity<Buffer>,
        buffer_position: text::Anchor,
        trigger: CompletionContext,
        window: &mut Window,
        cx: &mut Context<Editor>,
    ) -> Task<Result<Vec<CompletionResponse>>>;

    fn resolve_completions(
        &self,
        _buffer: Entity<Buffer>,
        _completion_indices: Vec<usize>,
        _completions: Rc<RefCell<Box<[Completion]>>>,
        _cx: &mut Context<Editor>,
    ) -> Task<Result<bool>> {
        Task::ready(Ok(false))
    }

    fn apply_additional_edits_for_completion(
        &self,
        _buffer: Entity<Buffer>,
        _completions: Rc<RefCell<Box<[Completion]>>>,
        _completion_index: usize,
        _push_to_history: bool,
        _cx: &mut Context<Editor>,
    ) -> Task<Result<Option<language::Transaction>>> {
        Task::ready(Ok(None))
    }

    fn is_completion_trigger(
        &self,
        buffer: &Entity<Buffer>,
        position: language::Anchor,
        text: &str,
        trigger_in_words: bool,
        cx: &mut Context<Editor>,
    ) -> bool;

    fn selection_changed(&self, _mat: Option<&StringMatch>, _window: &mut Window, _cx: &mut App) {}

    fn sort_completions(&self) -> bool {
        true
    }

    fn filter_completions(&self) -> bool {
        true
    }

    fn show_snippets(&self) -> bool {
        false
    }
}

impl CompletionProvider for Entity<Project> {
    fn completions(
        &self,
        _excerpt_id: ExcerptId,
        buffer: &Entity<Buffer>,
        buffer_position: text::Anchor,
        options: CompletionContext,
        _window: &mut Window,
        cx: &mut Context<Editor>,
    ) -> Task<Result<Vec<CompletionResponse>>> {
        self.update(cx, |project, cx| {
            let task = project.completions(buffer, buffer_position, options, cx);
            cx.background_spawn(task)
        })
    }

    fn resolve_completions(
        &self,
        buffer: Entity<Buffer>,
        completion_indices: Vec<usize>,
        completions: Rc<RefCell<Box<[Completion]>>>,
        cx: &mut Context<Editor>,
    ) -> Task<Result<bool>> {
        self.update(cx, |project, cx| {
            project.lsp_store().update(cx, |lsp_store, cx| {
                lsp_store.resolve_completions(buffer, completion_indices, completions, cx)
            })
        })
    }

    fn apply_additional_edits_for_completion(
        &self,
        buffer: Entity<Buffer>,
        completions: Rc<RefCell<Box<[Completion]>>>,
        completion_index: usize,
        push_to_history: bool,
        cx: &mut Context<Editor>,
    ) -> Task<Result<Option<language::Transaction>>> {
        self.update(cx, |project, cx| {
            project.lsp_store().update(cx, |lsp_store, cx| {
                lsp_store.apply_additional_edits_for_completion(
                    buffer,
                    completions,
                    completion_index,
                    push_to_history,
                    cx,
                )
            })
        })
    }

    fn is_completion_trigger(
        &self,
        buffer: &Entity<Buffer>,
        position: language::Anchor,
        text: &str,
        trigger_in_words: bool,
        cx: &mut Context<Editor>,
    ) -> bool {
        let mut chars = text.chars();
        let char = if let Some(char) = chars.next() {
            char
        } else {
            return false;
        };
        if chars.next().is_some() {
            return false;
        }

        let buffer = buffer.read(cx);
        let snapshot = buffer.snapshot();
        let classifier = snapshot
            .char_classifier_at(position)
            .scope_context(Some(CharScopeContext::Completion));
        if trigger_in_words && classifier.is_word(char) {
            return true;
        }

        buffer.completion_triggers().contains(text)
    }

    fn show_snippets(&self) -> bool {
        true
    }
}
