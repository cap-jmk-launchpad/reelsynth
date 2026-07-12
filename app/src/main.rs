//! ReelSynth S1 performance UI — matches `brand/mockups/s1-performance.html`.

mod midi_input;

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::egui;
use midi_input::{MidiDevices, MidiInputHandle};
use reelsynth::{load_preset, resolve_bank_for_preset, Patch, SynthEngine, WavetableBank};
use reelsynth_ui::{draw_s1, S1MidiDevices, S1ShellConfig, S1State};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

enum AudioCmd {
    NoteOn(u8, f32),
    NoteOff(u8),
    SetWtPosition(f32),
    SetFilterCutoff(f32),
    SetFilterResonance(f32),
    LoadPreset {
        patch: Patch,
        bank: WavetableBank,
    },
}

struct AudioHandle {
    tx: Sender<AudioCmd>,
    _stream: cpal::Stream,
    bank: Arc<RwLock<WavetableBank>>,
}

impl AudioHandle {
    fn send(&self, cmd: AudioCmd) {
        let _ = self.tx.send(cmd);
    }

    fn bank(&self) -> Arc<RwLock<WavetableBank>> {
        Arc::clone(&self.bank)
    }
}

fn start_audio(sample_rate: u32) -> Result<AudioHandle, String> {
    let bank = WavetableBank::factory_saw_morph();
    let patch = Patch::default_mono();
    let bank_shared = Arc::new(RwLock::new(bank.clone()));
    let mut engine = SynthEngine::new(bank, patch, sample_rate);

    let (tx, rx) = crossbeam_channel::unbounded::<AudioCmd>();
    let bank_for_audio = Arc::clone(&bank_shared);

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no audio output device".to_string())?;
    let config = device
        .default_output_config()
        .map_err(|e| e.to_string())?;
    let sr = config.sample_rate().0;
    if sr != sample_rate {
        engine = SynthEngine::new(WavetableBank::factory_saw_morph(), Patch::default_mono(), sr);
    }

    let mut engine = engine;
    let err_fn = |e| eprintln!("audio stream error: {e}");

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                drain_commands(&mut engine, &rx, &bank_for_audio);
                engine.process(data);
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_output_stream(
            &config.into(),
            move |data: &mut [i16], _| {
                drain_commands(&mut engine, &rx, &bank_for_audio);
                let mut buf = vec![0.0f32; data.len()];
                engine.process(&mut buf);
                for (out, sample) in data.iter_mut().zip(buf.iter()) {
                    *out = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                }
            },
            err_fn,
            None,
        ),
        other => return Err(format!("unsupported sample format: {other:?}")),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    Ok(AudioHandle {
        tx,
        _stream: stream,
        bank: bank_shared,
    })
}

