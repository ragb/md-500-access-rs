//! System area (`10 00 00 00`) — global settings.
//!
//! Two sub-blocks: **System Common** (`+00 00 00`, 34 bytes, spec *2-2) and
//! **System Control** (`+00 10 00`, 16 bytes, spec *2-3). They sit at different
//! offsets, so the [`System`] document is dumped/synced as two SysEx blocks
//! (handled in [`crate::device`]). Spec v1.00, not yet device-verified.

use serde::{Deserialize, Serialize};

use midi_access_core::byte_enum;
use midi_access_core::codec::bool_byte;

use crate::codec::{from_offset, read_nibbles, to_offset, write_nibbles, CodecError};
use crate::common::{CcNumber, MidiRxChannel, MidiTxChannel};

/// Encoded length of the System Common block.
pub const SYSTEM_COMMON_LEN: usize = 0x22; // 34
/// Encoded length of the System Control block.
pub const SYSTEM_CONTROL_LEN: usize = 0x10; // 16

// === enums ===

byte_enum! {
    /// Behaviour when switching banks. (`Bank wait mode`)
    BankWaitMode { Wait = 0, Immediate = 1 }
    valid = "0=wait, 1=immediate"
}
byte_enum! {
    /// How a knob's stored value engages when you turn it. (`Knob Mode`)
    KnobMode { Immediate = 0, Hook = 1 }
    valid = "0=immediate, 1=hook"
}
byte_enum! {
    /// Effect bypass behaviour. (`Bypass Mode`)
    BypassMode { Buffered = 0, True = 1 }
    valid = "0=buffered, 1=true"
}
byte_enum! {
    /// When a momentary pedal action fires. (`Pedal Action`)
    PedalAction { Push = 0, Release = 1 }
    valid = "0=push, 1=release"
}
byte_enum! {
    /// Footswitch operating mode. (`Foot Switch Mode`)
    FootSwitchMode { Normal = 0, Abc = 1, AbSimul = 2, SwDnUp = 3 }
    valid = "0=normal, 1=a/b/c, 2=a/b simul, 3=sw dn/up"
}
byte_enum! {
    /// What the main display shows. (`Display Type`)
    DisplayType { Time = 0, Bpm = 1, Patch = 2, Param = 3 }
    valid = "0=time, 1=bpm, 2=patch, 3=param"
}
byte_enum! {
    /// Which bank-select messages are transmitted. (`MIDI Bank Select OUT`)
    BankSelectOut { Msb = 0, MsbLsb = 1 }
    valid = "0=msb, 1=msb+lsb"
}
byte_enum! {
    /// Tempo-clock sync source. (`MIDI Sync`)
    MidiSync { Internal = 0, ExtUsb = 1, ExtMidi = 2, Auto = 3 }
    valid = "0=internal, 1=ext(usb), 2=ext(midi), 3=auto"
}
byte_enum! {
    /// Where realtime messages are routed out. (`MIDI source of real time message to output`)
    RealtimeOut { Int = 0, Usb = 1, Midi = 2 }
    valid = "0=int, 1=usb, 2=midi"
}
byte_enum! {
    /// MIDI/USB THRU routing. (`MIDI In THRU into Out`, `USB In THRU into Out`)
    ThruRoute { Off = 0, Usb = 1, Midi = 2, UsbMidi = 3 }
    valid = "0=off, 1=usb, 2=midi, 3=usb+midi"
}
byte_enum! {
    /// Whether a control follows the patch or a global system setting.
    /// (`TAP/CTL/CTL1/CTL2/EXP preference`)
    ControlPreference { Patch = 0, System = 1 }
    valid = "0=patch, 1=system"
}
byte_enum! {
    /// Function assigned to TAP / CTL1 / CTL2. (spec label: `(TAP|CTL)/CTL Function`)
    CtlFunction { Off = 0, Tap = 1, Reset = 2, Moment = 3, BankUp = 4, BankDown = 5 }
    valid = "0=off, 1=tap, 2=reset, 3=moment, 4=bank up, 5=bank down"
}
byte_enum! {
    /// Function assigned to the expression pedal. (`Exp Pedal Function`)
    ExpFunction { Off = 0, Rate = 1, Depth = 2, ELevel = 3, Param1 = 4, Param2 = 5 }
    valid = "0=off, 1=rate, 2=depth, 3=e.level, 4=param1, 5=param2"
}

