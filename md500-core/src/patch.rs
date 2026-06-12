//! PATCH parameters (`+00 10 00` / `+00 20 00` / `+00 30 00` within a bank) —
//! spec *2-6. The MD-500's central, 586-byte modulation-patch block.
//!
//! Layout (all offsets relative to the patch base):
//!
//! | Range          | Contents                                              |
//! |----------------|-------------------------------------------------------|
//! | `0x000..0x010` | Patch name (16 ASCII bytes)                           |
//! | `0x010..0x023` | [`PatchCommon`] — mode selector + shared LFO/EQ/output|
//! | `0x023..0x173` | the twelve modulation modes, laid out flat            |
//! | `0x173..0x180` | [`PatchControl`] — per-patch footswitch/expression    |
//! | `0x180..0x248` | eight [`Assign`] slots (25 bytes each)                |
//! | `0x248..0x24A` | Assign Input Sens + LED status                        |
//!
//! Every offset is always present on the wire regardless of the active mode (the
//! `mode` selector just chooses which block is audible), so [`Patch`] stores all
//! of them and decodes/encodes the block **sequentially** — a cursor walks the
//! bytes in spec order, which keeps the codec free of hand-written offset
//! arithmetic. Round-trip is byte-exact; the final cursor position is asserted to
//! equal [`PATCH_LEN`].
//!
//! Spec v1.00, not yet device-verified. Numeric depth/level/rate fields are typed
//! as integers (the device's wire form); only clearly-labelled selectors are
//! enums. Fields whose spec range/signedness is ambiguous in the source PDF are
//! kept as raw `u8` and noted; this can be tightened later without changing the
//! wire format.

use serde::{Deserialize, Serialize};

use midi_access_core::byte_enum;
use midi_access_core::codec::{
    bool_byte, read_ascii, signed_center, to_signed_center, write_ascii,
};

use crate::codec::{read_nibbles, write_nibbles, CodecError};
use crate::common::NoteValue;
use crate::system::{CtlFunction, ExpFunction};

/// Encoded length of one PATCH block.
pub const PATCH_LEN: usize = 0x24A; // 586
/// Length of the patch-name field.
pub const PATCH_NAME_LEN: usize = 16;
/// Number of assign slots.
pub const ASSIGN_COUNT: usize = 8;
/// Number of SLICER user-pattern steps.
pub const SLICER_STEPS: usize = 24;
/// Number of FILTER PATTERN frequency steps.
pub const FILTER_PATTERN_STEPS: usize = 24;

// === sequential reader / writer ===

struct Rd<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Rd<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self { b, i: 0 }
    }
    /// Next raw 7-bit byte.
    fn u8(&mut self) -> u8 {
        let v = self.b[self.i];
        self.i += 1;
        v
    }
    fn boolean(&mut self, field: &'static str) -> Result<bool, CodecError> {
        bool_byte(self.u8(), field)
    }
    fn note(&mut self) -> Result<NoteValue, CodecError> {
        NoteValue::from_byte(self.u8())
    }
    /// Centred-signed byte (value = raw − center).
    fn signed(
        &mut self,
        center: u8,
        lo: i32,
        hi: i32,
        field: &'static str,
    ) -> Result<i8, CodecError> {
        signed_center(self.u8(), center, lo, hi, field)
    }
    /// `w`-byte nibble-packed unsigned value.
    fn nib(&mut self, w: usize) -> u32 {
        let v = read_nibbles(&self.b[self.i..self.i + w]);
        self.i += w;
        v
    }
    /// `w`-byte nibble value with a display offset (`display = raw − offset`).
    fn nib_off(&mut self, w: usize, offset: i64) -> i64 {
        self.nib(w) as i64 - offset
    }
}

struct Wr {
    b: Vec<u8>,
}

impl Wr {
    fn new() -> Self {
        Self {
            b: Vec::with_capacity(PATCH_LEN),
        }
    }
    /// Push a raw value, rejecting anything outside the 7-bit wire range.
    fn u8(&mut self, v: u8, field: &'static str) -> Result<(), CodecError> {
        if v > 0x7F {
            return Err(CodecError::OutOfRange {
                field,
                value: v as i32,
                valid: "0..=127",
            });
        }
        self.b.push(v);
        Ok(())
    }
    fn boolean(&mut self, v: bool) {
        self.b.push(v as u8);
    }
    fn raw(&mut self, v: u8) {
        self.b.push(v);
    }
    fn signed(
        &mut self,
        v: i8,
        center: u8,
        lo: i32,
        hi: i32,
        field: &'static str,
    ) -> Result<(), CodecError> {
        self.b.push(to_signed_center(v, center, lo, hi, field)?);
        Ok(())
    }
    fn nib(&mut self, v: u32, w: usize) {
        self.b.extend_from_slice(&write_nibbles(v, w));
    }
    fn nib_ranged(
        &mut self,
        v: u32,
        w: usize,
        hi: u32,
        field: &'static str,
    ) -> Result<(), CodecError> {
        if v > hi {
            return Err(CodecError::OutOfRange {
                field,
                value: v as i32,
                valid: leak_hi(hi),
            });
        }
        self.nib(v, w);
        Ok(())
    }
    fn nib_off(
        &mut self,
        display: i64,
        w: usize,
        offset: i64,
        raw_hi: u32,
        field: &'static str,
    ) -> Result<(), CodecError> {
        let raw = display + offset;
        if !(0..=raw_hi as i64).contains(&raw) {
            return Err(CodecError::OutOfRange {
                field,
                value: display as i32,
                valid: leak_range(-offset, raw_hi as i64 - offset),
            });
        }
        self.nib(raw as u32, w);
        Ok(())
    }
}

fn leak_hi(hi: u32) -> &'static str {
    Box::leak(format!("0..={hi}").into_boxed_str())
}
fn leak_range(lo: i64, hi: i64) -> &'static str {
    Box::leak(format!("{lo}..={hi}").into_boxed_str())
}

// === selector enums (clearly labelled in the spec) ===

