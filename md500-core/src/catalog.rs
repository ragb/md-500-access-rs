//! Value catalogs and editor-facing parameter metadata for the MD-500.
//!
//! Most "named" values in the MD-500's typed model (modes, types, channels,
//! note divisions, CC numbers) already serialize as their own name via serde
//! enums, so they need no lookup table. The exceptions are the two large
//! integer-coded assign lists — `source` (0..=69) and `target` (0..=264) — which
//! are exposed as the `assign_source` / `assign_target` name↔number catalogs (see
//! [`crate::assigncat`]) so a preset can be authored by name; the wire values
//! stay integers.
//!
//! The [`Params`] table carries the editor labels / help / group for each field,
//! keyed by an **area-prefixed serde path** (`system.common.*`, `patch.chorus.*`,
//! …), the same scheme the CK crate uses to keep the two areas that both contain
//! a `common` block from colliding. Help text is distilled from the MD-500
//! owner's and editor manuals and the MIDI implementation.
//!
//! [`Catalogs`]: midi_access_core::Catalogs

use midi_access_core::meta::{choice, Choice, Kind, Level, ParamMeta};
use midi_access_core::{Catalogs, Params};
use serde_yaml::{Mapping, Value};

/// The MD-500's name↔number value catalogs — the two big assign lists and the
/// EQ corner-frequency tables.
pub struct Md500Catalogs;

/// The shared singleton referenced by the [`Device`](midi_access_core::Device) impl.
pub static MD500_CATALOGS: Md500Catalogs = Md500Catalogs;

/// The name↔number catalogs the MD-500 exposes.
const CATALOG_NAMES: &[&str] = &[
    "assign_source",
    "assign_target",
    "eq_low_freq",
    "eq_high_freq",
    "cut_low_freq",
    "cut_high_freq",
    "filter_mode",
    "polarity",
    "lfo_waveform_6",
    "phaser_stage",
    "bi_phase",
    "pattern_type",
    "pattern_step_count",
    "slicer_step_band",
    "slicer_output_mode",
    "slicer_fx_type",
    "rotary_speed",
    "tricho_lfo_mode",
    "scanner_mode",
    "onoff",
    "step_rate_note",
    "separation_deg",
    "waveform_10",
    "envelope_input",
    "slicer_pattern",
];

/// Low-band EQ corner-frequency labels (index 0..=16) — exactly the values
/// the device's Low Freq knob steps through. See MIDI Implementation §0x1C.
pub const EQ_LOW_FREQS: [&str; 17] = [
    "20.0 Hz", "25.0 Hz", "31.5 Hz", "40.0 Hz", "50.0 Hz", "63.0 Hz", "80.0 Hz", "100 Hz",
    "125 Hz", "160 Hz", "200 Hz", "250 Hz", "315 Hz", "400 Hz", "500 Hz", "630 Hz", "800 Hz",
];

/// High-band EQ corner-frequency labels (index 0..=14). See MIDI Implementation §0x1E.
pub const EQ_HIGH_FREQS: [&str; 15] = [
    "630 Hz", "800 Hz", "1.00 kHz", "1.25 kHz", "1.60 kHz", "2.00 kHz", "2.50 kHz", "3.15 kHz",
    "4.00 kHz", "5.00 kHz", "6.30 kHz", "8.00 kHz", "10.0 kHz", "12.5 kHz", "16.0 kHz",
];

/// Low-cut corner-frequency labels (index 0..=17) used by Chorus / Flanger /
/// Phaser `prime_*.low_cut` and `chorus.prime_low_cut`. Same Hz list as
/// [`EQ_LOW_FREQS`] but prefixed with `FLAT` (= no cut).
pub const CUT_LOW_FREQS: [&str; 18] = [
    "FLAT", "20.0 Hz", "25.0 Hz", "31.5 Hz", "40.0 Hz", "50.0 Hz", "63.0 Hz", "80.0 Hz", "100 Hz",
    "125 Hz", "160 Hz", "200 Hz", "250 Hz", "315 Hz", "400 Hz", "500 Hz", "630 Hz", "800 Hz",
];

/// High-cut corner-frequency labels (index 0..=15). Same as [`EQ_HIGH_FREQS`]
/// suffixed with `FLAT` (= no cut).
pub const CUT_HIGH_FREQS: [&str; 16] = [
    "630 Hz", "800 Hz", "1.00 kHz", "1.25 kHz", "1.60 kHz", "2.00 kHz", "2.50 kHz", "3.15 kHz",
    "4.00 kHz", "5.00 kHz", "6.30 kHz", "8.00 kHz", "10.0 kHz", "12.5 kHz", "16.0 kHz", "FLAT",
];

/// Filter-mode labels (LPF / HPF / BPF), shared by every filter sub-engine.
/// Index 0..=2.
pub const FILTER_MODES: [&str; 3] = ["LPF", "HPF", "BPF"];

/// Modulation polarity, index 0..=1.
pub const POLARITY: [&str; 2] = ["DOWN", "UP"];

/// The 6-entry LFO waveform list used by the Filter auto-wah engines.
/// Index 0..=5.
pub const LFO_WAVEFORM_6: [&str; 6] = ["SIN", "TRI", "SQR", "SAW-UP", "SAW-DOWN", "RAMP"];

/// Phaser stage count: 2 / 4 / 8 / 16 / 24 stages. Index 0..=4.
pub const PHASER_STAGE: [&str; 5] = ["2", "4", "8", "16", "24"];

/// Phaser bi-phase mode (just on/off). Index 0..=1. Spec §0x65 / §0x73.
pub const BI_PHASE: [&str; 2] = ["OFF", "ON"];

/// Filter PATTERN type — 10 factory patterns plus USER. Index 0..=10.
pub const PATTERN_TYPE: [&str; 11] = [
    "PAT1", "PAT2", "PAT3", "PAT4", "PAT5", "PAT6", "PAT7", "PAT8", "PAT9", "PAT10", "USER",
];

/// Filter PATTERN step count: 8 / 12 / 16 / 24 active steps. Index 0..=3.
pub const PATTERN_STEP_COUNT: [&str; 4] = ["8", "12", "16", "24"];

/// Slicer per-step frequency band. Index 0..=6.
pub const SLICER_STEP_BAND: [&str; 7] =
    ["THRU", "BAND1", "BAND2", "BAND3", "BAND4", "BAND5", "BAND6"];

/// Slicer output mode. Index 0..=4.
pub const SLICER_OUTPUT_MODE: [&str; 5] = ["MONO", "FIXED", "RANDOM", "PingPong", "AUTO"];

/// Slicer FX type. Index 0..=6.
pub const SLICER_FX_TYPE: [&str; 7] = [
    "OFF", "PITCH", "FLANGER", "PHASER", "SWEEP", "FILTER", "RING",
];

/// Rotary speed (slow / fast rotor). Index 0..=1.
pub const ROTARY_SPEED: [&str; 2] = ["SLOW", "FAST"];

/// Chorus TRI-CHO LFO mode (PRESET / MANUAL / both). Index 0..=2.
pub const TRICHO_LFO_MODE: [&str; 3] = ["PRESET", "MANUAL", "P+M"];

/// Vibrato SCANNER mode — Univibe-style scanner positions, Vibrato 1..3 and
/// Chorus 1..3. Index 0..=5.
pub const SCANNER_MODE: [&str; 6] = ["V1", "V2", "V3", "C1", "C2", "C3"];

/// Generic OFF / ON labels, for 1-bit fields that act as a toggle but are
/// typed as `u8` on the wire (e.g. Vibrato `trigger`). Index 0..=1.
pub const ONOFF: [&str; 2] = ["OFF", "ON"];

/// Step rate as a note division, with `OFF` at index 0 followed by note
/// divisions from `whole` down to `32nd`. Used by Flanger/Phaser step-rate
/// fields. Index 0..=16.
pub const STEP_RATE_NOTE: [&str; 17] = [
    "OFF",
    "whole",
    "dotted half",
    "whole triplet",
    "half",
    "dotted quarter",
    "half triplet",
    "quarter",
    "dotted 8th",
    "quarter triplet",
    "8th",
    "dotted 16th",
    "8th triplet",
    "16th",
    "dotted 32nd",
    "16th triplet",
    "32nd",
];

