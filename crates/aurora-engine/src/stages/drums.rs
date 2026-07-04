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
const GM_HIHAT_OPEN: u8 = 46;
const TICKS_PER_BEAT: f32 = 4.0;

/// Metric strength on 16th grid within one 4/4 bar.
const METRIC_STRENGTH_16: [f32; 16] = [
    1.00, 0.35, 0.55, 0.40, 0.70, 0.30, 0.50, 0.35, 0.85, 0.35, 0.55, 0.40, 0.65, 0.30, 0.50,
    0.35,
];

struct HihatPattern {
    mask: u16,
    open_mask: u16,
    velocity_curve: [u8; 16],
}

const HH_QUARTERS: HihatPattern = HihatPattern {
    mask: 0x1111,
    open_mask: 0,
    velocity_curve: [100, 0, 0, 0, 90, 0, 0, 0, 95, 0, 0, 0, 85, 0, 0, 0],
};

const HH_EIGHTHS: HihatPattern = HihatPattern {
    mask: 0x5555,
    open_mask: 0,
    velocity_curve: [110, 0, 85, 0, 95, 0, 80, 0, 105, 0, 85, 0, 90, 0, 75, 0],
};

const HH_SIXTEENTHS: HihatPattern = HihatPattern {
    mask: 0xFFFF,
    open_mask: 0,
    velocity_curve: [
        100, 55, 75, 50, 90, 50, 70, 45, 95, 55, 75, 50, 85, 50, 65, 45,
    ],
};

const HH_OFFBEAT_OPEN: HihatPattern = HihatPattern {
    mask: 0x5555,
    open_mask: 0x4040,
    velocity_curve: [100, 0, 80, 0, 90, 0, 70, 0, 95, 0, 80, 0, 85, 0, 65, 0],
};

/// Stage 10 — Drums: metric groove with accent emphasis and hi-hat density.
pub fn generate_drums(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let voice_id = drums_voice_id(state);
    let beats = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let total_measures = super::total_bars(&state.params) as usize;
    let density = state.params.drums.density;
    let fill_freq = state.params.drums.fill_frequency;
    let accent_emphasis = state
        .params
        .drums
        .accent_emphasis
        .clamp(0.0, 1.0);
    let hihat_density = state.params.drums.hihat_density.clamp(0.0, 1.0);
    let syncopation = state.params.rhythm.syncopation;
    let seed = state.params.search.seed.unwrap_or(42);

    let hihat_pattern = select_hihat_pattern(hihat_density, syncopation);

    for (mi, measure) in iter_measures_mut(&mut state.composition).enumerate() {
        if mi >= total_measures {
            break;
        }

        let is_final_measure = mi + 1 >= total_measures;
        let is_outro = mi + 1 >= total_measures.saturating_sub(1);

        let rhythm_accents: [f32; 4] = {
            let row = state.rhythm_accents.get(mi).cloned().unwrap_or_default();
            [
                row.first().copied().unwrap_or(1.0),
                row.get(1).copied().unwrap_or(0.4),
                row.get(2).copied().unwrap_or(0.7),
                row.get(3).copied().unwrap_or(0.4),
            ]
        };

        let is_phrase_end = measure.number.local == 4;
        let fill = is_phrase_end
            && fill_freq > 0.05
            && (mi + 1) % 4 == 0
            && !is_outro;

        // Backbone kick: beats 1 & 3 (ticks 0, 8)
        for tick in backbone_kick_ticks(accent_emphasis, density) {
            let beat = usize::from(tick / 4);
            if beat >= beats {
                continue;
            }
            let vel = drum_velocity(tick, 100, accent_emphasis, rhythm_accents[beat]);
            push_drum(
                measure,
                voice_id,
                mi,
                tick,
                GM_KICK,
                vel,
                created_at,
                "DRUM-KICK-001",
                "Kick",
            );
        }

        // Backbone snare: beats 2 & 4 (ticks 4, 12)
        for tick in [4u8, 12] {
            let beat = usize::from(tick / 4);
            if beat >= beats {
                continue;
            }
            let vel = drum_velocity(tick, 90, accent_emphasis, rhythm_accents[beat]);
            push_drum(
                measure,
                voice_id,
                mi,
                tick,
                GM_SNARE,
                vel,
                created_at,
                "DRUM-SNARE-001",
                "Snare",
            );
        }

        // Hi-hats from template (taper in outro / omit on final bar tail)
        let mut hat_mask = hihat_pattern.mask;
        if syncopation > 0.55 && !is_outro {
            hat_mask = apply_syncopation_to_mask(hat_mask);
        }
        if is_outro {
            hat_mask &= 0x1111;
        }
        if is_final_measure {
            hat_mask &= 0x0101;
        }
        for tick in 0..16u8 {
            if tick / 4 >= beats as u8 {
                break;
            }
            if (hat_mask >> tick) & 1 == 0 {
                continue;
            }
            let beat = (tick / 4) as usize;
            let note = if (hihat_pattern.open_mask >> tick) & 1 == 1 {
                GM_HIHAT_OPEN
            } else {
                GM_HIHAT_CLOSED
            };
            let base = hihat_pattern.velocity_curve[tick as usize] as f32 / 100.0;
            let metric = METRIC_STRENGTH_16[tick as usize];
            let blend = base * (metric * accent_emphasis + (1.0 - accent_emphasis) * 0.65);
            let vel = (45.0 + 75.0 * blend * rhythm_accents[beat] * (0.45 + 0.55 * hihat_density))
                .clamp(40.0, 110.0) as u8;
            push_drum(
                measure,
                voice_id,
                mi,
                tick,
                note,
                vel,
                created_at,
                "DRUM-HH-001",
                if note == GM_HIHAT_OPEN {
                    "Hi-Hat Open"
                } else {
                    "Hi-Hat"
                },
            );
        }

        // Optional embellishments (ghost snares / syncopated kicks)
        let budget = if is_outro {
            0
        } else {
            embellishment_budget(density, accent_emphasis)
        };
        let mut placed = 0u8;
        for tick in [2u8, 6, 10, 14, 3, 7, 11, 15] {
            if placed >= budget || tick / 4 >= beats as u8 {
                break;
            }
            let rng = ((seed.wrapping_mul(tick as u64 + 1).wrapping_add(mi as u64)) % 1000) as f32
                / 1000.0;
            if !should_place_optional_hit(tick, accent_emphasis, density, rng) {
                continue;
            }
            let beat = (tick / 4) as usize;
            let vel = drum_velocity(tick, 45, accent_emphasis, rhythm_accents[beat]);
            push_drum(
                measure,
                voice_id,
                mi,
                tick,
                GM_SNARE,
                vel,
                created_at,
                "DRUM-GHOST-001",
                "Ghost Snare",
            );
            placed += 1;
        }

        if fill {
            for tick in 8..16u8 {
                if tick / 4 >= beats as u8 {
                    break;
                }
                push_drum(
                    measure,
                    voice_id,
                    mi,
                    tick,
                    GM_SNARE,
                    105,
                    created_at,
                    "DRUM-FILL-001",
                    "Fill Snare",
                );
            }
        }
    }

    Ok(())
}

