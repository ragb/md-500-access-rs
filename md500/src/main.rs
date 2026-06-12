//! MD-500 CLI entry point.
//!
//! The whole command surface — ports / identity / dump / sync / show / lint /
//! diff / schema / catalog / resolve — is the generic engine in
//! `midi-access-cli`, dispatched through [`md500_core::Md500`]'s [`Device`] impl.
//!
//! [`Device`]: midi_access_core::Device

use std::process::ExitCode;

fn main() -> ExitCode {
    midi_access_cli::run::<md500_core::Md500>()
}
