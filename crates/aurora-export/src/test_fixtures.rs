//! Shared test fixtures for aurora-export.

use aurora_ast::{
    events::{Event, NoteEvent, TieSpec},
    nodes::Clef,
    types::{BeatOffset, NoteType, Pitch, WrittenDuration},
    CompositionBuilder, PipelineStageId, Provenance,
};
use aurora_core::NodeId;

pub fn sample_composition() -> aurora_ast::Composition {
    let mut comp = CompositionBuilder::new()
        .title("Test")
        .one_measure()
        .voices(1)
        .build();

    let note = NoteEvent {
        base: aurora_ast::TimedEventBase {
            id: NodeId::new(100, 0),
            offset: BeatOffset::zero(),
            duration: WrittenDuration {
                note_type: NoteType::Quarter,
                dots: 0,
                tuplet: None,
            },
            provenance: Provenance::generated(PipelineStageId::Melody, "2026-07-04"),
            visible: true,
        },
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
    };

    comp.movements[0].sections[0].phrases[0].measures[0].voices[0].events =
        vec![Event::Note(note)];

    let _ = Clef::Treble;
    comp
}
