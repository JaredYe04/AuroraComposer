use aurora_ast::{Composition, VoiceRole};
use aurora_core::ExportError;

use crate::ir::{IrEvent, MusicIr};

#[derive(Clone, Debug)]
pub enum AbcExportMode {
    MelodyOnly,
    LeadSheet,
    MultiVoice,
    Full,
}

#[derive(Clone, Debug)]
pub struct AbcExportConfig {
    pub mode: AbcExportMode,
    pub max_voices: u8,
    pub include_chord_symbols: bool,
    pub tune_number: u32,
    pub aurora_comments: bool,
}

impl Default for AbcExportConfig {
    fn default() -> Self {
        Self {
            mode: AbcExportMode::LeadSheet,
            max_voices: 1,
            include_chord_symbols: true,
            tune_number: 1,
            aurora_comments: true,
        }
    }
}

pub fn export_abc(
    comp: &Composition,
    ir: &MusicIr,
    config: &AbcExportConfig,
) -> Result<String, ExportError> {
    let voices: Vec<_> = ir
        .channel_index
        .iter()
        .filter(|v| !v.is_drum)
        .take(usize::from(config.max_voices.min(4)))
        .collect();

    if voices.is_empty() {
        return Err(ExportError::Abc("no non-drum voices to export".into()));
    }
    if ir.channel_index.len() > 4 && matches!(config.mode, AbcExportMode::Full) {
        return Err(ExportError::Abc("E_ABC_VOICE_OVERFLOW: more than 4 voices".into()));
    }

    let mut out = String::new();
    if config.aurora_comments {
        out.push_str("%%aurora-version: 0.1\n");
        out.push_str(&format!("%%aurora-generated: {}\n", ir.header.projected_at));
    }

    out.push_str(&format!("X:{}\n", config.tune_number));
    out.push_str(&format!("T:{}\n", escape_header(&comp.metadata.title)));
    if let Some(ref composer) = comp.metadata.composer {
        if !composer.is_empty() {
            out.push_str(&format!("C:{}\n", escape_header(composer)));
        }
    }

    let meter = comp.global.meter_map.default;
    out.push_str(&format!("M:{}/{}\n", meter.beats, meter.beat_type));
    out.push_str("L:1/8\n");

    let tempo = comp.global.tempo_map.default_bpm;
    if tempo > 0.0 {
        out.push_str(&format!("Q:1/4={}\n", tempo.round() as u32));
    }

    let key = key_header(comp);
    out.push_str(&format!("K:{key}\n"));

    let multi = matches!(config.mode, AbcExportMode::MultiVoice | AbcExportMode::Full)
        && voices.len() > 1;

    if multi {
        for (i, view) in voices.iter().enumerate() {
            out.push_str(&format!("V:{} name={}\n", i + 1, view.name));
        }
    }

    let ppq = ir.header.ppq;
    let measure_ticks =
        (measure_quarters(meter) * f64::from(ppq)).round() as u64;
    let measure_count = if measure_ticks > 0 {
        (ir.header.total_ticks / measure_ticks).max(1)
    } else {
        1
    };

    let chord_line = if config.include_chord_symbols
        && matches!(
            config.mode,
            AbcExportMode::LeadSheet | AbcExportMode::MultiVoice | AbcExportMode::Full
        )
    {
        Some(build_chord_line(comp, measure_count))
    } else {
        None
    };

    if let Some(ref w) = chord_line {
        out.push_str(&format!("w: {w}\n"));
    }

    for (vi, view) in voices.iter().enumerate() {
        if multi {
            out.push_str(&format!("V:{}\n", vi + 1));
        }

        for m in 0..measure_count {
            let start = m * measure_ticks;
            let end = start + measure_ticks;
            let mut notes: Vec<(u64, String)> = Vec::new();

            for &idx in &view.event_indices {
                match &ir.events[idx] {
                    IrEvent::Note(note) if note.base.tick >= start && note.base.tick < end => {
                        let rel = note.base.tick - start;
                        let abc = encode_note(note.midi, note.duration_ticks, ppq);
                        notes.push((rel, abc));
                    }
                    IrEvent::Rest(rest) if rest.base.tick >= start && rest.base.tick < end => {
                        let rel = rest.base.tick - start;
                        let abc = encode_rest(rest.duration_ticks, ppq);
                        notes.push((rel, abc));
                    }
                    _ => {}
                }
            }

            notes.sort_by_key(|(t, _)| *t);
            let bar = notes.into_iter().map(|(_, s)| s).collect::<Vec<_>>().join("");
            let barline = if m + 1 == measure_count { "|]" } else { "|" };
            out.push_str(&format!("{bar} {barline}\n"));
        }
    }

    Ok(out)
}

