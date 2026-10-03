//! Handles conversions of `language` items to and from the [`rpc`] protocol.

use crate::{CursorShape, Diagnostic, DiagnosticSourceKind, diagnostic_set::DiagnosticEntry};
use anyhow::{Context as _, Result};
use clock::ReplicaId;
use gpui::SharedString;
use lsp::{DiagnosticSeverity, LanguageServerId};
use rpc::proto;
use serde_json::Value;
use std::{ops::Range, str::FromStr, sync::Arc};
use text::*;

pub use proto::{BufferState, File, Operation};

use super::{point_from_lsp, point_to_lsp};

/// Deserializes a `[text::LineEnding]` from the RPC representation.
pub fn deserialize_line_ending(message: proto::LineEnding) -> text::LineEnding {
    match message {
        proto::LineEnding::Unix => text::LineEnding::Unix,
        proto::LineEnding::Windows => text::LineEnding::Windows,
    }
}

/// Serializes a [`text::LineEnding`] to be sent over RPC.
pub fn serialize_line_ending(message: text::LineEnding) -> proto::LineEnding {
    match message {
        text::LineEnding::Unix => proto::LineEnding::Unix,
        text::LineEnding::Windows => proto::LineEnding::Windows,
    }
}

/// Serializes a [`crate::Operation`] to be sent over RPC.
pub fn serialize_operation(operation: &crate::Operation) -> proto::Operation {
    match operation {
        crate::Operation::Buffer(text::Operation::Edit(edit)) => {
            proto::Operation::Edit(serialize_edit_operation(edit))
        }

        crate::Operation::Buffer(text::Operation::Undo(undo)) => {
            proto::Operation::Undo(proto::OperationUndo {
                replica_id: undo.timestamp.replica_id.as_u16() as u32,
                lamport_timestamp: undo.timestamp.value,
                version: serialize_version(&undo.version),
                counts: undo
                    .counts
                    .iter()
                    .map(|(edit_id, count)| proto::UndoCount {
                        replica_id: edit_id.replica_id.as_u16() as u32,
                        lamport_timestamp: edit_id.value,
                        count: *count,
                    })
                    .collect(),
            })
        }

        crate::Operation::UpdateSelections {
            selections,
            line_mode,
            lamport_timestamp,
            cursor_shape,
        } => proto::Operation::UpdateSelections(proto::OperationUpdateSelections {
            replica_id: lamport_timestamp.replica_id.as_u16() as u32,
            lamport_timestamp: lamport_timestamp.value,
            selections: serialize_selections(selections),
            line_mode: *line_mode,
            cursor_shape: serialize_cursor_shape(cursor_shape),
        }),

        crate::Operation::UpdateDiagnostics {
            lamport_timestamp,
            server_id,
            diagnostics,
        } => proto::Operation::UpdateDiagnostics(proto::UpdateDiagnostics {
            replica_id: lamport_timestamp.replica_id.as_u16() as u32,
            lamport_timestamp: lamport_timestamp.value,
            server_id: server_id.0 as u64,
            diagnostics: serialize_diagnostics(diagnostics.iter()),
        }),

        crate::Operation::UpdateCompletionTriggers {
            triggers,
            lamport_timestamp,
            server_id,
        } => proto::Operation::UpdateCompletionTriggers(proto::OperationUpdateCompletionTriggers {
            replica_id: lamport_timestamp.replica_id.as_u16() as u32,
            lamport_timestamp: lamport_timestamp.value,
            triggers: triggers.clone(),
            language_server_id: server_id.to_proto(),
        }),

        crate::Operation::UpdateLineEnding {
            line_ending,
            lamport_timestamp,
        } => proto::Operation::UpdateLineEnding(proto::OperationUpdateLineEnding {
            replica_id: lamport_timestamp.replica_id.as_u16() as u32,
            lamport_timestamp: lamport_timestamp.value,
            line_ending: serialize_line_ending(*line_ending),
        }),
    }
}

/// Serializes an [`EditOperation`] to be sent over RPC.
pub fn serialize_edit_operation(operation: &EditOperation) -> proto::OperationEdit {
    proto::OperationEdit {
        replica_id: operation.timestamp.replica_id.as_u16() as u32,
        lamport_timestamp: operation.timestamp.value,
        version: serialize_version(&operation.version),
        ranges: operation.ranges.iter().map(serialize_range).collect(),
        new_text: operation.new_text.iter().map(|text| text.to_string()).collect(),
    }
}

