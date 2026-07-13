//! Multi-bank wavetable loader with lazy dedupe by ID.

use std::collections::HashMap;

use crate::patch::{Oscillator, Patch};
use crate::wavetable::WavetableBank;

/// Resolved wavetable banks for a patch — one entry per unique `wavetable_id`.
#[derive(Clone, Debug)]
pub struct BankSet {
    banks: Vec<WavetableBank>,
    id_index: HashMap<String, usize>,
    default_index: usize,
}

impl BankSet {
    pub fn from_primary(primary: WavetableBank, patch: &Patch) -> Self {
        let mut banks = vec![primary.clone()];
        let mut id_index = HashMap::new();

        if let Some(id) = patch.wavetable_id.clone() {
            id_index.insert(id, 0);
        }

        for osc in &patch.oscillators {
            if let Some(id) = osc.wavetable_id.as_deref() {
                if !id_index.contains_key(id) {
                    let bank = load_bank_for_id(id, &primary);
                    let idx = banks.len();
                    banks.push(bank);
                    id_index.insert(id.to_string(), idx);
                }
            }
        }

        Self {
            banks,
            id_index,
            default_index: 0,
        }
    }

    pub fn banks(&self) -> &[WavetableBank] {
        &self.banks
    }

    pub fn primary(&self) -> &WavetableBank {
        &self.banks[self.default_index]
    }

    pub fn bank_for_osc(&self, patch: &Patch, osc_index: usize) -> usize {
        let osc = match patch.oscillators.get(osc_index) {
            Some(o) => o,
            None => return self.default_index,
        };
        self.index_for_osc(osc, patch)
    }

    pub fn index_for_osc(&self, osc: &Oscillator, patch: &Patch) -> usize {
        if let Some(id) = osc.wavetable_id.as_deref() {
            if let Some(&idx) = self.id_index.get(id) {
                return idx;
            }
        }
        if let Some(id) = patch.wavetable_id.as_deref() {
            if let Some(&idx) = self.id_index.get(id) {
                return idx;
            }
        }
        self.default_index
    }

    pub fn replace_primary(&mut self, bank: WavetableBank, patch: &Patch) {
        *self = Self::from_primary(bank, patch);
    }
}

fn load_bank_for_id(id: &str, fallback: &WavetableBank) -> WavetableBank {
    match id {
        "saw_morph" => WavetableBank::factory_saw_morph(),
        "square_morph" => WavetableBank::factory_square_morph(),
        "sine" => WavetableBank::factory_sine(),
        "formant" => WavetableBank::factory_formant(),
        "metallic" => WavetableBank::factory_metallic(),
        _ => fallback.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::Patch;

    #[test]
    fn dedupes_identical_ids() {
        let primary = WavetableBank::factory_saw_morph();
        let mut patch = Patch::default_mono();
        patch.ensure_oscillators(3);
        patch.oscillators[0].wavetable_id = Some("sine".into());
        patch.oscillators[1].wavetable_id = Some("saw_morph".into());
        patch.oscillators[2].wavetable_id = Some("sine".into());
        let set = BankSet::from_primary(primary, &patch);
        assert_eq!(set.banks().len(), 2);
        assert_eq!(
            set.bank_for_osc(&patch, 0),
            set.bank_for_osc(&patch, 2)
        );
    }
}
