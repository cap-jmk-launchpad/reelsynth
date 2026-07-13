//! Block-based realtime synthesizer engine (S0).

mod bank_set;
mod midi;
mod params;
mod voice_pool;
mod voice_rt;

pub use bank_set::BankSet;
pub use midi::{note_to_freq, MidiEvent};
pub use params::{EngineParams, Smoother};
pub use voice_pool::{VoicePool, MAX_VOICES};
pub use voice_rt::RtVoice;

use crate::fx::FxChain;
use crate::patch::Patch;
use crate::voice::render_note;
use crate::wavetable::WavetableBank;

/// Internal block size for voice summing (64–128 samples).
pub const BLOCK_SIZE: usize = 64;

/// Polyphonic wavetable synth engine with shared offline/realtime DSP.
pub struct SynthEngine {
    banks: BankSet,
    patch: Patch,
    pool: VoicePool,
    params: EngineParams,
    fx: FxChain,
    sample_rate: u32,
    global_time: f32,
}

impl SynthEngine {
    pub fn new(bank: WavetableBank, patch: Patch, sample_rate: u32) -> Self {
        let params = EngineParams::new(&patch, sample_rate as f32);
        let pool = VoicePool::new(&patch);
        let fx = FxChain::new(sample_rate);
        let banks = BankSet::from_primary(bank, &patch);
        Self {
            banks,
            patch,
            pool,
            params,
            fx,
            sample_rate,
            global_time: 0.0,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn patch(&self) -> &Patch {
        &self.patch
    }

    pub fn set_patch(&mut self, patch: Patch) {
        self.params.sync_from_patch(&patch);
        self.pool.reset_patch(&patch);
        self.fx.set_bypass(patch.fx_bypass.clone());
        self.banks = BankSet::from_primary(self.banks.primary().clone(), &patch);
        self.patch = patch;
    }

    pub fn bank(&self) -> &WavetableBank {
        self.banks.primary()
    }

    pub fn banks(&self) -> &[WavetableBank] {
        self.banks.banks()
    }

    /// Hot-swap wavetable bank and patch (preset load).
    pub fn load_preset(&mut self, bank: WavetableBank, patch: Patch) {
        self.banks = BankSet::from_primary(bank, &patch);
        self.set_patch(patch);
    }

    pub fn set_wt_position(&mut self, position: f32) {
        if let Some(osc) = self.patch.oscillators.get_mut(0) {
            osc.position = position.clamp(0.0, 255.0);
        }
    }

    pub fn set_filter_cutoff(&mut self, cutoff: f32) {
        self.patch.filter.cutoff = cutoff;
        self.params.filter_cutoff.set_target(cutoff);
    }

    pub fn set_filter_resonance(&mut self, resonance: f32) {
        self.patch.filter.resonance = resonance.clamp(0.0, 0.95);
    }

    pub fn set_filter_type(&mut self, filter_type: &str) {
        self.patch.filter.filter_type = filter_type.to_string();
    }

    pub fn set_filter_key_tracking(&mut self, key_tracking: f32) {
        self.patch.filter.key_tracking = key_tracking.clamp(0.0, 1.0);
    }

    pub fn set_filter_drive(&mut self, drive: f32) {
        self.patch.filter.drive = drive.clamp(0.0, 1.0);
    }

    pub fn set_filter2_cutoff(&mut self, cutoff: f32) {
        self.patch.filter2.cutoff = cutoff;
    }

    pub fn set_filter2_resonance(&mut self, resonance: f32) {
        self.patch.filter2.resonance = resonance.clamp(0.0, 0.95);
    }

    pub fn set_filter2_type(&mut self, filter_type: &str) {
        self.patch.filter2.filter_type = filter_type.to_string();
    }

    pub fn set_filter2_drive(&mut self, drive: f32) {
        self.patch.filter2.drive = drive.clamp(0.0, 1.0);
    }

    pub fn set_unison_stereo_spread(&mut self, spread: f32) {
        self.patch.unison_stereo_spread = spread.clamp(0.0, 1.0);
    }

    pub fn set_osc_type(&mut self, index: usize, osc_type: &str) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.osc_type = osc_type.to_string();
        }
    }

