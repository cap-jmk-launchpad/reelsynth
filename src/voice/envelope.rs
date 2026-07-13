//! ADSR envelope advancement.

use crate::patch::Envelope;

pub(crate) fn advance_envelope(
    level: &mut f32,
    stage: &mut u8,
    time: &mut f32,
    env: &Envelope,
    gate: bool,
    dt: f32,
) -> f32 {
    if gate {
        match *stage {
            0 => {
                *time += dt;
                let a = env.attack.max(1e-4);
                *level = (*time / a).min(1.0);
                if *level >= 1.0 {
                    *stage = 1;
                    *time = 0.0;
                }
            }
            1 => {
                *time += dt;
                let d = env.decay.max(1e-4);
                let t = (*time / d).min(1.0);
                *level = 1.0 + t * (env.sustain - 1.0);
                if t >= 1.0 {
                    *stage = 2;
                }
            }
            2 => *level = env.sustain,
            3 => {
                *stage = 0;
                *time = 0.0;
            }
            _ => {}
        }
    } else if *stage != 3 {
        *stage = 3;
        *time = 0.0;
    } else {
        *time += dt;
        let r = env.release.max(1e-4);
        let t = (*time / r).min(1.0);
        *level *= 1.0 - t;
    }
    *level
}

