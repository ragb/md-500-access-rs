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

/// The MD-500's (empty) value catalogs — the device has no name↔number tables.
pub struct Md500Catalogs;

/// The shared singleton referenced by the [`Device`](midi_access_core::Device) impl.
pub static MD500_CATALOGS: Md500Catalogs = Md500Catalogs;

/// The two name↔number catalogs the MD-500 exposes (the big assign lists).
const CATALOG_NAMES: &[&str] = &["assign_source", "assign_target"];

impl Catalogs for Md500Catalogs {
    fn resolve(&self, cat: &str, name: &str) -> Option<i64> {
        match cat {
            "assign_source" => crate::assigncat::source_index(name),
            "assign_target" => crate::assigncat::target_index(name),
            _ => None,
        }
    }
    fn label(&self, cat: &str, value: i64) -> Option<String> {
        let i = usize::try_from(value).ok()?;
        match cat {
            "assign_source" => crate::assigncat::source_name(i),
            "assign_target" => crate::assigncat::target_name(i).map(str::to_string),
            _ => None,
        }
    }
    fn names(&self) -> &[&str] {
        CATALOG_NAMES
    }
    fn as_value(&self) -> Value {
        use serde_yaml::Sequence;
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
    p("patch.common.low_freq", "Low EQ freq", "EQ", "Low-band corner frequency (index 0–16, 20 Hz–800 Hz)."),
    pk("patch.common.high_level", "High EQ level", "EQ", "High-band cut/boost, −50…+50.", Kind::Range{min:-50,max:50,unit:None}),
    p("patch.common.high_freq", "High EQ freq", "EQ", "High-band corner frequency (index 0–14, 630 Hz–16 kHz)."),
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
    p("patch.chorus.prime_waveform", "PRIME waveform", "Chorus", "Modulation waveform of the PRIME chorus (1–10)."),
    pl("patch.chorus.prime_sweetness", "PRIME sweetness", "Chorus", "Smooths and thickens the PRIME chorus voice."),
    pl("patch.chorus.prime_bell", "PRIME bell", "Chorus", "Adds bell-like shimmer to the PRIME chorus."),
    p("patch.chorus.prime_low_cut", "PRIME low cut", "Chorus", "Low-cut filter on the PRIME chorus (Flat, 20 Hz–800 Hz)."),
    p("patch.chorus.prime_high_cut", "PRIME high cut", "Chorus", "High-cut filter on the PRIME chorus (630 Hz–16 kHz, Flat)."),
    pl("patch.chorus.ce1_depth", "CE-1 depth", "Chorus", "Modulation depth of the CE-1 chorus/vibrato model."),
    p("patch.chorus.ce1_preamp_sw", "CE-1 preamp", "Chorus", "Engage the CE-1 preamp colouring."),
    p("patch.chorus.ce1_preamp_gain", "CE-1 preamp gain", "Chorus", "Drive of the CE-1 preamp (0–99)."),
    pl("patch.chorus.ce1_preamp_level", "CE-1 preamp level", "Chorus", "Output level of the CE-1 preamp."),
    p("patch.chorus.tricho_lfo_mode", "Tri-Cho LFO mode", "Chorus", "LFO mode of the 3-phase chorus (0–2)."),
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
    p("patch.flanger.prime_g.low_cut", "G low cut", "Flanger", "Low-cut filter (PRIME G)."),
    p("patch.flanger.prime_g.high_cut", "G high cut", "Flanger", "High-cut filter (PRIME G)."),
    p("patch.flanger.prime_g.separation", "G separation", "Flanger", "Stereo spread of the flange (PRIME G)."),
    p("patch.flanger.prime_g.step_rate", "G step rate", "Flanger", "Step-modulation rate; Off for smooth sweep (PRIME G)."),
    p("patch.flanger.prime_g.waveform", "G waveform", "Flanger", "Modulation waveform (PRIME G)."),
    p("patch.flanger.prime_g.input_sens", "G input sens", "Flanger", "Input sensitivity for envelope response (PRIME G)."),
    p("patch.flanger.prime_g.polarity", "G polarity", "Flanger", "Modulation polarity (PRIME G)."),
    pl("patch.flanger.prime_b.depth", "B depth", "Flanger", "Modulation depth (PRIME B)."),
    p("patch.flanger.prime_b.resonance", "B resonance", "Flanger", "Feedback/resonance of the flange comb (PRIME B)."),
    p("patch.flanger.prime_b.manual", "B manual", "Flanger", "Centre frequency of the flange sweep (PRIME B)."),
    p("patch.flanger.prime_b.turbo_sw", "B turbo", "Flanger", "Turbo makes the flange more intense (PRIME B)."),
    p("patch.flanger.prime_b.low_damp", "B low damp", "Flanger", "Damps the low end of the flange (PRIME B)."),
    p("patch.flanger.prime_b.high_damp", "B high damp", "Flanger", "Damps the high end of the flange (PRIME B)."),
    p("patch.flanger.prime_b.low_cut", "B low cut", "Flanger", "Low-cut filter (PRIME B)."),
    p("patch.flanger.prime_b.high_cut", "B high cut", "Flanger", "High-cut filter (PRIME B)."),
    p("patch.flanger.prime_b.separation", "B separation", "Flanger", "Stereo spread of the flange (PRIME B)."),
    p("patch.flanger.prime_b.step_rate", "B step rate", "Flanger", "Step-modulation rate; Off for smooth sweep (PRIME B)."),
    p("patch.flanger.prime_b.waveform", "B waveform", "Flanger", "Modulation waveform (PRIME B)."),
    p("patch.flanger.prime_b.input_sens", "B input sens", "Flanger", "Input sensitivity for envelope response (PRIME B)."),
    p("patch.flanger.prime_b.polarity", "B polarity", "Flanger", "Modulation polarity (PRIME B)."),

    // ===== Patch / Phaser =====
    pk("patch.phaser.phaser_type", "Phaser type", "Phaser", "Phaser voicing: PRIME G, PRIME B, or the vintage Script phaser.", Kind::Choice(&[choice("prime_g","Prime G"),choice("prime_b","Prime B"),choice("script","Script")])),
    p("patch.phaser.note", "Note", "Phaser", "Sets the rate from the tempo as a note value."),
    pl("patch.phaser.direct_level", "Direct level", "Phaser", "Level of the dry signal blended with the effect."),
    pl("patch.phaser.prime_g.depth", "G depth", "Phaser", "Modulation depth (PRIME G)."),
    p("patch.phaser.prime_g.resonance", "G resonance", "Phaser", "Resonance/feedback of the phase notches (PRIME G)."),
    p("patch.phaser.prime_g.manual", "G manual", "Phaser", "Centre frequency of the phase sweep (PRIME G)."),
    p("patch.phaser.prime_g.low_damp", "G low damp", "Phaser", "Damps the low end (PRIME G)."),
    p("patch.phaser.prime_g.high_damp", "G high damp", "Phaser", "Damps the high end (PRIME G)."),
    p("patch.phaser.prime_g.low_cut", "G low cut", "Phaser", "Low-cut filter (PRIME G)."),
    p("patch.phaser.prime_g.high_cut", "G high cut", "Phaser", "High-cut filter (PRIME G)."),
    p("patch.phaser.prime_g.separation", "G separation", "Phaser", "Stereo spread (PRIME G)."),
    p("patch.phaser.prime_g.waveform", "G waveform", "Phaser", "Modulation waveform (PRIME G)."),
    p("patch.phaser.prime_g.input_sens", "G input sens", "Phaser", "Input sensitivity for envelope response (PRIME G)."),
    p("patch.phaser.prime_g.polarity", "G polarity", "Phaser", "Modulation polarity (PRIME G)."),
    p("patch.phaser.prime_g.stage", "G stage", "Phaser", "Number of phaser stages (PRIME G)."),
    p("patch.phaser.prime_g.step_rate", "G step rate", "Phaser", "Step-modulation rate (PRIME G)."),
    p("patch.phaser.prime_g.bi_phase", "G bi-phase", "Phaser", "Bi-phase mode for a richer sweep (PRIME G)."),
    pl("patch.phaser.prime_b.depth", "B depth", "Phaser", "Modulation depth (PRIME B)."),
    p("patch.phaser.prime_b.resonance", "B resonance", "Phaser", "Resonance/feedback of the phase notches (PRIME B)."),
    p("patch.phaser.prime_b.manual", "B manual", "Phaser", "Centre frequency of the phase sweep (PRIME B)."),
    p("patch.phaser.prime_b.low_damp", "B low damp", "Phaser", "Damps the low end (PRIME B)."),
    p("patch.phaser.prime_b.high_damp", "B high damp", "Phaser", "Damps the high end (PRIME B)."),
    p("patch.phaser.prime_b.low_cut", "B low cut", "Phaser", "Low-cut filter (PRIME B)."),
    p("patch.phaser.prime_b.high_cut", "B high cut", "Phaser", "High-cut filter (PRIME B)."),
    p("patch.phaser.prime_b.separation", "B separation", "Phaser", "Stereo spread (PRIME B)."),
    p("patch.phaser.prime_b.waveform", "B waveform", "Phaser", "Modulation waveform (PRIME B)."),
    p("patch.phaser.prime_b.input_sens", "B input sens", "Phaser", "Input sensitivity for envelope response (PRIME B)."),
    p("patch.phaser.prime_b.polarity", "B polarity", "Phaser", "Modulation polarity (PRIME B)."),
    p("patch.phaser.prime_b.stage", "B stage", "Phaser", "Number of phaser stages (PRIME B)."),
    p("patch.phaser.prime_b.step_rate", "B step rate", "Phaser", "Step-modulation rate (PRIME B)."),
    p("patch.phaser.prime_b.bi_phase", "B bi-phase", "Phaser", "Bi-phase mode for a richer sweep (PRIME B)."),
    pl("patch.phaser.script_depth", "Script depth", "Phaser", "Modulation depth of the vintage Script phaser voicing."),

    // ===== Patch / C-Vibe =====
    pk("patch.cvibe.cvibe_type", "C-Vibe type", "C-Vibe", "PRIME or the SCANNER vibrato voicing (after the Boss CE-1).", Kind::Choice(&[choice("prime","Prime"),choice("scanner","Scanner")])),
    p("patch.cvibe.note", "Note", "C-Vibe", "Sets the rate from the tempo as a note value."),
    pl("patch.cvibe.depth", "Depth", "C-Vibe", "Modulation depth."),
    pl("patch.cvibe.direct_level", "Direct level", "C-Vibe", "Level of the dry signal blended with the effect."),

    // ===== Patch / Vibrato =====
    pk("patch.vibrato.vibrato_type", "Vibrato type", "Vibrato", "PRIME vibrato or the SCANNER (rotating-scanner) vibrato.", Kind::Choice(&[choice("prime","Prime"),choice("scanner","Scanner")])),
    p("patch.vibrato.note", "Note", "Vibrato", "Sets the rate from the tempo as a note value."),
    pl("patch.vibrato.direct_level", "Direct level", "Vibrato", "Level of the dry signal blended with the effect."),
    pl("patch.vibrato.prime_depth", "Depth", "Vibrato", "Modulation depth."),
    p("patch.vibrato.prime_color", "Color", "Vibrato", "Tonal colour of the vibrato voice."),
    p("patch.vibrato.trigger", "Trigger", "Vibrato", "Re-triggers the LFO from playing dynamics."),
    p("patch.vibrato.prime_rise_time", "Rise time", "Vibrato", "How quickly the vibrato fades in after a trigger."),
    p("patch.vibrato.prime_envelope_sens", "Envelope sens", "Vibrato", "Sensitivity of the envelope trigger."),
    p("patch.vibrato.prime_waveform", "Waveform", "Vibrato", "Modulation waveform."),
    p("patch.vibrato.prime_input_sens", "Input sens", "Vibrato", "Input sensitivity for envelope response."),
    p("patch.vibrato.scanner_mode", "Scanner mode", "Vibrato", "Scanner-vibrato variation."),

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
    p("patch.ring_mod.intelligent", "Intelligent", "Ring Mod", "Tracks the input pitch so the ring tone stays musical."),
    pl("patch.ring_mod.direct_level", "Direct level", "Ring Mod", "Level of the dry signal blended with the effect."),

    // ===== Patch / Rotary (rotary speaker simulation) =====
    p("patch.rotary.speed", "Speed", "Rotary", "Rotor speed select (Slow/Fast)."),
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
    p("patch.filter.auto_wah_g.filter_mode", "Auto Wah G mode", "Filter", "Filter type: LPF/BPF/HPF (Auto Wah G)."),
    p("patch.filter.auto_wah_g.waveform", "Auto Wah G waveform", "Filter", "LFO waveform (Auto Wah G)."),
    pl("patch.filter.auto_wah_b.depth", "Auto Wah B depth", "Filter", "LFO sweep depth (Auto Wah B)."),
    p("patch.filter.auto_wah_b.frequency", "Auto Wah B freq", "Filter", "Centre frequency of the wah (Auto Wah B)."),
    p("patch.filter.auto_wah_b.resonance", "Auto Wah B resonance", "Filter", "Filter resonance/peak (Auto Wah B)."),
    p("patch.filter.auto_wah_b.filter_mode", "Auto Wah B mode", "Filter", "Filter type: LPF/BPF/HPF (Auto Wah B)."),
    p("patch.filter.auto_wah_b.waveform", "Auto Wah B waveform", "Filter", "LFO waveform (Auto Wah B)."),
    p("patch.filter.touch_wah_g.filter_mode", "Touch Wah G mode", "Filter", "Filter type: LPF/BPF/HPF (Touch Wah G)."),
    p("patch.filter.touch_wah_g.polarity", "Touch Wah G polarity", "Filter", "Sweep up or down with input level (Touch Wah G)."),
    p("patch.filter.touch_wah_g.sens", "Touch Wah G sens", "Filter", "Sensitivity to playing dynamics (Touch Wah G)."),
    p("patch.filter.touch_wah_g.frequency", "Touch Wah G freq", "Filter", "Base centre frequency (Touch Wah G)."),
    p("patch.filter.touch_wah_g.resonance", "Touch Wah G resonance", "Filter", "Filter resonance/peak (Touch Wah G)."),
    p("patch.filter.touch_wah_g.decay", "Touch Wah G decay", "Filter", "How quickly the wah returns after a transient (Touch Wah G)."),
    p("patch.filter.touch_wah_b.filter_mode", "Touch Wah B mode", "Filter", "Filter type: LPF/BPF/HPF (Touch Wah B)."),
    p("patch.filter.touch_wah_b.polarity", "Touch Wah B polarity", "Filter", "Sweep up or down with input level (Touch Wah B)."),
    p("patch.filter.touch_wah_b.sens", "Touch Wah B sens", "Filter", "Sensitivity to playing dynamics (Touch Wah B)."),
    p("patch.filter.touch_wah_b.frequency", "Touch Wah B freq", "Filter", "Base centre frequency (Touch Wah B)."),
    p("patch.filter.touch_wah_b.resonance", "Touch Wah B resonance", "Filter", "Filter resonance/peak (Touch Wah B)."),
    p("patch.filter.touch_wah_b.decay", "Touch Wah B decay", "Filter", "How quickly the wah returns after a transient (Touch Wah B)."),
    p("patch.filter.pattern.filter_mode", "Pattern mode", "Filter", "Filter type for the pattern sequencer (LPF/BPF/HPF)."),
    p("patch.filter.pattern.pattern_type", "Pattern type", "Filter", "Preset or user step pattern."),
    p("patch.filter.pattern.step_number", "Pattern steps", "Filter", "Number of active steps in the pattern."),
    p("patch.filter.pattern.resonance", "Pattern resonance", "Filter", "Filter resonance for the pattern sequencer."),
    p("patch.filter.pattern.transition", "Pattern transition", "Filter", "Smoothness of the jump between steps."),
    p("patch.filter.pattern.frequencies", "Pattern step freqs", "Filter", "Cutoff frequency for each of the 24 pattern steps."),

    // ===== Patch / Slicer =====
    p("patch.slicer.note", "Note", "Slicer", "Sets the slice cycle from the tempo as a note value."),
    p("patch.slicer.pattern", "Pattern", "Slicer", "Preset slice pattern, or USER to program your own."),
    p("patch.slicer.fx_type", "FX type", "Slicer", "Per-step effect applied within the slice."),
    p("patch.slicer.step_number", "Step number", "Slicer", "Number of active steps in the user pattern."),
    p("patch.slicer.steps.length", "Step length", "Slicer", "Length of a user-pattern step."),
    p("patch.slicer.steps.level", "Step level", "Slicer", "Output level of a user-pattern step."),
    p("patch.slicer.steps.band", "Step band", "Slicer", "Frequency band used by a user-pattern step."),
    p("patch.slicer.steps.effect_level", "Step effect level", "Slicer", "Per-step effect amount."),
    p("patch.slicer.steps.effect_pitch", "Step effect pitch", "Slicer", "Per-step pitch of the slice effect."),
    p("patch.slicer.attack", "Attack", "Slicer", "Attack shape of each slice."),
    p("patch.slicer.duty", "Duty", "Slicer", "Duty cycle (on/off ratio) of the slices."),
    pl("patch.slicer.direct_level", "Direct level", "Slicer", "Level of the dry signal blended with the effect."),
    p("patch.slicer.output_mode", "Output mode", "Slicer", "Output routing of the slicer."),

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
        assert_eq!(MD500_CATALOGS.names(), &["assign_source", "assign_target"]);
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
        // as_value exposes both tables for the bundle.
        let v = MD500_CATALOGS.as_value();
        let m = v.as_mapping().unwrap();
        assert!(m.contains_key(Value::String("assign_source".into())));
        assert!(m.contains_key(Value::String("assign_target".into())));
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
