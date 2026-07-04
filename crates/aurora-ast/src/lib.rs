//! Aurora Composer Music AST — hierarchical score model.
//!
//! See `docs/02-music-model/ast.md`.

pub mod builder;
pub mod events;
pub mod nodes;
pub mod patch;
pub mod project;
pub mod provenance;
pub mod snapshot;
pub mod types;

pub use builder::CompositionBuilder;
pub use events::{
    Articulation, AutomationEvent, ChordEvent, Event, MarkerEvent, NoteEvent, Ornament, PitchRole,
    RestEvent, TimedEventBase, TieSpec,
};
pub use nodes::{
    AST_SCHEMA_VERSION, CadenceType, ChordQuality, ChordSymbol, Clef, Composition,
    CompositionMetadata, CompositionSource, DynamicLevel, EmotionProfile, GlobalAttributes,
    GlobalDisplayOptions, HarmonicFunction, HarmonySlot, InstrumentSpec, KeyChange, KeyMap,
    KeySignature, Margins, Measure, MeasureAttributes, MeasureNumber, MeasureNumberingStyle,
    MeasureVoice, MeterMap, Mode, Movement, MovementMetadata, PageLayout, Phrase, PhraseMetadata,
    PitchClass, ScoreLayout, Section, SectionMetadata, SectionRole, TempoMap, TempoSegment,
    TimeSignature, TimelinePosition, Voice, VoiceDef, VoiceExportSpec, VoiceId, VoiceLayoutId,
    VoiceRegistry, VoiceRole,
};
pub use provenance::{
    PipelineStageId, Provenance, ProvenanceAgent, ProvenanceRoot, ProvenanceSource, RuleRef,
    SearchContext, StateRef,
};
pub use project::{ParameterSnapshot, Project};
pub use types::{BeatOffset, NoteType, Pitch, PitchRange, Step, WrittenDuration};