fn select_hihat_pattern(hihat_density: f32, syncopation: f32) -> &'static HihatPattern {
    if hihat_density < 0.25 {
        &HH_QUARTERS
    } else if hihat_density < 0.55 {
        if syncopation > 0.5 {
            &HH_EIGHTHS
        } else {
            &HH_EIGHTHS
        }
    } else if hihat_density < 0.85 {
        &HH_SIXTEENTHS
    } else {
        &HH_OFFBEAT_OPEN
    }
}

fn backbone_kick_ticks(accent_emphasis: f32, density: f32) -> Vec<u8> {
    let mut ticks = vec![0u8, 8];
    if density > 0.55 && accent_emphasis < 0.6 {
        if density > 0.7 {
            ticks.push(6);
        }
        if density > 0.85 {
            ticks.push(14);
        }
    }
    ticks.sort_unstable();
    ticks.dedup();
    ticks
}

fn drum_velocity(tick: u8, instrument_base: u8, accent_emphasis: f32, rhythm_accent: f32) -> u8 {
    let metric = METRIC_STRENGTH_16[(tick % 16) as usize];
    let blend = metric * accent_emphasis + 0.5 * (1.0 - accent_emphasis);
    let v = instrument_base as f32 * (0.55 + 0.45 * blend + rhythm_accent * 0.2);
    v.clamp(1.0, 127.0) as u8
}

fn embellishment_budget(density: f32, accent_emphasis: f32) -> u8 {
    let raw = (density * 8.0) as u8;
    if accent_emphasis > 0.8 {
        raw.min(2)
    } else {
        raw.min(5)
    }
}

fn should_place_optional_hit(
    tick: u8,
    accent_emphasis: f32,
    density: f32,
    rng: f32,
) -> bool {
    if accent_emphasis > 0.7 && tick % 2 == 1 {
        return false;
    }
    let metric = METRIC_STRENGTH_16[(tick % 16) as usize];
    let weight = metric.powf(1.0 + accent_emphasis as f32 * 2.5);
    let threshold = 1.0 - density;
    rng > threshold / weight.max(0.05)
}

fn apply_syncopation_to_mask(mask: u16) -> u16 {
    let mut out = mask;
    for beat_tick in [0u8, 4, 8, 12] {
        out &= !(1 << beat_tick);
        out |= 1 << (beat_tick + 2);
    }
    out
}

fn push_drum(
    measure: &mut aurora_ast::Measure,
    voice_id: aurora_ast::VoiceId,
    mi: usize,
    tick: u8,
    gm_note: u8,
    velocity: u8,
    created_at: &str,
    rule_id: &str,
    name: &str,
) {
    let beat = f32::from(tick) / TICKS_PER_BEAT;
    let beat_idx = tick as usize / 4;
    let prov = make_note_provenance(
        PipelineStageId::Drums,
        created_at,
        vec![rule_id.into()],
        format!("{name} tick {tick} measure {}", measure.number.global),
    );
    let mut note = quarter_note(
        u64::try_from(40_000 + mi * 16 + usize::from(tick)).unwrap_or(40_000),
        beat_idx,
        gm_note,
        velocity,
        prov,
        true,
    );
    note.base.offset = aurora_ast::BeatOffset::new((beat * 4.0) as u32, 4);
    note.base.duration = WrittenDuration {
        note_type: NoteType::ThirtySecond,
        dots: 0,
        tuplet: None,
    };
    note.drum_map = Some(DrumMapEntry {
        gm_note,
        name: name.into(),
    });
    note.base.provenance.source = ProvenanceSource::Generated;
    note.base.provenance.agent = ProvenanceAgent::Engine {
        stage: PipelineStageId::Drums,
    };
    push_note(measure, voice_id, Event::Note(note));
}