byte_enum! {
    /// The active modulation algorithm (offset 0x10).
    ModulationMode {
        Chorus = 0, Flanger = 1, Phaser = 2, CVibe = 3, Vibrato = 4, Tremolo = 5,
        Dimension = 6, RingMod = 7, Rotary = 8, Filter = 9, Slicer = 10, Overtone = 11,
    }
    valid = "0..=11"
}
byte_enum! {
    /// Where the effect sits in the signal path (offset 0x20).
    InsertSw { Off = 0, Pre = 1, Post = 2 }
    valid = "0=off, 1=pre, 2=post"
}
byte_enum! {
    /// Mono vs stereo output (offset 0x21).
    OutputMode { Mono = 0, Stereo = 1 }
    valid = "0=mono, 1=stereo"
}
byte_enum! {
    /// CHORUS sub-algorithm (offset 0x23).
    ChorusType { Prime = 0, Ce1Chorus = 1, Ce1Vibrato = 2, TriCho = 3 }
    valid = "0=prime, 1=ce-1 chorus, 2=ce-1 vibrato, 3=tri-cho"
}
byte_enum! {
    /// FLANGER sub-algorithm (offset 0x38).
    FlangerType { PrimeG = 0, PrimeB = 1 }
    valid = "0=prime g, 1=prime b"
}
byte_enum! {
    /// C-VIBE sub-algorithm (offset 0x76).
    CVibeType { Prime = 0, Scanner = 1 }
    valid = "0=prime, 1=scanner"
}
byte_enum! {
    /// FILTER sub-algorithm (offset 0xB1).
    FilterType { AutoWahG = 0, AutoWahB = 1, TouchWahG = 2, TouchWahB = 3, Pattern = 4 }
    valid = "0=a-wah g, 1=a-wah b, 2=t-wah g, 3=t-wah b, 4=pattern"
}
byte_enum! {
    /// OVERTONE sub-algorithm (offset 0x167).
    OvertoneType { Overtone = 0, Detune = 1 }
    valid = "0=overtone, 1=detune"
}
byte_enum! {
    /// Whether an assign source latches or is momentary (assign +0x02).
    AssignSourceMode { Moment = 0, Toggle = 1 }
    valid = "0=moment, 1=toggle"
}
byte_enum! {
    /// Internal-LFO waveform for an assign's WAVE source (assign +0x15).
    AssignWaveForm { Saw = 0, Tri = 1, Sin = 2 }
    valid = "0=saw, 1=tri, 2=sin"
}
byte_enum! {
    /// Internal-pedal curve for an assign (assign +0x18).
    AssignPedalCurve { Linear = 0, Slow = 1, Fast = 2 }
    valid = "0=linear, 1=slow, 2=fast"
}
byte_enum! {
    /// PHASER sub-algorithm (offset 0x55).
    PhaserType { PrimeG = 0, PrimeB = 1, Script = 2 }
    valid = "0=prime g, 1=prime b, 2=script"
}
byte_enum! {
    /// VIBRATO sub-algorithm (offset 0x7A).
    VibratoType { Prime = 0, Scanner = 1 }
    valid = "0=prime, 1=scanner"
}
byte_enum! {
    /// TREMOLO sub-algorithm (offset 0x85).
    TremoloType { PrimeT = 0, PrimeP = 1, Twin = 2, Deluxe = 3 }
    valid = "0=prime t, 1=prime p, 2=twin, 3=deluxe"
}
byte_enum! {
    /// DIMENSION preset mode (offset 0x95) — Dimension D buttons 1–4, or USER.
    DimensionMode { M1 = 0, M2 = 1, M3 = 2, M4 = 3, User = 4 }
    valid = "0..=3 = modes 1..4, 4 = user"
}

// === common header (0x10..0x23) ===

/// Shared modulation parameters present for every mode (offsets 0x10..0x23).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchCommon {
    /// Active modulation algorithm (0x10).
    pub mode: ModulationMode,
    /// LFO rate, `1..=2000` = 0.01–20.00 Hz (0x11, 3 bytes).
    pub rate: u16,
    /// Tempo, `1..=96000` = ♩0.1–9600.0 (0x14, 5 bytes).
    pub bpm: u32,
    /// Initial LFO phase, `0..=23` (15° steps) (0x19).
    pub initial_phase: u8,
    /// Effect level, `0..=100` (0x1A).
    pub effect_level: u8,
    /// Low EQ level, `-50..=+50` (0x1B).
    pub low_level: i8,
    /// Low EQ frequency index, `0..=16` (0x1C).
    pub low_freq: u8,
    /// High EQ level, `-50..=+50` (0x1D).
    pub high_level: i8,
    /// High EQ frequency index, `0..=14` (0x1E).
    pub high_freq: u8,
    /// Hold tempo across patch changes (0x1F).
    pub tempo_hold: bool,
    /// Insert position (0x20).
    pub insert_sw: InsertSw,
    /// Output mode (0x21).
    pub output_mode: OutputMode,
    /// Output gain, `-6..=+6` dB (0x22).
    pub output_gain: i8,
}

impl PatchCommon {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            mode: ModulationMode::from_byte(r.u8())?,
            rate: r.nib(3) as u16,
            bpm: r.nib(5),
            initial_phase: r.u8(),
            effect_level: r.u8(),
            low_level: r.signed(50, -50, 50, "low_level")?,
            low_freq: r.u8(),
            high_level: r.signed(50, -50, 50, "high_level")?,
            high_freq: r.u8(),
            tempo_hold: r.boolean("tempo_hold")?,
            insert_sw: InsertSw::from_byte(r.u8())?,
            output_mode: OutputMode::from_byte(r.u8())?,
            output_gain: r.signed(6, -6, 6, "output_gain")?,
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.mode.to_byte());
        w.nib_ranged(self.rate as u32, 3, 2000, "rate")?;
        w.nib_ranged(self.bpm, 5, 96000, "bpm")?;
        w.u8(self.initial_phase, "initial_phase")?;
        w.u8(self.effect_level, "effect_level")?;
        w.signed(self.low_level, 50, -50, 50, "low_level")?;
        w.u8(self.low_freq, "low_freq")?;
        w.signed(self.high_level, 50, -50, 50, "high_level")?;
        w.u8(self.high_freq, "high_freq")?;
        w.boolean(self.tempo_hold);
        w.raw(self.insert_sw.to_byte());
        w.raw(self.output_mode.to_byte());
        w.signed(self.output_gain, 6, -6, 6, "output_gain")?;
        Ok(())
    }
}

// === CHORUS (0x23..0x38) ===