/// Stereo separation in 15° steps from 0° to 180° — Flanger/Phaser engine
/// separation field. Index 0..=12.
pub const SEPARATION_DEG: [&str; 13] = [
    "0deg", "15deg", "30deg", "45deg", "60deg", "75deg", "90deg", "105deg", "120deg", "135deg",
    "150deg", "165deg", "180deg",
];

/// LFO waveform numeric labels — wire 0..=9 displays as 1..=10. Used by Chorus
/// PRIME, Flanger/Phaser/Vibrato waveform fields per spec.
pub const WAVEFORM_10: [&str; 10] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];

/// Ring Modulator / Tremolo `intelligent` envelope-input selector — OFF or
/// tuned to a guitar / bass signal range. Index 0..=2.
pub const ENVELOPE_INPUT: [&str; 3] = ["OFF", "GUITAR", "BASS"];

/// Slicer pattern selector: 30 preset patterns (P1..P30), 20 "hold" patterns
/// (H1..H20), plus USER. Index 0..=50. Spec §0x68.
pub const SLICER_PATTERN: [&str; 51] = [
    "P1", "P2", "P3", "P4", "P5", "P6", "P7", "P8", "P9", "P10", "P11", "P12", "P13", "P14", "P15",
    "P16", "P17", "P18", "P19", "P20", "P21", "P22", "P23", "P24", "P25", "P26", "P27", "P28",
    "P29", "P30", "H1", "H2", "H3", "H4", "H5", "H6", "H7", "H8", "H9", "H10", "H11", "H12", "H13",
    "H14", "H15", "H16", "H17", "H18", "H19", "H20", "USER",
];

fn lookup_index(list: &[&str], name: &str) -> Option<i64> {
    list.iter().position(|&s| s == name).map(|i| i as i64)
}

/// Dispatch table for the simple index-keyed string catalogs in this file
/// (everything except the two big assign lists, which live in `assigncat`).
const SIMPLE_LISTS: &[(&str, &[&str])] = &[
    ("eq_low_freq", &EQ_LOW_FREQS),
    ("eq_high_freq", &EQ_HIGH_FREQS),
    ("cut_low_freq", &CUT_LOW_FREQS),
    ("cut_high_freq", &CUT_HIGH_FREQS),
    ("filter_mode", &FILTER_MODES),
    ("polarity", &POLARITY),
    ("lfo_waveform_6", &LFO_WAVEFORM_6),
    ("phaser_stage", &PHASER_STAGE),
    ("bi_phase", &BI_PHASE),
    ("pattern_type", &PATTERN_TYPE),
    ("pattern_step_count", &PATTERN_STEP_COUNT),
    ("slicer_step_band", &SLICER_STEP_BAND),
    ("slicer_output_mode", &SLICER_OUTPUT_MODE),
    ("slicer_fx_type", &SLICER_FX_TYPE),
    ("rotary_speed", &ROTARY_SPEED),
    ("tricho_lfo_mode", &TRICHO_LFO_MODE),
    ("scanner_mode", &SCANNER_MODE),
    ("onoff", &ONOFF),
    ("step_rate_note", &STEP_RATE_NOTE),
    ("separation_deg", &SEPARATION_DEG),
    ("waveform_10", &WAVEFORM_10),
    ("envelope_input", &ENVELOPE_INPUT),
    ("slicer_pattern", &SLICER_PATTERN),
];

fn simple_list(cat: &str) -> Option<&'static [&'static str]> {
    SIMPLE_LISTS
        .iter()
        .find(|(n, _)| *n == cat)
        .map(|(_, l)| *l)
}

impl Catalogs for Md500Catalogs {
    fn resolve(&self, cat: &str, name: &str) -> Option<i64> {
        match cat {
            "assign_source" => crate::assigncat::source_index(name),
            "assign_target" => crate::assigncat::target_index(name),
            other => simple_list(other).and_then(|l| lookup_index(l, name)),
        }
    }
    fn label(&self, cat: &str, value: i64) -> Option<String> {
        let i = usize::try_from(value).ok()?;
        match cat {
            "assign_source" => crate::assigncat::source_name(i),
            "assign_target" => crate::assigncat::target_name(i).map(str::to_string),
            other => simple_list(other).and_then(|l| l.get(i).map(|s| s.to_string())),
        }
    }
    fn names(&self) -> &[&str] {
        CATALOG_NAMES
    }
    fn as_value(&self) -> Value {
        use serde_yaml::Sequence;
        let to_seq =
            |xs: &[&str]| -> Sequence { xs.iter().map(|s| Value::String(s.to_string())).collect() };
        let src: Sequence = (0..crate::assigncat::SOURCE_COUNT)
            .filter_map(crate::assigncat::source_name)
            .map(Value::String)
            .collect();
        let tgt: Sequence = crate::assigncat::ASSIGN_TARGETS
            .iter()
            .map(|s| Value::String(s.to_string()))
            .collect();
        let mut m = Mapping::new();
        m.insert(Value::String("assign_source".into()), Value::Sequence(src));
        m.insert(Value::String("assign_target".into()), Value::Sequence(tgt));
        for (name, list) in SIMPLE_LISTS {
            m.insert(Value::String((*name).into()), Value::Sequence(to_seq(list)));
        }
        Value::Mapping(m)
    }
}

// === ParamMeta constructors (mirroring ck-core's `p` / `pl`) ===

/// A plain field.
const fn p(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
) -> ParamMeta {
    ParamMeta {
        path,
        label,
        group,
        help,
        level: Level::Plain,
        kind: None,
        catalog: None,
    }
}

/// A `0..N` magnitude (shown as a percentage by the editor).
const fn pl(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
) -> ParamMeta {
    ParamMeta {
        path,
        label,
        group,
        help,
        level: Level::Magnitude,
        kind: None,
        catalog: None,
    }
}

/// A field carrying an explicit [`Kind`] hint (range / choice / toggle).
const fn pk(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
    kind: Kind,
) -> ParamMeta {
    ParamMeta {
        path,
        label,
        group,
        help,
        level: Level::Plain,
        kind: Some(kind),
        catalog: None,
    }
}

/// A field tagged with a name↔number `catalog` (drives the editor picker and
/// `resolve`/`label`).
const fn pc(
    path: &'static str,
    label: &'static str,
    group: &'static str,
    help: &'static str,
    catalog: &'static str,
) -> ParamMeta {
    ParamMeta {
        path,
        label,
        group,
        help,
        level: Level::Plain,
        kind: None,
        catalog: Some(catalog),
    }
}

const MODE_CHOICES: &[Choice] = &[
    choice("chorus", "Chorus"),
    choice("flanger", "Flanger"),
    choice("phaser", "Phaser"),
    choice("c_vibe", "C-Vibe"),
    choice("vibrato", "Vibrato"),
    choice("tremolo", "Tremolo"),
    choice("dimension", "Dimension"),
    choice("ring_mod", "Ring Modulator"),
    choice("rotary", "Rotary"),
    choice("filter", "Filter"),
    choice("slicer", "Slicer"),
    choice("overtone", "Overtone"),
];

