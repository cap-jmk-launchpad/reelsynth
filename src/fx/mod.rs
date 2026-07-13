//! FX chain: types, bypass, and stereo processors.

mod chain;
mod processors;
mod types;

pub use chain::{effects_from_bypass, FxBypass, FxChain};
pub use types::{default_effects, EffectSlot, EffectType};

#[cfg(test)]
mod tests;
