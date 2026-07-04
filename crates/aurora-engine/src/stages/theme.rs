use aurora_ast::nodes::ThemeTransform;

use crate::motif::{apply_transform, generate_motif};

use super::common::make_note_provenance;
use super::{PhraseMotifPlan, PipelineState};

/// Stage 4 — Theme Planning: generate motif seeds and phrase-level development plans.
pub fn plan_themes(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let seed = state.params.search.seed.unwrap_or(42);
    let theme_count = state.params.theme.theme_count.max(1);
    let motif_len = state.params.theme.motif_length.max(1);
    let beats_per_measure = usize::from(state.params.rhythm.time_signature_beats.max(1));
    let phrase_measures = usize::from(state.params.form.phrase_length.max(1));
    let phrase_beats = phrase_measures * beats_per_measure;

    state.motifs.clear();
    state.phrase_motif_plans.clear();

    for t in 0..theme_count {
        let motif_id = format!("motif-theme-{t}-{motif_len}");
        let mix = seed.wrapping_add(u64::from(t).wrapping_mul(0x517C_C1B7));
        let motif = generate_motif(
            &motif_id,
            &state.params.theme,
            &state.params.mode,
            u32::from(t),
            mix,
        );
        state.motifs.insert(motif_id, motif);
    }

    for section in state
        .composition
        .movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
    {
        section.metadata.theme_refs.clear();
        for t in 0..theme_count {
            let transform = phrase_transform(seed, u32::from(t));
            section.metadata.theme_refs.push(aurora_ast::nodes::ThemeRef {
                theme_id: format!("theme-{t}"),
                transformation: transform,
            });
        }
    }

    let mut phrase_idx = 0u32;
    let mut global_beat = 0usize;
    for phrase in state
        .composition
        .movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
        .flat_map(|s| &mut s.phrases)
    {
        let theme_idx =
            (seed.wrapping_mul(u64::from(phrase_idx) + 1) % u64::from(theme_count.max(1))) as u32;
        let theme_id = format!("theme-{theme_idx}");
        let motif_id = format!("motif-{theme_id}-{motif_len}");
        phrase.metadata.motif_ref = Some(motif_id.clone());

        if state.params.form.phrase_model == "period" {
            phrase.metadata.contour_hint = Some(if phrase_idx % 2 == 0 {
                aurora_ast::nodes::ContourHint::Ascending
            } else {
                aurora_ast::nodes::ContourHint::Arch
            });
        }

        if phrase_idx % 2 == 1 {
            if let Some(base) = state.motifs.get(&motif_id).cloned() {
                let transform = phrase_transform(seed, phrase_idx);
                let suffix = transform_suffix(transform);
                let variant_id = format!("{motif_id}-{suffix}");
                let variant = apply_transform(&base, transform);
                state.motifs.insert(variant_id.clone(), variant);
                phrase.metadata.motif_ref = Some(variant_id);
            }
        }

        let active_motif_id = phrase.metadata.motif_ref.clone().unwrap_or(motif_id);
        let base_midi = state
            .motifs
            .get(&active_motif_id)
            .map(|m| m.base_midi)
            .unwrap_or(60 + state.params.mode.key % 12);

        let region_ratio = state
            .params
            .theme
            .repetition_ratio
            .max(state.params.melody.motif_weight * 0.85);
        let region_beats = ((phrase_beats as f32) * region_ratio).max(4.0) as usize;

        state.phrase_motif_plans.push(PhraseMotifPlan {
            phrase_index: phrase_idx,
            motif_id: active_motif_id,
            base_midi,
            region_beats,
            phrase_start_beat: global_beat,
        });

        global_beat += phrase.measures.len() * beats_per_measure;

        let _ = make_note_provenance(
            aurora_ast::PipelineStageId::ThemePlanning,
            created_at,
            vec![format!("FORM-DEV-00{}", (phrase_idx % 3) + 1)],
            format!("assigned {theme_id} to {}", phrase.metadata.phrase_id),
        );
        phrase_idx += 1;
    }

    Ok(())
}

fn phrase_transform(seed: u64, index: u32) -> ThemeTransform {
    let pick = (seed
        .wrapping_mul(u64::from(index) + 7)
        .wrapping_add(0xDEAD_BEEF)
        % 5) as u32;
    match pick {
        0 => ThemeTransform::Original,
        1 => ThemeTransform::Sequence,
        2 => ThemeTransform::Inversion,
        3 => ThemeTransform::Fragmentation,
        _ => ThemeTransform::Retrograde,
    }
}

fn transform_suffix(transform: ThemeTransform) -> &'static str {
    match transform {
        ThemeTransform::Original => "orig",
        ThemeTransform::Sequence => "seq",
        ThemeTransform::Inversion => "inv",
        ThemeTransform::Retrograde => "retro",
        ThemeTransform::Fragmentation => "frag",
        ThemeTransform::Augmentation => "aug",
        ThemeTransform::Diminution => "dim",
        ThemeTransform::ModalInterchange => "modal",
    }
}
