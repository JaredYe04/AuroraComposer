//! Bass candidate generation — root targeting, walk patterns, approach tones.

use aurora_ast::Event;
use aurora_rules::{
    CandidateGenerator, CandidatePatch, ChordSymbol as RuleChord, NodeId as RuleNodeId,
    SearchState, VoiceRole, search_note,
};

fn patch_midi(patch: &CandidatePatch) -> Option<u8> {
    patch.nodes_to_add.iter().find_map(|e| {
        if let Event::Note(n) = e {
            Some(n.pitch.midi)
        } else {
            None
        }
    })
}

pub struct BassCandidateGenerator {
    pub chord_grid: Vec<RuleChord>,
    pub measure_ids: Vec<RuleNodeId>,
    pub beats_per_measure: u8,
    pub bass_register: (u8, u8),
    pub jazz_walk: bool,
}

impl CandidateGenerator for BassCandidateGenerator {
    fn voice_role(&self) -> VoiceRole {
        VoiceRole::Bass
    }

    fn generate(&self, state: &SearchState) -> Vec<CandidatePatch> {
        let step = state.step_index as usize;
        let beat = step % usize::from(self.beats_per_measure);
        let measure_idx = step / usize::from(self.beats_per_measure);
        let chord = self
            .chord_grid
            .get(step)
            .or_else(|| self.chord_grid.get(measure_idx))
            .cloned()
            .unwrap_or_else(|| RuleChord::simple(0, aurora_ast::ChordQuality::Major, "C"));

        let next_chord = self
            .chord_grid
            .get(step + 1)
            .or_else(|| self.chord_grid.get(measure_idx + 1))
            .cloned()
            .unwrap_or_else(|| chord.clone());

        let prev_chord = if step > 0 {
            self.chord_grid.get(step - 1).cloned()
        } else {
            None
        };

        let chord_changed = prev_chord
            .as_ref()
            .is_none_or(|p| p.root.pc != chord.root.pc || p.quality != chord.quality);

        let measure_id = self
            .measure_ids
            .get(measure_idx)
            .copied()
            .unwrap_or(RuleNodeId::new(1, 0));

        let (min_midi, max_midi) = self.bass_register;
        let root_midi = 12 + chord.root.pc;
        let next_root = 12 + next_chord.root.pc;
        let mut candidates = Vec::new();

        // Strong beat + chord change: root of current chord
        if beat == 0 || chord_changed {
            for oct in 0..=2 {
                let midi = oct * 12 + chord.root.pc;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(measure_id, beat, midi, "bass_root"));
                }
            }
        }

        // Walk patterns — diatonic only unless jazz style
        if self.jazz_walk || beat > 0 {
            for delta in [0i16, 2, 4, 5, 7] {
                let midi = (root_midi as i16 + delta).clamp(min_midi as i16, max_midi as i16) as u8;
                candidates.push(make_patch(measure_id, beat, midi, "bass_walk"));
            }
            if self.jazz_walk {
                for delta in [-1i16, -2] {
                    let midi =
                        (root_midi as i16 + delta).clamp(min_midi as i16, max_midi as i16) as u8;
                    candidates.push(make_patch(measure_id, beat, midi, "bass_chromatic"));
                }
            }
        }

        // Approach next root: half-step or whole-step below
        if beat >= 2 || self.jazz_walk {
            for approach in [next_root.saturating_sub(1), next_root.saturating_sub(2)] {
                if approach >= min_midi && approach <= max_midi {
                    candidates.push(make_patch(measure_id, beat, approach, "bass_approach"));
                }
            }
        }

        if let Some(prev) = state.snapshot.last_pitch(VoiceRole::Bass) {
            for delta in [-2i16, -1, 1, 2, 3, 4, 5, 7] {
                let midi = (prev.midi as i16 + delta).clamp(min_midi as i16, max_midi as i16) as u8;
                if midi != prev.midi {
                    candidates.push(make_patch(measure_id, beat, midi, "bass_step"));
                }
            }
        }

        for pc in chord.voicing_pcs() {
            for oct in 0..=2 {
                let midi = oct * 12 + pc;
                if midi >= min_midi && midi <= max_midi {
                    candidates.push(make_patch(measure_id, beat, midi, "bass_chord_tone"));
                }
            }
        }

        candidates.sort_by_key(|c| c.nodes_to_add.len());
        candidates.dedup_by(|a, b| patch_midi(a) == patch_midi(b));

        if let Some(prev) = state.snapshot.last_pitch(VoiceRole::Bass) {
            candidates.retain(|c| patch_midi(c) != Some(prev.midi));
            if candidates.is_empty() {
                let escape = (prev.midi as i16 + 2).clamp(min_midi as i16, max_midi as i16) as u8;
                candidates.push(make_patch(measure_id, beat, escape, "bass_escape"));
            }
        }

        candidates
    }
}

fn make_patch(measure_id: RuleNodeId, _beat: usize, midi: u8, label: &str) -> CandidatePatch {
    CandidatePatch::single_note(
        aurora_rules::VoiceId(2),
        measure_id,
        search_note(midi, RuleNodeId::new(u64::from(midi), 0)),
        format!("{label}_{midi}"),
    )
}