/// Decode a bank number stored as `0..=98` (display `1..=99`).
fn bank_no_from_byte(b: u8, field: &'static str) -> Result<u8, CodecError> {
    if b <= 98 {
        Ok(b + 1)
    } else {
        Err(CodecError::InvalidValue {
            field,
            value: b,
            valid: "0..=98 (=bank 1..=99)",
        })
    }
}
fn bank_no_to_byte(n: u8, field: &'static str) -> Result<u8, CodecError> {
    if (1..=99).contains(&n) {
        Ok(n - 1)
    } else {
        Err(CodecError::OutOfRange {
            field,
            value: n as i32,
            valid: "1..=99",
        })
    }
}

// === System Common ===

/// Typed view of the System Common block (`10 00 00 00`, 34 bytes).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemCommon {
    /// Insert the effect into the send/return loop (offset 0x00).
    pub insert_loop_sw: bool,
    /// Bank switch timing (offset 0x01).
    pub bank_wait_mode: BankWaitMode,
    /// First bank reachable by BANK UP/DOWN, `1..=99` (offset 0x02).
    pub bank_extent_begin: u8,
    /// Last bank reachable by BANK UP/DOWN, `1..=99` (offset 0x03).
    pub bank_extent_end: u8,
    /// Lock the panel knobs (offset 0x04).
    pub knob_lock: bool,
    /// Knob pickup behaviour (offset 0x05).
    pub knob_mode: KnobMode,
    /// Bypass mode (offset 0x06).
    pub bypass_mode: BypassMode,
    /// Momentary pedal action timing (offset 0x07).
    pub pedal_action: PedalAction,
    /// Footswitch operating mode (offset 0x08).
    pub foot_switch_mode: FootSwitchMode,
    /// Display readout (offset 0x09).
    pub display_type: DisplayType,
    /// MIDI receive channel (offset 0x0A).
    pub rx_channel: MidiRxChannel,
    /// MIDI transmit channel (offset 0x0B).
    pub tx_channel: MidiTxChannel,
    /// Receive Program Change (offset 0x0C).
    pub pc_in: bool,
    /// Transmit Program Change (offset 0x0D).
    pub pc_out: bool,
    /// Bank-select messages to transmit (offset 0x0E).
    pub bank_select_out: BankSelectOut,
    /// Receive Control Change (offset 0x0F).
    pub cc_in: bool,
    /// Transmit Control Change (offset 0x10).
    pub cc_out: bool,
    /// CC# that drives RATE/VALUE (offset 0x11).
    pub rate_cc: CcNumber,
    /// CC# that drives DEPTH (offset 0x12).
    pub depth_cc: CcNumber,
    /// CC# that drives EFFECT LEVEL (offset 0x13).
    pub effect_level_cc: CcNumber,
    /// CC# that drives PARAM 1 (offset 0x14).
    pub param1_cc: CcNumber,
    /// CC# that drives PARAM 2 (offset 0x15).
    pub param2_cc: CcNumber,
    /// CC# for EFFECT on/off (offset 0x16).
    pub effect_cc: CcNumber,
    /// CC# for EFFECT A on/off (offset 0x17).
    pub effect_a_cc: CcNumber,
    /// CC# for EFFECT B on/off (offset 0x18).
    pub effect_b_cc: CcNumber,
    /// CC# for the CTL1 control (offset 0x19).
    pub ctl1_cc: CcNumber,
    /// CC# for the CTL2 control (offset 0x1A).
    pub ctl2_cc: CcNumber,
    /// CC# for the expression pedal (offset 0x1B).
    pub exp_cc: CcNumber,
    /// Tempo sync source (offset 0x1C).
    pub sync: MidiSync,
    /// Realtime-message output routing (offset 0x1D).
    pub realtime_out: RealtimeOut,
    /// MIDI IN → OUT THRU routing (offset 0x1E).
    pub midi_thru: ThruRoute,
    /// USB IN → OUT THRU routing (offset 0x1F).
    pub usb_thru: ThruRoute,
    /// PATCH A effect-switch state at power-on, for SIMUL (offset 0x20).
    pub simul_effect_sw_a: bool,
    /// PATCH B effect-switch state at power-on, for SIMUL (offset 0x21).
    pub simul_effect_sw_b: bool,
}

