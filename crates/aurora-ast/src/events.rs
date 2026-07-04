//! Timed event sum type and accessors.
//!
//! See `docs/02-music-model/events.md`.

use aurora_core::NodeId;
use serde::{Deserialize, Serialize};

use crate::nodes::{ChordSymbol, DynamicLevel};
use crate::provenance::Provenance;
use crate::types::{BeatOffset, Pitch, WrittenDuration};

/// Common header for timed events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimedEventBase {
    pub id: NodeId,
    pub offset: BeatOffset,
    pub duration: WrittenDuration,
    pub provenance: Provenance,
    #[serde(default = "default_visible")]
    pub visible: bool,
}

fn default_visible() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Event {
    Note(NoteEvent),
    Chord(ChordEvent),
    Rest(RestEvent),
    Marker(MarkerEvent),
    Automation(AutomationEvent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteEvent {
    pub base: TimedEventBase,
    pub pitch: Pitch,
    pub velocity: u8,
    pub tie: TieSpec,
    pub articulations: Vec<Articulation>,
    pub ornaments: Vec<Ornament>,
    pub lyric: Option<String>,
    pub pitch_role: Option<PitchRole>,
    pub stem_direction: Option<StemDirection>,
    pub beam_group: Option<u32>,
    #[serde(default)]
    pub is_drum: bool,
    pub drum_map: Option<DrumMapEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TieSpec {
    None,
    Start,
    Stop,
    Continue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PitchRole {
    ChordTone,
    PassingTone,
    NeighborTone,
    Appoggiatura,
    Suspension,
    Retardation,
    EscapeTone,
    PedalTone,
    Ornament,
    Unclassified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StemDirection {
    Up,
    Down,
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Articulation {
    Staccato,
    Staccatissimo,
    Tenuto,
    Accent,
    Marcato,
    Sforzato,
    Legato,
    Spiccato,
    BreathMark,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Ornament {
    Trill { upper: Option<Pitch> },
    Mordent { inverted: bool },
    Turn {
        upper: Option<Pitch>,
        lower: Option<Pitch>,
    },
    GraceNote {
        pitch: Pitch,
        steal_ratio: f32,
    },
    Tremolo { strokes: u8 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrumMapEntry {
    pub gm_note: u8,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChordEvent {
    pub base: TimedEventBase,
    pub pitches: Vec<ChordTone>,
    pub velocity: u8,
    pub articulations: Vec<Articulation>,
    pub symbol: Option<ChordSymbol>,
    pub arpeggiate: Option<ArpeggiateDirection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChordTone {
    pub pitch: Pitch,
    pub role: Option<ChordToneRole>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordToneRole {
    Root,
    Third,
    Fifth,
    Seventh,
    Ninth,
    Eleventh,
    Thirteenth,
    Added,
    Bass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArpeggiateDirection {
    Up,
    Down,
    Alternate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RestEvent {
    pub base: TimedEventBase,
    pub rest_type: RestType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestType {
    Normal,
    Measure,
    MultiMeasure(u16),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkerEvent {
    pub base: TimedEventBase,
    pub marker: MarkerKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MarkerKind {
    SectionBoundary {
        section_id: NodeId,
        label: Option<String>,
    },
    RehearsalMark {
        label: String,
    },
    DynamicMark {
        level: DynamicLevel,
        hairpin: Option<Hairpin>,
    },
    TempoMark {
        text: String,
        bpm_hint: Option<f64>,
    },
    ArticulationRegion {
        articulation: Articulation,
        scope: MarkerScope,
    },
    FiguredBass {
        figures: String,
    },
    Pedal {
        action: PedalAction,
    },
    Fermata {
        shape: FermataShape,
    },
    Caesura,
    Cue,
    TextExpression {
        text: String,
        placement: Placement,
    },
    MotifLabel {
        motif_id: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hairpin {
    Crescendo,
    Decrescendo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PedalAction {
    Down,
    Up,
    Half,
    SostenutoDown,
    SostenutoUp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerScope {
    NextEvent,
    UntilNextMarker,
    Voice,
    Measure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FermataShape {
    Normal,
    Short,
    Long,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Placement {
    Above,
    Below,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutomationEvent {
    pub base: TimedEventBase,
    pub target: AutomationTarget,
    pub value: f32,
    pub curve: AutomationCurve,
    pub end_value: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationTarget {
    Velocity,
    Expression,
    Modulation,
    Volume,
    Pan,
    PitchBend,
    TempoFactor,
    CustomCc(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationCurve {
    Step,
    Linear,
    Exponential,
    Logarithmic,
}

impl Event {
    #[must_use]
    pub fn base(&self) -> &TimedEventBase {
        match self {
            Self::Note(e) => &e.base,
            Self::Chord(e) => &e.base,
            Self::Rest(e) => &e.base,
            Self::Marker(e) => &e.base,
            Self::Automation(e) => &e.base,
        }
    }

    #[must_use]
    pub fn offset(&self) -> BeatOffset {
        self.base().offset
    }

    #[must_use]
    pub fn duration(&self) -> &WrittenDuration {
        &self.base().duration
    }

    #[must_use]
    pub fn provenance(&self) -> &Provenance {
        &self.base().provenance
    }

    #[must_use]
    pub fn id(&self) -> NodeId {
        self.base().id
    }

    #[must_use]
    pub fn is_pitched(&self) -> bool {
        matches!(self, Self::Note(_) | Self::Chord(_))
    }

    #[must_use]
    pub fn is_sounding(&self) -> bool {
        matches!(self, Self::Note(_) | Self::Chord(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{PipelineStageId, Provenance};
    use crate::types::{NoteType, Pitch};

    fn test_base() -> TimedEventBase {
        TimedEventBase {
            id: NodeId::new(1, 0),
            offset: BeatOffset::zero(),
            duration: WrittenDuration {
                note_type: NoteType::Quarter,
                dots: 0,
                tuplet: None,
            },
            provenance: Provenance::generated(PipelineStageId::Melody, "2026-01-01T00:00:00Z"),
            visible: true,
        }
    }

    #[test]
    fn event_serde_uses_kind_tag() {
        let event = Event::Note(NoteEvent {
            base: test_base(),
            pitch: Pitch::from_midi(60),
            velocity: 80,
            tie: TieSpec::None,
            articulations: vec![],
            ornaments: vec![],
            lyric: None,
            pitch_role: None,
            stem_direction: None,
            beam_group: None,
            is_drum: false,
            drum_map: None,
        });
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["kind"], "Note");
    }

    #[test]
    fn every_event_exposes_provenance() {
        let note = Event::Note(NoteEvent {
            base: test_base(),
            pitch: Pitch::from_midi(60),
            velocity: 80,
            tie: TieSpec::None,
            articulations: vec![],
            ornaments: vec![],
            lyric: None,
            pitch_role: None,
            stem_direction: None,
            beam_group: None,
            is_drum: false,
            drum_map: None,
        });
        assert!(matches!(
            note.provenance().source,
            crate::provenance::ProvenanceSource::Generated
        ));
    }

    #[test]
    fn note_is_pitched_and_sounding() {
        let note = Event::Note(NoteEvent {
            base: test_base(),
            pitch: Pitch::from_midi(60),
            velocity: 80,
            tie: TieSpec::None,
            articulations: vec![],
            ornaments: vec![],
            lyric: None,
            pitch_role: None,
            stem_direction: None,
            beam_group: None,
            is_drum: false,
            drum_map: None,
        });
        assert!(note.is_pitched());
        assert!(note.is_sounding());
    }

    #[test]
    fn rest_is_not_pitched() {
        let rest = Event::Rest(RestEvent {
            base: test_base(),
            rest_type: RestType::Normal,
        });
        assert!(!rest.is_pitched());
    }
}
