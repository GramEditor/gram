//! Maps row boundaries between the old and new buffers independently of layout.
//! Insertions repeat a boundary in the old buffer. deletions repeats one in the new buffer.

use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Change {
    pub new: Range<u32>,
    pub old: Range<u32>,
}

/// Emit monotone pairs of row boundaries, including the exclusive end boundary.
/// Clipping happens after pairing, so an excerpt starting inside a replacement
/// retains the replacement's original row correspondence.
pub(super) fn boundaries(new: Range<u32>, old: Range<u32>, changes: &[Change]) -> Vec<[u32; 2]> {
    let mut result = Vec::new();
    let first = changes.partition_point(|change| change.new.end < new.start && change.old.end < old.start);
    let mut cursor = first
        .checked_sub(1)
        .map_or([0, 0], |i| [changes[i].new.end, changes[i].old.end]);
    let mut append = |start: [u32; 2], end: [u32; 2]| {
        let count = (end[0] - start[0]).max(end[1] - start[1]);
        // Skip rows before the excerpt
        let first_visible = |side: usize, range: &Range<u32>| {
            if end[side] <= range.start || start[side] >= range.end {
                count
            } else {
                range.start.saturating_sub(start[side])
            }
        };
        let from = first_visible(0, &new).min(first_visible(1, &old));
        let to = count.min(new.end.saturating_sub(start[0]).max(old.end.saturating_sub(start[1])));
        for offset in from..=to {
            let pair = [
                (start[0] + offset.min(end[0] - start[0])).clamp(new.start, new.end),
                (start[1] + offset.min(end[1] - start[1])).clamp(old.start, old.end),
            ];
            if result.last() != Some(&pair) {
                result.push(pair);
            }
        }
    };
    for change in &changes[first..] {
        if change.new.start > new.end && change.old.start > old.end {
            break;
        }
        append(cursor, [change.new.start, change.old.start]);
        append([change.new.start, change.old.start], [change.new.end, change.old.end]);
        cursor = [change.new.end, change.old.end];
    }
    if cursor[0] <= new.end && cursor[1] <= old.end {
        append(cursor, [new.end, old.end]);
    }
    result
}

/// Padding needed at each boundary. Input rows exclude this alignment's own
/// padding, but include wrapping, headers and other display blocks.
pub(super) fn padding(rows: impl IntoIterator<Item = [u32; 2]>) -> Vec<[u32; 2]> {
    let mut totals = [0, 0];
    rows.into_iter()
        .map(|row| {
            let positions = [row[0] + totals[0], row[1] + totals[1]];
            let target = positions[0].max(positions[1]);
            let added = [target - positions[0], target - positions[1]];
            totals[0] += added[0];
            totals[1] += added[1];
            added
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insertion_and_following_context() {
        assert_eq!(
            boundaries(0..5, 0..3, &[Change { new: 1..3, old: 1..1 }]),
            vec![[0, 0], [1, 1], [2, 1], [3, 1], [4, 2], [5, 3]]
        );
    }

    #[test]
    fn test_clipped_uneven_replacement() {
        assert_eq!(
            boundaries(3..7, 2..5, &[Change { new: 1..5, old: 1..3 }]),
            vec![[3, 2], [3, 3], [4, 3], [5, 3], [6, 4], [7, 5]]
        );
    }

    #[test]
    fn test_deletion_at_end() {
        assert_eq!(
            boundaries(0..1, 0..4, &[Change { new: 1..1, old: 1..4 }]),
            vec![[0, 0], [1, 1], [1, 2], [1, 3], [1, 4]]
        );
    }

    #[test]
    fn test_wraps_and_headers_do_not_accumulate_drift() {
        let mut rows = Vec::new();
        let mut ends = [0, 0];
        for _ in 0..1000 {
            for height in [[2, 2], [3, 1], [0, 4], [2, 1], [1, 1]] {
                ends[0] += height[0];
                ends[1] += height[1];
                rows.push(ends);
            }
        }
        let gaps = padding(rows.iter().copied());
        let mut total = [0, 0];
        for (row, gap) in rows.into_iter().zip(gaps) {
            total[0] += gap[0];
            total[1] += gap[1];
            assert_eq!(row[0] + total[0], row[1] + total[1]);
        }
    }

    #[test]
    fn test_ten_thousand_changes_with_separate_excerpts() {
        let changes = (0..10_000)
            .map(|i| Change {
                new: i * 4 + 1..i * 4 + 3,
                old: i * 3 + 1..i * 3 + 2,
            })
            .collect::<Vec<_>>();
        for i in 0..10_000 {
            let pairs = boundaries(i * 4..i * 4 + 4, i * 3..i * 3 + 3, &changes);
            assert_eq!(pairs.first(), Some(&[i * 4, i * 3]));
            assert_eq!(pairs.last(), Some(&[i * 4 + 4, i * 3 + 3]));
            assert!(pairs.windows(2).all(|p| p[0][0] <= p[1][0] && p[0][1] <= p[1][1]));
            assert!(pairs.len() <= 7);
        }
    }

    #[test]
    fn test_deep_excerpt_inside_large_insertion() {
        let pairs = boundaries(
            999_990..1_000_000,
            0..1,
            &[Change {
                new: 0..1_000_000,
                old: 0..0,
            }],
        );
        assert_eq!(pairs.first(), Some(&[999_990, 0]));
        assert_eq!(pairs.last(), Some(&[1_000_000, 1]));
        assert_eq!(pairs.len(), 12);
    }
}
