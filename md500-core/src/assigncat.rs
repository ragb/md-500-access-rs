//! Assign **source** and **target** name catalogs — the two large index→name
//! lists from the MIDI implementation (ASSIGN `source` 0..=69 and `target`
//! 0..=264). Generated from `docs/spec/assign_targets.txt`; the source list is
//! formulaic (7 named controllers + CC#1..#31 + CC#64..#95).
//!
//! Values stay integers in the typed model; these tables let the editor / LLM
//! author an assignment by name (wired through the kit's `Catalogs`).

/// The seven named assign sources (indices 0..=6); the rest are CC numbers.
pub const SOURCE_NAMED: [&str; 7] = [
    "TAP/CTL", "EXP PDL", "CTL1 PDL", "CTL2 PDL", "INT PDL", "WAVE PDL", "INPUT",
];

/// Total number of assign sources (0..=69).
pub const SOURCE_COUNT: usize = 70;

/// The 265 assign targets, indexed 0..=264.
pub const ASSIGN_TARGETS: [&str; 265] = [
    "Effect Switch",
    "Rate",
    "BPM",
    "Mode",
    "Effect Level",
    "Insert Switch",
    "Output Gain",
    "Output Mode",
    "Trigger",
    "Chorus Type",
    "Chorus Note",
    "Chorus Depth",
    "Chorus PreDly",
    "Chorus Wavefm",
    "Chorus Sweet",
    "Chorus Bell",
    "Chorus ELevel",
    "Chorus DLevel",
    "Chorus LoLevel",
    "Chorus LoFreq",
    "Chorus HiLevel",
    "Chorus HiFreq",
    "Chorus LoCut",
    "Chorus HiCut",
    "CE1Cho Note",
    "CE1Cho Depth",
    "CE1Cho ELevel",
    "CE1Cho DLevel",
    "CE1Cho LoLevel",
    "CE1Cho LoFreq",
    "CE1Cho HiLevel",
    "CE1Cho HiFreq",
    "CE1Cho Preamp",
    "CE1Cho PreampG",
    "CE1Cho PreampL",
    "CE1Vib Note",
    "CE1Vib Depth",
    "CE1Vib ELevel",
    "CE1Vib DLevel",
    "CE1Vib LoLevel",
    "CE1Vib LoFreq",
    "CE1Vib HiLevel",
    "CE1Vib HiFreq",
    "CE1Vib Preamp",
    "CE1Vib PreampG",
    "CE1Vib PreampL",
    "TriCho Note",
    "TriCho LFO",
    "TriCho Intens1",
    "TriCho Intens2",
    "TriCho Intens3",
    "TriCho Bright",
    "TriCho LoLevel",
    "TriCho LoFreq",
    "TriCho HiLevel",
    "TriCho HiFreq",
    "TriCho ELevel",
    "TriCho DLevel",
    "Flanger Type",
    "FlangG Note",
    "FlangG Depth",
    "FlangG Reso",
    "FlangG Manual",
    "FlangG Turbo",
    "FlangG LoDamp",
    "FlangG HiDamp",
    "FlangG LoCut",
    "FlangG HiCut",
    "FlangG Separat",
    "FlangG Step",
    "FlangG Wavefm",
    "FlangG InSens",
    "FlangG Polarty",
    "FlangG ELevel",
    "FlangG DLevel",
    "FlangB Note",
    "FlangB Depth",
    "FlangB Reso",
    "FlangB Manual",
    "FlangB Turbo",
    "FlangB LoDamp",
    "FlangB HiDamp",
    "FlangB LoCut",
    "FlangB HiCut",
    "FlangB Separat",
    "FlangB Step",
    "FlangB Wavefm",
    "FlangB InSens",
    "FlangB Polarty",
    "FlangB ELevel",
    "FlangB DLevel",
    "Phaser Type",
    "PhaserG Note",
    "PhaserG Depth",
    "PhaserG Reso",
    "PhaserG Manual",
    "PhaserG LoDamp",
    "PhaserG HiDamp",
    "PhaserG LoCut",
    "PhaserG HiCut",
    "PhaserG Separt",
    "PhaserG Wavefm",
    "PhaserG InSens",
    "PhaserG Polrty",
    "PhaserG Stage",
    "PhaserG Step",
    "PhaserG BiPhas",
    "PhaserG ELevel",
    "PhaserG DLevel",
    "PhaserB Note",
    "PhaserB Depth",
    "PhaserB Reso",
    "PhaserB Manual",
    "PhaserB LoDamp",
    "PhaserB HiDamp",
    "PhaserB LoCut",
    "PhaserB HiCut",
    "PhaserB Separt",
    "PhaserB Wavefm",
    "PhaserB InSens",
    "PhaserB Polrty",
    "PhaserB Stage",
    "PhaserB Step",
    "PhaserB BiPhas",
    "PhaserB ELevel",
    "PhaserB DLevel",
    "Script Note",
    "Script Depth",
    "Script ELevel",
    "Script DLevel",
    "CVibe Type",
    "CVibe Note",
    "CVibe Depth",
    "CVibe ELevel",
    "CVibe DLevel",
    "Vibrato Type",
    "Vibrato Note",
    "Vibrato Depth",
    "Vibrato Color",
    "Vibrato Trig",
    "Vibrato Rise",
    "Vibrato EnvSns",
    "Vibrato Wavefm",
    "Vibrato InSens",
    "Vibrato ELevel",
    "Vibrato DLevel",
    "Scanner Speed",
    "Scanner Note",
    "Scanner Mode",
    "Scanner ELevel",
    "Scanner DLevel",
    "Trml Type",
    "Trml Note",
    "Trml Depth",
    "Trml Trigger",
    "Trml RiseTime",
    "Trml EnvSens",
    "Trml Waveform",
    "Trml InputSens",
    "Trml ELevel",
    "Trml DLevel",
    "Pan Note",
    "Pan Depth",
    "Pan Trigger",
    "Pan RiseTime",
    "Pan EnvSens",
    "Pan Waveform",
    "Pan InputSens",
    "Pan ELevel",
    "Pan DLevel",
    "Twin Speed",
    "Twin Note",
    "Twin Intensity",
    "Twin ELevel",
    "Twin DLevel",
    "Deluxe Speed",
    "Deluxe Note",
    "Deluxe Intnsty",
    "Deluxe ELevel",
    "Deluxe DLevel",
    "Dimension Mode",
    "Dimension Md1",
    "Dimension Md2",
    "Dimension Md3",
    "Dimension Md4",
    "Dimension Md5",
    "Dimension ELvl",
    "Dimension DLvl",
    "RingMod Freq",
    "RingMod Depth",
    "RingMod Rate",
    "RingMod ELevel",
    "RingMod DLevel",
    "Rotary Speed",
    "Rotary Slow",
    "Rotary Fast",
    "Rotary Rise",
    "Rotary Fall",
    "Rotary MicDist",
    "Rotary R/H",
    "Rotary Drive",
    "Rotary ELevel",
    "Rotary DLevel",
    "Filter Type",
    "AWahG Note",
    "AWahG Filter",
    "AWahG Depth",
    "AWahG Freq",
    "AWahG Reso",
    "AWahG Waveform",
    "AWahG ELevel",
    "AWahG DLevel",
    "AWahB Note",
    "AWahB Filter",
    "AWahB Depth",
    "AWahB Freq",
    "AWahB Reso",
    "AWahB Waveform",
    "AWahB ELevel",
    "AWahB DLevel",
    "TWahG Filter",
    "TWahG Polarity",
    "TWahG Sens",
    "TWahG Freq",
    "TWahG Reso",
    "TWahG Decay",
    "TWahG ELevel",
    "TWahG DLevel",
    "TWahB Filter",
    "TWahB Polarity",
    "TWahB Sens",
    "TWahB Freq",
    "TWahB Reso",
    "TWahB Decay",
    "TWahB ELevel",
    "TWahB DLevel",
    "PFilter Note",
    "PFilter Ptn",
    "PFilter Filter",
    "PFilter Reso",
    "PFilter Trans",
    "PFilter ELevel",
    "PFilter DLevel",
    "Slicer Note",
    "Slicer Pattern",
    "Slicer FxType",
    "Slicer Attack",
    "Slicer Duty",
    "Slicer ELevel",
    "Slicer DLevel",
    "Slicer OutMode",
    "Overton Type",
    "Overton Lower",
    "Overton Upper",
    "Overton Unison",
    "Overton Detune",
    "Overton Low",
    "Overton High",
    "Overton DLevel",
    "Detune Pitch1",
    "Detune ELevel1",
    "Detune Pitch2",
    "Detune ELevel2",
    "Detune DLevel",
    "LED",
];

