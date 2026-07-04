use aurora_ast::nodes::{ThemeRef, ThemeTransform};
use aurora_ast::PipelineStageId;

use super::common::make_note_provenance;
use super::PipelineState;

/// Stage 4 — Theme Planning: assign theme slots and motif refs per section/phrase.
pub fn plan_themes(state: &mut PipelineState, created_at: &str) -> Result<(), String> {
    let theme_count = state.params.theme.theme_count.max(1);
    let motif_len = state.params.theme.motif_length.max(1);

    for section in state
        .composition
        .movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
    {
        section.metadata.theme_refs.clear();
        for t in 0..theme_count {
            let transform = match t % 4 {
                0 => ThemeTransform::Original,
                1 => ThemeTransform::Sequence,
                2 => ThemeTransform::Inversion,
                _ => ThemeTransform::Augmentation,
            };
            section.metadata.theme_refs.push(ThemeRef {
                theme_id: format!("theme-{t}"),
                transformation: transform,
            });
        }
    }

    let mut phrase_idx = 0u32;
    for phrase in state
        .composition
        .movements
        .iter_mut()
        .flat_map(|m| &mut m.sections)
        .flat_map(|s| &mut s.phrases)
    {
        let theme_id = format!("theme-{}", phrase_idx % u32::from(theme_count));
        phrase.metadata.motif_ref = Some(format!("motif-{theme_id}-{motif_len}"));
        let _ = make_note_provenance(
            PipelineStageId::ThemePlanning,
            created_at,
            vec![format!("FORM-DEV-00{}", (phrase_idx % 3) + 1)],
            format!("assigned {theme_id} to {}", phrase.metadata.phrase_id),
        );
        phrase_idx += 1;
    }

    Ok(())
}
