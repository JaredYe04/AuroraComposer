//! Search-time evaluation context using `aurora-ast` types.
//!
//! Rule predicates operate on this lightweight snapshot during beam search;
//! full composition AST lives in `aurora_ast::AstSnapshot`.

use aurora_ast::{
    CadenceType, ChordSymbol, Event, KeySignature, Mode, NoteEvent, Pitch, PitchClass,
    TimedEventBase, VoiceId, VoiceRole, WrittenDuration,
};
use aurora_core::NodeId;
use aurora_ast::{BeatOffset, NoteType, TieSpec};
use aurora_core::ParameterBundle;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use aurora_ast::{CadenceType as AstCadenceType, ChordQuality as AstChordQuality};

/// Extension for pitch-class access during rule evaluation.
pub trait PitchExt {
    fn pitch_class(&self) -> PitchClass;
}

impl PitchExt for Pitch {
    fn pitch_class(&self) -> PitchClass {
        PitchClass {
            pc: self.midi % 12,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeatStrengthKind {
    Strong,
    Medium,
    Weak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeatStrength(pub BeatStrengthKind);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatchId(pub Uuid);

impl PatchId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PatchId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidatePatch {
    pub id: PatchId,
    pub voice_id: VoiceId,
    pub measure_id: NodeId,
    pub nodes_to_add: Vec<Event>,
    pub label: Option<String>,
}

impl CandidatePatch {
    #[must_use]
    pub fn single_note(
        voice_id: VoiceId,
        measure_id: NodeId,
        event: Event,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: PatchId::new(),
            voice_id,
            measure_id,
            nodes_to_add: vec![event],
            label: Some(label.into()),
        }
    }
}

/// Minimal note for search candidate patches.
#[must_use]
pub fn search_note(midi: u8, id: NodeId) -> Event {
    Event::Note(NoteEvent {
        base: TimedEventBase {
            id,
            offset: BeatOffset::zero(),
            duration: WrittenDuration {
                note_type: NoteType::Quarter,
                dots: 0,
                tuplet: None,
            },
            provenance: aurora_ast::Provenance::generated(
                aurora_ast::PipelineStageId::Melody,
                "search",
            ),
            visible: true,
        },
        pitch: Pitch::from_midi(midi),
        velocity: 80,
        tie: TieSpec::None,
        articulations: Vec::new(),
        ornaments: Vec::new(),
        lyric: None,
        pitch_role: None,
        stem_direction: None,
        beam_group: None,
        is_drum: false,
        drum_map: None,
    })
}

/// Search-time AST window (subset of full composition for rule evaluation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AstSnapshot {
    pub key: KeySignature,
    pub melody_pitches: Vec<Pitch>,
    pub bass_pitches: Vec<Pitch>,
    pub alto_pitches: Vec<Pitch>,
    pub current_chord: Option<ChordSymbol>,
    pub beat_offset: BeatOffset,
    pub beat_strength: BeatStrength,
    pub cadence: CadenceType,
    pub phrase_end: bool,
    pub is_phrase_end_measure: bool,
    pub grid_subdivision: u8,
    pub melody_register: (u8, u8),
    pub bass_register: (u8, u8),
    pub chord_grid: Option<Vec<ChordSymbol>>,
    pub beats_per_measure: u8,
}

impl Default for AstSnapshot {
    fn default() -> Self {
        Self {
            key: KeySignature {
                tonic: PitchClass { pc: 0 },
                mode: Mode::Major,
            },
            melody_pitches: Vec::new(),
            bass_pitches: Vec::new(),
            alto_pitches: Vec::new(),
            current_chord: None,
            beat_offset: BeatOffset::zero(),
            beat_strength: BeatStrength(BeatStrengthKind::Strong),
            cadence: CadenceType::None,
            phrase_end: false,
            is_phrase_end_measure: false,
            grid_subdivision: 4,
            melody_register: (60, 84),
            bass_register: (36, 60),
            chord_grid: None,
            beats_per_measure: 4,
        }
    }
}

impl AstSnapshot {
    #[must_use]
    pub fn with_chord_grid(mut self, grid: Vec<ChordSymbol>, beats_per_measure: u8) -> Self {
        self.chord_grid = Some(grid);
        self.beats_per_measure = beats_per_measure.max(1);
        self.current_chord = self.chord_grid.as_ref().and_then(|g| g.first().cloned());
        self
    }

    #[must_use]
    pub fn with_registers_from(mut self, params: &ParameterBundle) -> Self {
        self.melody_register = (
            params.register.melody_register_min,
            params.register.melody_register_max,
        );
        self.bass_register = (
            params.register.bass_register_min,
            params.register.bass_register_max,
        );
        self
    }

    #[must_use]
    pub fn for_step(&self, step: u32) -> Self {
        let mut next = self.clone();
        if let Some(grid) = &self.chord_grid {
            let beats = usize::from(self.beats_per_measure.max(1));
            let measure_idx = step as usize / beats;
            next.current_chord = grid.get(measure_idx).cloned();
            let beat = step as usize % beats;
            next.beat_strength = BeatStrength(if beat == 0 || beat == 2 {
                BeatStrengthKind::Strong
            } else {
                BeatStrengthKind::Weak
            });
        }
        next
    }

    #[must_use]
    pub fn apply(&self, patch: &CandidatePatch) -> Self {
        let mut next = self.clone();
        for event in &patch.nodes_to_add {
            if let Event::Note(note) = event {
                match patch.voice_id.0 {
                    0 => next.melody_pitches.push(note.pitch),
                    1 => next.alto_pitches.push(note.pitch),
                    2 => next.bass_pitches.push(note.pitch),
                    _ => next.melody_pitches.push(note.pitch),
                }
            }
        }
        if let Some(grid) = &self.chord_grid {
            let step = next.melody_pitches.len();
            let measure_idx = step / usize::from(self.beats_per_measure.max(1));
            next.current_chord = grid.get(measure_idx).cloned();
            let beat = step % usize::from(self.beats_per_measure.max(1));
            next.beat_strength = BeatStrength(if beat == 0 || beat == 2 {
                BeatStrengthKind::Strong
            } else {
                BeatStrengthKind::Weak
            });
        }
        next
    }

    #[must_use]
    pub fn last_pitch(&self, voice: VoiceRole) -> Option<Pitch> {
        match voice {
            VoiceRole::Melody | VoiceRole::Lead => self.melody_pitches.last().copied(),
            VoiceRole::Bass | VoiceRole::BassLine => self.bass_pitches.last().copied(),
            VoiceRole::Alto | VoiceRole::Tenor | VoiceRole::Inner => self.alto_pitches.last().copied(),
            VoiceRole::Drums | VoiceRole::Percussion => None,
            _ => self.melody_pitches.last().copied(),
        }
    }

    #[must_use]
    pub fn prev_melody_pitch(&self) -> Option<Pitch> {
        self.melody_pitches.last().copied()
    }
}

#[derive(Clone, Debug)]
pub struct EvaluationContext<'a> {
    pub snapshot: &'a AstSnapshot,
    pub patch: &'a CandidatePatch,
    pub voice_role: VoiceRole,
    pub step_index: u32,
}

impl<'a> EvaluationContext<'a> {
    #[must_use]
    pub fn candidate_pitch(&self) -> Option<Pitch> {
        self.patch.nodes_to_add.iter().find_map(|e| {
            if let Event::Note(n) = e {
                Some(n.pitch)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn prev_pitch(&self) -> Option<Pitch> {
        self.snapshot.last_pitch(self.voice_role)
    }

    #[must_use]
    pub fn interval_semitones(&self) -> Option<i16> {
        let curr = self.candidate_pitch()?;
        let prev = self.prev_pitch()?;
        Some(curr.midi as i16 - prev.midi as i16)
    }

    #[must_use]
    pub fn is_strong_beat(&self) -> bool {
        matches!(self.snapshot.beat_strength.0, BeatStrengthKind::Strong)
    }
}

/// Parameter → weight mapping helpers (scoring.md §11).
pub trait RuleWeightMapping {
    fn chord_tone_weight(&self) -> f64;
    fn cadence_strength_weight(&self) -> f64;
    fn stepwise_preference(&self) -> f64;
    fn leap_penalty(&self) -> f64;
    fn leap_limit_semitones(&self) -> u8;
    fn parallel_penalty(&self) -> f64;
    fn counterpoint_strictness(&self) -> f64;
    fn borrowed_chord_tolerance(&self) -> f64;
    fn harmony_complexity(&self) -> f64;
    fn repetition_ratio(&self) -> f64;
    fn syncopation(&self) -> f64;
    fn dissonance_tolerance(&self) -> f64;
}

impl RuleWeightMapping for ParameterBundle {
    fn chord_tone_weight(&self) -> f64 {
        lerp(0.0, 50.0, self.harmony.complexity as f64)
    }

    fn cadence_strength_weight(&self) -> f64 {
        lerp(0.0, 50.0, self.harmony.cadence_strength as f64)
    }

    fn stepwise_preference(&self) -> f64 {
        lerp(0.0, 30.0, self.voice.density as f64)
    }

    fn leap_penalty(&self) -> f64 {
        lerp(0.0, 20.0, self.voice.density as f64)
    }

    fn leap_limit_semitones(&self) -> u8 {
        7
    }

    fn parallel_penalty(&self) -> f64 {
        lerp(0.0, 100.0, self.counterpoint.parallel_penalty as f64)
    }

    fn counterpoint_strictness(&self) -> f64 {
        self.counterpoint.strictness as f64
    }

    fn borrowed_chord_tolerance(&self) -> f64 {
        self.scale.borrowed_chord_tolerance as f64
    }

    fn harmony_complexity(&self) -> f64 {
        self.harmony.complexity as f64
    }

    fn repetition_ratio(&self) -> f64 {
        self.theme.repetition_ratio as f64
    }

    fn syncopation(&self) -> f64 {
        self.rhythm.syncopation as f64
    }

    fn dissonance_tolerance(&self) -> f64 {
        self.harmony.dissonance as f64
    }
}

fn lerp(min: f64, max: f64, t: f64) -> f64 {
    min + (max - min) * t.clamp(0.0, 1.0)
}
