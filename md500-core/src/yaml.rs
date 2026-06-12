//! YAML string codecs + schema-header constants.
//!
//! The CLI wraps these with file I/O and a `# yaml-language-server: $schema=…`
//! header so editors auto-validate dumps against the JSON Schemas in `schemas/`.
//! The string codecs live here so the wasm crate can use them without `Path`.

use crate::bank::BankParams;
use crate::codec::CodecError;
use crate::patch::Patch;
use crate::pc_map::ProgramChangeMap;
use crate::setup::Setup;
use crate::system::System;

/// `yaml-language-server` schema header for each area's dump.
pub const SETUP_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/md500-setup.schema.json";
pub const SYSTEM_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/md500-system.schema.json";
pub const PC_MAP_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/md500-pc-map.schema.json";
pub const BANK_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/md500-bank.schema.json";
pub const PATCH_YAML_HEADER: &str =
    "# yaml-language-server: $schema=./schemas/md500-patch.schema.json";

macro_rules! yaml_codec {
    ($to:ident, $from:ident, $ty:ty) => {
        pub fn $to(v: &$ty) -> Result<String, CodecError> {
            serde_yaml::to_string(v).map_err(|e| CodecError::Yaml(e.to_string()))
        }
        pub fn $from(s: &str) -> Result<$ty, CodecError> {
            serde_yaml::from_str(s).map_err(|e| CodecError::Yaml(e.to_string()))
        }
    };
}

yaml_codec!(setup_to_yaml, setup_from_yaml, Setup);
yaml_codec!(system_to_yaml, system_from_yaml, System);
yaml_codec!(pc_map_to_yaml, pc_map_from_yaml, ProgramChangeMap);
yaml_codec!(bank_to_yaml, bank_from_yaml, BankParams);
yaml_codec!(patch_to_yaml, patch_from_yaml, Patch);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_yaml_round_trip() {
        let s = Setup::from_bytes(&[0x00, 0x00, 0x00]).unwrap();
        let y = setup_to_yaml(&s).unwrap();
        assert_eq!(setup_from_yaml(&y).unwrap(), s);
    }

    #[test]
    fn from_yaml_reports_yaml_error() {
        assert!(matches!(
            setup_from_yaml("not: : valid"),
            Err(CodecError::Yaml(_))
        ));
    }
}