/// Serializes an entry in the undo map to be sent over RPC.
pub fn serialize_undo_map_entry((edit_id, counts): (&clock::Lamport, &[(clock::Lamport, u32)])) -> proto::UndoMapEntry {
    proto::UndoMapEntry {
        replica_id: edit_id.replica_id.as_u16() as u32,
        local_timestamp: edit_id.value,
        counts: counts
            .iter()
            .map(|(undo_id, count)| proto::UndoCount {
                replica_id: undo_id.replica_id.as_u16() as u32,
                lamport_timestamp: undo_id.value,
                count: *count,
            })
            .collect(),
    }
}

/// Splits the given list of operations into chunks.
pub fn split_operations(mut operations: Vec<proto::Operation>) -> impl Iterator<Item = Vec<proto::Operation>> {
    #[cfg(any(test, feature = "test-support"))]
    const CHUNK_SIZE: usize = 5;

    #[cfg(not(any(test, feature = "test-support")))]
    const CHUNK_SIZE: usize = 100;

    let mut done = false;
    std::iter::from_fn(move || {
        if done {
            return None;
        }

        let operations = operations
            .drain(..std::cmp::min(CHUNK_SIZE, operations.len()))
            .collect::<Vec<_>>();
        if operations.is_empty() {
            done = true;
        }
        Some(operations)
    })
}

/// Serializes selections to be sent over RPC.
pub fn serialize_selections(selections: &Arc<[Selection<Anchor>]>) -> Vec<proto::Selection> {
    selections.iter().map(serialize_selection).collect()
}

/// Serializes a [`Selection`] to be sent over RPC.
pub fn serialize_selection(selection: &Selection<Anchor>) -> proto::Selection {
    proto::Selection {
        id: selection.id as u64,
        start: proto::EditorAnchor {
            anchor: serialize_anchor(&selection.start),
            excerpt_id: 0,
        },
        end: proto::EditorAnchor {
            anchor: serialize_anchor(&selection.end),
            excerpt_id: 0,
        },
        reversed: selection.reversed,
    }
}

/// Serializes a [`CursorShape`] to be sent over RPC.
pub fn serialize_cursor_shape(cursor_shape: &CursorShape) -> proto::CursorShape {
    match cursor_shape {
        CursorShape::Bar => proto::CursorShape::CursorBar,
        CursorShape::Block => proto::CursorShape::CursorBlock,
        CursorShape::Underline => proto::CursorShape::CursorUnderscore,
        CursorShape::Hollow => proto::CursorShape::CursorHollow,
    }
}

/// Deserializes a [`CursorShape`] from the RPC representation.
pub fn deserialize_cursor_shape(cursor_shape: proto::CursorShape) -> CursorShape {
    match cursor_shape {
        proto::CursorShape::CursorBar => CursorShape::Bar,
        proto::CursorShape::CursorBlock => CursorShape::Block,
        proto::CursorShape::CursorUnderscore => CursorShape::Underline,
        proto::CursorShape::CursorHollow => CursorShape::Hollow,
    }
}

/// Serializes a list of diagnostics to be sent over RPC.
pub fn serialize_diagnostics<'a>(
    diagnostics: impl IntoIterator<Item = &'a DiagnosticEntry<Anchor>>,
) -> Vec<proto::Diagnostic> {
    diagnostics
        .into_iter()
        .map(|entry| proto::Diagnostic {
            source: entry.diagnostic.source.clone(),
            source_kind: match entry.diagnostic.source_kind {
                DiagnosticSourceKind::Pulled => proto::DiagnosticSourceKind::Pulled,
                DiagnosticSourceKind::Pushed => proto::DiagnosticSourceKind::Pushed,
                DiagnosticSourceKind::Other => proto::DiagnosticSourceKind::Other,
            },
            start: serialize_anchor(&entry.range.start),
            end: serialize_anchor(&entry.range.end),
            message: entry.diagnostic.message.clone(),
            markdown: entry.diagnostic.markdown.clone(),
            severity: match entry.diagnostic.severity {
                DiagnosticSeverity::ERROR => proto::DiagnosticSeverity::Error,
                DiagnosticSeverity::WARNING => proto::DiagnosticSeverity::Warning,
                DiagnosticSeverity::INFORMATION => proto::DiagnosticSeverity::Information,
                DiagnosticSeverity::HINT => proto::DiagnosticSeverity::Hint,
                _ => proto::DiagnosticSeverity::None,
            },
            group_id: entry.diagnostic.group_id as u64,
            is_primary: entry.diagnostic.is_primary,
            underline: entry.diagnostic.underline,
            code: entry.diagnostic.code.as_ref().map(|s| s.to_string()),
            code_description: entry.diagnostic.code_description.as_ref().map(|s| s.to_string()),
            is_disk_based: entry.diagnostic.is_disk_based,
            is_unnecessary: entry.diagnostic.is_unnecessary,
            data: entry.diagnostic.data.as_ref().map(|data| data.to_string()),
            registration_id: entry.diagnostic.registration_id.as_ref().map(ToString::to_string),
        })
        .collect()
}