/// CHORUS-mode parameters (PRIME / CE-1 CHORUS / CE-1 VIBRATO / TRI-CHO).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chorus {
    pub chorus_type: ChorusType,
    pub note: NoteValue,
    pub direct_level: u8,
    /// PRIME modulation depth (0..=100).
    pub prime_depth: u8,
    /// PRIME pre-delay, 0..=400 ms (3 bytes).
    pub prime_pre_delay: u16,
    /// PRIME waveform, 0..=9 (= 1..10).
    pub prime_waveform: u8,
    pub prime_sweetness: u8,
    pub prime_bell: u8,
    /// PRIME low cut index, 0..=17.
    pub prime_low_cut: u8,
    /// PRIME high cut index, 0..=15.
    pub prime_high_cut: u8,
    pub ce1_depth: u8,
    pub ce1_preamp_sw: bool,
    /// CE-1 preamp gain, 0..=99.
    pub ce1_preamp_gain: u8,
    pub ce1_preamp_level: u8,
    /// TRI-CHO LFO mode, 0..=2.
    pub tricho_lfo_mode: u8,
    pub tricho_intensity1: u8,
    pub tricho_intensity2: u8,
    pub tricho_intensity3: u8,
    pub tricho_bright: bool,
}

impl Chorus {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            chorus_type: ChorusType::from_byte(r.u8())?,
            note: r.note()?,
            direct_level: r.u8(),
            prime_depth: r.u8(),
            prime_pre_delay: r.nib(3) as u16,
            prime_waveform: r.u8(),
            prime_sweetness: r.u8(),
            prime_bell: r.u8(),
            prime_low_cut: r.u8(),
            prime_high_cut: r.u8(),
            ce1_depth: r.u8(),
            ce1_preamp_sw: r.boolean("ce1_preamp_sw")?,
            ce1_preamp_gain: r.u8(),
            ce1_preamp_level: r.u8(),
            tricho_lfo_mode: r.u8(),
            tricho_intensity1: r.u8(),
            tricho_intensity2: r.u8(),
            tricho_intensity3: r.u8(),
            tricho_bright: r.boolean("tricho_bright")?,
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.chorus_type.to_byte());
        w.raw(self.note.to_byte());
        w.u8(self.direct_level, "chorus.direct_level")?;
        w.u8(self.prime_depth, "chorus.prime_depth")?;
        w.nib_ranged(
            self.prime_pre_delay as u32,
            3,
            400,
            "chorus.prime_pre_delay",
        )?;
        w.u8(self.prime_waveform, "chorus.prime_waveform")?;
        w.u8(self.prime_sweetness, "chorus.prime_sweetness")?;
        w.u8(self.prime_bell, "chorus.prime_bell")?;
        w.u8(self.prime_low_cut, "chorus.prime_low_cut")?;
        w.u8(self.prime_high_cut, "chorus.prime_high_cut")?;
        w.u8(self.ce1_depth, "chorus.ce1_depth")?;
        w.boolean(self.ce1_preamp_sw);
        w.u8(self.ce1_preamp_gain, "chorus.ce1_preamp_gain")?;
        w.u8(self.ce1_preamp_level, "chorus.ce1_preamp_level")?;
        w.u8(self.tricho_lfo_mode, "chorus.tricho_lfo_mode")?;
        w.u8(self.tricho_intensity1, "chorus.tricho_intensity1")?;
        w.u8(self.tricho_intensity2, "chorus.tricho_intensity2")?;
        w.u8(self.tricho_intensity3, "chorus.tricho_intensity3")?;
        w.boolean(self.tricho_bright);
        Ok(())
    }
}

// === FLANGER (0x38..0x55) ===

/// One FLANGER engine (PRIME G or PRIME B share this 13-byte layout).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlangerEngine {
    pub depth: u8,
    /// Resonance (raw 0..=100; display −100..0 on the device).
    pub resonance: u8,
    pub manual: u8,
    pub turbo_sw: bool,
    pub low_damp: u8,
    pub high_damp: u8,
    pub low_cut: u8,
    pub high_cut: u8,
    pub separation: u8,
    pub step_rate: u8,
    pub waveform: u8,
    pub input_sens: u8,
    pub polarity: u8,
}

impl FlangerEngine {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            depth: r.u8(),
            resonance: r.u8(),
            manual: r.u8(),
            turbo_sw: r.boolean("flanger.turbo_sw")?,
            low_damp: r.u8(),
            high_damp: r.u8(),
            low_cut: r.u8(),
            high_cut: r.u8(),
            separation: r.u8(),
            step_rate: r.u8(),
            waveform: r.u8(),
            input_sens: r.u8(),
            polarity: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        for (v, f) in [
            (self.depth, "flanger.depth"),
            (self.resonance, "flanger.resonance"),
            (self.manual, "flanger.manual"),
        ] {
            w.u8(v, f)?;
        }
        w.boolean(self.turbo_sw);
        for (v, f) in [
            (self.low_damp, "flanger.low_damp"),
            (self.high_damp, "flanger.high_damp"),
            (self.low_cut, "flanger.low_cut"),
            (self.high_cut, "flanger.high_cut"),
            (self.separation, "flanger.separation"),
            (self.step_rate, "flanger.step_rate"),
            (self.waveform, "flanger.waveform"),
            (self.input_sens, "flanger.input_sens"),
            (self.polarity, "flanger.polarity"),
        ] {
            w.u8(v, f)?;
        }
        Ok(())
    }
}

/// FLANGER-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flanger {
    pub flanger_type: FlangerType,
    pub note: NoteValue,
    pub direct_level: u8,
    pub prime_g: FlangerEngine,
    pub prime_b: FlangerEngine,
}

impl Flanger {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            flanger_type: FlangerType::from_byte(r.u8())?,
            note: r.note()?,
            direct_level: r.u8(),
            prime_g: FlangerEngine::read(r)?,
            prime_b: FlangerEngine::read(r)?,
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.flanger_type.to_byte());
        w.raw(self.note.to_byte());
        w.u8(self.direct_level, "flanger.direct_level")?;
        self.prime_g.write(w)?;
        self.prime_b.write(w)?;
        Ok(())
    }
}

// === PHASER (0x55..0x76) ===

/// One PHASER engine (PRIME G or PRIME B share this 14-byte layout).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaserEngine {
    pub depth: u8,
    pub resonance: u8,
    pub manual: u8,
    pub low_damp: u8,
    pub high_damp: u8,
    pub low_cut: u8,
    pub high_cut: u8,
    pub separation: u8,
    pub waveform: u8,
    pub input_sens: u8,
    pub polarity: u8,
    pub stage: u8,
    pub step_rate: u8,
    pub bi_phase: u8,
}

