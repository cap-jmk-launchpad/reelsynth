//! Factory wavetable catalog for the WT import menu.

use reelsynth::WavetableBank;

#[derive(Clone, Copy, Debug)]
pub struct FactoryBankEntry {
    pub id: &'static str,
    pub label: &'static str,
}

pub const FACTORY_BANKS: &[FactoryBankEntry] = &[
    FactoryBankEntry {
        id: "saw_morph",
        label: "Saw Morph",
    },
    FactoryBankEntry {
        id: "square_morph",
        label: "Square Morph",
    },
    FactoryBankEntry {
        id: "sine",
        label: "Sine",
    },
    FactoryBankEntry {
        id: "formant",
        label: "Formant",
    },
    FactoryBankEntry {
        id: "metallic",
        label: "Metallic",
    },
];

pub fn factory_bank(id: &str) -> Option<WavetableBank> {
    match id {
        "saw_morph" => Some(WavetableBank::factory_saw_morph()),
        "square_morph" => Some(WavetableBank::factory_square_morph()),
        "sine" => Some(WavetableBank::factory_sine()),
        "formant" => Some(WavetableBank::factory_formant()),
        "metallic" => Some(WavetableBank::factory_metallic()),
        _ => None,
    }
}

pub fn factory_label(id: &str) -> Option<&'static str> {
    FACTORY_BANKS
        .iter()
        .find(|e| e.id == id)
        .map(|e| e.label)
}
