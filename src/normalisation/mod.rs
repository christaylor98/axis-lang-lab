// Normalisation Module
//
// WAVE 1: NF Validation only
// WAVE 2: Normalisation pass framework
// WAVE 3: Desugaring transformations

pub mod desugaring;
pub mod nf_validation;
pub mod pass_framework;

pub use desugaring::*;
pub use nf_validation::*;
pub use pass_framework::*;