impl PhaserEngine {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            depth: r.u8(),
            resonance: r.u8(),
            manual: r.u8(),
            low_damp: r.u8(),
            high_damp: r.u8(),
            low_cut: r.u8(),
            high_cut: r.u8(),
            separation: r.u8(),
            waveform: r.u8(),
            input_sens: r.u8(),
            polarity: r.u8(),
            stage: r.u8(),
            step_rate: r.u8(),
            bi_phase: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        for (v, f) in [
            (self.depth, "phaser.depth"),
            (self.resonance, "phaser.resonance"),
            (self.manual, "phaser.manual"),
            (self.low_damp, "phaser.low_damp"),
            (self.high_damp, "phaser.high_damp"),
            (self.low_cut, "phaser.low_cut"),
            (self.high_cut, "phaser.high_cut"),
            (self.separation, "phaser.separation"),
            (self.waveform, "phaser.waveform"),
            (self.input_sens, "phaser.input_sens"),
            (self.polarity, "phaser.polarity"),
            (self.stage, "phaser.stage"),
            (self.step_rate, "phaser.step_rate"),
            (self.bi_phase, "phaser.bi_phase"),
        ] {
            w.u8(v, f)?;
        }
        Ok(())
    }
}

/// PHASER-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phaser {
    pub phaser_type: PhaserType,
    pub note: NoteValue,
    pub direct_level: u8,
    pub prime_g: PhaserEngine,
    pub prime_b: PhaserEngine,
    /// Reserved/dummy byte at 0x74.
    pub reserved_74: u8,
    /// SCRIPT modulation depth (0x75).
    pub script_depth: u8,
}

impl Phaser {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            phaser_type: PhaserType::from_byte(r.u8())?,
            note: r.note()?,
            direct_level: r.u8(),
            prime_g: PhaserEngine::read(r)?,
            prime_b: PhaserEngine::read(r)?,
            reserved_74: r.u8(),
            script_depth: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.phaser_type.to_byte());
        w.raw(self.note.to_byte());
        w.u8(self.direct_level, "phaser.direct_level")?;
        self.prime_g.write(w)?;
        self.prime_b.write(w)?;
        w.u8(self.reserved_74, "phaser.reserved_74")?;
        w.u8(self.script_depth, "phaser.script_depth")?;
        Ok(())
    }
}

// === C-VIBE (0x76..0x7A) ===

/// C-VIBE-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CVibe {
    pub cvibe_type: CVibeType,
    pub note: NoteValue,
    pub depth: u8,
    pub direct_level: u8,
}

impl CVibe {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            cvibe_type: CVibeType::from_byte(r.u8())?,
            note: r.note()?,
            depth: r.u8(),
            direct_level: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.cvibe_type.to_byte());
        w.raw(self.note.to_byte());
        w.u8(self.depth, "cvibe.depth")?;
        w.u8(self.direct_level, "cvibe.direct_level")?;
        Ok(())
    }
}

// === VIBRATO (0x7A..0x85) ===

/// VIBRATO-mode parameters (PRIME + SCANNER).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vibrato {
    pub vibrato_type: VibratoType,
    pub note: NoteValue,
    pub direct_level: u8,
    pub prime_depth: u8,
    pub prime_color: u8,
    pub trigger: u8,
    pub prime_rise_time: u8,
    pub prime_envelope_sens: u8,
    pub prime_waveform: u8,
    pub prime_input_sens: u8,
    /// SCANNER mode (0x84).
    pub scanner_mode: u8,
}

impl Vibrato {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            vibrato_type: VibratoType::from_byte(r.u8())?,
            note: r.note()?,
            direct_level: r.u8(),
            prime_depth: r.u8(),
            prime_color: r.u8(),
            trigger: r.u8(),
            prime_rise_time: r.u8(),
            prime_envelope_sens: r.u8(),
            prime_waveform: r.u8(),
            prime_input_sens: r.u8(),
            scanner_mode: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.vibrato_type.to_byte());
        w.raw(self.note.to_byte());
        for (v, f) in [
            (self.direct_level, "vibrato.direct_level"),
            (self.prime_depth, "vibrato.prime_depth"),
            (self.prime_color, "vibrato.prime_color"),
            (self.trigger, "vibrato.trigger"),
            (self.prime_rise_time, "vibrato.prime_rise_time"),
            (self.prime_envelope_sens, "vibrato.prime_envelope_sens"),
            (self.prime_waveform, "vibrato.prime_waveform"),
            (self.prime_input_sens, "vibrato.prime_input_sens"),
            (self.scanner_mode, "vibrato.scanner_mode"),
        ] {
            w.u8(v, f)?;
        }
        Ok(())
    }
}

// === TREMOLO (0x85..0x95) ===

/// TREMOLO-mode parameters (PRIME T / PRIME P / TWIN / DELUXE).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tremolo {
    pub tremolo_type: TremoloType,
    pub note: NoteValue,
    pub direct_level: u8,
    pub prime_t_depth: u8,
    pub reserved_89: u8,
    pub prime_t_rise_time: u8,
    pub prime_t_envelope_sens: u8,
    pub prime_t_waveform: u8,
    pub prime_t_input_sens: u8,
    pub prime_p_depth: u8,
    pub reserved_8f: u8,
    pub prime_p_rise_time: u8,
    pub prime_p_envelope_sens: u8,
    pub prime_p_waveform: u8,
    pub prime_p_input_sens: u8,
    pub twin_deluxe_intensity: u8,
}

impl Tremolo {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            tremolo_type: TremoloType::from_byte(r.u8())?,
            note: r.note()?,
            direct_level: r.u8(),
            prime_t_depth: r.u8(),
            reserved_89: r.u8(),
            prime_t_rise_time: r.u8(),
            prime_t_envelope_sens: r.u8(),
            prime_t_waveform: r.u8(),
            prime_t_input_sens: r.u8(),
            prime_p_depth: r.u8(),
            reserved_8f: r.u8(),
            prime_p_rise_time: r.u8(),
            prime_p_envelope_sens: r.u8(),
            prime_p_waveform: r.u8(),
            prime_p_input_sens: r.u8(),
            twin_deluxe_intensity: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.tremolo_type.to_byte());
        w.raw(self.note.to_byte());
        for (v, f) in [
            (self.direct_level, "tremolo.direct_level"),
            (self.prime_t_depth, "tremolo.prime_t_depth"),
            (self.reserved_89, "tremolo.reserved_89"),
            (self.prime_t_rise_time, "tremolo.prime_t_rise_time"),
            (self.prime_t_envelope_sens, "tremolo.prime_t_envelope_sens"),
            (self.prime_t_waveform, "tremolo.prime_t_waveform"),
            (self.prime_t_input_sens, "tremolo.prime_t_input_sens"),
            (self.prime_p_depth, "tremolo.prime_p_depth"),
            (self.reserved_8f, "tremolo.reserved_8f"),
            (self.prime_p_rise_time, "tremolo.prime_p_rise_time"),
            (self.prime_p_envelope_sens, "tremolo.prime_p_envelope_sens"),
            (self.prime_p_waveform, "tremolo.prime_p_waveform"),
            (self.prime_p_input_sens, "tremolo.prime_p_input_sens"),
            (self.twin_deluxe_intensity, "tremolo.twin_deluxe_intensity"),
        ] {
            w.u8(v, f)?;
        }
        Ok(())
    }
}

