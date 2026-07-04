//! Motif types for theme planning and melody realization.

mod generator;

pub use generator::generate_motif;

use aurora_ast::nodes::ThemeTransform;
use serde::{Deserialize, Serialize};

/// Semitone interval relative to the first pitch of the motif.
pub type Semitone = i8;

/// Duration class for one motif cell (one quarter beat).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotifDur {
    Quarter,
    TwoEighths,
    SyncopatedEighth,
    DottedQuarter,
    RestThenEighth,
}

/// A short melodic cell: interval pattern + rhythm weights.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Motif {
    pub id: String,
    /// Intervals from first pitch (first is always 0).
    pub intervals: Vec<Semitone>,
    /// Contour signature: -1, 0, +1 per interval step.
    pub contour: Vec<i8>,
    /// Rhythmic shape per cell (same length as intervals).
    pub rhythm: Vec<MotifDur>,
    /// Anchor pitch class for transposition (0–11).
    pub tonic_anchor: u8,
    /// Base MIDI for realization (octave 4–5 range).
    pub base_midi: u8,
}

impl Motif {
    /// Realize motif pitches at a given transposition base.
    pub fn realize_pitches(&self, base_midi: u8) -> Vec<u8> {
        let mut pitches = vec![base_midi];
        let mut current = base_midi as i16;
        for &interval in self.intervals.iter().skip(1) {
            current += i16::from(interval);
            pitches.push(current.clamp(0, 127) as u8);
        }
        pitches
    }

    /// Expected pitch at cell index (wraps for repetition).
    pub fn pitch_at(&self, cell_index: usize, base_midi: u8) -> u8 {
        let pitches = self.realize_pitches(base_midi);
        pitches[cell_index % pitches.len()]
    }

    pub fn rhythm_at(&self, cell_index: usize) -> MotifDur {
        self.rhythm
            .get(cell_index % self.rhythm.len())
            .copied()
            .unwrap_or(MotifDur::Quarter)
    }

    pub fn len(&self) -> usize {
        self.intervals.len().max(1)
    }

    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }
}

/// Tracks position within a motif during melody beam search.
#[derive(Clone, Debug, Default)]
pub struct MotifCursor {
    pub motif_id: Option<String>,
    pub cell_index: usize,
    pub base_midi: u8,
    pub active: bool,
    /// Phrase-local: how many beats the motif region spans.
    pub region_beats: usize,
}

impl MotifCursor {
    pub fn new(motif: &Motif, repetition_ratio: f32, phrase_beats: usize) -> Self {
        let region = ((phrase_beats as f32) * repetition_ratio).max(4.0) as usize;
        Self {
            motif_id: Some(motif.id.clone()),
            cell_index: 0,
            base_midi: motif.base_midi,
            active: true,
            region_beats: region,
        }
    }

    pub fn expected_pitch(&self, motif: &Motif) -> u8 {
        motif.pitch_at(self.cell_index, self.base_midi)
    }

    pub fn advance(&mut self, motif: &Motif) {
        self.cell_index = (self.cell_index + 1) % motif.len();
    }

    pub fn in_motif_region(&self, beat_in_phrase: usize) -> bool {
        self.active && beat_in_phrase < self.region_beats
    }
}

/// Apply a theme transform to a motif.
pub fn apply_transform(motif: &Motif, transform: ThemeTransform) -> Motif {
    match transform {
        ThemeTransform::Original => motif.clone(),
        ThemeTransform::Sequence => sequence_motif(motif, 2),
        ThemeTransform::Inversion => invert_motif(motif),
        ThemeTransform::Retrograde => retrograde_motif(motif),
        ThemeTransform::Augmentation => motif.clone(), // rhythm handled separately
        ThemeTransform::Diminution => fragment_motif(motif, 0, motif.intervals.len() / 2 + 1),
        ThemeTransform::Fragmentation => fragment_motif(motif, 0, (motif.intervals.len() / 2).max(2)),
        ThemeTransform::ModalInterchange => sequence_motif(motif, -2),
    }
}

fn sequence_motif(motif: &Motif, semitones: Semitone) -> Motif {
    Motif {
        intervals: motif.intervals.iter().map(|i| i + semitones).collect(),
        contour: motif.contour.clone(),
        rhythm: motif.rhythm.clone(),
        ..motif.clone()
    }
}

fn invert_motif(motif: &Motif) -> Motif {
    let mut intervals = vec![0i8];
    let mut prev = 0i8;
    for &interval in motif.intervals.iter().skip(1) {
        let step = interval - prev;
        intervals.push(-step);
        prev = interval;
    }
    let contour: Vec<i8> = intervals
        .windows(2)
        .map(|w| w[1].signum())
        .collect();
    Motif {
        intervals,
        contour,
        rhythm: motif.rhythm.clone(),
        ..motif.clone()
    }
}

fn retrograde_motif(motif: &Motif) -> Motif {
    let mut intervals: Vec<Semitone> = motif.intervals.iter().rev().copied().collect();
    // Re-anchor so first interval is 0
    if let Some(&first) = intervals.first() {
        intervals = intervals.iter().map(|i| i - first).collect();
    }
    Motif {
        intervals,
        contour: motif.contour.iter().rev().copied().collect(),
        rhythm: motif.rhythm.iter().rev().copied().collect(),
        ..motif.clone()
    }
}

fn fragment_motif(motif: &Motif, start: usize, end: usize) -> Motif {
    let end = end.min(motif.intervals.len());
    let start = start.min(end);
    let mut intervals = motif.intervals[start..end].to_vec();
    if intervals.is_empty() {
        intervals = vec![0];
    }
    // Re-anchor
    if let Some(&first) = intervals.first() {
        intervals = intervals.iter().map(|i| i - first).collect();
    }
    Motif {
        intervals,
        contour: motif.contour.get(start..end).unwrap_or(&[]).to_vec(),
        rhythm: motif.rhythm.get(start..end).unwrap_or(&[]).to_vec(),
        ..motif.clone()
    }
}

/// Compute interval-pattern similarity between realized melody and motif (0.0–1.0).
pub fn motif_interval_similarity(melody_pitches: &[u8], motif: &Motif) -> f64 {
    if melody_pitches.len() < 2 || motif.intervals.len() < 2 {
        return 0.0;
    }
    let motif_intervals: Vec<i8> = motif
        .intervals
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    let melody_intervals: Vec<i8> = melody_pitches
        .windows(2)
        .map(|w| (w[1] as i16 - w[0] as i16).clamp(-12, 12) as i8)
        .collect();

    let mut matches = 0usize;
    let compare_len = motif_intervals.len().min(melody_intervals.len());
    for i in 0..compare_len {
        if motif_intervals[i] == melody_intervals[i] {
            matches += 1;
        } else if motif_intervals[i].signum() == melody_intervals[i].signum() {
            matches += 1; // partial: same direction
        }
    }
    if compare_len == 0 {
        0.0
    } else {
        matches as f64 / compare_len as f64
    }
}
