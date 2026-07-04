//! P7 — Per-beat harmonic rhythm expansion.

use aurora_ast::{BeatOffset, ChordSymbol, HarmonySlot, Measure};

use super::templates::PlannedChord;

/// Map harmonic_rhythm (0–1) to approximate chord changes per measure.
pub fn chords_per_measure(harmonic_rhythm: f32, beats: u8) -> u8 {
    let base = 0.45 + harmonic_rhythm * 1.55;
    base.min(beats as f32).max(0.5).round() as u8
}

/// Beat offsets for chord changes within a bar.
pub fn change_beats(changes: u8, beats: u8) -> Vec<u32> {
    let b = u32::from(beats);
    match changes.max(1) {
        1 => vec![0],
        2 => vec![0, b / 2],
        3 => vec![0, b / 3, 2 * b / 3],
        _ => (0..u32::from(changes.min(beats))).collect(),
    }
}

/// Expand single-slot measures into multiple harmony slots per harmonic rhythm.
pub fn expand_measure_harmony_slots(
    measures: &mut [Measure],
    progression: &[PlannedChord],
    harmonic_rhythm: f32,
    beats_per_measure: u8,
) {
    let changes = chords_per_measure(harmonic_rhythm, beats_per_measure);
    let change_beats = change_beats(changes, beats_per_measure);

    for (mi, measure) in measures.iter_mut().enumerate() {
        let base = progression
            .get(mi)
            .or_else(|| progression.last())
            .cloned()
            .unwrap_or_else(|| {
                super::roman::make_chord(
                    0,
                    aurora_ast::ChordQuality::Major,
                    "I",
                    aurora_ast::HarmonicFunction::Tonic,
                )
            });

        let prev = progression.get(mi.saturating_sub(1));
        measure.harmony_slots.clear();

        for (idx, &start) in change_beats.iter().enumerate() {
            let end = change_beats.get(idx + 1).copied().unwrap_or(u32::from(beats_per_measure));
            let dur = end.saturating_sub(start).max(1);
            let slot_chord = slot_chord_for_beat(&base, prev, mi, start, changes, progression);
            measure.harmony_slots.push(make_slot(&slot_chord, start, dur, mi));
        }
        if measure.harmony_slots.is_empty() {
            measure.harmony_slots.push(make_slot(&base, 0, u32::from(beats_per_measure), mi));
        }
    }
}

fn slot_chord_for_beat(
    base: &PlannedChord,
    prev: Option<&PlannedChord>,
    measure_idx: usize,
    beat: u32,
    changes: u8,
    progression: &[PlannedChord],
) -> PlannedChord {
    if changes <= 1 {
        return base.clone();
    }
    // Half-bar: second half advances loop cell
    if beat > 0 {
        if let Some(next) = progression.get(measure_idx + 1) {
            return next.clone();
        }
        if let Some(p) = prev {
            return p.clone();
        }
    }
    base.clone()
}

fn make_slot(chord: &PlannedChord, start: u32, duration: u32, mi: usize) -> HarmonySlot {
    HarmonySlot {
        id: aurora_core::NodeId::new(u64::try_from(mi * 10 + start as usize).unwrap_or(1000), 0),
        offset: BeatOffset::new(start, 1),
        duration: BeatOffset::new(duration.max(1), 1),
        symbol: chord.symbol.clone(),
        roman_numeral: Some(chord.roman.clone()),
        function: Some(chord.function),
        provenance: aurora_ast::Provenance {
            source: aurora_ast::ProvenanceSource::Generated,
            stage: Some(aurora_ast::PipelineStageId::HarmonySkeleton),
            rule_ids: chord.rule_ids.clone(),
            rule_refs: vec![],
            eval_score: None,
            search: None,
            parent: None,
            created_at: String::new(),
            agent: aurora_ast::ProvenanceAgent::Engine {
                stage: aurora_ast::PipelineStageId::HarmonySkeleton,
            },
            parameters_hash: None,
            explanation: Some(format!("harmonic rhythm slot beat {start}")),
        },
    }
}

/// Build per-beat chord grid for beam search (length = bars × beats).
pub fn build_per_beat_chord_grid(measures: &[Measure], beats_per_measure: u8) -> Vec<ChordSymbol> {
    let bpm = beats_per_measure.max(1) as usize;
    let mut grid = Vec::new();
    for measure in measures {
        for beat in 0..bpm {
            let chord = chord_at_beat(measure, beat as u32)
                .unwrap_or_else(|| measure.harmony_slots.first().map(|s| s.symbol.clone()).unwrap_or_else(
                    || ChordSymbol::simple(0, aurora_ast::ChordQuality::Major, "C"),
                ));
            grid.push(chord);
        }
    }
    grid
}

pub fn chord_at_beat(measure: &Measure, beat: u32) -> Option<ChordSymbol> {
    measure.harmony_slots.iter().find_map(|slot| {
        let start = slot.offset.numer / slot.offset.denom.max(1);
        let dur = slot.duration.numer / slot.duration.denom.max(1);
        if beat >= start && beat < start + dur {
            Some(slot.symbol.clone())
        } else {
            None
        }
    })
}

/// Whether this step is a harmony boundary (chord change).
pub fn is_harmony_boundary(grid: &[ChordSymbol], step: usize) -> bool {
    if step == 0 {
        return true;
    }
    grid.get(step).map(|c| c.root.pc) != grid.get(step - 1).map(|c| c.root.pc)
}