// === DIMENSION (0x95..0x9C) ===

/// DIMENSION-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimension {
    pub mode: DimensionMode,
    pub user_mode1_sw: bool,
    pub user_mode2_sw: bool,
    pub user_mode3_sw: bool,
    pub user_mode4_sw: bool,
    pub user_mode5_sw: bool,
    pub direct_level: u8,
}

impl Dimension {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            mode: DimensionMode::from_byte(r.u8())?,
            user_mode1_sw: r.boolean("dimension.user_mode1_sw")?,
            user_mode2_sw: r.boolean("dimension.user_mode2_sw")?,
            user_mode3_sw: r.boolean("dimension.user_mode3_sw")?,
            user_mode4_sw: r.boolean("dimension.user_mode4_sw")?,
            user_mode5_sw: r.boolean("dimension.user_mode5_sw")?,
            direct_level: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.mode.to_byte());
        w.boolean(self.user_mode1_sw);
        w.boolean(self.user_mode2_sw);
        w.boolean(self.user_mode3_sw);
        w.boolean(self.user_mode4_sw);
        w.boolean(self.user_mode5_sw);
        w.u8(self.direct_level, "dimension.direct_level")?;
        Ok(())
    }
}

// === RING MOD (0x9C..0xA3) ===

/// RING MODULATION-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RingMod {
    pub frequency: u8,
    /// Frequency-modulation rate (3 bytes).
    pub freq_mod_rate: u16,
    pub freq_mod_depth: u8,
    pub intelligent: u8,
    pub direct_level: u8,
}

impl RingMod {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            frequency: r.u8(),
            freq_mod_rate: r.nib(3) as u16,
            freq_mod_depth: r.u8(),
            intelligent: r.u8(),
            direct_level: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.u8(self.frequency, "ringmod.frequency")?;
        w.nib(self.freq_mod_rate as u32, 3);
        w.u8(self.freq_mod_depth, "ringmod.freq_mod_depth")?;
        w.u8(self.intelligent, "ringmod.intelligent")?;
        w.u8(self.direct_level, "ringmod.direct_level")?;
        Ok(())
    }
}

// === ROTARY (0xA3..0xB1) ===

/// ROTARY-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rotary {
    /// Speed selector (SLOW/FAST), raw.
    pub speed: u8,
    /// Slow rotor rate (3 bytes).
    pub slow_rate: u16,
    /// Fast rotor rate (3 bytes).
    pub fast_rate: u16,
    pub rise_time: u8,
    pub fall_time: u8,
    pub mic_distance: u8,
    /// Rotor/Horn balance (2 bytes).
    pub rotor_horn_balance: u8,
    pub drive: u8,
    pub direct_level: u8,
}

impl Rotary {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            speed: r.u8(),
            slow_rate: r.nib(3) as u16,
            fast_rate: r.nib(3) as u16,
            rise_time: r.u8(),
            fall_time: r.u8(),
            mic_distance: r.u8(),
            rotor_horn_balance: r.nib(2) as u8,
            drive: r.u8(),
            direct_level: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.u8(self.speed, "rotary.speed")?;
        w.nib(self.slow_rate as u32, 3);
        w.nib(self.fast_rate as u32, 3);
        w.u8(self.rise_time, "rotary.rise_time")?;
        w.u8(self.fall_time, "rotary.fall_time")?;
        w.u8(self.mic_distance, "rotary.mic_distance")?;
        w.nib(self.rotor_horn_balance as u32, 2);
        w.u8(self.drive, "rotary.drive")?;
        w.u8(self.direct_level, "rotary.direct_level")?;
        Ok(())
    }
}

// === FILTER (0xB1..0xE7) ===

/// One AUTO WAH engine (G or B).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoWah {
    pub depth: u8,
    pub frequency: u8,
    pub resonance: u8,
    pub filter_mode: u8,
    pub waveform: u8,
}

/// One TOUCH WAH engine (G or B).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TouchWah {
    pub filter_mode: u8,
    pub polarity: u8,
    pub sens: u8,
    pub frequency: u8,
    pub resonance: u8,
    pub decay: u8,
}

/// FILTER PATTERN engine.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterPattern {
    pub filter_mode: u8,
    pub pattern_type: u8,
    pub step_number: u8,
    pub resonance: u8,
    pub transition: u8,
    /// Per-step filter frequencies (24 steps).
    pub frequencies: Vec<u8>,
}

/// FILTER-mode parameters (A-WAH G/B, T-WAH G/B, PATTERN).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filter {
    pub filter_type: FilterType,
    pub note: NoteValue,
    pub direct_level: u8,
    pub auto_wah_g: AutoWah,
    pub auto_wah_b: AutoWah,
    pub touch_wah_g: TouchWah,
    pub touch_wah_b: TouchWah,
    pub pattern: FilterPattern,
}

