//! Vertical D-pad target choice shared by the production runtime and the
//! fixture shell (spec 060, R1).
//!
//! Rows are entered at their leftmost item. When the source spans more than
//! one item of the nearest row (a full-width card above Pistol start),
//! focus lands on the row's leftmost item. Only a source that lines up with
//! exactly one item keeps its column. A source that overlaps nothing falls back
//! to the nearest centre.

use crate::ViewNode;
use crate::input::Action;

fn overlaps_horizontally(a: &ViewNode, b: &ViewNode) -> bool {
    b.x < a.x + a.w && b.x + b.w > a.x
}

fn same_row(a: &ViewNode, b: &ViewNode) -> bool {
    (a.y - b.y).abs() <= 1. || ((a.y + a.h / 2.) - (b.y + b.h / 2.)).abs() <= 1.
}

/// Node focus moves to for Up/Down from `current`, if any row lies that way.
pub fn vertical_target(nodes: &[ViewNode], current: &ViewNode, direction: Action) -> Option<i32> {
    let (cx, cy) = (current.x + current.w / 2., current.y + current.h / 2.);
    let candidates: Vec<(&ViewNode, f32, f32)> = nodes
        .iter()
        .filter(|n| n.actionable && n.id != current.id)
        .filter_map(|n| {
            let (dx, dy) = (n.x + n.w / 2. - cx, n.y + n.h / 2. - cy);
            let (along, separate_row) = match direction {
                Action::Up => (-dy, n.y + n.h <= current.y + 1.),
                Action::Down => (dy, n.y >= current.y + current.h - 1.),
                _ => return None,
            };
            (along > 1. && separate_row).then_some((n, along, dx.abs()))
        })
        .collect();
    let nearest_overlapping = candidates
        .iter()
        .filter(|(n, ..)| overlaps_horizontally(current, n))
        .min_by(|a, b| a.1.total_cmp(&b.1));
    if let Some(&(anchor, ..)) = nearest_overlapping {
        let row: Vec<_> = candidates
            .iter()
            .filter(|(n, ..)| same_row(anchor, n))
            .collect();
        let overlaps: Vec<_> = row
            .iter()
            .filter(|(n, ..)| overlaps_horizontally(current, n))
            .collect();
        if overlaps.len() == 1 {
            return Some(overlaps[0].0.id);
        }
        return row
            .iter()
            .min_by(|a, b| a.0.x.total_cmp(&b.0.x))
            .map(|(n, ..)| n.id);
    }
    candidates
        .iter()
        .min_by(|a, b| (a.1 * 100. + a.2).total_cmp(&(b.1 * 100. + b.2)))
        .map(|(n, ..)| n.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: i32, x: f32, y: f32, w: f32) -> ViewNode {
        ViewNode {
            id,
            x,
            y,
            w,
            h: 52.,
            actionable: true,
            ..Default::default()
        }
    }

    #[test]
    fn wider_sources_enter_the_whole_row_at_the_left_edge() {
        let row = [
            node(1, 0., 80., 100.),
            node(2, 120., 80., 100.),
            node(3, 240., 80., 100.),
        ];
        for y in [0., 160.] {
            let direction = if y == 0. { Action::Down } else { Action::Up };
            assert_eq!(
                vertical_target(&row, &node(0, 120., y, 220.), direction),
                Some(1)
            );
            for target in &row {
                assert_eq!(
                    vertical_target(&row, &node(0, target.x, y, target.w), direction),
                    Some(target.id)
                );
            }
        }
    }

    #[test]
    fn disabled_left_item_is_skipped_on_row_entry() {
        let mut row = [
            node(1, 0., 80., 100.),
            node(2, 120., 80., 100.),
            node(3, 240., 80., 100.),
        ];
        row[0].actionable = false;
        assert_eq!(
            vertical_target(&row, &node(0, 0., 0., 340.), Action::Down),
            Some(2)
        );
    }
}
