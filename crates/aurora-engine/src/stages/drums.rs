use aurora_ast::{
    events::DrumMapEntry, Event, NoteType, PipelineStageId, ProvenanceAgent, ProvenanceSource,
    WrittenDuration,
};

use super::common::{
    drums_voice_id, iter_measures_mut, make_note_provenance, push_note, quarter_note,
};
use super::PipelineState;

const GM_KICK: u8 = 36;
const GM_SNARE: u8 = 38;
const GM_HIHAT_CLOSED: u8 = 42;

/// Stage 10 — Drums: groove pattern on MIDI channel 10.
pub fn generate_drums(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let voice_id = drums_voice_id(state);
    let beats = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let density = state.params.drums.density;
    let fill_freq = state.params.drums.fill_frequency;
    let bar_count = super::total_bars(&state.params) as usize;

    for (mi, measure) in iter_measures_mut(&mut state.composition).enumerate() {
        let is_phrase_end = measure.number.local == 4;
        let fill = is_phrase_end && fill_freq > 0.05 && (mi + 1) % 4 == 0;

        for beat in 0..beats {
            let accent = state
                .rhythm_accents
                .get(mi)
                .and_then(|a| a.get(beat))
                .copied()
                .unwrap_or(0.5);

            if beat == 0 || (density > 0.4 && beat == 2) {
                let prov = make_note_provenance(
                    PipelineStageId::Drums,
                    created_at,
                    vec!["DRUM-KICK-001".into()],
                    format!("kick beat {} measure {}", beat + 1, measure.number.global),
                );
                push_note(
                    measure,
                    voice_id,
                    Event::Note(drum_hit(
                        u64::try_from(40_000 + mi * 8 + beat).unwrap_or(40_000),
                        beat,
                        GM_KICK,
                        100,
                        prov,
                        "Kick",
                    )),
                );
            }

            if beat == 1 || beat == 3 {
                let prov = make_note_provenance(
                    PipelineStageId::Drums,
                    created_at,
                    vec!["DRUM-SNARE-001".into()],
                    format!("snare beat {} measure {}", beat + 1, measure.number.global),
                );
                push_note(
                    measure,
                    voice_id,
                    Event::Note(drum_hit(
                        u64::try_from(50_000 + mi * 8 + beat).unwrap_or(50_000),
                        beat,
                        GM_SNARE,
                        90,
                        prov,
                        "Snare",
                    )),
                );
            }

            if density > 0.3 && accent > 0.35 {
                let prov = make_note_provenance(
                    PipelineStageId::Drums,
                    created_at,
                    vec!["DRUM-HH-001".into()],
                    format!("hi-hat beat {} measure {}", beat + 1, measure.number.global),
                );
                push_note(
                    measure,
                    voice_id,
                    Event::Note(drum_hit(
                        u64::try_from(60_000 + mi * 8 + beat).unwrap_or(60_000),
                        beat,
                        GM_HIHAT_CLOSED,
                        60,
                        prov,
                        "Hi-Hat",
                    )),
                );
            }

            if fill && beat >= beats - 1 {
                let prov = make_note_provenance(
                    PipelineStageId::Drums,
                    created_at,
                    vec!["DRUM-FILL-001".into()],
                    format!("fill measure {}", measure.number.global),
                );
                push_note(
                    measure,
                    voice_id,
                    Event::Note(drum_hit(
                        u64::try_from(70_000 + mi * 8 + beat).unwrap_or(70_000),
                        beat,
                        GM_SNARE,
                        110,
                        prov,
                        "Fill Snare",
                    )),
                );
            }
        }
    }

    let _ = bar_count;
    Ok(())
}

fn drum_hit(
    id: u64,
    beat: usize,
    gm_note: u8,
    velocity: u8,
    mut provenance: aurora_ast::Provenance,
    name: &str,
) -> aurora_ast::NoteEvent {
    provenance.source = ProvenanceSource::Generated;
    provenance.agent = ProvenanceAgent::Engine {
        stage: PipelineStageId::Drums,
    };
    let mut note = quarter_note(id, beat, gm_note, velocity, provenance, true);
    note.drum_map = Some(DrumMapEntry {
        gm_note,
        name: name.into(),
    });
    note.base.duration = WrittenDuration {
        note_type: NoteType::Eighth,
        dots: 0,
        tuplet: None,
    };
    note
}