impl Filter {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        let filter_type = FilterType::from_byte(r.u8())?;
        let note = r.note()?;
        let direct_level = r.u8();
        let auto_wah = |r: &mut Rd| AutoWah {
            depth: r.u8(),
            frequency: r.u8(),
            resonance: r.u8(),
            filter_mode: r.u8(),
            waveform: r.u8(),
        };
        let auto_wah_g = auto_wah(r);
        let auto_wah_b = auto_wah(r);
        let touch_wah = |r: &mut Rd| TouchWah {
            filter_mode: r.u8(),
            polarity: r.u8(),
            sens: r.u8(),
            frequency: r.u8(),
            resonance: r.u8(),
            decay: r.u8(),
        };
        let touch_wah_g = touch_wah(r);
        let touch_wah_b = touch_wah(r);
        let pattern = FilterPattern {
            filter_mode: r.u8(),
            pattern_type: r.u8(),
            step_number: r.u8(),
            resonance: r.u8(),
            transition: r.u8(),
            frequencies: (0..FILTER_PATTERN_STEPS).map(|_| r.u8()).collect(),
        };
        Ok(Self {
            filter_type,
            note,
            direct_level,
            auto_wah_g,
            auto_wah_b,
            touch_wah_g,
            touch_wah_b,
            pattern,
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.filter_type.to_byte());
        w.raw(self.note.to_byte());
        w.u8(self.direct_level, "filter.direct_level")?;
        for aw in [&self.auto_wah_g, &self.auto_wah_b] {
            for (v, f) in [
                (aw.depth, "filter.awah.depth"),
                (aw.frequency, "filter.awah.frequency"),
                (aw.resonance, "filter.awah.resonance"),
                (aw.filter_mode, "filter.awah.filter_mode"),
                (aw.waveform, "filter.awah.waveform"),
            ] {
                w.u8(v, f)?;
            }
        }
        for tw in [&self.touch_wah_g, &self.touch_wah_b] {
            for (v, f) in [
                (tw.filter_mode, "filter.twah.filter_mode"),
                (tw.polarity, "filter.twah.polarity"),
                (tw.sens, "filter.twah.sens"),
                (tw.frequency, "filter.twah.frequency"),
                (tw.resonance, "filter.twah.resonance"),
                (tw.decay, "filter.twah.decay"),
            ] {
                w.u8(v, f)?;
            }
        }
        let p = &self.pattern;
        for (v, f) in [
            (p.filter_mode, "filter.pattern.filter_mode"),
            (p.pattern_type, "filter.pattern.pattern_type"),
            (p.step_number, "filter.pattern.step_number"),
            (p.resonance, "filter.pattern.resonance"),
            (p.transition, "filter.pattern.transition"),
        ] {
            w.u8(v, f)?;
        }
        if p.frequencies.len() != FILTER_PATTERN_STEPS {
            return Err(CodecError::OutOfRange {
                field: "filter.pattern.frequencies",
                value: p.frequencies.len() as i32,
                valid: "exactly 24 entries",
            });
        }
        for v in &p.frequencies {
            w.u8(*v, "filter.pattern.frequency")?;
        }
        Ok(())
    }
}

// === SLICER (0xE7..0x167) ===

/// One SLICER user-pattern step.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlicerStep {
    pub length: u8,
    pub level: u8,
    pub band: u8,
    pub effect_level: u8,
    pub effect_pitch: u8,
}

/// SLICER-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slicer {
    pub note: NoteValue,
    pub pattern: u8,
    pub fx_type: u8,
    pub step_number: u8,
    /// 24 user-pattern steps.
    pub steps: Vec<SlicerStep>,
    pub attack: u8,
    pub duty: u8,
    pub direct_level: u8,
    pub output_mode: u8,
}

impl Slicer {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        let note = r.note()?;
        let pattern = r.u8();
        let fx_type = r.u8();
        let step_number = r.u8();
        let steps = (0..SLICER_STEPS)
            .map(|_| SlicerStep {
                length: r.u8(),
                level: r.u8(),
                band: r.u8(),
                effect_level: r.u8(),
                effect_pitch: r.u8(),
            })
            .collect();
        Ok(Self {
            note,
            pattern,
            fx_type,
            step_number,
            steps,
            attack: r.u8(),
            duty: r.u8(),
            direct_level: r.u8(),
            output_mode: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.note.to_byte());
        w.u8(self.pattern, "slicer.pattern")?;
        w.u8(self.fx_type, "slicer.fx_type")?;
        w.u8(self.step_number, "slicer.step_number")?;
        if self.steps.len() != SLICER_STEPS {
            return Err(CodecError::OutOfRange {
                field: "slicer.steps",
                value: self.steps.len() as i32,
                valid: "exactly 24 steps",
            });
        }
        for s in &self.steps {
            for (v, f) in [
                (s.length, "slicer.step.length"),
                (s.level, "slicer.step.level"),
                (s.band, "slicer.step.band"),
                (s.effect_level, "slicer.step.effect_level"),
                (s.effect_pitch, "slicer.step.effect_pitch"),
            ] {
                w.u8(v, f)?;
            }
        }
        w.u8(self.attack, "slicer.attack")?;
        w.u8(self.duty, "slicer.duty")?;
        w.u8(self.direct_level, "slicer.direct_level")?;
        w.u8(self.output_mode, "slicer.output_mode")?;
        Ok(())
    }
}

// === OVERTONE (0x167..0x173) ===

/// OVERTONE-mode parameters.
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overtone {
    pub overtone_type: OvertoneType,
    pub direct_level: u8,
    pub lower_level: u8,
    pub upper_level: u8,
    pub unison_level: u8,
    pub detune: u8,
    pub tone_low: u8,
    pub tone_high: u8,
    pub detune1_pitch: u8,
    pub detune1_level: u8,
    pub detune2_pitch: u8,
    pub detune2_level: u8,
}

impl Overtone {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            overtone_type: OvertoneType::from_byte(r.u8())?,
            direct_level: r.u8(),
            lower_level: r.u8(),
            upper_level: r.u8(),
            unison_level: r.u8(),
            detune: r.u8(),
            tone_low: r.u8(),
            tone_high: r.u8(),
            detune1_pitch: r.u8(),
            detune1_level: r.u8(),
            detune2_pitch: r.u8(),
            detune2_level: r.u8(),
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.overtone_type.to_byte());
        for (v, f) in [
            (self.direct_level, "overtone.direct_level"),
            (self.lower_level, "overtone.lower_level"),
            (self.upper_level, "overtone.upper_level"),
            (self.unison_level, "overtone.unison_level"),
            (self.detune, "overtone.detune"),
            (self.tone_low, "overtone.tone_low"),
            (self.tone_high, "overtone.tone_high"),
            (self.detune1_pitch, "overtone.detune1_pitch"),
            (self.detune1_level, "overtone.detune1_level"),
            (self.detune2_pitch, "overtone.detune2_pitch"),
            (self.detune2_level, "overtone.detune2_level"),
        ] {
            w.u8(v, f)?;
        }
        Ok(())
    }
}

// === per-patch control (0x173..0x180) ===

/// Per-patch footswitch / expression assignments (used when the matching System
/// Control `preference` is `PATCH`).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchControl {
    pub tap_ctl_function: CtlFunction,
    pub reserved_174: u8,
    pub ctl1_function: CtlFunction,
    pub ctl2_function: CtlFunction,
    pub exp_pedal_function: ExpFunction,
    /// Expression target min, display `-100..=10000` (4 bytes).
    pub exp_target_min: i32,
    /// Expression target max, display `-100..=10000` (4 bytes).
    pub exp_target_max: i32,
}

const EXP_TARGET_OFFSET: i64 = 100;
const EXP_TARGET_RAW_HI: u32 = 10100;

