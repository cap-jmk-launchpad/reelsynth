//! Shared waveform sampling for WT strip and views.

use egui::{Pos2, Rect};

pub fn frame_index(position: f32, num_frames: usize) -> usize {
    if num_frames == 0 {
        return 0;
    }
    let max = num_frames.saturating_sub(1);
    (position.round() as usize).min(max)
}

pub fn waveform_points(frame: &[f32], inner: Rect, sample_count: usize, amp: f32) -> Vec<Pos2> {
    if frame.is_empty() || sample_count < 2 {
        return Vec::new();
    }
    let mid_y = inner.center().y;
    let step = (frame.len() / sample_count).max(1);
    let denom = sample_count.saturating_sub(1).max(1) as f32;
    frame
        .iter()
        .step_by(step)
        .take(sample_count)
        .enumerate()
        .map(|(i, sample)| {
            let t = i as f32 / denom;
            let x = egui::lerp(inner.min.x..=inner.max.x, t);
            let y = mid_y - sample * inner.height() * amp;
            Pos2::new(x, y)
        })
        .collect()
}

pub fn peak_point(points: &[Pos2]) -> Option<Pos2> {
    points
        .iter()
        .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
        .copied()
}

/// Minimum pixel distance from `pos` to the waveform polyline.
pub fn nearest_waveform_distance(points: &[Pos2], pos: Pos2) -> f32 {
    if points.len() < 2 {
        return f32::INFINITY;
    }
    points
        .windows(2)
        .map(|seg| distance_point_to_segment(pos, seg[0], seg[1]))
        .fold(f32::INFINITY, f32::min)
}

/// True when `pos` is within `tolerance` px of the drawn waveform path.
pub fn hit_test_waveform(points: &[Pos2], pos: Pos2, tolerance: f32) -> bool {
    nearest_waveform_distance(points, pos) <= tolerance
}

/// Layer index whose polyline is nearest to `pos` within `max_dist` px.
///
/// Same nearest-wins rule as click-to-select on Design WT layer curves.
pub fn hovered_layer_from_pointer<'a>(
    layer_points: impl IntoIterator<Item = (usize, &'a [Pos2])>,
    pos: Pos2,
    max_dist: f32,
) -> Option<usize> {
    let mut best_idx = None;
    let mut best_dist = max_dist;
    for (idx, pts) in layer_points {
        let dist = nearest_waveform_distance(pts, pos);
        if dist < best_dist {
            best_dist = dist;
            best_idx = Some(idx);
        }
    }
    best_idx
}

fn distance_point_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len_sq = ab.x * ab.x + ab.y * ab.y;
    if len_sq <= f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let closest = Pos2::new(a.x + ab.x * t, a.y + ab.y * t);
    p.distance(closest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Rect;

    #[test]
    fn frame_index_clamps() {
        assert_eq!(frame_index(300.0, 64), 63);
        assert_eq!(frame_index(0.0, 0), 0);
    }

    #[test]
    fn waveform_points_bounds() {
        let frame: Vec<f32> = (0..256).map(|i| (i as f32 * 0.1).sin()).collect();
        let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(200.0, 80.0));
        let pts = waveform_points(&frame, rect, 32, 0.45);
        assert_eq!(pts.len(), 32);
        for p in &pts {
            assert!(p.x >= rect.min.x && p.x <= rect.max.x);
            assert!(p.y >= rect.min.y && p.y <= rect.max.y);
        }
    }

    #[test]
    fn peak_point_finds_minimum_y() {
        let pts = vec![
            Pos2::new(0.0, 10.0),
            Pos2::new(1.0, 5.0),
            Pos2::new(2.0, 12.0),
        ];
        assert_eq!(peak_point(&pts).unwrap().y, 5.0);
    }

    #[test]
    fn hit_test_waveform_near_line() {
        let pts = vec![
            Pos2::new(0.0, 50.0),
            Pos2::new(100.0, 50.0),
        ];
        assert!(hit_test_waveform(&pts, Pos2::new(50.0, 50.0), 8.0));
        assert!(hit_test_waveform(&pts, Pos2::new(50.0, 55.0), 8.0));
        assert!(!hit_test_waveform(&pts, Pos2::new(50.0, 70.0), 8.0));
    }

    #[test]
    fn hovered_layer_from_pointer_picks_nearest_within_tolerance() {
        let a = vec![Pos2::new(0.0, 40.0), Pos2::new(100.0, 40.0)];
        let b = vec![Pos2::new(0.0, 60.0), Pos2::new(100.0, 60.0)];
        let layers = [(0usize, a.as_slice()), (1usize, b.as_slice())];

        assert_eq!(
            hovered_layer_from_pointer(layers, Pos2::new(50.0, 42.0), 14.0),
            Some(0)
        );
        assert_eq!(
            hovered_layer_from_pointer(layers, Pos2::new(50.0, 58.0), 14.0),
            Some(1)
        );
        assert_eq!(
            hovered_layer_from_pointer(layers, Pos2::new(50.0, 100.0), 14.0),
            None
        );
    }

    #[test]
    fn hovered_layer_from_pointer_nearest_wins_on_tie_break_distance() {
        let far = vec![Pos2::new(0.0, 20.0), Pos2::new(100.0, 20.0)];
        let near = vec![Pos2::new(0.0, 50.0), Pos2::new(100.0, 50.0)];
        let layers = [(3usize, far.as_slice()), (7usize, near.as_slice())];
        assert_eq!(
            hovered_layer_from_pointer(layers, Pos2::new(50.0, 52.0), 14.0),
            Some(7)
        );
    }
}