/// Serializes an [`Anchor`] to be sent over RPC.
pub fn serialize_anchor(anchor: &Anchor) -> proto::Anchor {
    proto::Anchor {
        replica_id: anchor.timestamp.replica_id.as_u16() as u32,
        timestamp: anchor.timestamp.value,
        offset: anchor.offset as u64,
        bias: match anchor.bias {
            Bias::Left => proto::Bias::Left,
            Bias::Right => proto::Bias::Right,
        },
        buffer_id: anchor.buffer_id.map(Into::into),
    }
}

pub fn serialize_anchor_range(range: Range<Anchor>) -> proto::AnchorRange {
    proto::AnchorRange {
        start: serialize_anchor(&range.start),
        end: serialize_anchor(&range.end),
    }
}

/// Deserializes an [`Range<Anchor>`] from the RPC representation.
pub fn deserialize_anchor_range(range: proto::AnchorRange) -> Result<Range<Anchor>> {
    Ok(
        deserialize_anchor(range.start)
            ..deserialize_anchor(range.end),
    )
}

/// Deserializes an [`crate::Operation`] from the RPC representation.
pub fn deserialize_operation(message: proto::Operation) -> Result<crate::Operation> {
    Ok(match message {
        proto::Operation::Edit(edit) => {
            crate::Operation::Buffer(text::Operation::Edit(deserialize_edit_operation(edit)))
        }
        proto::Operation::Undo(undo) => crate::Operation::Buffer(text::Operation::Undo(UndoOperation {
            timestamp: clock::Lamport {
                replica_id: ReplicaId::new(undo.replica_id as u16),
                value: undo.lamport_timestamp,
            },
            version: deserialize_version(&undo.version),
            counts: undo
                .counts
                .into_iter()
                .map(|c| {
                    (
                        clock::Lamport {
                            replica_id: ReplicaId::new(c.replica_id as u16),
                            value: c.lamport_timestamp,
                        },
                        c.count,
                    )
                })
                .collect(),
        })),
        proto::Operation::UpdateSelections(message) => {
            let selections = message
                .selections
                .into_iter()
                .map(|selection| {
                    Selection {
                        id: selection.id as usize,
                        start: deserialize_anchor(selection.start.anchor),
                        end: deserialize_anchor(selection.end.anchor),
                        reversed: selection.reversed,
                        goal: SelectionGoal::None,
                    }
                })
                .collect::<Vec<_>>();

            crate::Operation::UpdateSelections {
                lamport_timestamp: clock::Lamport {
                    replica_id: ReplicaId::new(message.replica_id as u16),
                    value: message.lamport_timestamp,
                },
                selections: Arc::from(selections),
                line_mode: message.line_mode,
                cursor_shape: deserialize_cursor_shape(
                    proto::CursorShape::try_from(message.cursor_shape).context("Missing cursor shape")?,
                ),
            }
        }
        proto::Operation::UpdateDiagnostics(message) => crate::Operation::UpdateDiagnostics {
            lamport_timestamp: clock::Lamport {
                replica_id: ReplicaId::new(message.replica_id as u16),
                value: message.lamport_timestamp,
            },
            server_id: LanguageServerId(message.server_id as usize),
            diagnostics: deserialize_diagnostics(message.diagnostics),
        },
        proto::Operation::UpdateCompletionTriggers(message) => crate::Operation::UpdateCompletionTriggers {
            triggers: message.triggers,
            lamport_timestamp: clock::Lamport {
                replica_id: ReplicaId::new(message.replica_id as u16),
                value: message.lamport_timestamp,
            },
            server_id: LanguageServerId::from_proto(message.language_server_id),
        },
        proto::Operation::UpdateLineEnding(message) => crate::Operation::UpdateLineEnding {
            lamport_timestamp: clock::Lamport {
                replica_id: ReplicaId::new(message.replica_id as u16),
                value: message.lamport_timestamp,
            },
            line_ending: deserialize_line_ending(
                proto::LineEnding::try_from(message.line_ending).context("missing line_ending")?,
            ),
        },
    })
}