fn drain_commands(
    engine: &mut SynthEngine,
    rx: &Receiver<AudioCmd>,
    bank_shared: &Arc<RwLock<WavetableBank>>,
) {
    loop {
        match rx.try_recv() {
            Ok(AudioCmd::NoteOn(n, v)) => engine.note_on(n, v),
            Ok(AudioCmd::NoteOff(n)) => engine.note_off(n),
            Ok(AudioCmd::SetWtPosition(p)) => engine.set_wt_position(p),
            Ok(AudioCmd::SetFilterCutoff(c)) => engine.set_filter_cutoff(c),
            Ok(AudioCmd::SetFilterResonance(r)) => engine.set_filter_resonance(r),
            Ok(AudioCmd::LoadPreset { patch, bank }) => {
                engine.load_preset(bank.clone(), patch);
                if let Ok(mut g) = bank_shared.write() {
                    *g = engine.bank().clone();
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
}

fn resolve_bank(path: &Path, preset: &Patch) -> Result<WavetableBank, String> {
    resolve_bank_for_preset(path, preset).or_else(|_| match preset.wavetable_id.as_deref() {
        Some("saw_morph") => Ok(WavetableBank::factory_saw_morph()),
        Some(id) => Err(format!("could not resolve wavetable for id {id}")),
        None => Ok(WavetableBank::factory_saw_morph()),
    })
}

fn sync_state_from_patch(state: &mut S1State, patch: &Patch) {
    state.preset_name = patch.name.clone();
    state.preset_category = preset_category_label(patch);
    state.wt_position = patch
        .oscillators
        .first()
        .map(|o| o.position)
        .unwrap_or(0.0);
    state.filter_cutoff = patch.filter.cutoff;
    state.filter_resonance = patch.filter.resonance;
}

fn preset_category_label(patch: &Patch) -> String {
    let wt = patch
        .wavetable_id
        .as_deref()
        .unwrap_or("wavetable")
        .replace('_', " ");
    format!("Preset · Wavetable · {wt}")
}

fn patch_from_state(state: &S1State, base: &Patch) -> Patch {
    let mut patch = base.clone();
    patch.name = state.preset_name.clone();
    if let Some(osc) = patch.oscillators.get_mut(0) {
        osc.position = state.wt_position;
    }
    patch.filter.cutoff = state.filter_cutoff;
    patch.filter.resonance = state.filter_resonance;
    patch
}

fn main() -> eframe::Result<()> {
    let audio = match start_audio(44100) {
        Ok(a) => Some(Arc::new(a)),
        Err(e) => {
            eprintln!("audio init failed: {e}");
            None
        }
    };

    let midi_devices = MidiDevices::enumerate();
    let (midi_note_tx, midi_note_rx) = crossbeam_channel::unbounded::<(u8, bool, f32)>();

    eframe::run_native(
        "ReelSynth",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280.0, 720.0])
                .with_min_inner_size([1024.0, 640.0])
                .with_title("ReelSynth"),
            ..Default::default()
        },
        Box::new(move |cc| {
            reelsynth_ui_theme::apply(&cc.egui_ctx);
            Ok(Box::new(ReelSynthApp::new(
                audio.clone(),
                midi_devices,
                midi_note_tx,
                midi_note_rx,
            )))
        }),
    )
}

struct ReelSynthApp {
    audio: Option<Arc<AudioHandle>>,
    state: S1State,
    current_patch: Patch,
    preset_path: Option<PathBuf>,
    midi_devices: MidiDevices,
    midi_selected: usize,
    midi_handle: MidiInputHandle,
    midi_note_tx: Sender<(u8, bool, f32)>,
    midi_note_rx: Receiver<(u8, bool, f32)>,
}

impl ReelSynthApp {
    fn new(
        audio: Option<Arc<AudioHandle>>,
        midi_devices: MidiDevices,
        midi_note_tx: Sender<(u8, bool, f32)>,
        midi_note_rx: Receiver<(u8, bool, f32)>,
    ) -> Self {
        let status = if audio.is_some() {
            "Audio OK — click keys, QWERTY (Z–M), or MIDI".into()
        } else {
            "No audio — UI only".into()
        };
        let midi_handle = MidiInputHandle::disconnected();
        Self {
            audio,
            state: S1State {
                status,
                ..S1State::default()
            },
            current_patch: Patch::default_mono(),
            preset_path: None,
            midi_devices,
            midi_selected: 0,
            midi_handle,
            midi_note_tx,
            midi_note_rx,
        }
    }

    fn note_on(&mut self, note: u8, velocity: f32) {
        if self.state.keys_down.insert(note) {
            if let Some(a) = &self.audio {
                a.send(AudioCmd::NoteOn(note, velocity));
            }
        }
    }

    fn note_off(&mut self, note: u8) {
        if self.state.keys_down.remove(&note) {
            if let Some(a) = &self.audio {
                a.send(AudioCmd::NoteOff(note));
            }
        }
    }

    fn sync_params(&mut self) {
        if let Some(a) = &self.audio {
            a.send(AudioCmd::SetWtPosition(self.state.wt_position));
            a.send(AudioCmd::SetFilterCutoff(self.state.filter_cutoff));
            a.send(AudioCmd::SetFilterResonance(self.state.filter_resonance));
        }
        self.current_patch = patch_from_state(&self.state, &self.current_patch);
    }

    fn connect_midi(&mut self, index: usize) {
        self.midi_selected = index;
        self.midi_handle = match MidiInputHandle::connect(
            &self.midi_devices,
            index,
            self.midi_note_tx.clone(),
        ) {
            Ok(h) => {
                let label = self
                    .midi_devices
                    .names
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| "MIDI".into());
                self.state.midi_device = if index == 0 {
                    "None".into()
                } else {
                    label.clone()
                };
                if index == 0 {
                    self.state.status = "MIDI disconnected".into();
                } else {
                    self.state.status = format!("MIDI: {label}");
                }
                h
            }
            Err(e) => {
                self.state.status = e;
                MidiInputHandle::disconnected()
            }
        };
    }

    fn open_preset(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("ReelSynth Preset", &["reelpreset"])
            .pick_file()
        else {
            return;
        };

        match load_preset(&path) {
            Ok(patch) => match resolve_bank(&path, &patch) {
                Ok(bank) => {
                    if let Some(a) = &self.audio {
                        a.send(AudioCmd::LoadPreset {
                            patch: patch.clone(),
                            bank,
                        });
                    }
                    sync_state_from_patch(&mut self.state, &patch);
                    self.current_patch = patch;
                    self.preset_path = Some(path);
                    self.state.status = format!(
                        "Loaded {}",
                        self.preset_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("preset")
                    );
                }
                Err(e) => self.state.status = e,
            },
            Err(e) => self.state.status = format!("Open failed: {e}"),
        }
    }

    fn save_preset(&mut self) {
        let path = if let Some(p) = &self.preset_path {
            Some(p.clone())
        } else {
            let default_name = format!(
                "{}.reelpreset",
                self.state
                    .preset_name
                    .replace(['/', '\\'], "_")
                    .trim()
            );
            rfd::FileDialog::new()
                .add_filter("ReelSynth Preset", &["reelpreset"])
                .set_file_name(&default_name)
                .save_file()
        };

        let Some(mut path) = path else {
            return;
        };

        if path.extension().is_none() {
            path.set_extension("reelpreset");
        }

        self.current_patch = patch_from_state(&self.state, &self.current_patch);
        match self.current_patch.to_json() {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(()) => {
                    self.preset_path = Some(path.clone());
                    self.state.status = format!(
                        "Saved {}",
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("preset")
                    );
                }
                Err(e) => self.state.status = format!("Save failed: {e}"),
            },
            Err(e) => self.state.status = format!("Serialize failed: {e}"),
        }
    }

    fn bank_for_ui(&self) -> Option<WavetableBank> {
        self.audio
            .as_ref()
            .and_then(|a| a.bank().read().ok().map(|g| (*g).clone()))
    }
}