impl PatchControl {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            tap_ctl_function: CtlFunction::from_byte(r.u8())?,
            reserved_174: r.u8(),
            ctl1_function: CtlFunction::from_byte(r.u8())?,
            ctl2_function: CtlFunction::from_byte(r.u8())?,
            exp_pedal_function: ExpFunction::from_byte(r.u8())?,
            exp_target_min: r.nib_off(4, EXP_TARGET_OFFSET) as i32,
            exp_target_max: r.nib_off(4, EXP_TARGET_OFFSET) as i32,
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.raw(self.tap_ctl_function.to_byte());
        w.u8(self.reserved_174, "control.reserved_174")?;
        w.raw(self.ctl1_function.to_byte());
        w.raw(self.ctl2_function.to_byte());
        w.raw(self.exp_pedal_function.to_byte());
        w.nib_off(
            self.exp_target_min as i64,
            4,
            EXP_TARGET_OFFSET,
            EXP_TARGET_RAW_HI,
            "control.exp_target_min",
        )?;
        w.nib_off(
            self.exp_target_max as i64,
            4,
            EXP_TARGET_OFFSET,
            EXP_TARGET_RAW_HI,
            "control.exp_target_max",
        )?;
        Ok(())
    }
}

// === assign slots (0x180..0x248) ===

/// One of the eight ASSIGN slots (25 bytes). `source` (`0..=69`) and `target`
/// (`0..=264`) index large controller / parameter-target lists kept as integers;
/// `target_min`/`target_max` are the device's scaled target values (display
/// `-100..=2400000`).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assign {
    pub sw: bool,
    /// Source selector, `0..=69`.
    pub source: u8,
    pub source_mode: AssignSourceMode,
    /// Target parameter index, `0..=264` (3 bytes).
    pub target: u16,
    /// Target minimum, display `-100..=2400000` (6 bytes).
    pub target_min: i32,
    /// Target maximum, display `-100..=2400000` (6 bytes).
    pub target_max: i32,
    /// Active-range low, `0..=126`.
    pub active_range_lo: u8,
    /// Active-range high, `1..=127`.
    pub active_range_hi: u8,
    /// Wave rate, `0..=113` (0..100 plus note values).
    pub wave_rate: u8,
    pub wave_form: AssignWaveForm,
    /// Internal-pedal trigger source, `0..=68`.
    pub internal_pedal_trigger: u8,
    /// Internal-pedal time, `0..=100`.
    pub internal_pedal_time: u8,
    pub internal_pedal_curve: AssignPedalCurve,
}

const ASSIGN_TARGET_OFFSET: i64 = 100;
const ASSIGN_TARGET_RAW_HI: u32 = 2400100;

impl Assign {
    fn read(r: &mut Rd) -> Result<Self, CodecError> {
        Ok(Self {
            sw: r.boolean("assign.sw")?,
            source: r.u8(),
            source_mode: AssignSourceMode::from_byte(r.u8())?,
            target: r.nib(3) as u16,
            target_min: r.nib_off(6, ASSIGN_TARGET_OFFSET) as i32,
            target_max: r.nib_off(6, ASSIGN_TARGET_OFFSET) as i32,
            active_range_lo: r.u8(),
            active_range_hi: r.u8(),
            wave_rate: r.u8(),
            wave_form: AssignWaveForm::from_byte(r.u8())?,
            internal_pedal_trigger: r.u8(),
            internal_pedal_time: r.u8(),
            internal_pedal_curve: AssignPedalCurve::from_byte(r.u8())?,
        })
    }
    fn write(&self, w: &mut Wr) -> Result<(), CodecError> {
        w.boolean(self.sw);
        w.u8(self.source, "assign.source")?;
        w.raw(self.source_mode.to_byte());
        w.nib_ranged(self.target as u32, 3, 264, "assign.target")?;
        w.nib_off(
            self.target_min as i64,
            6,
            ASSIGN_TARGET_OFFSET,
            ASSIGN_TARGET_RAW_HI,
            "assign.target_min",
        )?;
        w.nib_off(
            self.target_max as i64,
            6,
            ASSIGN_TARGET_OFFSET,
            ASSIGN_TARGET_RAW_HI,
            "assign.target_max",
        )?;
        w.u8(self.active_range_lo, "assign.active_range_lo")?;
        w.u8(self.active_range_hi, "assign.active_range_hi")?;
        w.u8(self.wave_rate, "assign.wave_rate")?;
        w.raw(self.wave_form.to_byte());
        w.u8(self.internal_pedal_trigger, "assign.internal_pedal_trigger")?;
        w.u8(self.internal_pedal_time, "assign.internal_pedal_time")?;
        w.raw(self.internal_pedal_curve.to_byte());
        Ok(())
    }
}

// === the full patch ===

/// A complete MD-500 patch (586 bytes).
#[cfg_attr(feature = "tsify", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "tsify", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patch {
    /// Patch name, up to 16 ASCII characters.
    pub name: String,
    pub common: PatchCommon,
    pub chorus: Chorus,
    pub flanger: Flanger,
    pub phaser: Phaser,
    pub cvibe: CVibe,
    pub vibrato: Vibrato,
    pub tremolo: Tremolo,
    pub dimension: Dimension,
    pub ring_mod: RingMod,
    pub rotary: Rotary,
    pub filter: Filter,
    pub slicer: Slicer,
    pub overtone: Overtone,
    pub control: PatchControl,
    /// The eight assign slots.
    pub assigns: Vec<Assign>,
    /// Shared assign input sensitivity, `0..=100` (0x248).
    pub assign_input_sens: u8,
    /// Whether the LED reflects patch state (0x249).
    pub led_status: bool,
}