    pub fn set_osc_pulse_width(&mut self, index: usize, pw: f32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.pulse_width = pw.clamp(0.05, 0.95);
        }
    }

    pub fn set_osc_morph(&mut self, index: usize, a: f32, b: f32, amount: f32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.morph_a = a.clamp(0.0, 255.0);
            osc.morph_b = b.clamp(0.0, 255.0);
            osc.morph_amount = amount.clamp(0.0, 1.0);
            if amount > 0.0 {
                osc.position = osc.morph_a + (osc.morph_b - osc.morph_a) * amount;
            }
        }
    }

    pub fn set_osc_warp(&mut self, index: usize, mode: &str, amount: f32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.warp_mode = mode.to_string();
            osc.warp_amount = amount.clamp(0.0, 1.0);
        }
    }

    pub fn set_envelope(&mut self, envelope: crate::patch::Envelope) {
        self.patch.envelope = envelope;
    }

    pub fn set_filter_envelope(&mut self, envelope: crate::patch::Envelope) {
        self.patch.filter_envelope = envelope;
    }

    pub fn set_lfo_rate(&mut self, rate: f32) {
        self.patch.lfo.rate = rate.max(0.0);
    }

    pub fn set_lfo_depth(&mut self, depth: f32) {
        self.patch.lfo.depth = depth.clamp(0.0, 1.0);
    }

    pub fn set_osc_level(&mut self, index: usize, level: f32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.level = level.clamp(0.0, 1.0);
        }
    }

    pub fn set_osc_pan(&mut self, index: usize, pan: f32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.pan = pan.clamp(-1.0, 1.0);
        }
    }

    pub fn set_osc_detune(&mut self, index: usize, detune: f32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.detune = detune.clamp(-2400.0, 2400.0);
        }
    }

    pub fn set_osc_unison(&mut self, index: usize, unison: u32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.unison = unison.clamp(1, 8);
        }
    }

    pub fn set_osc_position(&mut self, index: usize, position: f32) {
        self.patch.ensure_oscillators(index + 1);
        if let Some(osc) = self.patch.oscillators.get_mut(index) {
            osc.position = position.clamp(0.0, 255.0);
        }
    }

    pub fn set_sub_level(&mut self, level: f32) {
        self.patch.sub_level = level.clamp(0.0, 1.0);
    }

    pub fn set_noise_level(&mut self, level: f32) {
        self.patch.noise_level = level.clamp(0.0, 1.0);
    }

    pub fn set_mod_matrix(&mut self, slots: Vec<crate::patch::ModSlot>) {
        self.patch.mod_matrix = slots;
    }

    pub fn set_fx_bypass(&mut self, bypass: crate::fx::FxBypass) {
        self.patch.fx_bypass = bypass.clone();
        self.fx.set_bypass(bypass);
    }

    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let freq = note_to_freq(note);
        self.pool
            .note_on(&self.patch, note, freq, velocity, self.global_time);
    }

    pub fn note_off(&mut self, note: u8) {
        self.pool.note_off(note);
    }

    pub fn handle_event(&mut self, event: MidiEvent) {
        match event {
            MidiEvent::NoteOn { note, velocity } => self.note_on(note, velocity),
            MidiEvent::NoteOff { note } => self.note_off(note),
        }
    }

    /// Render one block of mono audio into `out` (L+R average).
    pub fn process(&mut self, out: &mut [f32]) {
        for chunk in out.chunks_mut(BLOCK_SIZE) {
            self.process_block_mono(chunk);
        }
    }

    /// Render interleaved stereo `[L,R,L,R,…]`.
    pub fn process_stereo(&mut self, out: &mut [f32]) {
        let frames = out.len() / 2;
        for chunk_start in (0..frames).step_by(BLOCK_SIZE) {
            let chunk_frames = (frames - chunk_start).min(BLOCK_SIZE);
            self.process_block_stereo(&mut out[chunk_start * 2..(chunk_start + chunk_frames) * 2]);
        }
    }

    fn process_block_mono(&mut self, out: &mut [f32]) {
        let sr = self.sample_rate as f32;
        let dt = 1.0 / sr;
        let bank_slice = self.banks.banks().to_vec();

        for sample in out.iter_mut() {
            self.params.filter_cutoff.process();
            self.params.master_gain.process();
            let mut patch = self.patch.clone();
            patch.filter.cutoff = self.params.filter_cutoff.current();
            let bank_for_osc = |oi: usize| self.banks.bank_for_osc(&patch, oi);

            let mut acc_l = 0.0f32;
            let mut acc_r = 0.0f32;
            for voice in self.pool.voices_mut() {
                let [l, r] = voice.process_sample(
                    &bank_slice,
                    &bank_for_osc,
                    &patch,
                    self.global_time,
                    dt,
                    sr,
                );
                acc_l += l;
                acc_r += r;
            }
            let gain = self.params.master_gain.current();
            let mono = self.fx.process_sample((acc_l + acc_r) * 0.5 * gain);
            *sample = mono;
            self.global_time += dt;
        }
    }

    fn process_block_stereo(&mut self, out: &mut [f32]) {
        let sr = self.sample_rate as f32;
        let dt = 1.0 / sr;
        let frames = out.len() / 2;
        let bank_slice = self.banks.banks().to_vec();

        for frame in 0..frames {
            self.params.filter_cutoff.process();
            self.params.master_gain.process();
            let mut patch = self.patch.clone();
            patch.filter.cutoff = self.params.filter_cutoff.current();
            let bank_for_osc = |oi: usize| self.banks.bank_for_osc(&patch, oi);

            let mut acc_l = 0.0f32;
            let mut acc_r = 0.0f32;
            for voice in self.pool.voices_mut() {
                let [l, r] = voice.process_sample(
                    &bank_slice,
                    &bank_for_osc,
                    &patch,
                    self.global_time,
                    dt,
                    sr,
                );
                acc_l += l;
                acc_r += r;
            }
            let gain = self.params.master_gain.current();
            let [l, r] = self.fx.process_stereo(acc_l * gain, acc_r * gain);
            out[frame * 2] = l;
            out[frame * 2 + 1] = r;
            self.global_time += dt;
        }
    }

    /// Offline reference render using the same patch/bank (for golden tests).
    pub fn render_offline(&self, freq: f32, duration: f32) -> Vec<f32> {
        let bank_for_osc = |oi: usize| self.banks.bank_for_osc(&self.patch, oi);
        let mut audio = render_note(
            self.banks.banks(),
            bank_for_osc,
            freq,
            duration,
            self.sample_rate,
            &self.patch,
        );
        let mut fx = FxChain::new(self.sample_rate);
        fx.set_bypass(self.patch.fx_bypass.clone());
        for sample in audio.iter_mut() {
            *sample = fx.process_sample(*sample);
        }
        audio
    }
}