/// The full editor-facing parameter table, ordered by area then struct-field
/// order. Paths are area-prefixed and match the typed serde structures; repeated
/// sub-structures (the eight assigns, the 24 slicer steps) use one representative
/// path that the editor reuses for every instance.
#[rustfmt::skip]
static PARAMS: &[ParamMeta] = &[
    // ===== Setup =====
    p("setup.current_patch", "Current patch", "Setup", "The patch the pedal is currently on (bank 1–99, slot A/B/C). Writing it recalls that patch."),

    // ===== System / Common =====
    p("system.common.insert_loop_sw", "Insert loop", "System", "Routes the effect through the external send/return (insert) loop."),
    pk("system.common.bank_wait_mode", "Bank wait mode", "System", "WAIT holds the patch change until you confirm; IMMEDIATE switches at once.", Kind::Choice(&[choice("wait","Wait"),choice("immediate","Immediate")])),
    pk("system.common.bank_extent_begin", "Bank range start", "System", "First bank reachable with BANK UP/DOWN (1–99).", Kind::Range{min:1,max:99,unit:None}),
    pk("system.common.bank_extent_end", "Bank range end", "System", "Last bank reachable with BANK UP/DOWN (1–99).", Kind::Range{min:1,max:99,unit:None}),
    p("system.common.knob_lock", "Knob lock", "System", "Disables the panel knobs so a stage bump can't change the sound."),
    pk("system.common.knob_mode", "Knob mode", "System", "IMMEDIATE jumps to the knob value; HOOK waits until the knob passes the stored value.", Kind::Choice(&[choice("immediate","Immediate"),choice("hook","Hook")])),
    pk("system.common.bypass_mode", "Bypass mode", "System", "BUFFERED keeps the signal buffered when bypassed; TRUE is a hard-wire bypass.", Kind::Choice(&[choice("buffered","Buffered"),choice("true","True")])),
    pk("system.common.pedal_action", "Pedal action", "System", "Whether a momentary control fires on PUSH or on RELEASE.", Kind::Choice(&[choice("push","Push"),choice("release","Release")])),
    pk("system.common.foot_switch_mode", "Footswitch mode", "System", "How the onboard footswitches behave: NORMAL, A/B/C patch select, A/B SIMUL (run two at once), or SW DN/UP.", Kind::Choice(&[choice("normal","Normal"),choice("abc","A/B/C"),choice("ab_simul","A/B Simul"),choice("sw_dn_up","Sw Dn/Up")])),
    pk("system.common.display_type", "Display", "System", "What the main display shows by default: time, BPM, patch, or the edited parameter.", Kind::Choice(&[choice("time","Time"),choice("bpm","BPM"),choice("patch","Patch"),choice("param","Param")])),
    p("system.common.rx_channel", "Receive channel", "MIDI", "MIDI channel the pedal responds to (1–16, or Off)."),
    p("system.common.tx_channel", "Transmit channel", "MIDI", "MIDI channel the pedal sends on (1–16, Rx to follow the receive channel, or Off)."),
    p("system.common.pc_in", "Program Change in", "MIDI", "Receive Program Change messages to switch patches."),
    p("system.common.pc_out", "Program Change out", "MIDI", "Send a Program Change when the patch changes."),
    pk("system.common.bank_select_out", "Bank Select out", "MIDI", "Which bank-select bytes are sent: MSB only, or MSB+LSB.", Kind::Choice(&[choice("msb","MSB"),choice("msb_lsb","MSB+LSB")])),
    p("system.common.cc_in", "Control Change in", "MIDI", "Receive Control Change messages to move the assignable controls."),
    p("system.common.cc_out", "Control Change out", "MIDI", "Send a Control Change when a knob or control moves."),
    p("system.common.rate_cc", "Rate CC", "MIDI", "CC number that drives the RATE/VALUE knob (Off, CC#1–31, CC#64–95)."),
    p("system.common.depth_cc", "Depth CC", "MIDI", "CC number that drives the DEPTH knob."),
    p("system.common.effect_level_cc", "Effect Level CC", "MIDI", "CC number that drives the E.LEVEL knob."),
    p("system.common.param1_cc", "Param 1 CC", "MIDI", "CC number that drives the PARAM 1 knob."),
    p("system.common.param2_cc", "Param 2 CC", "MIDI", "CC number that drives the PARAM 2 knob."),
    p("system.common.effect_cc", "Effect on/off CC", "MIDI", "CC number that toggles the effect."),
    p("system.common.effect_a_cc", "Effect A on/off CC", "MIDI", "CC number that toggles effect A (SIMUL)."),
    p("system.common.effect_b_cc", "Effect B on/off CC", "MIDI", "CC number that toggles effect B (SIMUL)."),
    p("system.common.ctl1_cc", "CTL1 CC", "MIDI", "CC number for the external CTL1 control."),
    p("system.common.ctl2_cc", "CTL2 CC", "MIDI", "CC number for the external CTL2 control."),
    p("system.common.exp_cc", "Expression CC", "MIDI", "CC number for the expression pedal."),
    pk("system.common.sync", "Tempo sync", "MIDI", "Clock source for tempo: internal, external USB/MIDI clock, or auto.", Kind::Choice(&[choice("internal","Internal"),choice("ext_usb","Ext (USB)"),choice("ext_midi","Ext (MIDI)"),choice("auto","Auto")])),
    pk("system.common.realtime_out", "Realtime out", "MIDI", "Where MIDI realtime (clock/start/stop) messages are sent.", Kind::Choice(&[choice("int","Internal"),choice("usb","USB"),choice("midi","MIDI")])),
    pk("system.common.midi_thru", "MIDI IN→OUT thru", "MIDI", "Echo data arriving at MIDI IN to which outputs.", Kind::Choice(&[choice("off","Off"),choice("usb","USB"),choice("midi","MIDI"),choice("usb_midi","USB+MIDI")])),
    pk("system.common.usb_thru", "USB IN→OUT thru", "MIDI", "Echo data arriving at USB IN to which outputs.", Kind::Choice(&[choice("off","Off"),choice("usb","USB"),choice("midi","MIDI"),choice("usb_midi","USB+MIDI")])),
    p("system.common.simul_effect_sw_a", "SIMUL A power-on state", "System", "Whether patch A's effect is on at power-up in SIMUL mode."),
    p("system.common.simul_effect_sw_b", "SIMUL B power-on state", "System", "Whether patch B's effect is on at power-up in SIMUL mode."),

    // ===== System / Control =====
    pk("system.control.tap_ctl_preference", "TAP/CTL follows", "Control", "Whether the TAP/CTL switch uses the patch's setting or the system setting.", Kind::Choice(&[choice("patch","Patch"),choice("system","System")])),
    p("system.control.tap_ctl_function", "TAP/CTL function", "Control", "Function of the TAP/CTL footswitch (Off, Tap, Reset, Moment, Bank Up, Bank Down)."),
    pk("system.control.ctl1_preference", "CTL1 follows", "Control", "Whether CTL1 uses the patch's setting or the system setting.", Kind::Choice(&[choice("patch","Patch"),choice("system","System")])),
    p("system.control.ctl1_function", "CTL1 function", "Control", "Function of the external CTL1 footswitch."),
    pk("system.control.ctl2_preference", "CTL2 follows", "Control", "Whether CTL2 uses the patch's setting or the system setting.", Kind::Choice(&[choice("patch","Patch"),choice("system","System")])),
    p("system.control.ctl2_function", "CTL2 function", "Control", "Function of the external CTL2 footswitch."),
    pk("system.control.exp_preference", "EXP follows", "Control", "Whether the expression pedal uses the patch's setting or the system setting.", Kind::Choice(&[choice("patch","Patch"),choice("system","System")])),
    p("system.control.exp_pedal_function", "Expression function", "Control", "What the expression pedal controls (Off, Rate, Depth, E.Level, Param 1, Param 2)."),
    p("system.control.exp_target_min", "Expression min", "Control", "Value the expression target reaches at heel position (display −100…23000)."),
    p("system.control.exp_target_max", "Expression max", "Control", "Value the expression target reaches at toe position (display −100…23000)."),

    // ===== Bank (SIMUL) =====
    pk("bank.structure", "Structure", "Bank", "How the two SIMUL effects are chained: in series or in parallel.", Kind::Choice(&[choice("series","Series"),choice("parallel","Parallel")])),
    pk("bank.insert_position", "Insert position", "Bank", "Where the bank's effects sit relative to the insert loop.", Kind::Choice(&[choice("off","Off"),choice("pre","Pre"),choice("post","Post"),choice("middle","Middle")])),
    pk("bank.output_mode", "Output mode", "Bank", "MIX sums both effects; A/B routes them to separate outputs.", Kind::Choice(&[choice("mix","Mix"),choice("ab","A/B")])),
    p("bank.ab_sync_sw", "A/B sync", "Bank", "Synchronise the tempo/LFO of effects A and B."),

    // ===== Program Change map =====
    p("pc_map.entries", "PC map", "Program Change", "Maps each incoming Program Change (across 3 bank-select banks) to a stored patch."),

    // ===== Patch / Common =====
    pk("patch.common.mode", "Mode", "Modulation", "The active modulation algorithm. Each mode has its own parameter set; only the selected mode is audible.", Kind::Choice(MODE_CHOICES)),
    pk("patch.common.rate", "Rate", "Modulation", "Modulation LFO rate, 0.01–20.00 Hz (stored 1–2000). The RATE/VALUE knob.", Kind::Range{min:1,max:2000,unit:Some("0.01Hz")}),
    pk("patch.common.bpm", "BPM", "Modulation", "Tempo used when Rate is set by note value, ♩0.1–9600.0 (stored 1–96000).", Kind::Range{min:1,max:96000,unit:None}),
    pk("patch.common.initial_phase", "Initial phase", "Modulation", "LFO start phase in 15° steps (0–345°).", Kind::Range{min:0,max:23,unit:Some("15deg")}),
    pl("patch.common.effect_level", "Effect Level", "Modulation", "Wet (effect) signal level. The E.LEVEL knob."),
    pk("patch.common.low_level", "Low EQ level", "EQ", "Low-band cut/boost, −50…+50.", Kind::Range{min:-50,max:50,unit:None}),
    pc("patch.common.low_freq", "Low EQ freq", "EQ", "Low-band corner frequency: 20 Hz to 800 Hz.", "eq_low_freq"),
    pk("patch.common.high_level", "High EQ level", "EQ", "High-band cut/boost, −50…+50.", Kind::Range{min:-50,max:50,unit:None}),
    pc("patch.common.high_freq", "High EQ freq", "EQ", "High-band corner frequency: 630 Hz to 16 kHz.", "eq_high_freq"),
    p("patch.common.tempo_hold", "Tempo hold", "Modulation", "Keep the current tempo (and Rate) when switching patches."),
    pk("patch.common.insert_sw", "Insert", "Output", "Where the effect sits in the chain.", Kind::Choice(&[choice("off","Off"),choice("pre","Pre"),choice("post","Post")])),
    pk("patch.common.output_mode", "Output", "Output", "Mono or stereo output.", Kind::Choice(&[choice("mono","Mono"),choice("stereo","Stereo")])),
    pk("patch.common.output_gain", "Output gain", "Output", "Output trim, −6…+6 dB.", Kind::Range{min:-6,max:6,unit:Some("dB")}),

    // ===== Patch / Chorus =====
    pk("patch.chorus.chorus_type", "Chorus type", "Chorus", "PRIME is the MD-500's own chorus; CE-1 CHORUS/VIBRATO model the Boss CE-1; TRI-CHO is the 3-phase chorus that took the '80s by storm.", Kind::Choice(&[choice("prime","Prime"),choice("ce1_chorus","CE-1 Chorus"),choice("ce1_vibrato","CE-1 Vibrato"),choice("tri_cho","Tri-Cho")])),
    p("patch.chorus.note", "Note", "Chorus", "Sets the rate from the tempo as a note value."),
    pl("patch.chorus.direct_level", "Direct level", "Chorus", "Level of the dry signal blended with the effect."),
    pl("patch.chorus.prime_depth", "PRIME depth", "Chorus", "Modulation depth of the PRIME chorus."),
    pk("patch.chorus.prime_pre_delay", "PRIME pre-delay", "Chorus", "Pre-delay before the chorus voice, 0–400 ms.", Kind::Range{min:0,max:400,unit:Some("ms")}),
    pc("patch.chorus.prime_waveform", "PRIME waveform", "Chorus", "Modulation waveform of the PRIME chorus.", "waveform_10"),
    pl("patch.chorus.prime_sweetness", "PRIME sweetness", "Chorus", "Smooths and thickens the PRIME chorus voice."),
    pl("patch.chorus.prime_bell", "PRIME bell", "Chorus", "Adds bell-like shimmer to the PRIME chorus."),
    pc("patch.chorus.prime_low_cut", "PRIME low cut", "Chorus", "Low-cut filter on the PRIME chorus.", "cut_low_freq"),
    pc("patch.chorus.prime_high_cut", "PRIME high cut", "Chorus", "High-cut filter on the PRIME chorus.", "cut_high_freq"),
    pl("patch.chorus.ce1_depth", "CE-1 depth", "Chorus", "Modulation depth of the CE-1 chorus/vibrato model."),
    p("patch.chorus.ce1_preamp_sw", "CE-1 preamp", "Chorus", "Engage the CE-1 preamp colouring."),
    p("patch.chorus.ce1_preamp_gain", "CE-1 preamp gain", "Chorus", "Drive of the CE-1 preamp (0–99)."),
    pl("patch.chorus.ce1_preamp_level", "CE-1 preamp level", "Chorus", "Output level of the CE-1 preamp."),
    pc("patch.chorus.tricho_lfo_mode", "Tri-Cho LFO mode", "Chorus", "LFO mode of the 3-phase chorus.", "tricho_lfo_mode"),
    pl("patch.chorus.tricho_intensity1", "Tri-Cho intensity 1", "Chorus", "Depth of the 3-phase chorus voice 1."),
    pl("patch.chorus.tricho_intensity2", "Tri-Cho intensity 2", "Chorus", "Depth of the 3-phase chorus voice 2."),
    pl("patch.chorus.tricho_intensity3", "Tri-Cho intensity 3", "Chorus", "Depth of the 3-phase chorus voice 3."),
    p("patch.chorus.tricho_bright", "Tri-Cho bright", "Chorus", "Brightens the 3-phase chorus."),

    // ===== Patch / Flanger (PRIME G and PRIME B share the engine layout) =====
    pk("patch.flanger.flanger_type", "Flanger type", "Flanger", "PRIME G and PRIME B are two voicings of the flanger.", Kind::Choice(&[choice("prime_g","Prime G"),choice("prime_b","Prime B")])),
    p("patch.flanger.note", "Note", "Flanger", "Sets the rate from the tempo as a note value."),
    pl("patch.flanger.direct_level", "Direct level", "Flanger", "Level of the dry signal blended with the effect."),
    pl("patch.flanger.prime_g.depth", "G depth", "Flanger", "Modulation depth (PRIME G)."),
    p("patch.flanger.prime_g.resonance", "G resonance", "Flanger", "Feedback/resonance of the flange comb (PRIME G)."),
    p("patch.flanger.prime_g.manual", "G manual", "Flanger", "Centre frequency of the flange sweep (PRIME G)."),
    p("patch.flanger.prime_g.turbo_sw", "G turbo", "Flanger", "Turbo makes the flange more intense (PRIME G)."),
    p("patch.flanger.prime_g.low_damp", "G low damp", "Flanger", "Damps the low end of the flange (PRIME G)."),
    p("patch.flanger.prime_g.high_damp", "G high damp", "Flanger", "Damps the high end of the flange (PRIME G)."),
    pc("patch.flanger.prime_g.low_cut", "G low cut", "Flanger", "Low-cut filter (PRIME G).", "cut_low_freq"),
    pc("patch.flanger.prime_g.high_cut", "G high cut", "Flanger", "High-cut filter (PRIME G).", "cut_high_freq"),
    pc("patch.flanger.prime_g.separation", "G separation", "Flanger", "Stereo spread of the flange (PRIME G).", "separation_deg"),
    pc("patch.flanger.prime_g.step_rate", "G step rate", "Flanger", "Step-modulation rate; OFF for smooth sweep (PRIME G).", "step_rate_note"),
    pc("patch.flanger.prime_g.waveform", "G waveform", "Flanger", "Modulation waveform (PRIME G).", "waveform_10"),
    p("patch.flanger.prime_g.input_sens", "G input sens", "Flanger", "Input sensitivity for envelope response (PRIME G)."),
    pc("patch.flanger.prime_g.polarity", "G polarity", "Flanger", "Modulation polarity (PRIME G).", "polarity"),
    pl("patch.flanger.prime_b.depth", "B depth", "Flanger", "Modulation depth (PRIME B)."),
    p("patch.flanger.prime_b.resonance", "B resonance", "Flanger", "Feedback/resonance of the flange comb (PRIME B)."),
    p("patch.flanger.prime_b.manual", "B manual", "Flanger", "Centre frequency of the flange sweep (PRIME B)."),
    p("patch.flanger.prime_b.turbo_sw", "B turbo", "Flanger", "Turbo makes the flange more intense (PRIME B)."),
    p("patch.flanger.prime_b.low_damp", "B low damp", "Flanger", "Damps the low end of the flange (PRIME B)."),
    p("patch.flanger.prime_b.high_damp", "B high damp", "Flanger", "Damps the high end of the flange (PRIME B)."),
    pc("patch.flanger.prime_b.low_cut", "B low cut", "Flanger", "Low-cut filter (PRIME B).", "cut_low_freq"),
    pc("patch.flanger.prime_b.high_cut", "B high cut", "Flanger", "High-cut filter (PRIME B).", "cut_high_freq"),
    pc("patch.flanger.prime_b.separation", "B separation", "Flanger", "Stereo spread of the flange (PRIME B).", "separation_deg"),
    pc("patch.flanger.prime_b.step_rate", "B step rate", "Flanger", "Step-modulation rate; OFF for smooth sweep (PRIME B).", "step_rate_note"),
    pc("patch.flanger.prime_b.waveform", "B waveform", "Flanger", "Modulation waveform (PRIME B).", "waveform_10"),
    p("patch.flanger.prime_b.input_sens", "B input sens", "Flanger", "Input sensitivity for envelope response (PRIME B)."),
    pc("patch.flanger.prime_b.polarity", "B polarity", "Flanger", "Modulation polarity (PRIME B).", "polarity"),

    // ===== Patch / Phaser =====
    pk("patch.phaser.phaser_type", "Phaser type", "Phaser", "Phaser voicing: PRIME G, PRIME B, or the vintage Script phaser.", Kind::Choice(&[choice("prime_g","Prime G"),choice("prime_b","Prime B"),choice("script","Script")])),
    p("patch.phaser.note", "Note", "Phaser", "Sets the rate from the tempo as a note value."),
    pl("patch.phaser.direct_level", "Direct level", "Phaser", "Level of the dry signal blended with the effect."),
    pl("patch.phaser.prime_g.depth", "G depth", "Phaser", "Modulation depth (PRIME G)."),
    p("patch.phaser.prime_g.resonance", "G resonance", "Phaser", "Resonance/feedback of the phase notches (PRIME G)."),
    p("patch.phaser.prime_g.manual", "G manual", "Phaser", "Centre frequency of the phase sweep (PRIME G)."),
    p("patch.phaser.prime_g.low_damp", "G low damp", "Phaser", "Damps the low end (PRIME G)."),
    p("patch.phaser.prime_g.high_damp", "G high damp", "Phaser", "Damps the high end (PRIME G)."),
    pc("patch.phaser.prime_g.low_cut", "G low cut", "Phaser", "Low-cut filter (PRIME G).", "cut_low_freq"),
    pc("patch.phaser.prime_g.high_cut", "G high cut", "Phaser", "High-cut filter (PRIME G).", "cut_high_freq"),
    pc("patch.phaser.prime_g.separation", "G separation", "Phaser", "Stereo spread (PRIME G).", "separation_deg"),
    pc("patch.phaser.prime_g.waveform", "G waveform", "Phaser", "Modulation waveform (PRIME G).", "waveform_10"),
    p("patch.phaser.prime_g.input_sens", "G input sens", "Phaser", "Input sensitivity for envelope response (PRIME G)."),
    pc("patch.phaser.prime_g.polarity", "G polarity", "Phaser", "Modulation polarity (PRIME G).", "polarity"),
    pc("patch.phaser.prime_g.stage", "G stage", "Phaser", "Number of phaser stages (PRIME G).", "phaser_stage"),
    pc("patch.phaser.prime_g.step_rate", "G step rate", "Phaser", "Step-modulation rate (PRIME G).", "step_rate_note"),
    pc("patch.phaser.prime_g.bi_phase", "G bi-phase", "Phaser", "Bi-phase mode for a richer sweep (PRIME G).", "bi_phase"),
    pl("patch.phaser.prime_b.depth", "B depth", "Phaser", "Modulation depth (PRIME B)."),
    p("patch.phaser.prime_b.resonance", "B resonance", "Phaser", "Resonance/feedback of the phase notches (PRIME B)."),
    p("patch.phaser.prime_b.manual", "B manual", "Phaser", "Centre frequency of the phase sweep (PRIME B)."),
    p("patch.phaser.prime_b.low_damp", "B low damp", "Phaser", "Damps the low end (PRIME B)."),
    p("patch.phaser.prime_b.high_damp", "B high damp", "Phaser", "Damps the high end (PRIME B)."),
    pc("patch.phaser.prime_b.low_cut", "B low cut", "Phaser", "Low-cut filter (PRIME B).", "cut_low_freq"),
    pc("patch.phaser.prime_b.high_cut", "B high cut", "Phaser", "High-cut filter (PRIME B).", "cut_high_freq"),
    pc("patch.phaser.prime_b.separation", "B separation", "Phaser", "Stereo spread (PRIME B).", "separation_deg"),
    pc("patch.phaser.prime_b.waveform", "B waveform", "Phaser", "Modulation waveform (PRIME B).", "waveform_10"),
    p("patch.phaser.prime_b.input_sens", "B input sens", "Phaser", "Input sensitivity for envelope response (PRIME B)."),
    pc("patch.phaser.prime_b.polarity", "B polarity", "Phaser", "Modulation polarity (PRIME B).", "polarity"),
    pc("patch.phaser.prime_b.stage", "B stage", "Phaser", "Number of phaser stages (PRIME B).", "phaser_stage"),
    pc("patch.phaser.prime_b.step_rate", "B step rate", "Phaser", "Step-modulation rate (PRIME B).", "step_rate_note"),
    pc("patch.phaser.prime_b.bi_phase", "B bi-phase", "Phaser", "Bi-phase mode for a richer sweep (PRIME B).", "bi_phase"),
    pl("patch.phaser.script_depth", "Script depth", "Phaser", "Modulation depth of the vintage Script phaser voicing."),

    // ===== Patch / C-Vibe =====
    pk("patch.cvibe.cvibe_type", "C-Vibe type", "C-Vibe", "Univibe-style voicing — Chorus or Vibrato position of the scanner.", Kind::Choice(&[choice("chorus","Chorus"),choice("vibrato","Vibrato")])),
    p("patch.cvibe.note", "Note", "C-Vibe", "Sets the rate from the tempo as a note value."),
    pl("patch.cvibe.depth", "Depth", "C-Vibe", "Modulation depth."),
    pl("patch.cvibe.direct_level", "Direct level", "C-Vibe", "Level of the dry signal blended with the effect."),

    // ===== Patch / Vibrato =====
    pk("patch.vibrato.vibrato_type", "Vibrato type", "Vibrato", "PRIME vibrato or the SCANNER (rotating-scanner) vibrato.", Kind::Choice(&[choice("prime","Prime"),choice("scanner","Scanner")])),
    p("patch.vibrato.note", "Note", "Vibrato", "Sets the rate from the tempo as a note value."),
    pl("patch.vibrato.direct_level", "Direct level", "Vibrato", "Level of the dry signal blended with the effect."),
    pl("patch.vibrato.prime_depth", "Depth", "Vibrato", "Modulation depth."),
    p("patch.vibrato.prime_color", "Color", "Vibrato", "Tonal colour of the vibrato voice."),
    pc("patch.vibrato.trigger", "Trigger", "Vibrato", "Re-triggers the LFO from playing dynamics.", "onoff"),
    p("patch.vibrato.prime_rise_time", "Rise time", "Vibrato", "How quickly the vibrato fades in after a trigger."),
    p("patch.vibrato.prime_envelope_sens", "Envelope sens", "Vibrato", "Sensitivity of the envelope trigger."),
    pc("patch.vibrato.prime_waveform", "Waveform", "Vibrato", "Modulation waveform.", "waveform_10"),
    p("patch.vibrato.prime_input_sens", "Input sens", "Vibrato", "Input sensitivity for envelope response."),
    pc("patch.vibrato.scanner_mode", "Scanner mode", "Vibrato", "Univibe-style scanner position (V1–V3 vibrato, C1–C3 chorus).", "scanner_mode"),

    // ===== Patch / Tremolo =====
    pk("patch.tremolo.tremolo_type", "Tremolo type", "Tremolo", "PRIME T (tremolo), PRIME P (auto-pan), TWIN, or DELUXE voicing.", Kind::Choice(&[choice("prime_t","Prime T"),choice("prime_p","Prime P"),choice("twin","Twin"),choice("deluxe","Deluxe")])),
    p("patch.tremolo.note", "Note", "Tremolo", "Sets the rate from the tempo as a note value."),
    pl("patch.tremolo.direct_level", "Direct level", "Tremolo", "Level of the dry signal blended with the effect."),
    pl("patch.tremolo.prime_t_depth", "T depth", "Tremolo", "Modulation depth (PRIME T)."),
    p("patch.tremolo.prime_t_rise_time", "T rise time", "Tremolo", "Fade-in time after a trigger (PRIME T)."),
    p("patch.tremolo.prime_t_envelope_sens", "T envelope sens", "Tremolo", "Envelope trigger sensitivity (PRIME T)."),
    p("patch.tremolo.prime_t_waveform", "T waveform", "Tremolo", "Modulation waveform (PRIME T)."),
    p("patch.tremolo.prime_t_input_sens", "T input sens", "Tremolo", "Input sensitivity (PRIME T)."),
    pl("patch.tremolo.prime_p_depth", "P depth", "Tremolo", "Modulation depth (PRIME P)."),
    p("patch.tremolo.prime_p_rise_time", "P rise time", "Tremolo", "Fade-in time after a trigger (PRIME P)."),
    p("patch.tremolo.prime_p_envelope_sens", "P envelope sens", "Tremolo", "Envelope trigger sensitivity (PRIME P)."),
    p("patch.tremolo.prime_p_waveform", "P waveform", "Tremolo", "Modulation waveform (PRIME P)."),
    p("patch.tremolo.prime_p_input_sens", "P input sens", "Tremolo", "Input sensitivity (PRIME P)."),
    pl("patch.tremolo.twin_deluxe_intensity", "Twin/Deluxe intensity", "Tremolo", "Tremolo intensity for the TWIN and DELUXE voicings."),

    // ===== Patch / Dimension (models the Roland Dimension D) =====
    pk("patch.dimension.mode", "Mode", "Dimension", "Dimension D preset 1–4 (the original SDD-320 buttons), or USER to combine buttons via the switches below.", Kind::Choice(&[choice("m1","1"),choice("m2","2"),choice("m3","3"),choice("m4","4"),choice("user","User")])),
    p("patch.dimension.user_mode1_sw", "User mode 1", "Dimension", "Enable Dimension button 1 in USER mode."),
    p("patch.dimension.user_mode2_sw", "User mode 2", "Dimension", "Enable Dimension button 2 in USER mode."),
    p("patch.dimension.user_mode3_sw", "User mode 3", "Dimension", "Enable Dimension button 3 in USER mode."),
    p("patch.dimension.user_mode4_sw", "User mode 4", "Dimension", "Enable Dimension button 4 in USER mode."),
    p("patch.dimension.user_mode5_sw", "User mode 5", "Dimension", "Enable Dimension button 5 in USER mode."),
    pl("patch.dimension.direct_level", "Direct level", "Dimension", "Level of the dry signal blended with the effect."),

    // ===== Patch / Ring Mod =====
    p("patch.ring_mod.frequency", "Frequency", "Ring Mod", "Carrier frequency of the ring modulator."),
    p("patch.ring_mod.freq_mod_rate", "Freq mod rate", "Ring Mod", "Rate at which the carrier frequency is modulated."),
    pl("patch.ring_mod.freq_mod_depth", "Freq mod depth", "Ring Mod", "Depth of the carrier-frequency modulation."),
    pc("patch.ring_mod.intelligent", "Intelligent", "Ring Mod", "Envelope follower input matching — off, or tuned to the guitar / bass signal range.", "envelope_input"),
    pl("patch.ring_mod.direct_level", "Direct level", "Ring Mod", "Level of the dry signal blended with the effect."),

    // ===== Patch / Rotary (rotary speaker simulation) =====
    pc("patch.rotary.speed", "Speed", "Rotary", "Rotor speed select.", "rotary_speed"),
    p("patch.rotary.slow_rate", "Slow rate", "Rotary", "Rotor rate in the Slow setting."),
    p("patch.rotary.fast_rate", "Fast rate", "Rotary", "Rotor rate in the Fast setting."),
    p("patch.rotary.rise_time", "Rise time", "Rotary", "How quickly the rotor accelerates to Fast."),
    p("patch.rotary.fall_time", "Fall time", "Rotary", "How quickly the rotor decelerates to Slow."),
    p("patch.rotary.mic_distance", "Mic distance", "Rotary", "Distance of the virtual mics from the cabinet."),
    p("patch.rotary.rotor_horn_balance", "Rotor/Horn balance", "Rotary", "Balance between the low rotor and high horn."),
    pl("patch.rotary.drive", "Drive", "Rotary", "Overdrive into the rotary cabinet."),
    pl("patch.rotary.direct_level", "Direct level", "Rotary", "Level of the dry signal blended with the effect."),

    // ===== Patch / Filter =====
    pk("patch.filter.filter_type", "Filter type", "Filter", "A-WAH (auto/LFO wah), T-WAH (touch wah), or PATTERN (programmed filter steps), each in a G/B voicing.", Kind::Choice(&[choice("auto_wah_g","Auto Wah G"),choice("auto_wah_b","Auto Wah B"),choice("touch_wah_g","Touch Wah G"),choice("touch_wah_b","Touch Wah B"),choice("pattern","Pattern")])),
    p("patch.filter.note", "Note", "Filter", "Sets the rate from the tempo as a note value."),
    pl("patch.filter.direct_level", "Direct level", "Filter", "Level of the dry signal blended with the effect."),
    pl("patch.filter.auto_wah_g.depth", "Auto Wah G depth", "Filter", "LFO sweep depth (Auto Wah G)."),
    p("patch.filter.auto_wah_g.frequency", "Auto Wah G freq", "Filter", "Centre frequency of the wah (Auto Wah G)."),
    p("patch.filter.auto_wah_g.resonance", "Auto Wah G resonance", "Filter", "Filter resonance/peak (Auto Wah G)."),
    pc("patch.filter.auto_wah_g.filter_mode", "Auto Wah G mode", "Filter", "Filter type (Auto Wah G).", "filter_mode"),
    pc("patch.filter.auto_wah_g.waveform", "Auto Wah G waveform", "Filter", "LFO waveform (Auto Wah G).", "lfo_waveform_6"),
    pl("patch.filter.auto_wah_b.depth", "Auto Wah B depth", "Filter", "LFO sweep depth (Auto Wah B)."),
    p("patch.filter.auto_wah_b.frequency", "Auto Wah B freq", "Filter", "Centre frequency of the wah (Auto Wah B)."),
    p("patch.filter.auto_wah_b.resonance", "Auto Wah B resonance", "Filter", "Filter resonance/peak (Auto Wah B)."),
    pc("patch.filter.auto_wah_b.filter_mode", "Auto Wah B mode", "Filter", "Filter type (Auto Wah B).", "filter_mode"),
    pc("patch.filter.auto_wah_b.waveform", "Auto Wah B waveform", "Filter", "LFO waveform (Auto Wah B).", "lfo_waveform_6"),
    pc("patch.filter.touch_wah_g.filter_mode", "Touch Wah G mode", "Filter", "Filter type (Touch Wah G).", "filter_mode"),
    pc("patch.filter.touch_wah_g.polarity", "Touch Wah G polarity", "Filter", "Sweep up or down with input level (Touch Wah G).", "polarity"),
    p("patch.filter.touch_wah_g.sens", "Touch Wah G sens", "Filter", "Sensitivity to playing dynamics (Touch Wah G)."),
    p("patch.filter.touch_wah_g.frequency", "Touch Wah G freq", "Filter", "Base centre frequency (Touch Wah G)."),
    p("patch.filter.touch_wah_g.resonance", "Touch Wah G resonance", "Filter", "Filter resonance/peak (Touch Wah G)."),
    p("patch.filter.touch_wah_g.decay", "Touch Wah G decay", "Filter", "How quickly the wah returns after a transient (Touch Wah G)."),
    pc("patch.filter.touch_wah_b.filter_mode", "Touch Wah B mode", "Filter", "Filter type (Touch Wah B).", "filter_mode"),
    pc("patch.filter.touch_wah_b.polarity", "Touch Wah B polarity", "Filter", "Sweep up or down with input level (Touch Wah B).", "polarity"),
    p("patch.filter.touch_wah_b.sens", "Touch Wah B sens", "Filter", "Sensitivity to playing dynamics (Touch Wah B)."),
    p("patch.filter.touch_wah_b.frequency", "Touch Wah B freq", "Filter", "Base centre frequency (Touch Wah B)."),
    p("patch.filter.touch_wah_b.resonance", "Touch Wah B resonance", "Filter", "Filter resonance/peak (Touch Wah B)."),
    p("patch.filter.touch_wah_b.decay", "Touch Wah B decay", "Filter", "How quickly the wah returns after a transient (Touch Wah B)."),
    pc("patch.filter.pattern.filter_mode", "Pattern mode", "Filter", "Filter type for the pattern sequencer.", "filter_mode"),
    pc("patch.filter.pattern.pattern_type", "Pattern type", "Filter", "Preset (PAT1–PAT10) or USER step pattern.", "pattern_type"),
    pc("patch.filter.pattern.step_number", "Pattern steps", "Filter", "Number of active steps in the pattern.", "pattern_step_count"),
    p("patch.filter.pattern.resonance", "Pattern resonance", "Filter", "Filter resonance for the pattern sequencer."),
    p("patch.filter.pattern.transition", "Pattern transition", "Filter", "Smoothness of the jump between steps."),
    p("patch.filter.pattern.frequencies", "Pattern step freqs", "Filter", "Cutoff frequency for each of the 24 pattern steps."),

    // ===== Patch / Slicer =====
    p("patch.slicer.note", "Note", "Slicer", "Sets the slice cycle from the tempo as a note value."),
    pc("patch.slicer.pattern", "Pattern", "Slicer", "Preset slice pattern (P1–P30, H1–H20), or USER to program your own.", "slicer_pattern"),
    pc("patch.slicer.fx_type", "FX type", "Slicer", "Per-step effect applied within the slice.", "slicer_fx_type"),
    p("patch.slicer.step_number", "Step number", "Slicer", "Number of active steps in the user pattern."),
    p("patch.slicer.steps.length", "Step length", "Slicer", "Length of a user-pattern step."),
    p("patch.slicer.steps.level", "Step level", "Slicer", "Output level of a user-pattern step."),
    pc("patch.slicer.steps.band", "Step band", "Slicer", "Frequency band used by a user-pattern step.", "slicer_step_band"),
    p("patch.slicer.steps.effect_level", "Step effect level", "Slicer", "Per-step effect amount."),
    p("patch.slicer.steps.effect_pitch", "Step effect pitch", "Slicer", "Per-step pitch of the slice effect."),
    p("patch.slicer.attack", "Attack", "Slicer", "Attack shape of each slice."),
    p("patch.slicer.duty", "Duty", "Slicer", "Duty cycle (on/off ratio) of the slices."),
    pl("patch.slicer.direct_level", "Direct level", "Slicer", "Level of the dry signal blended with the effect."),
    pc("patch.slicer.output_mode", "Output mode", "Slicer", "Output routing of the slicer.", "slicer_output_mode"),

    // ===== Patch / Overtone =====
    pk("patch.overtone.overtone_type", "Overtone type", "Overtone", "OVERTONE adds harmonics; DETUNE adds pitch-shifted layers.", Kind::Choice(&[choice("overtone","Overtone"),choice("detune","Detune")])),
    pl("patch.overtone.direct_level", "Direct level", "Overtone", "Level of the dry signal blended with the effect."),
    pl("patch.overtone.lower_level", "Lower level", "Overtone", "Level of the lower (sub) overtone."),
    pl("patch.overtone.upper_level", "Upper level", "Overtone", "Level of the upper overtone."),
    pl("patch.overtone.unison_level", "Unison level", "Overtone", "Level of the unison voice."),
    p("patch.overtone.detune", "Detune", "Overtone", "Amount of detuning between voices."),
    p("patch.overtone.tone_low", "Tone low", "Overtone", "Low tone shaping of the overtone voices."),
    p("patch.overtone.tone_high", "Tone high", "Overtone", "High tone shaping of the overtone voices."),
    p("patch.overtone.detune1_pitch", "Detune 1 pitch", "Overtone", "Pitch offset of detune voice 1."),
    pl("patch.overtone.detune1_level", "Detune 1 level", "Overtone", "Level of detune voice 1."),
    p("patch.overtone.detune2_pitch", "Detune 2 pitch", "Overtone", "Pitch offset of detune voice 2."),
    pl("patch.overtone.detune2_level", "Detune 2 level", "Overtone", "Level of detune voice 2."),

    // ===== Patch / Control (per-patch footswitch & expression) =====
    p("patch.control.tap_ctl_function", "TAP/CTL function", "Patch Control", "Per-patch function of the TAP/CTL switch (used when the System preference is Patch)."),
    p("patch.control.ctl1_function", "CTL1 function", "Patch Control", "Per-patch function of the external CTL1 switch."),
    p("patch.control.ctl2_function", "CTL2 function", "Patch Control", "Per-patch function of the external CTL2 switch."),
    p("patch.control.exp_pedal_function", "Expression function", "Patch Control", "Per-patch function of the expression pedal."),
    p("patch.control.exp_target_min", "Expression min", "Patch Control", "Expression-pedal heel value (display −100…10000)."),
    p("patch.control.exp_target_max", "Expression max", "Patch Control", "Expression-pedal toe value (display −100…10000)."),

    // ===== Patch / Assign (8 slots; one representative path set) =====
    p("patch.assigns.sw", "Assign on", "Assign", "Enable this assignment."),
    pc("patch.assigns.source", "Source", "Assign", "Controller that drives the assignment (footswitch, expression, internal/wave pedal, input envelope, or a CC#).", "assign_source"),
    pk("patch.assigns.source_mode", "Source mode", "Assign", "Momentary (active while held) or toggle (latches).", Kind::Choice(&[choice("moment","Moment"),choice("toggle","Toggle")])),
    pc("patch.assigns.target", "Target", "Assign", "Parameter the source controls (the full per-mode target list).", "assign_target"),
    p("patch.assigns.target_min", "Target min", "Assign", "Target value at the source's minimum."),
    p("patch.assigns.target_max", "Target max", "Assign", "Target value at the source's maximum."),
    pk("patch.assigns.active_range_lo", "Active range low", "Assign", "Lower bound of the source range that is active (0–126).", Kind::Range{min:0,max:126,unit:None}),
    pk("patch.assigns.active_range_hi", "Active range high", "Assign", "Upper bound of the source range that is active (1–127).", Kind::Range{min:1,max:127,unit:None}),
    p("patch.assigns.wave_rate", "Wave rate", "Assign", "Rate of the internal wave-pedal LFO (when the source is WAVE PDL)."),
    pk("patch.assigns.wave_form", "Wave form", "Assign", "Waveform of the internal wave-pedal LFO.", Kind::Choice(&[choice("saw","Saw"),choice("tri","Tri"),choice("sin","Sin")])),
    p("patch.assigns.internal_pedal_trigger", "Internal pedal trigger", "Assign", "Trigger source for the internal pedal."),
    p("patch.assigns.internal_pedal_time", "Internal pedal time", "Assign", "Travel time of the internal pedal."),
    pk("patch.assigns.internal_pedal_curve", "Internal pedal curve", "Assign", "Response curve of the internal pedal.", Kind::Choice(&[choice("linear","Linear"),choice("slow","Slow"),choice("fast","Fast")])),

    // ===== Patch / trailing =====
    pl("patch.assign_input_sens", "Assign input sens", "Assign", "Shared input sensitivity for envelope-following assign sources."),
    p("patch.led_status", "LED status", "Patch", "Whether the on/off LED reflects this patch's state when recalled."),
];