/// Name of assign source `i` (0..=69), or `None`.
pub fn source_name(i: usize) -> Option<String> {
    match i {
        0..=6 => Some(SOURCE_NAMED[i].to_string()),
        7..=37 => Some(format!("CC#{}", i - 6)), // CC#1..#31
        38..=69 => Some(format!("CC#{}", i + 26)), // CC#64..#95
        _ => None,
    }
}

/// Index of an assign source name (case-insensitive), or `None`.
pub fn source_index(name: &str) -> Option<i64> {
    if let Some(p) = SOURCE_NAMED
        .iter()
        .position(|n| n.eq_ignore_ascii_case(name))
    {
        return Some(p as i64);
    }
    let num: u8 = name
        .strip_prefix("CC#")
        .or_else(|| name.strip_prefix("cc#"))?
        .parse()
        .ok()?;
    match num {
        1..=31 => Some(num as i64 + 6),
        64..=95 => Some(num as i64 - 64 + 38),
        _ => None,
    }
}

/// Name of assign target `i` (0..=264), or `None`.
pub fn target_name(i: usize) -> Option<&'static str> {
    ASSIGN_TARGETS.get(i).copied()
}

/// Index of an assign target name (case-insensitive), or `None`.
pub fn target_index(name: &str) -> Option<i64> {
    ASSIGN_TARGETS
        .iter()
        .position(|n| n.eq_ignore_ascii_case(name))
        .map(|p| p as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_round_trips() {
        for i in 0..SOURCE_COUNT {
            let n = source_name(i).unwrap();
            assert_eq!(source_index(&n), Some(i as i64), "{n}");
        }
        assert_eq!(source_name(0).as_deref(), Some("TAP/CTL"));
        assert_eq!(source_name(7).as_deref(), Some("CC#1"));
        assert_eq!(source_name(38).as_deref(), Some("CC#64"));
        assert_eq!(source_name(69).as_deref(), Some("CC#95"));
        assert!(source_name(70).is_none());
    }

    #[test]
    fn target_round_trips() {
        assert_eq!(ASSIGN_TARGETS.len(), 265);
        assert_eq!(target_name(0), Some("Effect Switch"));
        assert_eq!(target_name(264), Some("LED"));
        for (i, n) in ASSIGN_TARGETS.iter().enumerate() {
            assert_eq!(target_index(n), Some(i as i64), "{n}");
        }
    }
}