impl SystemCommon {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() != SYSTEM_COMMON_LEN {
            return Err(CodecError::WrongLength {
                expected: SYSTEM_COMMON_LEN,
                actual: b.len(),
            });
        }
        Ok(Self {
            insert_loop_sw: bool_byte(b[0x00], "insert_loop_sw")?,
            bank_wait_mode: BankWaitMode::from_byte(b[0x01])?,
            bank_extent_begin: bank_no_from_byte(b[0x02], "bank_extent_begin")?,
            bank_extent_end: bank_no_from_byte(b[0x03], "bank_extent_end")?,
            knob_lock: bool_byte(b[0x04], "knob_lock")?,
            knob_mode: KnobMode::from_byte(b[0x05])?,
            bypass_mode: BypassMode::from_byte(b[0x06])?,
            pedal_action: PedalAction::from_byte(b[0x07])?,
            foot_switch_mode: FootSwitchMode::from_byte(b[0x08])?,
            display_type: DisplayType::from_byte(b[0x09])?,
            rx_channel: MidiRxChannel::from_byte(b[0x0A])?,
            tx_channel: MidiTxChannel::from_byte(b[0x0B])?,
            pc_in: bool_byte(b[0x0C], "pc_in")?,
            pc_out: bool_byte(b[0x0D], "pc_out")?,
            bank_select_out: BankSelectOut::from_byte(b[0x0E])?,
            cc_in: bool_byte(b[0x0F], "cc_in")?,
            cc_out: bool_byte(b[0x10], "cc_out")?,
            rate_cc: CcNumber::from_byte(b[0x11], "rate_cc")?,
            depth_cc: CcNumber::from_byte(b[0x12], "depth_cc")?,
            effect_level_cc: CcNumber::from_byte(b[0x13], "effect_level_cc")?,
            param1_cc: CcNumber::from_byte(b[0x14], "param1_cc")?,
            param2_cc: CcNumber::from_byte(b[0x15], "param2_cc")?,
            effect_cc: CcNumber::from_byte(b[0x16], "effect_cc")?,
            effect_a_cc: CcNumber::from_byte(b[0x17], "effect_a_cc")?,
            effect_b_cc: CcNumber::from_byte(b[0x18], "effect_b_cc")?,
            ctl1_cc: CcNumber::from_byte(b[0x19], "ctl1_cc")?,
            ctl2_cc: CcNumber::from_byte(b[0x1A], "ctl2_cc")?,
            exp_cc: CcNumber::from_byte(b[0x1B], "exp_cc")?,
            sync: MidiSync::from_byte(b[0x1C])?,
            realtime_out: RealtimeOut::from_byte(b[0x1D])?,
            midi_thru: ThruRoute::from_byte(b[0x1E])?,
            usb_thru: ThruRoute::from_byte(b[0x1F])?,
            simul_effect_sw_a: bool_byte(b[0x20], "simul_effect_sw_a")?,
            simul_effect_sw_b: bool_byte(b[0x21], "simul_effect_sw_b")?,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut b = vec![0u8; SYSTEM_COMMON_LEN];
        b[0x00] = self.insert_loop_sw as u8;
        b[0x01] = self.bank_wait_mode.to_byte();
        b[0x02] = bank_no_to_byte(self.bank_extent_begin, "bank_extent_begin")?;
        b[0x03] = bank_no_to_byte(self.bank_extent_end, "bank_extent_end")?;
        b[0x04] = self.knob_lock as u8;
        b[0x05] = self.knob_mode.to_byte();
        b[0x06] = self.bypass_mode.to_byte();
        b[0x07] = self.pedal_action.to_byte();
        b[0x08] = self.foot_switch_mode.to_byte();
        b[0x09] = self.display_type.to_byte();
        b[0x0A] = self.rx_channel.to_byte()?;
        b[0x0B] = self.tx_channel.to_byte()?;
        b[0x0C] = self.pc_in as u8;
        b[0x0D] = self.pc_out as u8;
        b[0x0E] = self.bank_select_out.to_byte();
        b[0x0F] = self.cc_in as u8;
        b[0x10] = self.cc_out as u8;
        b[0x11] = self.rate_cc.to_byte("rate_cc")?;
        b[0x12] = self.depth_cc.to_byte("depth_cc")?;
        b[0x13] = self.effect_level_cc.to_byte("effect_level_cc")?;
        b[0x14] = self.param1_cc.to_byte("param1_cc")?;
        b[0x15] = self.param2_cc.to_byte("param2_cc")?;
        b[0x16] = self.effect_cc.to_byte("effect_cc")?;
        b[0x17] = self.effect_a_cc.to_byte("effect_a_cc")?;
        b[0x18] = self.effect_b_cc.to_byte("effect_b_cc")?;
        b[0x19] = self.ctl1_cc.to_byte("ctl1_cc")?;
        b[0x1A] = self.ctl2_cc.to_byte("ctl2_cc")?;
        b[0x1B] = self.exp_cc.to_byte("exp_cc")?;
        b[0x1C] = self.sync.to_byte();
        b[0x1D] = self.realtime_out.to_byte();
        b[0x1E] = self.midi_thru.to_byte();
        b[0x1F] = self.usb_thru.to_byte();
        b[0x20] = self.simul_effect_sw_a as u8;
        b[0x21] = self.simul_effect_sw_b as u8;
        Ok(b)
    }
}

// === System Control ===

/// Typed view of the System Control block (`10 00 10 00`, 16 bytes).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemControl {
    /// Whether TAP/CTL follows the patch or the system (offset 0x00).
    pub tap_ctl_preference: ControlPreference,
    /// Function of the TAP/CTL footswitch (offset 0x01).
    pub tap_ctl_function: CtlFunction,
    /// Whether CTL1 follows the patch or the system (offset 0x02).
    pub ctl1_preference: ControlPreference,
    /// Function of the external CTL1 footswitch (offset 0x03).
    pub ctl1_function: CtlFunction,
    /// Whether CTL2 follows the patch or the system (offset 0x04).
    pub ctl2_preference: ControlPreference,
    /// Function of the external CTL2 footswitch (offset 0x05).
    pub ctl2_function: CtlFunction,
    /// Whether the expression pedal follows the patch or the system (offset 0x06).
    pub exp_preference: ControlPreference,
    /// Function of the expression pedal (offset 0x07).
    pub exp_pedal_function: ExpFunction,
    /// Expression-pedal target minimum, display `-100..=23000` (offset 0x08, 4 bytes).
    pub exp_target_min: i32,
    /// Expression-pedal target maximum, display `-100..=23000` (offset 0x0C, 4 bytes).
    pub exp_target_max: i32,
}

