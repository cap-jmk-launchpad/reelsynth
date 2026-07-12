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
    points.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap())
}
