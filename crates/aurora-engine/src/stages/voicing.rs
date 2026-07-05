//! Shared chord voicing — inversions, register placement, voice-leading.

use aurora_ast::{ChordQuality, ChordSymbol};

/// Pick inversion index 0=root, 1=first, 2=second (seeded by step for variety).
pub fn inversion_index(step: usize, chord: &ChordSymbol, prev_bass_pc: Option<u8>) -> usize {
    let pcs = chord.voicing_pcs();
    if pcs.len() < 2 {
        return 0;
    }
    if step % 4 == 0 {
        return 0;
    }
    if let Some(prev_bass) = prev_bass_pc {
        let mut best = 0usize;
        let mut best_dist = u8::MAX;
        for (i, &pc) in pcs.iter().enumerate().take(3) {
            let up = (pc + 12 - prev_bass) % 12;
            let down = (prev_bass + 12 - pc) % 12;
            let dist = up.min(down);
            if dist < best_dist {
                best_dist = dist;
                best = i;
            }
        }
        return best % pcs.len();
    }
    0
}

/// Build MIDI voicing with optional inversion and slash bass.
pub fn voice_chord(
    chord: &ChordSymbol,
    inversion: usize,
    melody_midi: Option<u8>,
    prev: &[u8],
    register: (u8, u8),
    density: f32,
) -> Vec<u8> {
    let mut pcs = chord.voicing_pcs();
    if pcs.is_empty() {
        return vec![60];
    }

    let note_count = if density >= 0.75 {
        pcs.len().min(4)
    } else if density >= 0.45 {
        3
    } else {
        2
    };

    // Rotate for inversion
    let inv = inversion % pcs.len();
    if inv > 0 {
        pcs.rotate_left(inv);
    }

    // Slash bass overrides lowest note
    if let Some(bass) = chord.bass {
        pcs.retain(|&pc| pc != bass.pc);
        pcs.insert(0, bass.pc);
    }

    pcs.truncate(note_count);

    let mut best = Vec::new();
    let mut best_score = f32::NEG_INFINITY;

    for base_oct in 3..=5 {
        let mut voicing: Vec<u8> = pcs
            .iter()
            .enumerate()
            .map(|(i, &pc)| (base_oct + i as u8).saturating_mul(12) + pc)
            .collect();
        voicing.sort_unstable();

        if voicing.iter().any(|&m| m < register.0 || m > register.1) {
            continue;
        }
        if let Some(mel) = melody_midi {
            if voicing.iter().any(|&v| v >= mel.saturating_sub(3)) {
                continue;
            }
        }

        let mut score = 10.0f32;
        if !prev.is_empty() {
            for (v, p) in voicing.iter().zip(prev.iter()) {
                let motion = (*v as i16 - *p as i16).unsigned_abs();
                score -= motion as f32 * 0.2;
            }
        }
        score += density * voicing.len() as f32;
        if inv > 0 {
            score += 1.5;
        }

        if score > best_score {
            best_score = score;
            best = voicing;
        }
    }

    if best.is_empty() {
        let pc = pcs[0];
        best.push(
            register
                .0
                .max(48)
                .min(register.1)
                .div_euclid(12)
                .saturating_mul(12)
                + pc,
        );
    }
    best
}

/// Single arpeggio tone for weak-beat comping.
pub fn arpeggio_tone(
    chord: &ChordSymbol,
    step: usize,
    register: (u8, u8),
) -> u8 {
    let pcs = chord.voicing_pcs();
    let pc = pcs[step % pcs.len()];
    for oct in (3..=5).rev() {
        let midi = oct * 12 + pc;
        if midi >= register.0 && midi <= register.1 {
            return midi;
        }
    }
    register.0.max(48).div_euclid(12) * 12 + pc
}

/// Occasionally substitute sus4 color on subdominant/dominant (returns new quality).
pub fn sus_color(chord: &ChordSymbol, step: usize) -> ChordQuality {
    if matches!(
        chord.quality,
        ChordQuality::Major | ChordQuality::Sus2 | ChordQuality::Sus4
    ) && (step % 7 == 3 || step % 11 == 5)
    {
        ChordQuality::Sus4
    } else {
        chord.quality
    }
}
