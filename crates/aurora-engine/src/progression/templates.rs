//! Progression template library — functional harmony + classic pop/rock/jazz cells.

use aurora_ast::{ChordQuality, ChordSymbol, HarmonicFunction, Mode, PitchClass};

use super::mode::{default_function, default_triad_quality, ModeScale};

/// How scale degrees map to pitch classes when realizing a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarmonicBasis {
    /// Ionian / functional major-key harmony (melody mode is independent).
    Functional,
    /// Scale degrees follow the melody mode (modal harmony).
    Modal,
}

/// Which chord begins the loop — affects minor vs major emotional entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TonicAnchor {
    /// Starts on I (e.g. C Am F G — bright major feel).
    Major,
    /// Starts on vi (e.g. Am F C G — relative minor feel).
    RelativeMinor,
    /// Neutral — no preference.
    Any,
}

/// One slot in a progression template (scale degree based).
#[derive(Clone, Debug)]
pub struct RomanSlot {
    pub roman: &'static str,
    pub degree: u8,
    pub accidental: i8,
    pub quality: Option<ChordQuality>,
    pub function: HarmonicFunction,
}

/// Named progression template.
#[derive(Clone, Debug)]
pub struct ProgressionTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub slots: Vec<RomanSlot>,
    pub loop_friendly: bool,
    pub style_tags: &'static [&'static str],
    pub mode_tags: &'static [Mode],
    pub min_complexity: f32,
    pub harmonic_basis: HarmonicBasis,
    pub tonic_anchor: TonicAnchor,
}

#[derive(Clone, Debug)]
pub struct PlannedChord {
    pub symbol: ChordSymbol,
    pub roman: String,
    pub function: HarmonicFunction,
    pub chromatic: Option<super::chromatic::ChromaticKind>,
    pub rule_ids: Vec<String>,
}

impl ProgressionTemplate {
    pub fn realize(&self, tonic: u8) -> Vec<PlannedChord> {
        self.realize_mode(tonic, Mode::Major)
    }

    pub fn realize_mode(&self, tonic: u8, mode: Mode) -> Vec<PlannedChord> {
        let harmonic_mode = match self.harmonic_basis {
            HarmonicBasis::Functional => Mode::Major,
            HarmonicBasis::Modal => mode,
        };
        let scale = ModeScale::from_mode(harmonic_mode);
        self.slots
            .iter()
            .map(|s| {
                let quality = s.quality.unwrap_or_else(|| default_triad_quality(harmonic_mode, s.degree));
                let pc = scale.degree_pc(tonic, s.degree, s.accidental);
                PlannedChord {
                    symbol: ChordSymbol {
                        root: PitchClass { pc },
                        quality,
                        extensions: vec![],
                        bass: None,
                        raw: format!(
                            "{}{}",
                            chord_root_name(pc),
                            quality_suffix(quality)
                        ),
                    },
                    roman: s.roman.to_string(),
                    function: s.function,
                    chromatic: None,
                    rule_ids: vec![],
                }
            })
            .collect()
    }
}

fn slot(
    roman: &'static str,
    degree: u8,
    quality: ChordQuality,
    function: HarmonicFunction,
) -> RomanSlot {
    RomanSlot {
        roman,
        degree,
        accidental: 0,
        quality: Some(quality),
        function,
    }
}

fn flat(roman: &'static str, degree: u8, quality: ChordQuality, function: HarmonicFunction) -> RomanSlot {
    RomanSlot {
        roman,
        degree,
        accidental: -1,
        quality: Some(quality),
        function,
    }
}

fn tpl(
    id: &'static str,
    name: &'static str,
    slots: Vec<RomanSlot>,
    loop_friendly: bool,
    style_tags: &'static [&'static str],
    mode_tags: &'static [Mode],
    min_complexity: f32,
    harmonic_basis: HarmonicBasis,
    tonic_anchor: TonicAnchor,
) -> ProgressionTemplate {
    ProgressionTemplate {
        id,
        name,
        slots,
        loop_friendly,
        style_tags,
        mode_tags,
        min_complexity,
        harmonic_basis,
        tonic_anchor,
    }
}