/// Raw upper bound for the expression target min/max nibble value (display +offset).
const EXP_TARGET_RAW_HI: u32 = 23100;
const EXP_TARGET_OFFSET: i64 = 100;
const EXP_TARGET_WIDTH: usize = 4;

impl SystemControl {
    pub fn from_bytes(b: &[u8]) -> Result<Self, CodecError> {
        if b.len() != SYSTEM_CONTROL_LEN {
            return Err(CodecError::WrongLength {
                expected: SYSTEM_CONTROL_LEN,
                actual: b.len(),
            });
        }
        Ok(Self {
            tap_ctl_preference: ControlPreference::from_byte(b[0x00])?,
            tap_ctl_function: CtlFunction::from_byte(b[0x01])?,
            ctl1_preference: ControlPreference::from_byte(b[0x02])?,
            ctl1_function: CtlFunction::from_byte(b[0x03])?,
            ctl2_preference: ControlPreference::from_byte(b[0x04])?,
            ctl2_function: CtlFunction::from_byte(b[0x05])?,
            exp_preference: ControlPreference::from_byte(b[0x06])?,
            exp_pedal_function: ExpFunction::from_byte(b[0x07])?,
            exp_target_min: from_offset(read_nibbles(&b[0x08..0x0C]), EXP_TARGET_OFFSET) as i32,
            exp_target_max: from_offset(read_nibbles(&b[0x0C..0x10]), EXP_TARGET_OFFSET) as i32,
        })
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut b = vec![0u8; SYSTEM_CONTROL_LEN];
        b[0x00] = self.tap_ctl_preference.to_byte();
        b[0x01] = self.tap_ctl_function.to_byte();
        b[0x02] = self.ctl1_preference.to_byte();
        b[0x03] = self.ctl1_function.to_byte();
        b[0x04] = self.ctl2_preference.to_byte();
        b[0x05] = self.ctl2_function.to_byte();
        b[0x06] = self.exp_preference.to_byte();
        b[0x07] = self.exp_pedal_function.to_byte();
        let min = to_offset(
            self.exp_target_min as i64,
            EXP_TARGET_OFFSET,
            EXP_TARGET_RAW_HI,
            "exp_target_min",
        )?;
        let max = to_offset(
            self.exp_target_max as i64,
            EXP_TARGET_OFFSET,
            EXP_TARGET_RAW_HI,
            "exp_target_max",
        )?;
        b[0x08..0x0C].copy_from_slice(&write_nibbles(min, EXP_TARGET_WIDTH));
        b[0x0C..0x10].copy_from_slice(&write_nibbles(max, EXP_TARGET_WIDTH));
        Ok(b)
    }
}

