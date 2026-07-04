//! Roman numeral utilities and pitch-class helpers.

use aurora_ast::{ChordQuality, ChordSymbol, HarmonicFunction, Mode, PitchClass};

use super::templates::{chord_root_name, quality_suffix, PlannedChord};

/// Scale degree (1–7) to pitch class in major.
pub fn major_degree_pc(tonic: u8, degree: u8) -> u8 {
    let steps: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
    (tonic + steps[(degree.saturating_sub(1) % 7) as usize]) % 12
}

/// Infer scale degree from roman numeral string.
pub fn diatonic_degree(roman: &str) -> Option<u8> {
    let r = roman.trim_start_matches(['b', '#', '♭', '♯']);
    if r.starts_with("vii") {
        return Some(7);
    }
    if r.starts_with("vi") {
        return Some(6);
    }
    if r.starts_with('V') || r.starts_with('v') && r.contains('7') {
        return Some(5);
    }
    if r.starts_with("IV") || r.starts_with("iv") {
        return Some(4);
    }
    if r.starts_with("iii") {
        return Some(3);
    }
    if r.starts_with("ii") {
        return Some(2);
    }
    if r.starts_with('I') || r.starts_with('i') {
        return Some(1);
    }
    None
}

pub fn make_chord(
    root_pc: u8,
    quality: ChordQuality,
    roman: &str,
    function: HarmonicFunction,
) -> PlannedChord {
    PlannedChord {
        symbol: ChordSymbol {
            root: PitchClass { pc: root_pc % 12 },
            quality,
            extensions: vec![],
            bass: None,
            raw: format!(
                "{}{}",
                chord_root_name(root_pc),
                quality_suffix(quality)
            ),
        },
        roman: roman.to_string(),
        function,
        chromatic: None,
        rule_ids: vec![],
    }
}

/// Secondary dominant root for target scale degree in major.
pub fn secondary_dominant_of(degree: u8, tonic: u8) -> PlannedChord {
    let target = major_degree_pc(tonic, degree);
    let dom = (target + 7) % 12;
    let roman = match degree {
        2 => "V7/ii",
        4 => "V7/IV",
        5 => "V7/V",
        6 => "V7/vi",
        7 => "V7/vii",
        _ => "V7/x",
    };
    make_chord(dom, ChordQuality::Dominant7, roman, HarmonicFunction::Dominant)
}

/// Recompute roman/function for a chord under a new tonic.
pub fn reanalyze_chord(chord: &PlannedChord, tonic: u8, mode: Mode) -> (String, HarmonicFunction) {
    let pc = chord.symbol.root.pc;
    let interval = (pc + 12 - tonic) % 12;
    match (interval, mode, chord.symbol.quality) {
        (0, Mode::Major, ChordQuality::Major | ChordQuality::Major7) => {
            ("I".into(), HarmonicFunction::Tonic)
        }
        (0, Mode::Major, ChordQuality::Minor | ChordQuality::Minor7) => {
            ("i".into(), HarmonicFunction::Tonic)
        }
        (2, Mode::Major, ChordQuality::Minor | ChordQuality::Minor7) => {
            ("ii".into(), HarmonicFunction::Predominant)
        }
        (5, Mode::Major, _) => ("IV".into(), HarmonicFunction::Subdominant),
        (7, Mode::Major, ChordQuality::Dominant7) => {
            ("V7".into(), HarmonicFunction::Dominant)
        }
        (7, Mode::Major, _) => ("V".into(), HarmonicFunction::Dominant),
        (9, Mode::Major, ChordQuality::Minor) => ("vi".into(), HarmonicFunction::Tonic),
        (10, Mode::Major, ChordQuality::Dominant7) => {
            ("bVII7".into(), HarmonicFunction::Dominant)
        }
        (10, Mode::Major, _) => ("bVII".into(), HarmonicFunction::Subdominant),
        (8, Mode::Major, ChordQuality::Minor) => ("bVI".into(), HarmonicFunction::Subdominant),
        (5, Mode::Major, ChordQuality::Minor) => ("iv".into(), HarmonicFunction::Subdominant),
        _ => (chord.roman.clone(), chord.function),
    }
}