/// The editor-facing parameter table.
pub fn params() -> Params {
    Params(PARAMS)
}

/// The full catalog [`Bundle`](midi_access_core::Bundle) — `{ device, params,
/// catalogs, defaults }` — the same object the CLI's `catalog` command prints.
/// Exposed so the wasm layer can hand it to the editor verbatim (single source
/// of truth for labels/help, no parallel TS table).
pub fn bundle() -> midi_access_core::Bundle {
    use midi_access_core::Device;
    midi_access_core::Bundle {
        device: crate::device::Md500::NAME,
        params: params(),
        catalogs: MD500_CATALOGS.as_value(),
        defaults: Value::Mapping(Mapping::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn assign_catalogs_resolve_and_label() {
        assert_eq!(MD500_CATALOGS.names(), CATALOG_NAMES);
        assert!(MD500_CATALOGS.resolve("bogus", "x").is_none());
        // source: named + CC#.
        assert_eq!(MD500_CATALOGS.resolve("assign_source", "TAP/CTL"), Some(0));
        assert_eq!(MD500_CATALOGS.resolve("assign_source", "CC#64"), Some(38));
        assert_eq!(
            MD500_CATALOGS.label("assign_source", 38).as_deref(),
            Some("CC#64")
        );
        // target: by name.
        assert_eq!(
            MD500_CATALOGS.resolve("assign_target", "Effect Switch"),
            Some(0)
        );
        assert_eq!(MD500_CATALOGS.resolve("assign_target", "LED"), Some(264));
        assert_eq!(
            MD500_CATALOGS.label("assign_target", 0).as_deref(),
            Some("Effect Switch")
        );
        // as_value exposes all tables for the bundle.
        let v = MD500_CATALOGS.as_value();
        let m = v.as_mapping().unwrap();
        assert!(m.contains_key(Value::String("assign_source".into())));
        assert!(m.contains_key(Value::String("assign_target".into())));
        assert!(m.contains_key(Value::String("eq_low_freq".into())));
        assert!(m.contains_key(Value::String("eq_high_freq".into())));
    }

    #[test]
    fn eq_freq_catalogs_resolve_and_label() {
        assert_eq!(MD500_CATALOGS.resolve("eq_low_freq", "20.0 Hz"), Some(0));
        assert_eq!(MD500_CATALOGS.resolve("eq_low_freq", "800 Hz"), Some(16));
        assert!(MD500_CATALOGS.resolve("eq_low_freq", "nope").is_none());
        assert_eq!(
            MD500_CATALOGS.label("eq_low_freq", 0).as_deref(),
            Some("20.0 Hz")
        );
        assert_eq!(
            MD500_CATALOGS.label("eq_low_freq", 16).as_deref(),
            Some("800 Hz")
        );
        assert!(MD500_CATALOGS.label("eq_low_freq", 17).is_none());

        assert_eq!(MD500_CATALOGS.resolve("eq_high_freq", "630 Hz"), Some(0));
        assert_eq!(MD500_CATALOGS.resolve("eq_high_freq", "16.0 kHz"), Some(14));
        assert_eq!(
            MD500_CATALOGS.label("eq_high_freq", 14).as_deref(),
            Some("16.0 kHz")
        );
        assert!(MD500_CATALOGS.label("eq_high_freq", 15).is_none());
    }

    #[test]
    fn new_simple_catalogs_round_trip() {
        // Spot-check each of the new lists: lookup by name, label back from index.
        let cases = [
            ("cut_low_freq", "FLAT", 0i64, "FLAT"),
            ("cut_low_freq", "800 Hz", 17, "800 Hz"),
            ("cut_high_freq", "630 Hz", 0, "630 Hz"),
            ("cut_high_freq", "FLAT", 15, "FLAT"),
            ("filter_mode", "LPF", 0, "LPF"),
            ("filter_mode", "BPF", 2, "BPF"),
            ("polarity", "DOWN", 0, "DOWN"),
            ("polarity", "UP", 1, "UP"),
            ("lfo_waveform_6", "SIN", 0, "SIN"),
            ("lfo_waveform_6", "RAMP", 5, "RAMP"),
            ("phaser_stage", "2", 0, "2"),
            ("phaser_stage", "24", 4, "24"),
            ("pattern_type", "PAT1", 0, "PAT1"),
            ("pattern_type", "USER", 10, "USER"),
            ("pattern_step_count", "8", 0, "8"),
            ("pattern_step_count", "24", 3, "24"),
            ("slicer_step_band", "THRU", 0, "THRU"),
            ("slicer_step_band", "BAND6", 6, "BAND6"),
            ("slicer_output_mode", "MONO", 0, "MONO"),
            ("slicer_output_mode", "AUTO", 4, "AUTO"),
            ("slicer_fx_type", "OFF", 0, "OFF"),
            ("slicer_fx_type", "RING", 6, "RING"),
            ("rotary_speed", "SLOW", 0, "SLOW"),
            ("rotary_speed", "FAST", 1, "FAST"),
            ("tricho_lfo_mode", "PRESET", 0, "PRESET"),
            ("tricho_lfo_mode", "P+M", 2, "P+M"),
            ("scanner_mode", "V1", 0, "V1"),
            ("scanner_mode", "C3", 5, "C3"),
            ("onoff", "OFF", 0, "OFF"),
            ("onoff", "ON", 1, "ON"),
            ("bi_phase", "OFF", 0, "OFF"),
            ("bi_phase", "ON", 1, "ON"),
            ("step_rate_note", "OFF", 0, "OFF"),
            ("step_rate_note", "32nd", 16, "32nd"),
            ("separation_deg", "0deg", 0, "0deg"),
            ("separation_deg", "180deg", 12, "180deg"),
            ("waveform_10", "1", 0, "1"),
            ("waveform_10", "10", 9, "10"),
            ("envelope_input", "OFF", 0, "OFF"),
            ("envelope_input", "BASS", 2, "BASS"),
            ("slicer_pattern", "P1", 0, "P1"),
            ("slicer_pattern", "H20", 49, "H20"),
            ("slicer_pattern", "USER", 50, "USER"),
        ];
        for (cat, name, idx, back) in cases {
            assert_eq!(
                MD500_CATALOGS.resolve(cat, name),
                Some(idx),
                "{cat}: resolve {name}"
            );
            assert_eq!(
                MD500_CATALOGS.label(cat, idx).as_deref(),
                Some(back),
                "{cat}: label {idx}"
            );
        }
    }

    #[test]
    fn params_are_unique_and_addressable() {
        let p = params();
        // No duplicate paths.
        let mut seen = HashSet::new();
        for m in p.as_slice() {
            assert!(seen.insert(m.path), "duplicate param path {}", m.path);
            assert!(!m.label.is_empty(), "{} has no label", m.path);
            assert!(!m.help.is_empty(), "{} has no help", m.path);
        }
        // Spot-check coverage across areas.
        assert!(p.get("setup.current_patch").is_some());
        assert!(p.get("system.common.rx_channel").is_some());
        assert!(p.get("bank.structure").is_some());
        assert!(p.get("patch.common.mode").is_some());
        assert!(p.get("patch.flanger.prime_b.depth").is_some());
        assert!(p.get("patch.assigns.target").is_some());
        assert!(p.as_slice().len() > 200);
    }

    #[test]
    fn groups_collect_fields() {
        assert!(params().in_group("Chorus").count() >= 15);
        assert!(params().in_group("MIDI").count() >= 10);
    }
}
