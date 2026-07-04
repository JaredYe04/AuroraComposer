//! Block-chord accompaniment voice (piano / strings) with inversions and rhythmic comping.

use aurora_ast::{ChordQuality, ChordSymbol, Event, NoteType, PipelineStageId, VoiceId, WrittenDuration};

use super::common::{
    accompaniment_enabled, accompaniment_voice_id, collect_melody_per_beat,
    collect_per_beat_chord_grid, iter_measures_mut, make_note_provenance, push_note,
};
use super::rhythm_patterns::{beat_offset_from_frac, comp_hits_for_beat, CompHit};
use super::voicing::{arpeggio_tone, inversion_index, sus_color, voice_chord};
use super::PipelineState;

const SUBDIV: u32 = 4;

/// Generate chord accompaniment when [`accompaniment_enabled`].
pub fn generate_chord_voice(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    if !accompaniment_enabled(state) {
        return Ok(());
    }

    let voice_id = accompaniment_voice_id();
    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let bar_count = super::total_bars(&state.params);
    let total_steps = bar_count as usize * beats_per_measure;

    let grid = collect_per_beat_chord_grid(state, total_steps);
    let melody_per_beat = collect_melody_per_beat(state, total_steps);
    let register = (
        state.params.accompaniment.register_min,
        state.params.accompaniment.register_max,
    );
    let density = state.params.accompaniment.voicing_density.clamp(0.0, 1.0);
    let syncopation = state.params.rhythm.syncopation.clamp(0.0, 1.0);

    let segments = chord_segments(&grid);
    let mut note_id = 30_000u64;
    let mut prev_voicing: Vec<u8> = Vec::new();
    let mut prev_bass_pc: Option<u8> = None;
    let mut global_step = 0usize;
    let mut seg_idx = 0usize;

    for (measure_idx, measure) in iter_measures_mut(&mut state.composition).enumerate() {
        for beat in 0..beats_per_measure {
            if global_step >= total_steps {
                break;
            }
            while seg_idx + 1 < segments.len() && global_step >= segments[seg_idx + 1].start {
                seg_idx += 1;
            }
            let segment = &segments[seg_idx.min(segments.len().saturating_sub(1))];
            let beat_in_seg = global_step - segment.start;

            let mut hits = comp_hits_for_beat(beat, measure_idx, syncopation, density);
            if beat_in_seg == 0 || global_step == segment.start {
                if hits.is_empty() {
                    hits.push(CompHit {
                        offset_frac: 0.0,
                        note_type: NoteType::Quarter,
                        velocity_scale: 1.0,
                    });
                }
            } else if hits.is_empty() {
                global_step += 1;
                continue;
            }

            let melody_midi = melody_per_beat.get(global_step).and_then(|m| *m);
            let mut chord = segment.chord.clone();
            if syncopation > 0.25 {
                chord.quality = sus_color(&chord, global_step);
            }

            let inv = inversion_index(global_step, &chord, prev_bass_pc);
            let strong_beat = beat == 0 || beat == 2;

            for (hit_idx, hit) in hits.iter().enumerate() {
                let full_block = hit_idx == 0
                    && (strong_beat
                        || beat_in_seg == 0
                        || matches!(
                            hit.note_type,
                            NoteType::Quarter | NoteType::Half | NoteType::Whole
                        ));

                if full_block {
                    let voicing = voice_chord(
                        &chord,
                        inv,
                        melody_midi,
                        &prev_voicing,
                        register,
                        density,
                    );

                    let note_type = if beat_in_seg == 0
                        && segment.end - segment.start >= beats_per_measure
                    {
                        NoteType::Half
                    } else {
                        hit.note_type
                    };

                    for (idx, midi) in voicing.iter().copied().enumerate() {
                        let prov = make_note_provenance(
                            PipelineStageId::Counterpoint,
                            created_at,
                            vec!["HARM-VOICE-001".into(), "ACCOMP-001".into()],
                            format!("chord_voice step {global_step} note {idx}"),
                        );
                        let mut note = super::common::quarter_note(
                            note_id,
                            beat,
                            midi,
                            ((66.0 + idx as f32 * 3.0) * hit.velocity_scale) as u8,
                            prov,
                            false,
                        );
                        note.base.duration.note_type = note_type;
                        note.base.offset = beat_offset_from_frac(beat as u32, hit.offset_frac, SUBDIV);
                        push_note(measure, voice_id, Event::Note(note));
                        note_id += 1;
                    }
                    prev_bass_pc = voicing.first().map(|m| m % 12);
                    prev_voicing = voicing;
                } else if density >= 0.35 {
                    let midi = arpeggio_tone(&chord, global_step + inv + hit_idx, register);
                    let prov = make_note_provenance(
                        PipelineStageId::Counterpoint,
                        created_at,
                        vec!["HARM-VOICE-002-ARP".into()],
                        format!("chord_arp step {global_step}"),
                    );
                    let mut note = super::common::quarter_note(
                        note_id,
                        beat,
                        midi,
                        (58.0 * hit.velocity_scale) as u8,
                        prov,
                        false,
                    );
                    note.base.duration = WrittenDuration {
                        note_type: hit.note_type,
                        dots: 0,
                        tuplet: None,
                    };
                    note.base.offset = beat_offset_from_frac(beat as u32, hit.offset_frac, SUBDIV);
                    push_note(measure, voice_id, Event::Note(note));
                    note_id += 1;
                }
            }

            global_step += 1;
        }
    }

    let _ = VoiceId(0);
    Ok(())
}

struct ChordSegment {
    start: usize,
    end: usize,
    chord: ChordSymbol,
}

fn chord_segments(grid: &[ChordSymbol]) -> Vec<ChordSegment> {
    if grid.is_empty() {
        return vec![];
    }
    let mut segments = Vec::new();
    let mut start = 0usize;
    for i in 1..=grid.len() {
        let ends = i == grid.len()
            || grid[i].root.pc != grid[start].root.pc
            || grid[i].quality != grid[start].quality;
        if ends {
            segments.push(ChordSegment {
                start,
                end: i,
                chord: grid[start].clone(),
            });
            start = i;
        }
    }
    segments
}

#[allow(dead_code)]
fn with_quality(chord: &ChordSymbol, q: ChordQuality) -> ChordSymbol {
    let mut c = chord.clone();
    c.quality = q;
    c
}