pub fn template_library() -> Vec<ProgressionTemplate> {
    vec![
        // --- Canonical four-chord schemas (Open Music Theory / pop research) ---
        tpl(
            "POP-AXIS",
            "I-V-vi-IV",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
            ],
            true,
            &["pop", "rock", "contemporary", "worship"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "POP-CANON",
            "I-V-vi-IV (C G Am F)",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
            ],
            true,
            &["pop", "rock"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "POP-PLG",
            "I-IV-vi-V (C F Am G)",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "folk", "country"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "POP-AXIS-R",
            "vi-IV-I-V (Am F C G)",
            vec![
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "ballad", "anthem"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::RelativeMinor,
        ),
        tpl(
            "POP-EMO",
            "vi-V-IV-V (Am F C G variant)",
            vec![
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "emo", "indie"],
            &[],
            0.15,
            HarmonicBasis::Functional,
            TonicAnchor::RelativeMinor,
        ),
        tpl(
            "POP-HOP",
            "IV-V-vi-I (hopscotch)",
            vec![
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            true,
            &["pop", "contemporary"],
            &[],
            0.2,
            HarmonicBasis::Functional,
            TonicAnchor::Any,
        ),
        tpl(
            "CP-50S",
            "I-vi-IV-V (doo-wop)",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "doo-wop", "classical"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "CP-CYCLE",
            "I-vi-ii-V (gospel / rhythm changes)",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("ii", 2, ChordQuality::Minor, HarmonicFunction::Predominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["jazz", "pop", "gospel", "soul"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "CP-FUND",
            "I-IV-V-I",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            true,
            &["classical", "folk", "rock"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "CP-PLAGAL",
            "I-ii-IV-I",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("ii", 2, ChordQuality::Minor, HarmonicFunction::Predominant),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            true,
            &["classical", "pop"],
            &[],
            0.1,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "CP-MIX-8",
            "I-V-vi-iii-IV-I-ii-V",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("iii", 3, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("ii", 2, ChordQuality::Minor, HarmonicFunction::Predominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "rock", "worship"],
            &[],
            0.25,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "CP-IV-CHAIN",
            "IV-V-iii-vi-ii-V-I",
            vec![
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("iii", 3, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("ii", 2, ChordQuality::Minor, HarmonicFunction::Predominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            false,
            &["pop", "film", "rock"],
            &[],
            0.3,
            HarmonicBasis::Functional,
            TonicAnchor::Any,
        ),
        tpl(
            "POP-RICH",
            "I-V/vi-vi-IV-bVII-V",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("V/vi", 5, ChordQuality::Dominant7, HarmonicFunction::Dominant),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                flat("bVII", 7, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "rock"],
            &[],
            0.35,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "POP-COLOR",
            "I-IV-iv-V (modal interchange)",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                RomanSlot {
                    roman: "iv",
                    degree: 4,
                    accidental: 0,
                    quality: Some(ChordQuality::Minor),
                    function: HarmonicFunction::Subdominant,
                },
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "rock"],
            &[],
            0.3,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "CP-ANDALUS",
            "i-bVII-bVI-V (Andalusian cadence)",
            vec![
                slot("i", 1, ChordQuality::Minor, HarmonicFunction::Tonic),
                flat("bVII", 7, ChordQuality::Major, HarmonicFunction::Subdominant),
                flat("bVI", 6, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "rock", "flamenco", "metal"],
            &[],
            0.15,
            HarmonicBasis::Functional,
            TonicAnchor::RelativeMinor,
        ),
        tpl(
            "MIN-AND",
            "i-bVII-bVI-V",
            vec![
                slot("i", 1, ChordQuality::Minor, HarmonicFunction::Tonic),
                flat("bVII", 7, ChordQuality::Major, HarmonicFunction::Subdominant),
                flat("bVI", 6, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "rock"],
            &[Mode::NaturalMinor],
            0.0,
            HarmonicBasis::Modal,
            TonicAnchor::RelativeMinor,
        ),
        tpl(
            "CP-ROCK-MIX",
            "I-bVII-IV-I (mixolydian borrow)",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                flat("bVII", 7, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            true,
            &["rock", "pop"],
            &[],
            0.1,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "VI-IV-I-I",
            "vi-IV-I-I (Am F C C)",
            vec![
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            true,
            &["pop", "ballad"],
            &[],
            0.0,
            HarmonicBasis::Functional,
            TonicAnchor::RelativeMinor,
        ),
        tpl(
            "DOR-FUS",
            "i-IV-bVII-IV",
            vec![
                slot("i", 1, ChordQuality::Minor, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                flat("bVII", 7, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
            ],
            true,
            &["pop", "fusion", "rock"],
            &[Mode::Dorian],
            0.0,
            HarmonicBasis::Modal,
            TonicAnchor::RelativeMinor,
        ),
        tpl(
            "MIXO-POP",
            "I-bVII-IV-I",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                flat("bVII", 7, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            true,
            &["rock", "pop"],
            &[Mode::Mixolydian],
            0.0,
            HarmonicBasis::Modal,
            TonicAnchor::Major,
        ),
        tpl(
            "LYD-FLT",
            "I-II-I-V",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("II", 2, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
            ],
            true,
            &["pop", "film"],
            &[Mode::Lydian],
            0.0,
            HarmonicBasis::Modal,
            TonicAnchor::Major,
        ),
        tpl(
            "PHR-CAD",
            "i-bII-bVII-i",
            vec![
                slot("i", 1, ChordQuality::Minor, HarmonicFunction::Tonic),
                flat("bII", 2, ChordQuality::Major, HarmonicFunction::Subdominant),
                flat("bVII", 7, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("i", 1, ChordQuality::Minor, HarmonicFunction::Tonic),
            ],
            true,
            &["world", "metal"],
            &[Mode::Phrygian],
            0.0,
            HarmonicBasis::Modal,
            TonicAnchor::RelativeMinor,
        ),
        tpl(
            "JAZZ-IIV",
            "ii-V-I",
            vec![
                slot("ii7", 2, ChordQuality::Minor7, HarmonicFunction::Predominant),
                slot("V7", 5, ChordQuality::Dominant7, HarmonicFunction::Dominant),
                slot("Imaj7", 1, ChordQuality::Major7, HarmonicFunction::Tonic),
            ],
            false,
            &["jazz", "swing", "bebop"],
            &[],
            0.45,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "JAZZ-TA",
            "I-VI-ii-V",
            vec![
                slot("Imaj7", 1, ChordQuality::Major7, HarmonicFunction::Tonic),
                slot("VI7", 6, ChordQuality::Dominant7, HarmonicFunction::Dominant),
                slot("ii7", 2, ChordQuality::Minor7, HarmonicFunction::Predominant),
                slot("V7", 5, ChordQuality::Dominant7, HarmonicFunction::Dominant),
            ],
            true,
            &["jazz", "swing"],
            &[],
            0.45,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
        tpl(
            "WAVE-8",
            "I-IV-V/vi-vi-iv-V-I",
            vec![
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
                slot("IV", 4, ChordQuality::Major, HarmonicFunction::Subdominant),
                slot("V/vi", 5, ChordQuality::Dominant7, HarmonicFunction::Dominant),
                slot("vi", 6, ChordQuality::Minor, HarmonicFunction::Tonic),
                RomanSlot {
                    roman: "iv",
                    degree: 4,
                    accidental: 0,
                    quality: Some(ChordQuality::Minor),
                    function: HarmonicFunction::Subdominant,
                },
                slot("V", 5, ChordQuality::Major, HarmonicFunction::Dominant),
                slot("I", 1, ChordQuality::Major, HarmonicFunction::Tonic),
            ],
            false,
            &["pop", "film"],
            &[],
            0.45,
            HarmonicBasis::Functional,
            TonicAnchor::Major,
        ),
    ]
}

pub fn select_template(genre: &str, jazz: bool, loop_mode: bool) -> ProgressionTemplate {
    let genre = genre.to_lowercase();
    let library = template_library();

    if jazz {
        return library
            .iter()
            .find(|t| t.id == "JAZZ-TA" || (t.id == "JAZZ-IIV" && !loop_mode))
            .or_else(|| library.iter().find(|t| t.id == "JAZZ-IIV"))
            .cloned()
            .unwrap_or_else(|| {
                template_library()
                    .into_iter()
                    .find(|t| t.id == "JAZZ-IIV")
                    .unwrap_or_else(|| template_library()[0].clone())
            });
    }

    if genre.contains("classical") || genre.contains("baroque") {
        return library
            .iter()
            .find(|t| t.id == "CP-FUND")
            .cloned()
            .unwrap_or_else(|| {
                template_library()
                    .into_iter()
                    .find(|t| t.id == "CP-FUND")
                    .unwrap_or_else(|| template_library()[0].clone())
            });
    }

    if loop_mode {
        library
            .iter()
            .find(|t| t.id == "POP-RICH")
            .or_else(|| library.iter().find(|t| t.id == "POP-AXIS"))
            .cloned()
            .unwrap_or_else(|| library[0].clone())
    } else {
        library
            .iter()
            .find(|t| t.id == "WAVE-8")
            .or_else(|| library.iter().find(|t| t.id == "CP-CYCLE"))
            .cloned()
            .unwrap_or_else(|| library[4].clone())
    }
}

pub fn templates_for_mode(mode: Mode, complexity: f32, loop_mode: bool) -> Vec<ProgressionTemplate> {
    template_library()
        .into_iter()
        .filter(|t| {
            let mode_ok = t.mode_tags.is_empty()
                || t.mode_tags.contains(&mode)
                || t.harmonic_basis == HarmonicBasis::Functional;
            t.min_complexity <= complexity
                && mode_ok
                && (loop_mode || !t.loop_friendly || t.id == "WAVE-8")
        })
        .filter(|t| loop_mode || !t.loop_friendly || t.slots.len() >= 4)
        .collect()
}

pub fn chord_root_name(pc: u8) -> &'static str {
    match pc % 12 {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "Eb",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "Ab",
        9 => "A",
        10 => "Bb",
        11 => "B",
        _ => "C",
    }
}

pub fn quality_suffix(q: ChordQuality) -> &'static str {
    match q {
        ChordQuality::Major => "",
        ChordQuality::Major7 => "maj7",
        ChordQuality::Minor => "m",
        ChordQuality::Minor7 => "m7",
        ChordQuality::Dominant7 => "7",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dorian_template_realizes_iv_major() {
        let lib = template_library();
        let dorfus = lib.iter().find(|t| t.id == "DOR-FUS").unwrap();
        let chords = dorfus.realize_mode(0, Mode::Dorian);
        assert_eq!(chords[1].symbol.root.pc, 5); // F major
        assert_eq!(chords[1].symbol.quality, ChordQuality::Major);
    }
}
