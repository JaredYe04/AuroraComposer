//! P6 — HarmonyPad block voicing for homophonic texture.

use aurora_ast::{Event, PipelineStageId, VoiceId};

use super::common::{
    harmony_pad_enabled, harmony_pad_voice_id, iter_measures_mut, make_note_provenance, push_note,
    quarter_note,
};
use super::rhythm_patterns::comp_hits_for_beat;
use super::PipelineState;

/// Generate HarmonyPad inner voices (alto + tenor block) when homophonic.
pub fn generate_harmony_pad(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    if !harmony_pad_enabled(state) {
        return Ok(());
    }

    let pad_id = harmony_pad_voice_id();
    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let grid = &state.per_beat_chord_grid;
    let melody_pitches = super::common::collect_melody_pitches(state);

    let pad_register = (48u8, 72u8);
    let density = state.params.texture.homophony_polyphony_balance.clamp(0.0, 1.0);
    let syncopation = state.params.rhythm.syncopation.clamp(0.0, 1.0);
    let mut global_step = 0usize;
    let mut prev_alto: Option<u8> = None;
    let mut prev_tenor: Option<u8> = None;
    let mut note_id = 20_000u64;

    for (measure_idx, measure) in iter_measures_mut(&mut state.composition).enumerate() {
        for beat in 0..beats_per_measure {
            let hits = comp_hits_for_beat(beat, measure_idx, syncopation, density);
            if hits.is_empty() && beat != 0 {
                global_step += 1;
                continue;
            }

            let melody_midi = melody_pitches.get(global_step).copied();
            let chord = grid.get(global_step).cloned().unwrap_or_else(|| {
                aurora_ast::ChordSymbol::simple(
                    state.params.mode.key % 12,
                    aurora_ast::ChordQuality::Major,
                    "C",
                )
            });

            let bass_est = chord.root.pc + 36; // estimated bass octave
            let (alto, tenor) = voice_pad_block(
                &chord,
                melody_midi,
                bass_est,
                prev_alto,
                prev_tenor,
                pad_register,
                state.params.texture.homophony_polyphony_balance > 0.92,
            );

            for (midi, label) in [(tenor, "pad_tenor"), (alto, "pad_alto")] {
                let prov = make_note_provenance(
                    PipelineStageId::Counterpoint,
                    created_at,
                    vec!["HARM-VOICE-001".into(), "VL-CT-001".into()],
                    format!("{label} step {global_step}"),
                );
                push_note(
                    measure,
                    pad_id,
                    Event::Note(quarter_note(note_id, beat, midi, 70, prov, false)),
                );
                note_id += 1;
            }

            prev_alto = Some(alto);
            prev_tenor = Some(tenor);
            global_step += 1;
        }
    }

    let _ = VoiceId(0);
    Ok(())
}

fn voice_pad_block(
    chord: &aurora_ast::ChordSymbol,
    melody_midi: Option<u8>,
    bass_est: u8,
    prev_alto: Option<u8>,
    prev_tenor: Option<u8>,
    register: (u8, u8),
    close: bool,
) -> (u8, u8) {
    let pcs: Vec<u8> = chord.pitch_classes().to_vec();
    let melody_pc = melody_midi.map(|m| m % 12);

    let mut best = (
        prev_alto.unwrap_or(60),
        prev_tenor.unwrap_or(55),
    );
    let mut best_score = f32::NEG_INFINITY;

    for &a_pc in &pcs {
        for &t_pc in &pcs {
            for a_oct in 3..=5 {
                for t_oct in 3..=5 {
                    let alto = a_oct * 12 + a_pc;
                    let tenor = t_oct * 12 + t_pc;
                    if alto < register.0 || alto > register.1 {
                        continue;
                    }
                    if tenor < register.0 || tenor > register.1 {
                        continue;
                    }
                    if tenor >= alto {
                        continue;
                    }
                    if tenor <= bass_est {
                        continue;
                    }
                    if let Some(m) = melody_midi {
                        if alto >= m {
                            continue;
                        }
                    }
                    if let Some(mpc) = melody_pc {
                        if a_pc == mpc && t_pc == mpc {
                            continue;
                        }
                    }

                    let mut score = 0.0f32;
                    score += 5.0; // chord tone base
                    if let (Some(pa), Some(pt)) = (prev_alto, prev_tenor) {
                        if pa % 12 == a_pc {
                            score += 0.8;
                        }
                        if pt % 12 == t_pc {
                            score += 0.8;
                        }
                        let motion_a = (alto as i16 - pa as i16).unsigned_abs();
                        let motion_t = (tenor as i16 - pt as i16).unsigned_abs();
                        score -= (motion_a + motion_t) as f32 * 0.3;
                    }
                    if close {
                        if melody_midi.map(|m| m - alto).unwrap_or(12) <= 12 {
                            score += 2.0;
                        }
                    }
                    if score > best_score {
                        best_score = score;
                        best = (alto, tenor);
                    }
                }
            }
        }
    }
    best
}