fn keyboard_note(key: egui::Key) -> Option<u8> {
    use egui::Key;
    Some(match key {
        Key::Z => 48,
        Key::S => 49,
        Key::X => 50,
        Key::D => 51,
        Key::C => 52,
        Key::V => 53,
        Key::G => 54,
        Key::B => 55,
        Key::H => 56,
        Key::N => 57,
        Key::J => 58,
        Key::M => 59,
        _ => return None,
    })
}

impl eframe::App for ReelSynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok((note, on, vel)) = self.midi_note_rx.try_recv() {
            if on {
                self.note_on(note, vel.max(0.05));
            } else {
                self.note_off(note);
            }
        }

        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed,
                    repeat: false,
                    ..
                } = event
                {
                    if let Some(note) = keyboard_note(*key) {
                        if *pressed {
                            self.note_on(note, 0.9);
                        } else {
                            self.note_off(note);
                        }
                    }
                }
            }
        });

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: reelsynth_ui_theme::Tokens::default().bg,
                ..Default::default()
            })
            .show(ctx, |ui| {
                let bank = self.bank_for_ui();
                let bank_ref = bank.as_ref();
                let midi = S1MidiDevices {
                    names: &self.midi_devices.names,
                    selected: self.midi_selected,
                };
                let config = S1ShellConfig {
                    show_wt_editor: true,
                };
                let actions = draw_s1(ui, ui.max_rect(), &mut self.state, bank_ref, &midi, &config);

                if let Some(n) = actions.note_on {
                    self.note_on(n, 0.9);
                }
                if let Some(n) = actions.note_off {
                    self.note_off(n);
                }
                if actions.params_changed {
                    self.sync_params();
                }
                if actions.open_preset {
                    self.open_preset();
                }
                if actions.save_preset {
                    self.save_preset();
                }
                if let Some(idx) = actions.midi_device_selected {
                    if idx != self.midi_selected {
                        self.connect_midi(idx);
                    }
                }
            });

        if self.audio.is_some() {
            ctx.request_repaint();
        }
    }
}