fn build_chord_line(comp: &Composition, measure_count: u64) -> String {
    let mut symbols = Vec::new();
    let mut global = 0u32;

    'outer: for movement in &comp.movements {
        for section in &movement.sections {
            for phrase in &section.phrases {
                for measure in &phrase.measures {
                    if u64::from(global) >= measure_count {
                        break 'outer;
                    }
                    let sym = measure
                        .harmony_slots
                        .first()
                        .map(|s| s.symbol.raw.clone())
                        .unwrap_or_else(|| "z".into());
                    symbols.push(sym);
                    global += 1;
                }
            }
        }
    }

    while symbols.len() < measure_count as usize {
        symbols.push("z".into());
    }

    symbols.join(" | ") + " |"
}

fn key_header(comp: &Composition) -> String {
    let key = comp.global.key_map.default;
    let names = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];
    let root = names[key.tonic.pc as usize % 12];
    match key.mode {
        aurora_ast::nodes::Mode::Major => root.to_string(),
        aurora_ast::nodes::Mode::NaturalMinor
        | aurora_ast::nodes::Mode::HarmonicMinor
        | aurora_ast::nodes::Mode::MelodicMinor => format!("{root}m"),
        _ => root.to_string(),
    }
}

fn encode_note(midi: u8, duration_ticks: u32, ppq: u16) -> String {
    let pitch = encode_pitch(midi);
    let dur = encode_duration(duration_ticks, ppq);
    if dur == 1 {
        pitch
    } else if dur < 1 {
        format!("{pitch}/{}/", 2)
    } else {
        format!("{pitch}{dur}")
    }
}

fn encode_rest(duration_ticks: u32, ppq: u16) -> String {
    let dur = encode_duration(duration_ticks, ppq);
    if dur == 1 {
        "z".into()
    } else {
        format!("z{dur}")
    }
}

/// ABC duration multiplier relative to `L:1/8`.
fn encode_duration(duration_ticks: u32, ppq: u16) -> u32 {
    let eighths = (f64::from(duration_ticks) / f64::from(ppq) * 2.0).round() as u32;
    eighths.max(1)
}

fn encode_pitch(midi: u8) -> String {
    let semitone = midi % 12;
    let octave = i16::from(midi) / 12 - 1;
    let (letter, acc) = match semitone {
        0 => ('C', 0),
        1 => ('C', 1),
        2 => ('D', 0),
        3 => ('D', 1),
        4 => ('E', 0),
        5 => ('F', 0),
        6 => ('F', 1),
        7 => ('G', 0),
        8 => ('G', 1),
        9 => ('A', 0),
        10 => ('A', 1),
        _ => ('B', 0),
    };

    let mut s = String::new();
    if acc > 0 {
        s.push('^');
    } else if acc < 0 {
        s.push('_');
    }

    let rel = octave - 4;
    match rel.cmp(&0) {
        std::cmp::Ordering::Less => {
            s.push(letter);
            for _ in 0..(-rel) {
                s.push(',');
            }
        }
        std::cmp::Ordering::Equal => {
            s.push(letter);
        }
        std::cmp::Ordering::Greater => {
            s.push(letter.to_ascii_lowercase());
            for _ in 1..rel {
                s.push('\'');
            }
        }
    }
    s
}

fn escape_header(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_ascii() { c } else { '?' })
        .collect()
}

fn measure_quarters(meter: aurora_ast::TimeSignature) -> f64 {
    f64::from(meter.beats) * (4.0 / f64::from(meter.beat_type))
}

pub fn abc_voice_count_warning(comp: &Composition) -> Option<String> {
    let melodic = comp
        .voice_registry
        .voices
        .iter()
        .filter(|v| !matches!(v.role, VoiceRole::Drums | VoiceRole::Percussion))
        .count();
    if melodic > 2 {
        Some(format!(
            "ABC export may be unreadable with {melodic} melodic voices; consider MusicXML"
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{project_ast_to_ir, DEFAULT_PPQ};
    use crate::test_fixtures::sample_composition;

    #[test]
    fn export_abc_has_required_headers() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let abc = export_abc(&comp, &ir, &AbcExportConfig::default()).unwrap();
        assert!(abc.contains("X:1\n"));
        assert!(abc.contains("T:Test\n"));
        assert!(abc.contains("M:4/4\n"));
        assert!(abc.contains("L:1/8\n"));
        assert!(abc.contains("K:"));
    }

    #[test]
    fn export_abc_encodes_middle_c() {
        let comp = sample_composition();
        let ir = project_ast_to_ir(&comp, DEFAULT_PPQ).unwrap();
        let abc = export_abc(
            &comp,
            &ir,
            &AbcExportConfig {
                mode: AbcExportMode::MelodyOnly,
                include_chord_symbols: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(abc.contains("C2"));
    }
}