impl Patch {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() != PATCH_LEN {
            return Err(CodecError::WrongLength {
                expected: PATCH_LEN,
                actual: bytes.len(),
            });
        }
        let mut r = Rd::new(bytes);
        let name = read_ascii(&bytes[..PATCH_NAME_LEN], "name")?;
        r.i = PATCH_NAME_LEN;
        let patch = Self {
            name,
            common: PatchCommon::read(&mut r)?,
            chorus: Chorus::read(&mut r)?,
            flanger: Flanger::read(&mut r)?,
            phaser: Phaser::read(&mut r)?,
            cvibe: CVibe::read(&mut r)?,
            vibrato: Vibrato::read(&mut r)?,
            tremolo: Tremolo::read(&mut r)?,
            dimension: Dimension::read(&mut r)?,
            ring_mod: RingMod::read(&mut r)?,
            rotary: Rotary::read(&mut r)?,
            filter: Filter::read(&mut r)?,
            slicer: Slicer::read(&mut r)?,
            overtone: Overtone::read(&mut r)?,
            control: PatchControl::read(&mut r)?,
            assigns: (0..ASSIGN_COUNT)
                .map(|_| Assign::read(&mut r))
                .collect::<Result<Vec<_>, _>>()?,
            assign_input_sens: r.u8(),
            led_status: r.boolean("led_status")?,
        };
        debug_assert_eq!(r.i, PATCH_LEN, "patch decoder consumed wrong byte count");
        Ok(patch)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut w = Wr::new();
        w.b.extend_from_slice(&write_ascii(&self.name, PATCH_NAME_LEN, "name")?);
        self.common.write(&mut w)?;
        self.chorus.write(&mut w)?;
        self.flanger.write(&mut w)?;
        self.phaser.write(&mut w)?;
        self.cvibe.write(&mut w)?;
        self.vibrato.write(&mut w)?;
        self.tremolo.write(&mut w)?;
        self.dimension.write(&mut w)?;
        self.ring_mod.write(&mut w)?;
        self.rotary.write(&mut w)?;
        self.filter.write(&mut w)?;
        self.slicer.write(&mut w)?;
        self.overtone.write(&mut w)?;
        self.control.write(&mut w)?;
        if self.assigns.len() != ASSIGN_COUNT {
            return Err(CodecError::OutOfRange {
                field: "assigns",
                value: self.assigns.len() as i32,
                valid: "exactly 8 assign slots",
            });
        }
        for a in &self.assigns {
            a.write(&mut w)?;
        }
        w.u8(self.assign_input_sens, "assign_input_sens")?;
        w.boolean(self.led_status);
        if w.b.len() != PATCH_LEN {
            return Err(CodecError::OutOfRange {
                field: "patch",
                value: w.b.len() as i32,
                valid: "586 bytes",
            });
        }
        Ok(w.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A spec-shaped patch byte block: name "INIT" then deterministic values that
    /// satisfy every typed field's constraints.
    fn sample_bytes() -> Vec<u8> {
        let mut b = vec![0u8; PATCH_LEN];
        // name "INIT"
        b[0..4].copy_from_slice(b"INIT");
        for byte in &mut b[4..PATCH_NAME_LEN] {
            *byte = 0x20; // spaces
        }
        // common: mode=0, rate=500 (nibbles at 0x11..0x14), bpm=1200 (0x14..0x19)
        b[0x11..0x14].copy_from_slice(&write_nibbles(500, 3));
        b[0x14..0x19].copy_from_slice(&write_nibbles(1200, 5));
        b[0x19] = 12; // initial_phase
        b[0x1A] = 80; // effect_level
        b[0x1B] = 50; // low_level → 0
        b[0x1D] = 75; // high_level → +25
        b[0x20] = 1; // insert_sw = pre
        b[0x21] = 1; // output_mode = stereo
        b[0x22] = 6; // output_gain → 0
                     // chorus pre-delay (0x27..0x2A) = 200 ms
        b[0x27..0x2A].copy_from_slice(&write_nibbles(200, 3));
        // ring mod rate (0x9D..0xA0), rotary rates (0xA4.., 0xA7..), rotor/horn (0xAD..0xAF)
        b[0x9D..0xA0].copy_from_slice(&write_nibbles(100, 3));
        b[0xA4..0xA7].copy_from_slice(&write_nibbles(50, 3));
        b[0xA7..0xAA].copy_from_slice(&write_nibbles(900, 3));
        b[0xAD..0xAF].copy_from_slice(&write_nibbles(20, 2));
        // control exp targets (0x178.., 0x17C..): raw 100 → display 0
        b[0x178..0x17C].copy_from_slice(&write_nibbles(100, 4));
        b[0x17C..0x180].copy_from_slice(&write_nibbles(10100, 4));
        // each assign: target(3)+min(6)+max(6) need valid nibbles; default 0 is fine
        // but active_range_hi default 0 is below min 1 — set to 1 per slot.
        for s in 0..ASSIGN_COUNT {
            let base = 0x180 + s * 25;
            // target_min/max raw = 100 → display 0
            b[base + 6..base + 12].copy_from_slice(&write_nibbles(100, 6));
            b[base + 12..base + 18].copy_from_slice(&write_nibbles(100, 6));
            b[base + 0x13] = 1; // active_range_hi
        }
        b[0x248] = 100; // assign_input_sens
        b
    }

    #[test]
    fn full_patch_round_trips_byte_exact() {
        let bytes = sample_bytes();
        let p = Patch::from_bytes(&bytes).unwrap();
        assert_eq!(p.name, "INIT");
        assert_eq!(p.common.mode, ModulationMode::Chorus);
        assert_eq!(p.common.rate, 500);
        assert_eq!(p.common.bpm, 1200);
        assert_eq!(p.common.low_level, 0);
        assert_eq!(p.common.high_level, 25);
        assert_eq!(p.common.insert_sw, InsertSw::Pre);
        assert_eq!(p.chorus.prime_pre_delay, 200);
        assert_eq!(p.ring_mod.freq_mod_rate, 100);
        assert_eq!(p.rotary.slow_rate, 50);
        assert_eq!(p.rotary.fast_rate, 900);
        assert_eq!(p.control.exp_target_min, 0);
        assert_eq!(p.control.exp_target_max, 10000);
        assert_eq!(p.assigns.len(), 8);
        assert_eq!(p.assigns[0].target_min, 0);
        assert_eq!(p.filter.pattern.frequencies.len(), 24);
        assert_eq!(p.slicer.steps.len(), 24);
        assert_eq!(p.assign_input_sens, 100);
        // byte-exact round trip
        assert_eq!(p.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(matches!(
            Patch::from_bytes(&[0u8; 100]),
            Err(CodecError::WrongLength { .. })
        ));
    }

    #[test]
    fn yaml_round_trips() {
        let p = Patch::from_bytes(&sample_bytes()).unwrap();
        let y = serde_yaml::to_string(&p).unwrap();
        let back: Patch = serde_yaml::from_str(&y).unwrap();
        assert_eq!(p, back);
        assert_eq!(back.to_bytes().unwrap(), p.to_bytes().unwrap());
    }

    #[test]
    fn yaml_uses_named_enums() {
        let p = Patch::from_bytes(&sample_bytes()).unwrap();
        let y = serde_yaml::to_string(&p).unwrap();
        assert!(y.contains("mode: chorus"));
        assert!(y.contains("insert_sw: pre"));
        assert!(y.contains("output_mode: stereo"));
    }
}