/// Deserializes an [`EditOperation`] from the RPC representation.
pub fn deserialize_edit_operation(edit: proto::OperationEdit) -> EditOperation {
    EditOperation {
        timestamp: clock::Lamport {
            replica_id: ReplicaId::new(edit.replica_id as u16),
            value: edit.lamport_timestamp,
        },
        version: deserialize_version(&edit.version),
        ranges: edit.ranges.into_iter().map(deserialize_range).collect(),
        new_text: edit.new_text.into_iter().map(Arc::from).collect(),
    }
}

/// Deserializes an entry in the undo map from the RPC representation.
pub fn deserialize_undo_map_entry(entry: proto::UndoMapEntry) -> (clock::Lamport, Vec<(clock::Lamport, u32)>) {
    (
        clock::Lamport {
            replica_id: ReplicaId::new(entry.replica_id as u16),
            value: entry.local_timestamp,
        },
        entry
            .counts
            .into_iter()
            .map(|undo_count| {
                (
                    clock::Lamport {
                        replica_id: ReplicaId::new(undo_count.replica_id as u16),
                        value: undo_count.lamport_timestamp,
                    },
                    undo_count.count,
                )
            })
            .collect(),
    )
}

/// Deserializes selections from the RPC representation.
pub fn deserialize_selections(selections: Vec<proto::Selection>) -> Arc<[Selection<Anchor>]> {
    selections.into_iter().map(deserialize_selection).collect()
}

/// Deserializes a [`Selection`] from the RPC representation.
pub fn deserialize_selection(selection: proto::Selection) -> Selection<Anchor> {
    Selection {
        id: selection.id as usize,
        start: deserialize_anchor(selection.start.anchor),
        end: deserialize_anchor(selection.end.anchor),
        reversed: selection.reversed,
        goal: SelectionGoal::None,
    }
}

/// Deserializes a list of diagnostics from the RPC representation.
pub fn deserialize_diagnostics(diagnostics: Vec<proto::Diagnostic>) -> Arc<[DiagnosticEntry<Anchor>]> {
    diagnostics
        .into_iter()
        .filter_map(|diagnostic| {
            let data = if let Some(data) = diagnostic.data {
                Some(Value::from_str(&data).ok()?)
            } else {
                None
            };
            Some(DiagnosticEntry {
                range: deserialize_anchor(diagnostic.start)..deserialize_anchor(diagnostic.end),
                diagnostic: Diagnostic {
                    source: diagnostic.source,
                    severity: match proto::DiagnosticSeverity::try_from(diagnostic.severity).ok()? {
                        proto::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
                        proto::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
                        proto::DiagnosticSeverity::Information => DiagnosticSeverity::INFORMATION,
                        proto::DiagnosticSeverity::Hint => DiagnosticSeverity::HINT,
                        proto::DiagnosticSeverity::None => return None,
                    },
                    message: diagnostic.message,
                    markdown: diagnostic.markdown,
                    group_id: diagnostic.group_id as usize,
                    code: diagnostic.code.map(lsp::NumberOrString::from_string),
                    code_description: diagnostic.code_description.and_then(|s| lsp::Uri::from_str(&s).ok()),
                    is_primary: diagnostic.is_primary,
                    is_disk_based: diagnostic.is_disk_based,
                    is_unnecessary: diagnostic.is_unnecessary,
                    underline: diagnostic.underline,
                    registration_id: diagnostic.registration_id.map(SharedString::from),
                    source_kind: match proto::DiagnosticSourceKind::try_from(diagnostic.source_kind).ok()? {
                        proto::DiagnosticSourceKind::Pulled => DiagnosticSourceKind::Pulled,
                        proto::DiagnosticSourceKind::Pushed => DiagnosticSourceKind::Pushed,
                        proto::DiagnosticSourceKind::Other => DiagnosticSourceKind::Other,
                    },
                    data,
                },
            })
        })
        .collect()
}

/// Deserializes an [`Anchor`] from the RPC representation.
pub fn deserialize_anchor(anchor: proto::Anchor) -> Anchor {
    let buffer_id = match anchor.buffer_id {
        None => None,
        // The only "error" is if buffer ID is 0,
        // which we'll just treat as a missing buffer
        Some(id) => BufferId::new(id).ok()
    };
    Anchor {
        timestamp: clock::Lamport {
            replica_id: ReplicaId::new(anchor.replica_id as u16),
            value: anchor.timestamp,
        },
        offset: anchor.offset as usize,
        bias: match proto::Bias::from(anchor.bias) {
            proto::Bias::Left => Bias::Left,
            proto::Bias::Right => Bias::Right,
        },
        buffer_id,
    }
}