// === combined System document ===

/// The full System document — both sub-blocks, dumped/synced together.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub common: SystemCommon,
    pub control: SystemControl,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_common() -> Vec<u8> {
        let mut b = vec![0u8; SYSTEM_COMMON_LEN];
        b[0x01] = 1; // bank_wait_mode = immediate
        b[0x02] = 0; // bank_extent_begin → 1
        b[0x03] = 98; // bank_extent_end → 99
        b[0x08] = 2; // foot_switch_mode = a/b simul
        b[0x0A] = 0; // rx ch 1
        b[0x0B] = 16; // tx = rx
        b[0x11] = 1; // rate_cc = CC#1
        b[0x12] = 32; // depth_cc = CC#64
        b[0x1C] = 3; // sync = auto
        b
    }

    #[test]
    fn common_round_trips() {
        let bytes = sample_common();
        let c = SystemCommon::from_bytes(&bytes).unwrap();
        assert_eq!(c.bank_extent_begin, 1);
        assert_eq!(c.bank_extent_end, 99);
        assert_eq!(c.rate_cc, CcNumber::Number(1));
        assert_eq!(c.depth_cc, CcNumber::Number(64));
        assert_eq!(c.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn control_round_trips_with_signed_exp_targets() {
        let mut b = vec![0u8; SYSTEM_CONTROL_LEN];
        b[0x01] = 1; // tap function = tap
                     // exp_target_min = display -100 → raw 0.
                     // exp_target_max = display 23000 → raw 23100 = nibbles 5,9,3,12 → 0x5,0x9,0x3,0xC.
        b[0x0C..0x10].copy_from_slice(&write_nibbles(23100, 4));
        let c = SystemControl::from_bytes(&b).unwrap();
        assert_eq!(c.exp_target_min, -100);
        assert_eq!(c.exp_target_max, 23000);
        assert_eq!(c.to_bytes().unwrap(), b);
    }

    #[test]
    fn common_rejects_wrong_length() {
        assert!(matches!(
            SystemCommon::from_bytes(&[0u8; 10]),
            Err(CodecError::WrongLength { .. })
        ));
    }

    #[test]
    fn yaml_named_enums() {
        let c = SystemCommon::from_bytes(&sample_common()).unwrap();
        let y = serde_yaml::to_string(&c).unwrap();
        assert!(y.contains("bank_wait_mode: immediate"));
        assert!(y.contains("foot_switch_mode: ab_simul"));
        assert!(y.contains("sync: auto"));
        assert!(y.contains("tx_channel: rx"));
    }
}
