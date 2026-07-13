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
}