/// Returns a `[clock::Lamport`] timestamp for the given [`proto::Operation`].
pub fn lamport_timestamp_for_operation(operation: &proto::Operation) -> Option<clock::Lamport> {
    let (replica_id, value) = match operation {
        proto::Operation::Edit(op) => (op.replica_id, op.lamport_timestamp),
        proto::Operation::Undo(op) => (op.replica_id, op.lamport_timestamp),
        proto::Operation::UpdateDiagnostics(op) => (op.replica_id, op.lamport_timestamp),
        proto::Operation::UpdateSelections(op) => (op.replica_id, op.lamport_timestamp),
        proto::Operation::UpdateCompletionTriggers(op) => (op.replica_id, op.lamport_timestamp),
        proto::Operation::UpdateLineEnding(op) => (op.replica_id, op.lamport_timestamp),
    };

    Some(clock::Lamport {
        replica_id: ReplicaId::new(replica_id as u16),
        value,
    })
}

/// Serializes a [`Transaction`] to be sent over RPC.
pub fn serialize_transaction(transaction: &Transaction) -> proto::Transaction {
    proto::Transaction {
        id: serialize_timestamp(transaction.id),
        edit_ids: transaction.edit_ids.iter().copied().map(serialize_timestamp).collect(),
        start: serialize_version(&transaction.start),
    }
}

/// Deserializes a [`Transaction`] from the RPC representation.
pub fn deserialize_transaction(transaction: proto::Transaction) -> Result<Transaction> {
    Ok(Transaction {
        id: deserialize_timestamp(transaction.id),
        edit_ids: transaction.edit_ids.into_iter().map(deserialize_timestamp).collect(),
        start: deserialize_version(&transaction.start),
    })
}

/// Serializes a [`clock::Lamport`] timestamp to be sent over RPC.
pub fn serialize_timestamp(timestamp: clock::Lamport) -> proto::LamportTimestamp {
    proto::LamportTimestamp {
        replica_id: timestamp.replica_id.as_u16() as u32,
        value: timestamp.value,
    }
}

/// Deserializes a [`clock::Lamport`] timestamp from the RPC representation.
pub fn deserialize_timestamp(timestamp: proto::LamportTimestamp) -> clock::Lamport {
    clock::Lamport {
        replica_id: ReplicaId::new(timestamp.replica_id as u16),
        value: timestamp.value,
    }
}

/// Serializes a range of [`FullOffset`]s to be sent over RPC.
pub fn serialize_range(range: &Range<FullOffset>) -> proto::Range {
    proto::Range {
        start: range.start.0 as u64,
        end: range.end.0 as u64,
    }
}

/// Deserializes a range of [`FullOffset`]s from the RPC representation.
pub fn deserialize_range(range: proto::Range) -> Range<FullOffset> {
    FullOffset(range.start as usize)..FullOffset(range.end as usize)
}

/// Deserializes a clock version from the RPC representation.
pub fn deserialize_version(message: &[proto::VectorClockEntry]) -> clock::Global {
    let mut version = clock::Global::new();
    for entry in message {
        version.observe(clock::Lamport {
            replica_id: ReplicaId::new(entry.replica_id as u16),
            value: entry.timestamp,
        });
    }
    version
}

/// Serializes a clock version to be sent over RPC.
pub fn serialize_version(version: &clock::Global) -> Vec<proto::VectorClockEntry> {
    version
        .iter()
        .map(|entry| proto::VectorClockEntry {
            replica_id: entry.replica_id.as_u16() as u32,
            timestamp: entry.value,
        })
        .collect()
}

pub fn serialize_lsp_edit(edit: lsp::TextEdit) -> proto::TextEdit {
    let start = point_from_lsp(edit.range.start).0;
    let end = point_from_lsp(edit.range.end).0;
    proto::TextEdit {
        new_text: edit.new_text,
        lsp_range_start: proto::PointUtf16 {
            row: start.row,
            column: start.column,
        },
        lsp_range_end: proto::PointUtf16 {
            row: end.row,
            column: end.column,
        },
    }
}

pub fn deserialize_lsp_edit(edit: proto::TextEdit) -> lsp::TextEdit {
    let start = edit.lsp_range_start;
    let start = PointUtf16::new(start.row, start.column);
    let end = edit.lsp_range_end;
    let end = PointUtf16::new(end.row, end.column);
    lsp::TextEdit {
        range: lsp::Range {
            start: point_to_lsp(start),
            end: point_to_lsp(end),
        },
        new_text: edit.new_text,
    }
}
