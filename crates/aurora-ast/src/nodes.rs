//! AST hierarchy nodes: composition form, voices, measures, and global attributes.
//!
//! See `docs/02-music-model/ast.md`, `voices.md`, `timeline.md`, `score.md`.

use aurora_core::{NodeId, ParameterBundle};
use serde::{Deserialize, Serialize};

use crate::events::Event;
use crate::provenance::{Provenance, ProvenanceRoot};
use crate::types::{BeatOffset, PitchRange};

/// AST schema version stored on [`Composition`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstSchemaVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

pub const AST_SCHEMA_VERSION: AstSchemaVersion = AstSchemaVersion {
    major: 0,
    minor: 1,
    patch: 0,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Composition {
    pub id: NodeId,
    pub schema_version: AstSchemaVersion,
    pub metadata: CompositionMetadata,
    pub global: GlobalAttributes,
    pub voice_registry: VoiceRegistry,
    pub movements: Vec<Movement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompositionMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub copyright: Option<String>,
    pub license: Option<String>,
    pub created_at: String,
    pub modified_at: String,
    pub language: Option<String>,
    pub parameters_used: ParameterBundle,
    pub emotion_profile: Option<EmotionProfile>,
    pub provenance_root: ProvenanceRoot,
    pub tags: Vec<String>,
    pub source: CompositionSource,
    pub layout: ScoreLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositionSource {
    Generated,
    Imported,
    Hybrid,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmotionProfile {
    pub valence: f32,
    pub arousal: f32,
    pub weight_deltas: std::collections::HashMap<String, f32>,
    pub tempo_delta_bpm: f32,
    pub harmonic_color_bias: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreLayout {
    pub staff_spacing: f32,
    pub measure_numbering: MeasureNumberingStyle,
    pub part_list_order: Vec<VoiceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasureNumberingStyle {
    EveryMeasure,
    EverySystem,
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalAttributes {
    pub default_key: KeySignature,
    pub default_meter: TimeSignature,
    pub tempo_map: TempoMap,
    pub key_map: KeyMap,
    pub meter_map: MeterMap,
    pub dynamics_baseline: DynamicLevel,
    pub pickup_measure: Option<PickupSpec>,
    pub display: GlobalDisplayOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PickupSpec {
    pub duration: BeatOffset,
    pub notated_measure_number: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalDisplayOptions {
    pub show_metronome: bool,
    pub show_rehearsal_marks: bool,
    pub page_layout: PageLayout,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageLayout {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub margins_mm: Margins,
    pub system_distance: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TempoMap {
    pub default_bpm: f64,
    pub segments: Vec<TempoSegment>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TempoSegment {
    pub start: TimelinePosition,
    pub bpm: f64,
    pub ramp: Option<TempoRamp>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TempoRamp {
    pub end: TimelinePosition,
    pub end_bpm: f64,
    pub curve: RampCurve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RampCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Exponential,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimelinePosition {
    pub global_measure: u32,
    pub beat: BeatOffset,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeterMap {
    pub default: TimeSignature,
    pub changes: Vec<MeterChange>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeterChange {
    pub at_measure: u32,
    pub meter: TimeSignature,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyMap {
    pub default: KeySignature,
    pub changes: Vec<KeyChange>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyChange {
    pub at: TimelinePosition,
    pub key: KeySignature,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySignature {
    pub tonic: PitchClass,
    pub mode: Mode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchClass {
    pub pc: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Major,
    NaturalMinor,
    HarmonicMinor,
    MelodicMinor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    pub beats: u8,
    pub beat_type: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynamicLevel {
    Pppp,
    Ppp,
    Pp,
    P,
    Mp,
    Mf,
    F,
    Ff,
    Fff,
    Ffff,
    Sfz,
    Fp,
    Rf,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChordSymbol {
    pub root: PitchClass,
    pub quality: ChordQuality,
    pub extensions: Vec<Extension>,
    pub bass: Option<PitchClass>,
    pub raw: String,
}

impl ChordSymbol {
    /// Triad pitch classes (root, third, fifth) for rule evaluation and candidate generation.
    #[must_use]
    pub fn pitch_classes(&self) -> [u8; 3] {
        let root = self.root.pc;
        let (third, fifth) = match self.quality {
            ChordQuality::Sus2 => (2, 7),
            ChordQuality::Sus4 => (5, 7),
            ChordQuality::Minor | ChordQuality::Minor7 | ChordQuality::HalfDiminished7 => (3, 7),
            ChordQuality::Diminished | ChordQuality::Diminished7 => (3, 6),
            ChordQuality::Augmented => (4, 8),
            _ => (4, 7),
        };
        [
            root,
            (root + third) % 12,
            (root + fifth) % 12,
        ]
    }

    /// All pitch classes for voicing (triad + 7th when quality implies it).
    #[must_use]
    pub fn voicing_pcs(&self) -> Vec<u8> {
        let mut pcs: Vec<u8> = self.pitch_classes().into_iter().collect();
        if let Some(seventh) = self.seventh_pc() {
            if !pcs.contains(&seventh) {
                pcs.push(seventh);
            }
        }
        if let Some(bass) = self.bass {
            if !pcs.contains(&bass.pc) {
                pcs.insert(0, bass.pc);
            }
        }
        pcs.sort_unstable();
        pcs.dedup();
        pcs
    }

    fn seventh_pc(&self) -> Option<u8> {
        let root = self.root.pc;
        Some(match self.quality {
            ChordQuality::Dominant7 | ChordQuality::Minor7 | ChordQuality::HalfDiminished7 => {
                (root + 10) % 12
            }
            ChordQuality::Major7 => (root + 11) % 12,
            ChordQuality::Diminished7 => (root + 9) % 12,
            _ => return None,
        })
    }

    #[must_use]
    pub fn simple(root_pc: u8, quality: ChordQuality, raw: impl Into<String>) -> Self {
        Self {
            root: PitchClass { pc: root_pc % 12 },
            quality,
            extensions: Vec::new(),
            bass: None,
            raw: raw.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    Dominant7,
    Major7,
    Minor7,
    HalfDiminished7,
    Diminished7,
    Sus2,
    Sus4,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Extension {
    Sixth,
    Seventh,
    Ninth,
    Eleventh,
    Thirteenth,
    Add9,
    Add11,
    Flat9,
    Sharp11,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Movement {
    pub id: NodeId,
    pub metadata: MovementMetadata,
    pub sections: Vec<Section>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MovementMetadata {
    pub title: Option<String>,
    pub ordinal: u16,
    pub key_override: Option<KeySignature>,
    pub tempo_override: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub id: NodeId,
    pub metadata: SectionMetadata,
    pub markers: Vec<SectionMarker>,
    pub phrases: Vec<Phrase>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionMetadata {
    pub role: SectionRole,
    pub label: Option<String>,
    pub theme_refs: Vec<ThemeRef>,
    pub key_area: Option<KeySignature>,
    pub repeat: Option<RepeatSpec>,
    pub energy_level: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionRole {
    Intro,
    Verse,
    PreChorus,
    Chorus,
    Bridge,
    Breakdown,
    Build,
    Drop,
    Outro,
    Coda,
    Exposition,
    Development,
    Recapitulation,
    Transition,
    Interlude,
    Custom(u32),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeRef {
    pub theme_id: String,
    pub transformation: ThemeTransform,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeTransform {
    Original,
    Sequence,
    Inversion,
    Retrograde,
    Augmentation,
    Diminution,
    Fragmentation,
    ModalInterchange,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RepeatSpec {
    pub count: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionMarker {
    pub id: NodeId,
    pub offset: BeatOffset,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Phrase {
    pub id: NodeId,
    pub metadata: PhraseMetadata,
    pub measures: Vec<Measure>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhraseMetadata {
    pub phrase_id: String,
    pub cadence: Option<CadenceType>,
    pub motif_ref: Option<String>,
    pub contour_hint: Option<ContourHint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CadenceType {
    PerfectAuthentic,
    ImperfectAuthentic,
    Half,
    Plagal,
    Deceptive,
    Phrygian,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContourHint {
    Ascending,
    Descending,
    Arch,
    Wave,
    Static,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Measure {
    pub id: NodeId,
    pub number: MeasureNumber,
    pub attributes: MeasureAttributes,
    pub harmony_slots: Vec<HarmonySlot>,
    pub voices: Vec<MeasureVoice>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureNumber {
    pub local: u16,
    pub global: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasureAttributes {
    pub meter: Option<TimeSignature>,
    pub key: Option<KeySignature>,
    pub repeat_start: bool,
    pub repeat_end: bool,
    pub repeat_count: Option<u8>,
    pub volta: Option<VoltaSpec>,
    pub rehearsal_mark: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoltaSpec {
    pub ending_number: u8,
    pub repeat_count: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HarmonySlot {
    pub id: NodeId,
    pub offset: BeatOffset,
    pub duration: BeatOffset,
    pub symbol: ChordSymbol,
    pub roman_numeral: Option<String>,
    pub function: Option<HarmonicFunction>,
    pub provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarmonicFunction {
    Tonic,
    Subdominant,
    Dominant,
    Predominant,
    Passing,
    Neighbor,
    Cadential,
}

/// Global voice definition (`VoiceDef` in spec). Re-exported as `Voice` for API ergonomics.
pub type Voice = VoiceDef;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceRegistry {
    pub voices: Vec<VoiceDef>,
    pub groups: Vec<VoiceGroup>,
    pub default_layout: VoiceLayoutId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceGroupId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceLayoutId(pub u16);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceDef {
    pub id: VoiceId,
    pub name: String,
    pub role: VoiceRole,
    pub register: PitchRange,
    pub midi_channel: u8,
    pub group: Option<VoiceGroupId>,
    pub instrument: InstrumentSpec,
    pub export: VoiceExportSpec,
    pub priority: u8,
    pub mutable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceRole {
    Melody,
    Alto,
    Tenor,
    Bass,
    Inner,
    HarmonyPad,
    Drums,
    Percussion,
    Lead,
    Accompaniment,
    BassLine,
    Guitar,
    Piano,
    Strings,
    Brass,
    Woodwinds,
    Custom(u32),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstrumentSpec {
    pub gm_program: u8,
    pub name: String,
    pub transposition: i8,
    pub clef: Clef,
    pub staff_lines: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clef {
    Treble,
    Bass,
    Alto,
    Tenor,
    Percussion,
    Tab,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceExportSpec {
    pub musicxml_part_id: String,
    pub staff_index: u8,
    pub abbrev: Option<String>,
    pub hide_if_empty: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceGroup {
    pub id: VoiceGroupId,
    pub name: String,
    pub kind: VoiceGroupKind,
    pub member_voices: Vec<VoiceId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceGroupKind {
    RhythmSection,
    HarmonicBed,
    LeadSection,
    StringSection,
    BrassSection,
    Custom,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeasureVoice {
    pub voice_id: VoiceId,
    pub events: Vec<Event>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{PipelineStageId, Provenance};

    #[test]
    fn ast_schema_version_is_zero_one_zero() {
        assert_eq!(AST_SCHEMA_VERSION.major, 0);
        assert_eq!(AST_SCHEMA_VERSION.minor, 1);
    }

    #[test]
    fn harmony_slot_has_provenance() {
        let slot = HarmonySlot {
            id: NodeId::new(1, 0),
            offset: BeatOffset::zero(),
            duration: BeatOffset::new(1, 1),
            symbol: ChordSymbol {
                root: PitchClass { pc: 0 },
                quality: ChordQuality::Major,
                extensions: vec![],
                bass: None,
                raw: "C".into(),
            },
            roman_numeral: Some("I".into()),
            function: Some(HarmonicFunction::Tonic),
            provenance: Provenance::generated(PipelineStageId::HarmonySkeleton, "2026-01-01"),
        };
        assert!(slot.provenance.stage.is_some());
    }

    #[test]
    fn voice_type_alias_matches_voice_def() {
        let voice: Voice = VoiceDef {
            id: VoiceId(0),
            name: "Melody".into(),
            role: VoiceRole::Melody,
            register: PitchRange {
                min_midi: 60,
                max_midi: 84,
                preferred_min: 62,
                preferred_max: 80,
            },
            midi_channel: 1,
            group: None,
            instrument: InstrumentSpec {
                gm_program: 0,
                name: "Piano".into(),
                transposition: 0,
                clef: Clef::Treble,
                staff_lines: 5,
            },
            export: VoiceExportSpec {
                musicxml_part_id: "P1".into(),
                staff_index: 0,
                abbrev: None,
                hide_if_empty: false,
            },
            priority: 0,
            mutable: true,
        };
        assert_eq!(voice.role, VoiceRole::Melody);
    }
}
